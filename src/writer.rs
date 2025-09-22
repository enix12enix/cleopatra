// src/writer.rs
use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::time::{sleep, Duration};
use sqlx::{SqlitePool};
use crate::models::CreateTestResult;
use crate::config::WriterConfig;
use crate::db::upsert_test_result;

#[derive(Clone)]
pub struct Writer {
    sender: Sender<CreateTestResult>,
}

impl Writer {
    pub async fn enqueue(&self, result: CreateTestResult) -> Result<(), String> {
        self.sender
            .send(result)
            .await
            .map_err(|e| format!("Failed to enqueue result: {}", e))
    }

    pub fn shutdown(&self) {
        // Dropping a clone closes one strong reference.
        // Once all senders are dropped, the receiver gets `None`.
        drop(self.sender.clone());
    }
}

pub async fn start_writer(config: &WriterConfig, writer_pool: SqlitePool) -> Writer {
    let (tx, mut rx): (Sender<CreateTestResult>, Receiver<CreateTestResult>) =
        channel(config.batch_size);

    let batch_size = config.batch_size;
    let flush_interval = Duration::from_millis(config.flush_interval_ms);

    tokio::spawn(async move {
        let mut conn = writer_pool.acquire().await.expect("Failed to acquire writer connection");
        
        let mut buffer = Vec::with_capacity(batch_size);

        loop {
            tokio::select! {
                maybe_item = rx.recv() => {
                    match maybe_item {
                        Some(item) => {
                            buffer.push(item);
                            
                            // Flush immediately if we've reached batch size
                            if buffer.len() >= batch_size {
                                flush(&mut conn, &buffer).await;
                                buffer.clear();
                            }
                        },
                        None => {
                            // Channel closed -> final flush and exit
                            if !buffer.is_empty() {
                                flush(&mut conn, &buffer).await;
                            }
                            println!("Writer shutdown complete.");
                            break;
                        }
                    }
                }
                // periodically flush
                _ = sleep(flush_interval), if !buffer.is_empty() => {
                    flush(&mut conn, &buffer).await;
                    buffer.clear();
                }
            }
        }
    });

    Writer { sender: tx }
}

async fn flush(conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>, buffer: &[CreateTestResult]) {
    if buffer.is_empty() {
        return;
    }

    println!("Flushing {} results to DB", buffer.len());

    if let Err(e) = do_flush(conn, buffer).await {
        eprintln!("Error flushing results: {}", e);
    }
}

async fn do_flush(conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>, buffer: &[CreateTestResult]) -> Result<(), sqlx::Error> {
    use sqlx::Acquire;
    
    let mut tx = conn.begin().await?;

    for item in buffer {
        upsert_test_result(
            &mut *tx,
            item,
        )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

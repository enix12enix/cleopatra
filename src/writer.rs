// src/writer.rs
use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::time::{sleep, Duration};
use sqlx::{Connection, SqliteConnection, Transaction};
use crate::models::CreateTestResult;
use crate::config::WriterConfig;
use crate::routes::utils::upsert_test_result;

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

pub async fn start_writer(config: &WriterConfig, db_url: &str) -> Writer {
    let (tx, mut rx): (Sender<CreateTestResult>, Receiver<CreateTestResult>) =
        channel(config.batch_size);

    let batch_size = config.batch_size;
    let flush_interval = Duration::from_millis(config.flush_interval_ms);
    let db_url = db_url.to_string();

    tokio::spawn(async move {
        // Dedicated connection (not pooled)
        let mut conn = SqliteConnection::connect(&db_url)
            .await
            .expect("Failed to create writer connection");

        let mut buffer = Vec::with_capacity(batch_size);

        loop {
            tokio::select! {
                maybe_item = rx.recv() => {
                    match maybe_item {
                        Some(item) => buffer.push(item),
                        None => {
                            // Channel closed -> final flush and exit
                            if !buffer.is_empty() {
                                flush(&mut conn, &mut buffer).await;
                            }
                            println!("Writer shutdown complete.");
                            break;
                        }
                    }
                }
                _ = sleep(flush_interval), if !buffer.is_empty() => {
                    flush(&mut conn, &mut buffer).await;
                }
            }

            if buffer.len() >= batch_size {
                flush(&mut conn, &mut buffer).await;
            }
        }
    });

    Writer { sender: tx }
}

async fn flush(conn: &mut SqliteConnection, buffer: &mut Vec<CreateTestResult>) {
    if buffer.is_empty() {
        return;
    }

    println!("Flushing {} results to DB", buffer.len());

    if let Err(e) = do_flush(conn, buffer).await {
        eprintln!("Error flushing results: {}", e);
    }

    buffer.clear();
}

async fn do_flush(conn: &mut SqliteConnection, buffer: &[CreateTestResult]) -> Result<(), sqlx::Error> {
    let mut tx: Transaction<'_, sqlx::Sqlite> = conn.begin().await?;

    for item in buffer {
        upsert_test_result(
            &mut *tx,
            item.execution_id,
            item.name.clone(),
            item.platform.clone(),
            item.description.clone(),
            item.status.clone(),
            item.execution_time,
            item.log.clone(),
            item.screenshot_id,
            item.created_by.clone(),
            item.time_created,
        )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

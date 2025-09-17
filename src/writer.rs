// src/writer.rs
use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::time::{sleep, Duration};
use sqlx::{Connection, SqliteConnection, Transaction, Row};
use crate::models::CreateTestResult;
use crate::config::WriterConfig;

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
        // Runtime-checked query (no compile-time DB needed)
        let existing = sqlx::query(
            r#"
            SELECT id, counter FROM test_result
            WHERE execution_id = ? AND name = ?
            "#
        )
        .bind(&item.execution_id)
        .bind(&item.name)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(record) = existing {
            let new_counter: i64 = record.get::<i64, _>("counter") + 1;
            let id: i64 = record.get::<i64, _>("id");

            sqlx::query(
                r#"
                UPDATE test_result
                SET platform = ?,
                    description = ?,
                    status = ?,
                    execution_time = ?,
                    counter = ?,
                    log = ?,
                    screenshot_id = ?,
                    created_by = ?,
                    time_created = ?
                WHERE id = ?
                "#
            )
            .bind(&item.platform)
            .bind(&item.description)
            .bind(&item.status)
            .bind(&item.execution_time)
            .bind(new_counter)
            .bind(&item.log)
            .bind(&item.screenshot_id)
            .bind(&item.created_by)
            .bind(&item.time_created)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO test_result (
                    execution_id, name, platform, description,
                    status, execution_time, counter, log,
                    screenshot_id, created_by, time_created
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&item.execution_id)
            .bind(&item.name)
            .bind(&item.platform)
            .bind(&item.description)
            .bind(&item.status)
            .bind(&item.execution_time)
            .bind(1_i64) // counter starts at 1
            .bind(&item.log)
            .bind(&item.screenshot_id)
            .bind(&item.created_by)
            .bind(&item.time_created)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

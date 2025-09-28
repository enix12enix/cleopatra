// src/writer.rs
use async_trait::async_trait;
use sqlx::{Acquire, Pool, Sqlite, Transaction};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::time::{Duration, sleep};

use crate::config::Config;
use crate::database::upsert_test_result;
use crate::models::CreateTestResult;

#[async_trait]
pub trait Writer: Send + Sync + 'static {
    type Message: Send + 'static;
    type Datasource: Clone + Send + Sync + 'static;
    type Error: std::fmt::Display + Send + 'static;

    fn sender(&self) -> &Sender<Self::Message>;
    fn config_name() -> &'static str;

    async fn flush_db(
        &self,
        ds: &Self::Datasource,
        buffer: &[Self::Message],
    ) -> Result<(), Self::Error>;

    async fn new(config: &Config, ds: Self::Datasource) -> Self
    where
        Self: Sized;

    async fn enqueue(&self, message: Self::Message) -> Result<(), String> {
        self.sender()
            .clone()
            .send(message)
            .await
            .map_err(|e| format!("Failed to enqueue message: {}", e))
    }

    fn shutdown(&self) {
        drop(self.sender().clone());
    }
}

/// --- Start the writer loop
pub async fn start_writer_loop<W>(
    writer: Arc<W>,
    mut rx: Receiver<W::Message>,
    datasource: W::Datasource,
    batch_size: usize,
    flush_interval_ms: u64,
) where
    W: Writer + 'static,
{
    tokio::spawn(async move {
        let flush_interval = Duration::from_millis(flush_interval_ms);
        let mut buffer = Vec::with_capacity(batch_size);

        loop {
            tokio::select! {
                maybe_item = rx.recv() => {
                    match maybe_item {
                        Some(item) => {
                            buffer.push(item);
                            if buffer.len() >= batch_size {
                                if let Err(e) = writer.flush_db(&datasource, &buffer).await {
                                    eprintln!("Error flushing: {}", e);
                                }
                                buffer.clear();
                            }
                        }
                        None => {
                            if !buffer.is_empty() {
                                let _ = writer.flush_db(&datasource, &buffer).await;
                            }
                            println!("Writer shutdown complete");
                            break;
                        }
                    }
                }
                _ = sleep(flush_interval), if !buffer.is_empty() => {
                    if let Err(e) = writer.flush_db(&datasource, &buffer).await {
                        eprintln!("Error flushing: {}", e);
                    }
                    buffer.clear();
                }
            }
        }
    });
}

/// --- DefaultWriter for SQLite ---
#[derive(Clone)]
pub struct DefaultWriter {
    sender: Sender<CreateTestResult>,
}

#[async_trait]
impl Writer for DefaultWriter {
    type Message = CreateTestResult;
    type Datasource = Pool<Sqlite>;
    type Error = sqlx::Error;

    fn sender(&self) -> &Sender<Self::Message> {
        &self.sender
    }

    fn config_name() -> &'static str {
        "main"
    }

    async fn new(config: &Config, ds: Self::Datasource) -> Self {
        let writer_config = config
            .writers
            .get(Self::config_name())
            .expect("Writer config not found");

        let (tx, rx) = channel(writer_config.batch_size);
        let writer = Arc::new(Self { sender: tx.clone() });

        start_writer_loop(
            writer.clone(),
            rx,
            ds.clone(),
            writer_config.batch_size,
            writer_config.flush_interval_ms,
        )
        .await;

        Arc::try_unwrap(writer).unwrap_or_else(|arc| (*arc).clone())
    }

    /// --- Transactional flush
    async fn flush_db(
        &self,
        ds: &Self::Datasource,
        buffer: &[Self::Message],
    ) -> Result<(), Self::Error> {
        if buffer.is_empty() {
            return Ok(());
        }

        println!("flush data to db :: {}", buffer.len());

        // Acquire a connection from the pool
        let mut conn = ds.acquire().await?;

        // Start a transaction
        let mut tx: Transaction<'_, Sqlite> = conn.begin().await?;

        // Execute all upserts inside this transaction
        for item in buffer {
            upsert_test_result(&mut tx, item).await?;
        }

        // Commit transaction
        tx.commit().await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum WriterName {
    Main,
}

pub struct WriterManager {
    writers: HashMap<WriterName, Box<dyn AnyWriter + Send + Sync>>,
}

#[async_trait]
pub trait AnyWriter: Send + Sync {
    async fn enqueue_boxed(&self, message: Box<dyn std::any::Any + Send>) -> Result<(), String>;
    fn shutdown(&self);
}

#[async_trait]
impl<T> AnyWriter for Arc<T>
where
    T: Writer + 'static,
    T::Message: 'static,
{
    async fn enqueue_boxed(&self, message: Box<dyn std::any::Any + Send>) -> Result<(), String> {
        let msg = *message
            .downcast::<T::Message>()
            .map_err(|_| "Failed to downcast message".to_string())?;
        self.enqueue(msg).await
    }

    fn shutdown(&self) {
        Writer::shutdown(&**self)
    }
}

impl WriterManager {
    pub fn new() -> Self {
        Self {
            writers: HashMap::new(),
        }
    }

    pub fn insert<W>(&mut self, name: WriterName, writer: W)
    where
        W: Writer + 'static,
    {
        self.writers.insert(name, Box::new(Arc::new(writer)));
    }

    pub async fn enqueue(
        &self,
        name: WriterName,
        message: Box<dyn std::any::Any + Send>,
    ) -> Result<(), String> {
        let writer = self.writers.get(&name).ok_or("Writer not found")?;
        writer.enqueue_boxed(message).await
    }

    pub fn shutdown_all(&self) {
        for w in self.writers.values() {
            w.shutdown();
        }
    }
}

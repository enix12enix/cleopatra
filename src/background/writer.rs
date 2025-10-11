// src/background/writer.rs
use async_trait::async_trait;
use sqlx::{Acquire, Pool, Sqlite, Transaction};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{sleep, Instant};

use crossbeam_queue::ArrayQueue;

use crate::config::Config;
use crate::database::upsert_test_result;
use crate::models::CreateTestResult;

// the interface for writer
#[async_trait]
pub trait Writer: Send + Sync + 'static {
    type Message: Send + Sync + Clone + 'static;
    type Datasource: Clone + Send + Sync + 'static;
    type Error: std::fmt::Display + Send + 'static;

    fn sender(&self) -> &Sender<Self::Message>;
    fn config_name() -> &'static str;

    #[allow(dead_code)]
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

/// --- DefaultWriter for SQLite (dispatcher + lock-free ring buffer)
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

        let batch_size = writer_config.batch_size;
        let flush_interval_ms = writer_config.flush_interval_ms;
        let queue_capacity = (batch_size * 16).max(1024);

        let (tx, mut rx): (Sender<CreateTestResult>, Receiver<CreateTestResult>) =
            channel(writer_config.batch_size * 8);

        let queue = Arc::new(ArrayQueue::<CreateTestResult>::new(queue_capacity));
        let closed = Arc::new(AtomicBool::new(false));
        let ds_clone_for_writer = ds.clone();

        // Dispatcher task: channel -> queue
        {
            let queue = Arc::clone(&queue);
            let closed = Arc::clone(&closed);
            tokio::spawn(async move {
                while let Some(item) = rx.recv().await {
                    loop {
                        match queue.push(item.clone()) {
                            Ok(()) => break,
                            Err(_) => {
                                tokio::task::yield_now().await;
                            }
                        }
                    }
                }
                closed.store(true, Ordering::SeqCst);
            });
        }

        // Writer task: queue -> db
        {
            let queue = Arc::clone(&queue);
            let closed = Arc::clone(&closed);
            let batch_size = batch_size;
            let flush_interval = Duration::from_millis(flush_interval_ms);

            tokio::spawn(async move {
                let mut buffer: Vec<CreateTestResult> = Vec::with_capacity(batch_size);
                let mut last_flush = Instant::now();

                loop {
                    // read message from queue and add it to buffer
                    while buffer.len() < batch_size {
                        match queue.pop() {
                            Some(item) => buffer.push(item),
                            None => break,
                        }
                    }

                    let now = Instant::now();
                    let time_elapsed = now.duration_since(last_flush);

                    // flush data
                    if !buffer.is_empty() && (buffer.len() >= batch_size || time_elapsed >= flush_interval) {
                        if let Err(e) = flush_to_sqlite(&ds_clone_for_writer, &buffer).await {
                            eprintln!("Error flushing to sqlite: {}", e);
                        } else {
                            buffer.clear();
                            last_flush = Instant::now();
                        }
                    }

                    // exit if queue is closed and data is cleared
                    if closed.load(Ordering::SeqCst) {
                        while let Some(item) = queue.pop() {
                            buffer.push(item);
                            if buffer.len() >= batch_size {
                                if let Err(e) = flush_to_sqlite(&ds_clone_for_writer, &buffer).await {
                                    eprintln!("Error flushing to sqlite at shutdown: {}", e);
                                }
                                buffer.clear();
                            }
                        }

                        if !buffer.is_empty() {
                            if let Err(e) = flush_to_sqlite(&ds_clone_for_writer, &buffer).await {
                                eprintln!("Error flushing to sqlite at shutdown: {}", e);
                            }
                            buffer.clear();
                        }

                        break;
                    }

                    // avoid busy loop
                    sleep(Duration::from_millis(10)).await;
                }

                println!("DefaultWriter writer task exiting cleanly");
            });
        }

        Self { sender: tx }
    }

    async fn flush_db(
        &self,
        ds: &Self::Datasource,
        buffer: &[Self::Message],
    ) -> Result<(), Self::Error> {
        flush_to_sqlite(ds, buffer).await
    }
}

async fn flush_to_sqlite(
    ds: &Pool<Sqlite>,
    buffer: &[CreateTestResult],
) -> Result<(), sqlx::Error> {
    if buffer.is_empty() {
        return Ok(());
    }

    let mut conn = ds.acquire().await?;
    let mut tx: Transaction<'_, Sqlite> = conn.begin().await?;

    for item in buffer {
        upsert_test_result(&mut tx, item).await?;
    }

    tx.commit().await?;
    println!("flush data to db :: {}", buffer.len());
    Ok(())
}

/// --- Writer Manager
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

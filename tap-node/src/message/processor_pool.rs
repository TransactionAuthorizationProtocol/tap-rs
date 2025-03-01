//! Concurrent message processing pool
//!
//! This module provides a task pool for processing messages concurrently.

use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinHandle;

use crate::error::{Error, Result};
use tap_core::message::TapMessage;

use super::MessageProcessor;

/// Configuration for the processor pool
#[derive(Debug, Clone)]
pub struct ProcessorPoolConfig {
    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Size of the message queue
    pub queue_size: usize,
}

impl Default for ProcessorPoolConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 32,
            queue_size: 1000,
        }
    }
}

/// Concurrent message processor pool
pub struct ProcessorPool<T: MessageProcessor> {
    /// The message processor to use
    _processor: Arc<T>,
    /// Sender for the message queue
    sender: mpsc::Sender<TapMessage>,
    /// Task handles for the worker tasks
    _task_handles: Vec<JoinHandle<()>>,
    /// Semaphore for limiting concurrent tasks
    _semaphore: Arc<Semaphore>,
}

impl<T: MessageProcessor + 'static> ProcessorPool<T> {
    /// Create a new processor pool
    pub fn new(processor: Arc<T>, config: ProcessorPoolConfig) -> Self {
        let (sender, receiver) = mpsc::channel::<TapMessage>(config.queue_size);
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));

        // Create a mutex-wrapped receiver that can be shared among workers
        let shared_receiver = Arc::new(tokio::sync::Mutex::new(receiver));

        // Spawn worker tasks
        let mut task_handles = Vec::with_capacity(config.max_concurrent_tasks);
        for i in 0..config.max_concurrent_tasks {
            let worker_processor = processor.clone();
            let worker_receiver = shared_receiver.clone();
            let worker_semaphore = semaphore.clone();

            // Spawn a new worker task
            let handle = tokio::spawn(Self::worker_task(
                i,
                worker_processor,
                worker_receiver,
                worker_semaphore,
            ));

            task_handles.push(handle);
        }

        Self {
            _processor: processor,
            sender,
            _task_handles: task_handles,
            _semaphore: semaphore,
        }
    }

    /// Submit a message for processing
    pub async fn submit(&self, message: TapMessage) -> Result<()> {
        self.sender
            .send(message)
            .await
            .map_err(|_| Error::Dispatch("Failed to submit message to processor pool".to_string()))
    }

    /// Worker task that processes messages
    async fn worker_task(
        _id: usize,
        processor: Arc<T>,
        receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<TapMessage>>>,
        semaphore: Arc<Semaphore>,
    ) {
        loop {
            // Acquire a permit from the semaphore
            let _permit = semaphore.acquire().await.unwrap();

            // Get the next message
            let message = {
                let mut receiver_guard = receiver.lock().await;
                match receiver_guard.recv().await {
                    Some(msg) => msg,
                    None => {
                        // Channel closed, exit the worker
                        break;
                    }
                }
            };

            // Process the message asynchronously - using spawn for truly concurrent execution
            let processor_clone = processor.clone();
            tokio::spawn(async move {
                // We use process_incoming here, but this could be adapted for different processing
                let _ = processor_clone.process_incoming(message).await;
                // Note: in a real implementation, we would handle the result and possibly log errors
            });
        }
    }
}

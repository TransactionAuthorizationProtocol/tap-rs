//! Processor pool for concurrent message processing.
//!
//! This module provides a processor pool for handling concurrent message processing.

use tap_msg::didcomm::PlainMessage;
use tokio::sync::mpsc::{channel, Sender};
use tokio::time::Duration;

use crate::error::{Error, Result};
use crate::message::processor::PlainMessageProcessor;
use crate::message::{CompositePlainMessageProcessor, PlainMessageProcessorType};

/// Configuration for the processor pool
#[derive(Debug, Clone)]
pub struct ProcessorPoolConfig {
    /// The number of worker tasks to create
    pub workers: usize,
    /// The capacity of the message channel
    pub channel_capacity: usize,
    /// The maximum duration to wait for a worker to process a message
    pub worker_timeout: Duration,
}

impl Default for ProcessorPoolConfig {
    fn default() -> Self {
        Self {
            workers: 4,
            channel_capacity: 100,
            worker_timeout: Duration::from_secs(30),
        }
    }
}

/// Processor pool for concurrent message processing
#[derive(Clone)]
pub struct ProcessorPool {
    /// The message processor to use
    processor: CompositePlainMessageProcessor,
    /// Channel for submitting messages for processing
    tx: Sender<PlainMessage>,
}

impl ProcessorPool {
    /// Create a new processor pool
    pub fn new(config: ProcessorPoolConfig) -> Self {
        let (tx, mut rx) = channel::<PlainMessage>(config.channel_capacity);
        let processors: Vec<PlainMessageProcessorType> = Vec::new();
        let processor = CompositePlainMessageProcessor::new(processors);
        let processor_for_workers = processor.clone();

        // Spawn a single task to distribute messages to workers
        tokio::spawn(async move {
            // Create worker channels
            let mut worker_channels = Vec::with_capacity(config.workers);
            for _ in 0..config.workers {
                let (worker_tx, mut worker_rx) = channel::<PlainMessage>(config.channel_capacity);
                worker_channels.push(worker_tx);

                let worker_processor = processor_for_workers.clone();
                let worker_timeout = config.worker_timeout;

                // Spawn a worker to process messages from its channel
                tokio::spawn(async move {
                    while let Some(message) = worker_rx.recv().await {
                        match tokio::time::timeout(
                            worker_timeout,
                            worker_processor.process_incoming(message),
                        )
                        .await
                        {
                            Ok(result) => {
                                if let Err(e) = result {
                                    eprintln!("Error processing message: {}", e);
                                }
                            }
                            Err(_) => {
                                eprintln!(
                                    "PlainMessage processing timed out after {:?}",
                                    worker_timeout
                                );
                            }
                        }
                    }
                });
            }

            // Round-robin distribute messages to workers
            let mut current_worker = 0;
            while let Some(message) = rx.recv().await {
                if worker_channels.is_empty() {
                    break;
                }

                // Try to send to the current worker, or move to the next one if fails
                let mut attempts = 0;
                while attempts < worker_channels.len() {
                    match worker_channels[current_worker].send(message.clone()).await {
                        Ok(_) => break,
                        Err(_) => {
                            current_worker = (current_worker + 1) % worker_channels.len();
                            attempts += 1;
                        }
                    }
                }

                // Advance to next worker
                current_worker = (current_worker + 1) % worker_channels.len();
            }
        });

        Self { processor, tx }
    }

    /// Submit a message for processing
    pub async fn submit(&self, message: PlainMessage) -> Result<()> {
        self.tx.send(message).await.map_err(|e| {
            Error::Processing(format!("Failed to submit message to processor pool: {}", e))
        })
    }

    /// Add a processor to the pool
    pub fn add_processor(&mut self, processor: PlainMessageProcessorType) {
        self.processor.add_processor(processor);
    }
}

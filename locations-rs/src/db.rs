use scylla::transport::errors::QueryError;
use scylla::batch::Batch;
use std::sync::Arc;
use async_channel::Receiver;
use tracing::{error, info};
use thiserror::Error;
use scylla::{Session, SessionBuilder};
use std::process;

const BATCH_SIZE: usize = 200;

#[derive(Error, Debug)]
pub enum OutputError {
    #[error("Scylla query error: {0}")]
    ScyllaError(#[from] QueryError),
    #[error("Channel receive error: {0}")]
    ChannelError(#[from] async_channel::RecvError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub async fn create_db_session(uri: &str, keyspace: &str) -> Arc<Session> {
    match SessionBuilder::new()
        .known_node(uri)
        .use_keyspace(keyspace, true)
        .build()
        .await
    {
        Ok(session) => Arc::new(session),
        Err(e) => {
            eprintln!("Failed to connect to Scylla DB at {}: {}", uri, e);
            process::exit(1);
        }
    }
}

pub async fn handle_db_insertions(session: Arc<Session>, rx_db: Receiver<Vec<String>>) -> Result<(), OutputError> {
    info!("Database connection established");
    let stmt = session.prepare("INSERT INTO locations (location_id) VALUES (?)").await?;

    let mut batch = Batch::default();
    let mut batch_values = Vec::with_capacity(BATCH_SIZE);

    while let Ok(location_ids) = rx_db.recv().await {
        for location_id in location_ids {
            batch.append_statement(stmt.clone());
            batch_values.push((location_id,));

            if batch_values.len() >= BATCH_SIZE {
                execute_batch(&session, &batch, &batch_values).await;
                batch_values.clear();
                batch = Batch::default();
            }
        }
    }

    // Execute any remaining batches
    if !batch_values.is_empty() {
        execute_batch(&session, &batch, &batch_values).await;
    }

    info!("Database insertion handler completed");
    Ok(())
}


async fn execute_batch(
    session: &Session,
    batch: &Batch,
    values: &[(String,)],
) {
    let mut retry_count: i32 = 0;
    let max_retries: i32 = 5;

    loop {
        match session.batch(batch, values).await {
            Ok(_) => break,
            Err(e) => {
                if retry_count >= max_retries {
                    error!("Failed to execute batch after {} retries: {:?}", max_retries, e);
                    return;
                }
                error!(attempt = retry_count + 1, "Error executing batch: {:?}. Retrying...", e);
                retry_count += 1;
            }
        }
    }
}
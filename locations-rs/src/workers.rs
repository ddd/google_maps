use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_tls::HttpsConnector;
use bytes::Bytes;
use http_body_util::Empty;
use async_channel::{Receiver, Sender};
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::fs::File;
use crate::status::CounterType;

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Channel send error for String: {0}")]
    ChannelSendErrorString(#[from] async_channel::SendError<String>),
    #[error("Channel send error for Vec<String>: {0}")]
    ChannelSendErrorUser(#[from] async_channel::SendError<Vec<String>>),
    #[error("Receive error: {0}")]
    ReceiveError(#[from] async_channel::RecvError),
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}


pub async fn fetcher(
    mut client: Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    max_retries: usize,
    rx_fetcher: Receiver<Vec<maps::tiles::Tile>>,
    tx_out: Sender<Vec<String>>,
    status: Arc<super::status::ProgramStatus>,
) -> Result<(), WorkerError> {

    while let Ok(tiles) = rx_fetcher.recv().await {
        status.update_last_tile(tiles[0].clone());
        let location_ids = super::retry::view_tiles_with_retries(&mut client, tiles, &status, max_retries).await?;
        if location_ids.len() != 0 {
            status.increment_count(CounterType::Found, location_ids.len());
            tx_out.send(location_ids).await?;
        }
    }

    Ok(())
}

pub async fn file_writer(rx_out: Receiver<Vec<String>>, filename: &str) -> Result<(), crate::db::OutputError> {
    let file = File::create(filename).await?;
    let mut writer = BufWriter::new(file);

    while let Ok(locations) = rx_out.recv().await {
        for location in locations {
            writer.write_all(location.as_bytes()).await?;
            writer.write_all(b"\n").await?;
        }
    }

    writer.flush().await?;
    Ok(())
}

use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_tls::HttpsConnector;
use tokio::time::{sleep, Duration};
use tracing::{warn, error};
use http_body_util::Empty;
use std::sync::Arc;
use bytes::Bytes;
use crate::status::CounterType;

pub async fn view_tiles_with_retries(
    client: &mut Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    tiles: Vec<maps::tiles::Tile>,
    status: &Arc<super::status::ProgramStatus>,
    max_retries: usize,
) -> Result<Vec<String>, super::workers::WorkerError> {
    let delay = Duration::from_secs(1);

    for i in 1..max_retries+1 {
        status.increment(CounterType::Request);
        match maps::tiles::view_tiles(&client, &tiles).await {
            Ok(response) => {
                return Ok(response)
            },
            Err(e) => {
                match e {
                    maps::tiles::FetchTilesError::UnexpectedStatusCode(status_code) => {
                        warn!("Unexpected status code {} when fetching tiles.", status_code);
                        sleep(delay).await;
                    }
                    _ => {
                        status.increment(CounterType::Error);
                        error!("Error in view_tiles_with_retries (attempt {}): {}", i, e);
                        sleep(delay).await;
                    }
                }
            }
        }
    }

    status.increment(CounterType::Failed);
    return Err(super::workers::WorkerError::MaxRetriesExceeded);
}
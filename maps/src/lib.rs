use thiserror::Error;
use tonic::transport::{Channel, ClientTlsConfig};
pub mod tiles;
mod places;

use mapsjs::maps_js_internal_service_client::MapsJsInternalServiceClient;
pub use places::{Place, GetPlaceError};

mod mapsjs {
    tonic::include_proto!("google.internal.maps.mapsjs.v1");
}

use places::GetPlaceRequest;

#[derive(Error, Debug)]
pub enum MapsJsInternalServiceClientError {
    #[error("Tonic transport error: {0}")]
    TonicTransport(#[from] tonic::transport::Error),

    #[error("Tonic status error: {0}")]
    TonicStatus(#[from] tonic::Status),

    #[error("Invalid metadata value: {0}")]
    InvalidMetadata(#[from] tonic::metadata::errors::InvalidMetadataValue),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Other: {0}")]
    Other(String)
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Tonic transport error: {0}")]
    TonicTransport(#[from] tonic::transport::Error),

    #[error("Tonic status error: {0}")]
    TonicStatus(#[from] tonic::Status),

    #[error("Invalid metadata value: {0}")]
    InvalidMetadata(#[from] tonic::metadata::errors::InvalidMetadataValue),

    #[error("Rate limited")]
    RateLimited,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Not found")]
    NotFound,

    #[error("Other error: {0}")]
    Other(String),
}


pub struct MapsJsInternalClient {
    client: MapsJsInternalServiceClient<Channel>,
}

impl MapsJsInternalClient {
    pub async fn new() -> Result<Self, MapsJsInternalServiceClientError> {
        let channel = Channel::from_static("https://maps.googleapis.com")
            .tls_config(ClientTlsConfig::new())?
            .connect()
            .await?;

        let client = MapsJsInternalServiceClient::new(channel);

        Ok(Self {
            client,
        })
    }

    pub async fn from_channel(channel: Channel) -> Result<Self, MapsJsInternalServiceClientError> {
        let client = MapsJsInternalServiceClient::new(channel);

        Ok(Self {
            client,
        })
    }

    pub fn get_place(&mut self, location_id: String) -> GetPlaceRequest {
        GetPlaceRequest {
            client: &mut self.client,
            location_id
        }
    }
}

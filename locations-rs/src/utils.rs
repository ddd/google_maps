use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use hyper_tls::HttpsConnector;
use native_tls::TlsConnector;
use http_body_util::Empty;
use bytes::Bytes;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),
    #[error("TLS error: {0}")]
    TlsError(#[from] native_tls::Error),
}

pub fn initialize_client() -> Result<Client<HttpsConnector<HttpConnector>, Empty<Bytes>>, ClientError> {
    let mut http = HttpConnector::new();
    http.enforce_http(false);

    let tls = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    // Create an HTTPS connector using the HTTP connector and the custom TLS connector
    let https = HttpsConnector::from((http, tls.into()));
    
    // Create the client with the custom service
    let client = Client::builder(TokioExecutor::new())
        .build::<_, Empty<Bytes>>(https);

    Ok(client)
}
use thiserror::Error;
use std::string::FromUtf8Error;
use hyper_util::client::legacy;

#[derive(Error, Debug)]
pub enum FetchTilesError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] hyper::Error),

    #[error("HTTP error: {0}")]
    HttpBuildError(#[from] hyper::http::Error),

    #[error("Invalid URI: {0}")]
    UriError(#[from] hyper::http::uri::InvalidUri),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] hyper::header::InvalidHeaderValue),

    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] FromUtf8Error),

    #[error("Hyper client error: {0}")]
    HyperClientError(#[from] legacy::Error),

    #[error("Unexpected status code: {0}")]
    UnexpectedStatusCode(u16)
}
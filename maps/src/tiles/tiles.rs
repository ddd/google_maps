use hyper_tls::HttpsConnector;
use hyper::{Request, Method};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper_util::{client::legacy::Client, client::legacy::connect::HttpConnector};
use serde_json::Value;


pub async fn view_tiles(client: &Client<HttpsConnector<HttpConnector>, Empty<Bytes>>, tiles: &Vec<crate::tiles::types::Tile>) -> Result<Vec<String>, crate::tiles::error::FetchTilesError> {
    
    let pb = crate::tiles::format::format_tiles(tiles);

    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("https://maps.googleapis.com/maps/vt?pb={}", pb))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 6.1; rv:31.0) Gecko/20100101 Firefox/31.0")
        .body(Empty::new())?;

    let resp = client.request(req).await?;

    let status = resp.status();
    if status != hyper::StatusCode::OK {
        return Err(crate::tiles::error::FetchTilesError::UnexpectedStatusCode(status.as_u16()));
    }

    let body_bytes = resp.into_body().collect().await?.to_bytes();
    let body = String::from_utf8(body_bytes.to_vec())?;

    let parsed: Value = serde_json::from_str(&body)?;
    let mut feature_ids = Vec::new();
    
    if let Some(array) = parsed.as_array() {
        for item in array {
            if let Some(features) = item["features"].as_array() {
                for feature in features {
                    if let Some(id) = feature["id"].as_str() {
                        // Convert id to a number (assuming it's numeric)
                        if let Ok(id_num) = id.parse::<u64>() {
                            // Convert to base16 (hexadecimal) and store as string
                            let base16_id = format!("{:x}", id_num);
                            feature_ids.push(base16_id);
                        }
                    }
                }
            }
        }
    }
    Ok(feature_ids)
}
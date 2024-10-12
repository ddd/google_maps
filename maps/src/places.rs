use std::str::FromStr;
use thiserror::Error;
use crate::*;
use tonic::Request;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use regex::Regex;
use urlencoding::decode;
use lazy_static::lazy_static;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"/url\?q=([^&]*)&").unwrap();
}

pub struct GetPlaceRequest<'a> {
    pub client: &'a mut MapsJsInternalServiceClient<Channel>,
    pub location_id: String,
}

#[derive(Debug, Error)]
pub enum GetPlaceError {
    #[error("Request error: {0}")]
    RequestError(#[from] RequestError),
}

use mapsjs::GetEntityDetailsRequest;
use mapsjs::get_entity_details_request::{
    EntityQuery,
    entity_query::Entity,
    LocalizationContext
};

#[derive(Debug)]
pub struct Place {
    pub location_id: String,
    pub title: Option<String>,
    pub local_language_title: Option<String>,
    pub rating: Option<i32>,
    pub phone: Option<String>,
    pub url: Option<String>,
    pub menu_url: Option<String>,
    pub global_code: Option<String>,
    pub compound_code: Option<String>,
    pub altitude: Option<i64>,
    pub longitude: Option<i64>,
    pub latitude: Option<i64>
}

impl<'a> GetPlaceRequest<'a> {
    pub async fn send(self) -> Result<Place, GetPlaceError> {
        let mut request = Request::new(GetEntityDetailsRequest {
            entity_query: Some(EntityQuery{
                entity: Some(Entity{
                    feature_id: self.location_id.clone()
                })
            }),
            localization_context: Some(LocalizationContext{
                language: "en-US".to_string(),
                region: "US".to_string()
            }),
        });

        request.metadata_mut().insert("x-goog-fieldmask", MetadataValue::from_str("entityDetailsResult(title,localLanguageTitle,singleLineAddress,numRatingStars,phoneNumber,authorityPageLink.url,menuLink.url,category),camera.location").map_err(|e| RequestError::InvalidMetadata(e))?);

        let response = self.client.get_entity_details(request).await
            .map_err(|e| match e.code() {
                tonic::Code::ResourceExhausted => RequestError::RateLimited,
                tonic::Code::NotFound => RequestError::NotFound,
                _ => RequestError::TonicStatus(e),
            })?
            .into_inner();

        let entity_details = response.entity_details_result
            .ok_or_else(|| RequestError::Other("Missing entity details".to_string()))?;
        let camera = response.camera
            .ok_or_else(|| RequestError::Other("Missing camera information".to_string()))?;
        let location = camera.location
            .ok_or_else(|| RequestError::Other("Missing location information".to_string()))?;

        let extract_and_decode_url = |url: Option<String>| -> Option<String> {
            url.and_then(|u| {
                URL_REGEX.captures(&u).and_then(|cap| {
                    cap.get(1).and_then(|m| decode(m.as_str()).ok().map(|s| s.into_owned()))
                })
            })
        };

        Ok(Place {
            location_id: self.location_id,
            title: Some(entity_details.title),
            local_language_title: Some(entity_details.local_language_title),
            rating: Some(entity_details.num_rating_stars),
            phone: Some(entity_details.phone_number),
            url: extract_and_decode_url(entity_details.authority_page_link.map(|link| link.url)),
            menu_url: extract_and_decode_url(entity_details.menu_link.map(|link| link.url)),
            global_code: entity_details.plus_code.as_ref()
                .and_then(|code| code.global_code.as_ref())
                .map(|global| global.raw_text.clone()),
            compound_code: entity_details.plus_code.as_ref()
                .and_then(|code| code.compound_code.as_ref())
                .map(|compound| compound.compound_code.clone()),
            altitude: location.altitude.parse().ok(),
            longitude: location.longitude.parse().ok(),
            latitude: location.latitude.parse().ok(),
        })
    }
}
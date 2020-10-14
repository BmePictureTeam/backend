use aide::openapi::v3::macros::api;
use thiserror::Error;
use time::Duration;
use uuid::Uuid;

#[api]
#[serde(rename_all = "camelCase")]
pub struct CreateImageRequest {
    pub title: String,
    pub description: Option<String>,
    pub categories: Vec<String>,
}

#[api]
#[serde(rename_all = "camelCase")]
pub struct CreateImageResponse {
    pub id: Uuid,
}

#[derive(Debug, Error)]
pub enum CreateImageError {
    #[error("there was an unexpected error")]
    Unexpected,
}

#[derive(Debug, Error)]
pub enum UploadImageError {
    #[error("the given identifier is invalid")]
    InvalidId,
    #[error("the image was already uploaded")]
    AlreadyUploaded,
    #[error("the image upload exceeded the {}s timeout, please create a new image", .0.whole_seconds())]
    TimeOut(Duration),
    #[error("expected a file, but got none")]
    ExpectedFile,
    #[error("there was an unexpected error during the upload process")]
    Unexpected,
}

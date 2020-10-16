use aide::openapi::v3::macros::api;
use thiserror::Error;
use time::Duration;
use uuid::Uuid;

#[api]
#[serde(rename_all = "camelCase")]
pub struct CreateImageRequest {
    pub title: String,
    pub description: Option<String>,
    pub categories: Vec<Uuid>,
}

#[api]
#[serde(rename_all = "camelCase")]
pub struct CreateImageResponse {
    pub id: Uuid,
}

#[derive(Debug, Error)]
pub enum CreateImageError {
    #[error("the image must have at least one category")]
    NoCategory,
    #[error(r#"the the image category "{0}" was not found"#)]
    CategoryNotFound(Uuid),
    #[error("there was an unexpected error")]
    Unexpected,
}

#[api]
pub struct SearchImagesQuery {
    pub search: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[api]
pub struct Image {
    pub title: String,
    pub description: Option<String>,
    pub categories: Vec<Uuid>,
}

#[api]
pub struct SearchImagesResponse {
    pub images: Vec<Image>,
}

#[derive(Debug, Error)]
pub enum SearchImagesError {
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

#[api]
pub struct RateImageRequest {
    pub rating: u32,
}

#[derive(Debug, Error)]
pub enum RateImageError {
    #[error("the image was not found")]
    ImageNotFound,
    #[error("own image cannot be rated")]
    OwnImage,
    #[error("the rating must be between 1 and 5")]
    InvalidRating,
    #[error("there was an unexpected error")]
    Unexpected,
}

#[api]
pub struct GetImageRatingResponse {
    pub average: f32,
    pub rating_count: u32,
}

#[derive(Debug, Error)]
pub enum GetImageRatingsError {
    #[error("the image was not found")]
    ImageNotFound,
    #[error("there was an unexpected error")]
    Unexpected,
}

#[api]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub image_count: u32,
}

#[api]
pub struct GetCategoriesResponse {
    pub categories: Vec<Category>,
}

#[derive(Debug, Error)]
pub enum GetCategoriesError {
    #[error("there was an unexpected error")]
    Unexpected,
}

#[api]
pub struct CreateCategoryRequest {
    pub name: String,
}

#[api]
pub struct CreateCategoryResponse {
    pub id: Uuid,
}

#[derive(Debug, Error)]
pub enum CreateCategoryError {
    #[error("only admins are allowed to create categories")]
    NotAllowed,
    #[error("there was an unexpected error")]
    Unexpected,
}

#[api]
pub struct RenameCategoryRequest {
    pub name: String,
}


#[derive(Debug, Error)]
pub enum RenameCategoryError {
    #[error("only admins are allowed to create categories")]
    NotAllowed,
    #[error("there category was not found")]
    CategoryNotFound,
    #[error("there was an unexpected error")]
    Unexpected,
}

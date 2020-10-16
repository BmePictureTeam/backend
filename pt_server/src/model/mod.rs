use aide::openapi::v3::macros::api;

pub mod auth;
pub mod error;
pub mod image;

#[api]
#[derive(Debug)]
pub struct Pagination {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

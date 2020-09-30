use aide::openapi::v3::macros::api;

/// A generic error.
#[api]
pub struct GenericError {
    pub message: String,
}

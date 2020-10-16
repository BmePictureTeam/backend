use aide::openapi::v3::macros::api;

/// A generic error.
#[api]
pub struct GenericError {
    pub message: String,
}

impl Default for GenericError {
    fn default() -> Self {
        Self {
            message: "an unexpected error happened".to_string()
        }
    }
}
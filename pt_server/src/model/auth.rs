use aide::openapi::v3::macros::api;
use thiserror::Error;

#[api]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[api]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token: String,
}

#[api]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[api]
#[serde(rename_all = "camelCase")]
pub struct InvalidRegisterRequest {
    pub error: String,
}

#[api]
#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("the given e-mail address already exists")]
    EmailExists,
    #[error("the given e-mail is invalid")]
    InvalidEmail,
    #[error("invalid password: {0}")]
    InvalidPassword(PasswordError),
    #[error("an unexpected error happened")]
    Unexpected,
}

#[api]
#[derive(Debug, Error)]
pub enum LoginError {
    #[error("user was not found")]
    UserNotFound,
    #[error("incorrect password")]
    IncorrectPassword,
    #[error("unexpected error")]
    Unexpected,
}

#[api]
#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("password must be longer than {0} characters")]
    TooShort(usize),
    #[error("password must not be longer than {0} characters")]
    TooLong(usize),
}

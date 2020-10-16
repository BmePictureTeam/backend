use crate::{model::error::GenericError, services::auth::UserInfo, services::AuthService};
use actix_web::{FromRequest, HttpResponse, ResponseError, http::StatusCode, web};
use futures::Future;
use std::pin::Pin;
use thiserror::Error;

pub struct SessionToken(UserInfo);

impl SessionToken {
    pub fn user_info(&self) -> &UserInfo {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum InvalidTokenError {
    #[error("authorization token is missing")]
    Missing,
    #[error("invalid authorization token: {0}")]
    Jwt(jwt::errors::Error),
}

impl ResponseError for InvalidTokenError {
    fn status_code(&self) -> StatusCode {
        match self {
            InvalidTokenError::Missing => StatusCode::UNAUTHORIZED,
            InvalidTokenError::Jwt(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(GenericError {
            message: self.to_string(),
        })
    }
}

impl FromRequest for SessionToken {
    type Error = InvalidTokenError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let auth_service = (*req
            .app_data::<web::Data<Box<dyn AuthService>>>()
            .expect("auth service was not provided with request"))
        .clone();

        match req.headers().get("authorization") {
            Some(val) => {
                let token = val
                    .to_str()
                    .map(|s| s.trim_start_matches("Bearer ").to_string())
                    .unwrap_or_default();
                    
                Box::pin(async move {
                    auth_service
                        .validate_token(&token)
                        .await
                        .map(|info| SessionToken(info))
                        .map_err(|e| InvalidTokenError::Jwt(e))
                })
            }
            None => Box::pin(futures::future::ready(Err(InvalidTokenError::Missing))),
        }
    }
}

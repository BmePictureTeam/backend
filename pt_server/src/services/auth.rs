use crate::{config::Config, db::app_user::AppUser};
use aide::openapi::v3::macros::api;
use async_trait::async_trait;
use jwt::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use slog::{error, Logger};
use sqlx::PgPool;
use thiserror::Error;
use time::prelude::*;
use uuid::Uuid;

use super::Service;

/// User info used inside the user tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTokenClaims {
    /// Expiration time, Unix timestamp.
    pub exp: usize,
    /// Issuer
    pub iss: String,
    /// Subject    
    pub sub: String,

    /// Info about the user.
    pub user: UserInfo,
}

pub type Token = String;

#[async_trait]
pub trait AuthService: Service {
    async fn login(&self, email: &str, password: &str) -> Result<Token, LoginError>;
    async fn validate_token(&self, token: &str) -> Result<UserInfo, jwt::errors::Error>;
}

#[api]
#[derive(Debug, Error)]
pub enum LoginError {
    #[error("user was not found")]
    UserNotFound,
    #[error("incorrect password")]
    IncorrectPassword,
    #[error("unexpected error")]
    UnexpectedError,
}

#[derive(Debug, Clone)]
pub struct DefaultAuthService {
    pool: PgPool,
    logger: Logger,
    config: Config,
}

impl DefaultAuthService {
    pub fn new(config: &Config, logger: Logger, pool: PgPool) -> Self {
        Self {
            logger,
            pool,
            config: config.clone(),
        }
    }
}

#[async_trait]
impl AuthService for DefaultAuthService {
    async fn login(&self, email: &str, password: &str) -> Result<Token, LoginError> {
        let final_email = email.trim().to_lowercase();

        let user = AppUser::by_email(&final_email, &self.pool)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                LoginError::UnexpectedError
            })?
            .ok_or(LoginError::UserNotFound)?;

        if !argon2::verify_encoded(&user.password_hash, password.trim().as_bytes())
            .map_err(|_| LoginError::IncorrectPassword)?
        {
            return Err(LoginError::IncorrectPassword);
        };

        let claims = UserTokenClaims {
            exp: 24.hours().whole_seconds() as usize,
            iss: "pictureTeam".into(),
            sub: "appUser".into(),
            user: UserInfo {
                id: user.id,
                admin: user.is_admin,
            },
        };

        jwt::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.token_secret.as_bytes()),
        )
        .map_err(|e| {
            error!(&self.logger, "unexpected jwt error";
                "error" => e.to_string()
            );
            LoginError::UnexpectedError
        })
    }

    async fn validate_token(&self, token: &str) -> Result<UserInfo, jwt::errors::Error> {
        jwt::decode::<UserTokenClaims>(
            &token,
            &DecodingKey::from_secret(self.config.token_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|t| t.claims.user)
    }
}

use crate::{
    config::Config, db::app_user::AppUser, model::auth::LoginError, model::auth::RegisterError,
    util::random_string, util::validate_email,
};
use async_trait::async_trait;
use jwt::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use slog::{error, Logger};
use sqlx::PgPool;
use time::{prelude::*, OffsetDateTime};
use uuid::Uuid;

use super::Service;

/// User info used inside the user tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppUserTokenClaims {
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
    async fn register(&self, email: &str, password: &str) -> Result<(), RegisterError>;
    async fn login(&self, email: &str, password: &str) -> Result<Token, LoginError>;
    async fn validate_token(&self, token: &str) -> Result<UserInfo, jwt::errors::Error>;
}
dyn_clone::clone_trait_object!(AuthService);

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
    async fn register(&self, email: &str, password: &str) -> Result<(), RegisterError> {
        let final_email = email.trim().to_lowercase();

        if !validate_email(&final_email) {
            return Err(RegisterError::InvalidEmail);
        }

        let password_hash = argon2::hash_encoded(
            password.trim().as_bytes(),
            random_string(64).as_bytes(),
            &argon2::Config::default(),
        )
        .unwrap();

        let existing_user = AppUser::by_email(&final_email, &self.pool)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                RegisterError::Unexpected
            })?;

        if existing_user.is_some() {
            return Err(RegisterError::EmailExists);
        }

        AppUser::new(&self.pool, &final_email, &password_hash, false)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                RegisterError::Unexpected
            })?;

        Ok(())
    }

    async fn login(&self, email: &str, password: &str) -> Result<Token, LoginError> {
        let final_email = email.trim().to_lowercase();

        let user = AppUser::by_email(&final_email, &self.pool)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                LoginError::Unexpected
            })?
            .ok_or(LoginError::UserNotFound)?;

        if !argon2::verify_encoded(&user.password_hash, password.trim().as_bytes())
            .map_err(|_| LoginError::IncorrectPassword)?
        {
            return Err(LoginError::IncorrectPassword);
        };

        let claims = AppUserTokenClaims {
            exp: (OffsetDateTime::now_utc() + 24.hours()).timestamp() as usize,
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
            LoginError::Unexpected
        })
    }

    async fn validate_token(&self, token: &str) -> Result<UserInfo, jwt::errors::Error> {
        jwt::decode::<AppUserTokenClaims>(
            token,
            &DecodingKey::from_secret(self.config.token_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|t| t.claims.user)
    }
}

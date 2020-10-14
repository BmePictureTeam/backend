use aide::openapi::v3::macros::api;
use async_trait::async_trait;
use slog::{error, Logger};
use sqlx::PgPool;
use thiserror::Error;

use crate::{config::Config, db::app_user::AppUser, util::random_string, util::validate_email};

use super::Service;

#[async_trait]
pub trait AppUserService: Service {
    async fn register(&self, email: &str, password: &str) -> Result<(), RegisterError>;
}

#[derive(Debug, Clone)]
pub struct DefaultAppUserService {
    pool: PgPool,
    logger: Logger,
    #[allow(dead_code)]
    config: Config,
}

impl DefaultAppUserService {
    pub fn new(config: &Config, logger: Logger, pool: PgPool) -> Self {
        Self {
            logger,
            pool,
            config: config.clone(),
        }
    }
}

#[async_trait]
impl AppUserService for DefaultAppUserService {
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
                RegisterError::UnexpectedError
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
                RegisterError::UnexpectedError
            })?;

        Ok(())
    }
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
    UnexpectedError,
}

#[api]
#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("password must be longer than {0} characters")]
    TooShort(usize),
    #[error("password must not be longer than {0} characters")]
    TooLong(usize),
}

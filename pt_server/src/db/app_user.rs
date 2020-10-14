use sqlx::{query_file, query_file_as, Error, PgPool};
use uuid::Uuid;

use super::image::{Image, NewImage};

pub struct AppUser {
    pub id: Uuid,
    pub created: time::PrimitiveDateTime,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
}

impl AppUser {
    pub async fn new(
        pool: &PgPool,
        email: &str,
        password_hash: &str,
        admin: bool,
    ) -> Result<Uuid, anyhow::Error> {
        Ok(
            query_file!("queries/app_user/create.sql", email, password_hash, admin)
                .fetch_one(pool)
                .await
                .map(|v| v.id)?,
        )
    }

    pub async fn by_id(id: Uuid, pool: &PgPool) -> Result<Option<AppUser>, anyhow::Error> {
        let res = query_file_as!(AppUser, "queries/app_user/get_by_id.sql", id)
            .fetch_one(pool)
            .await;

        match res {
            Ok(u) => Ok(Some(u)),
            Err(e) => match e {
                Error::RowNotFound => Ok(None),
                _ => Err(e.into()),
            },
        }
    }

    pub async fn by_email(email: &str, pool: &PgPool) -> Result<Option<AppUser>, anyhow::Error> {
        let res = query_file_as!(AppUser, "queries/app_user/get_by_email.sql", email)
        .fetch_one(pool)
        .await;

    match res {
        Ok(u) => Ok(Some(u)),
        Err(e) => match e {
            Error::RowNotFound => Ok(None),
            _ => Err(e.into()),
        },
    }
    }
}

/// Methods for an instance
impl AppUser {
    pub async fn add_image(&self, image: NewImage, pool: &PgPool) -> Result<Uuid, anyhow::Error> {
        Image::new(self.id, image, pool).await
    }

    pub async fn save(&self, pool: &PgPool) -> Result<(), anyhow::Error> {
        query_file!(
            "queries/app_user/update.sql",
            &self.id,
            &self.email,
            &self.password_hash,
            &self.is_admin
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

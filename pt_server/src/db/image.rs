use sqlx::{query_file, query_file_as, Error, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

/// New image without ID
pub struct NewImage {
    pub title: String,
    pub description: Option<String>,
}

pub struct Image {
    pub id: Uuid,
    pub created: OffsetDateTime,
    pub upload_date: Option<OffsetDateTime>,
    pub title: String,
    pub description: Option<String>,
}

/// An image with an owner ID.
pub struct OwnedImage {
    pub image: Image,
    /// An [AppUser] that owns the image.
    pub owner_id: Uuid,
}

impl Image {
    pub async fn by_id(id: Uuid, pool: &PgPool) -> Result<Option<OwnedImage>, sqlx::Error> {
        let res = query_file!("queries/image/get_by_id.sql", id)
            .fetch_one(pool)
            .await;

        match res {
            Ok(image) => Ok(Some(OwnedImage {
                image: Image {
                    id: image.id,
                    created: image.created,
                    upload_date: image.upload_date,
                    title: image.title,
                    description: image.description,
                },
                owner_id: image.app_user_id,
            })),
            Err(e) => match e {
                Error::RowNotFound => Ok(None),
                _ => Err(e.into()),
            },
        }
    }

    pub async fn by_app_user_id(
        app_user_id: Uuid,
        pool: &PgPool,
    ) -> Result<Vec<Image>, sqlx::Error> {
        Ok(
            query_file_as!(Image, "queries/image/get_by_app_user_id.sql", app_user_id)
                .fetch_all(pool)
                .await?,
        )
    }

    pub async fn new(
        app_user_id: Uuid,
        image: NewImage,
        pool: &PgPool,
    ) -> Result<Uuid, sqlx::Error> {
        Ok(query_file!(
            "queries/image/create.sql",
            app_user_id,
            image.title,
            image.description
        )
        .fetch_one(pool)
        .await
        .map(|v| v.id)?)
    }
}

impl Image {
    pub async fn save(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        query_file!(
            "queries/image/update.sql",
            self.id,
            self.upload_date,
            self.title,
            self.description
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

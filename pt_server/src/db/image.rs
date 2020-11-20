use sqlx::{query_file, query_file_as, Error, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{category::Category, rating::Rating};

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
    pub app_user_id: Uuid,
}

impl Image {
    pub async fn by_id(id: Uuid, pool: &PgPool) -> Result<Option<Image>, sqlx::Error> {
        let res = query_file_as!(Image, "queries/image/by_id.sql", id)
            .fetch_one(pool)
            .await;

        match res {
            Ok(image) => Ok(Some(image.into())),
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
            query_file_as!(Image, "queries/image/by_app_user_id.sql", app_user_id)
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

    pub async fn search(
        s: Option<&str>,
        offset: Option<i64>,
        limit: Option<i64>,
        pool: &PgPool,
    ) -> Result<Vec<Image>, sqlx::Error> {
        match s {
            Some(s) => query_file_as!(
                Image,
                "queries/image/search.sql",
                s,
                offset.unwrap_or(0),
                limit.unwrap_or(10)
            )
            .fetch_all(pool)
            .await
            .map(|images| images.into_iter().map(Into::into).collect()),
            None => query_file_as!(
                Image,
                "queries/image/search_no_str.sql",
                offset.unwrap_or(0),
                limit.unwrap_or(10)
            )
            .fetch_all(pool)
            .await
            .map(|images| images.into_iter().map(Into::into).collect()),
        }
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

    pub async fn rate(&self, user_id: Uuid, rating: i32, pool: &PgPool) -> Result<(), sqlx::Error> {
        Rating::new(user_id, self.id, rating).save(pool).await
    }

    pub async fn ratings(&self, pool: &PgPool) -> Result<Vec<Rating>, sqlx::Error> {
        Rating::by_image(self.id, pool).await
    }

    pub async fn categories(&self, pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
        Category::by_image_id(self.id, pool).await
    }
}

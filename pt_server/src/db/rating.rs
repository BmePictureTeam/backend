use sqlx::{query_file, query_file_as, PgPool};
use uuid::Uuid;

pub struct Rating {
    pub app_user_id: Uuid,
    pub image_id: Uuid,
    pub rating: i32,
}

impl Rating {
    pub fn new(app_user_id: Uuid, image_id: Uuid, rating: i32) -> Self {
        Self {
            app_user_id,
            image_id,
            rating,
        }
    }

    pub async fn by_image(image_id: Uuid, pool: &PgPool) -> Result<Vec<Rating>, sqlx::Error> {
        query_file_as!(Rating, "queries/rating/all_by_image.sql", image_id)
            .fetch_all(pool)
            .await
    }
}

impl Rating {
    pub async fn save(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        query_file!(
            "queries/rating/upsert.sql",
            self.app_user_id,
            self.image_id,
            self.rating
        )
        .execute(pool)
        .await
        .map(|_| ())
    }
}

pub struct UserRating {
    pub email: String,
    pub average_rating: Option<f64>,
}

impl UserRating {
    pub async fn all(pool: &PgPool) -> Result<Vec<UserRating>, sqlx::Error> {
        query_file_as!(UserRating, "queries/rating/all_by_users.sql")
            .fetch_all(pool)
            .await
    }
}

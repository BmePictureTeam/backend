use sqlx::{query_file, query_file_as, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

pub struct Category {
    pub id: Uuid,
    pub created: OffsetDateTime,
    pub category_name: String,
}

impl Category {
    pub async fn by_id(id: Uuid, pool: &PgPool) -> Result<Option<Category>, sqlx::Error> {
        let res = query_file_as!(Category, "queries/category/by_id.sql", id)
            .fetch_one(pool)
            .await;

        match res {
            Ok(c) => Ok(Some(c)),
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(e),
            },
        }
    }

    pub async fn by_image_id(id: Uuid, pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
        query_file_as!(Category, "queries/category/by_image_id.sql", id)
            .fetch_all(pool)
            .await
    }

    pub async fn new(name: &str, pool: &PgPool) -> Result<Uuid, sqlx::Error> {
        query_file!("queries/category/create.sql", name)
            .fetch_one(pool)
            .await
            .map(|res| res.id)
    }

    pub async fn all(pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
        query_file_as!(Category, "queries/category/all.sql")
            .fetch_all(pool)
            .await
    }

    pub async fn delete(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        query_file!("queries/category/remove_images.sql", self.id)
            .execute(&mut tx)
            .await?;

        query_file!("queries/category/delete.sql", self.id)
            .execute(&mut tx)
            .await?;

        tx.commit().await
    }
}

impl Category {
    pub async fn image_count(&self, pool: &PgPool) -> Result<i64, sqlx::Error> {
        query_file!("queries/category/image_count.sql", &self.id)
            .fetch_one(pool)
            .await
            .map(|res| res.count.unwrap_or(0))
    }

    pub async fn add_image(&self, image_id: Uuid, pool: &PgPool) -> Result<(), sqlx::Error> {
        query_file!("queries/category/add_image.sql", &self.id, image_id)
            .execute(pool)
            .await
            .map(|_| ())
    }

    pub async fn save(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        query_file!("queries/category/update.sql", &self.id, &self.category_name)
            .execute(pool)
            .await
            .map(|_| ())
    }
}

pub struct CategoryExt {
    pub category: Category,
    pub image_count: i64,
}

impl CategoryExt {
    pub async fn all(pool: &PgPool) -> Result<Vec<CategoryExt>, sqlx::Error> {
        let categories: Vec<Category> = query_file_as!(Category, "queries/category/all.sql")
            .fetch_all(pool)
            .await?;

        let mut categories_ext = Vec::with_capacity(categories.len());
        for category in categories {
            categories_ext.push(CategoryExt {
                image_count: category.image_count(pool).await?,
                category,
            });
        }

        Ok(categories_ext)
    }
}

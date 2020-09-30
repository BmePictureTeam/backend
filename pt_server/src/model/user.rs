use sqlx::{query_as, Error, PgPool};
use uuid::Uuid;

pub struct User {
    pub id: Uuid,
    pub name: String,
}

impl User {
    pub async fn by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, anyhow::Error> {
        let res = query_as!(
            User,
            "
            SELECT * FROM users
            WHERE users.id = $1
            ",
            id
        )
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

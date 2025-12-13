use async_trait::async_trait;
use sqlx::{PgPool, Pool};

use crate::domain::user::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: User) -> Result<User, sqlx::Error>;
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<(), sqlx::Error>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn save(&self, user: User) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as::<_, (i32,)>(
            r#"
            INSERT INTO users (name, email)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(&user.name)
        .bind(&user.email)
        .fetch_one(&self.pool)
        .await?;

        Ok(user.with_id(row.0))
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
        Ok(sqlx::query_as::<_, User>(
            r#"
        SELECT id, name, email
        FROM users
        WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
        Ok(sqlx::query_as::<_, User>(
            r#"
        SELECT id, name, email
        FROM users
        ORDER BY id
        "#,
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn delete(&self, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

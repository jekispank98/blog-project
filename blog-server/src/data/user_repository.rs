use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create_user(
    pool: &PgPool,
    id: Uuid,
    email: &str,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash)
        VALUES ($1, $2, $3)
        "#,
    )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, email, password_hash, created_at
        FROM users
        WHERE email = $1
        "#,
    )
        .bind(email)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| UserRow {
        id: r.get("id"),
        email: r.get("email"),
        password_hash: r.get("password_hash"),
        created_at: r.get("created_at"),
    }))
}
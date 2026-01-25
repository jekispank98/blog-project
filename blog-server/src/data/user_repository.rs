use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create_user(
    pool: &PgPool,
    id: Uuid,
    username: &str,
    email: &str,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash)
        VALUES ($1, $2, $3, $4)
        "#,
    )
        .bind(id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn find_by_email_or_username(pool: &PgPool, identifier: &str) -> Result<Option<UserRow>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, email, username, password_hash, created_at
        FROM users
        WHERE email = $1 OR username = $1
        "#,
    )
        .bind(identifier) // Привязываем один и тот же аргумент к обоим условиям
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| UserRow {
        id: r.get("id"),
        email: r.get("email"),
        username: r.get("username"),
        password_hash: r.get("password_hash"),
        created_at: r.get("created_at"),
    }))
}
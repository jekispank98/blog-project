use sqlx::{PgPool, Row};
use crate::domain::error::ParserError;
use crate::domain::post::Post;

#[derive(Debug)]
pub struct PostRow {
    id: String,
    title: String,
    content: String,
    author_id: String,
    created_at: u64,
    updated_at: u64,
}

impl PostRow {
    pub fn new(id: String, title: String, content: String, author_id: String, created_at: u64, updated_at: u64) -> Self {
        PostRow {
            id,
            title,
            content,
            author_id,
            created_at,
            updated_at,
        }
    }
}
pub async fn create(
    pool: &PgPool,
    id: &str,
    title: &str,
    content: &str,
    author_id: &str,
    created_at: i64,
) -> Result<PostRow, ParserError> {
    sqlx::query(
        r#"
        INSERT INTO posts (id, title, content, author_id, created_at, updated_at)
        VALUES ($1::uuid, $2, $3, $4::uuid, $5, $6)
        "#,
    )
    .bind(id)
    .bind(title)
    .bind(content)
    .bind(author_id)
    .bind(created_at)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(PostRow {
        id: id.to_string(),
        title: title.to_string(),
        content: content.to_string(),
        author_id: author_id.to_string(),
        created_at: created_at as u64,
        updated_at: created_at as u64
    })
}

pub async fn find_by_id(pool: &PgPool, id: &str) -> Result<Option<Post>, ParserError> {
    let row = sqlx::query(
        r#"
        SELECT id, title, content, author_id, created_at, updated_at
        FROM posts
        WHERE id = $1
        "#,
    )
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| {
        Post::new(
            r.get("id"),
            r.get("title"),
            r.get("content"),
            r.get("author_id"),
            r.get::<i64, _>("created_at") as u64,
            r.get::<i64, _>("updated_at") as u64,
        )
    }))
}


pub async fn delete_post(pool: PgPool, id: &str) -> Result<(), ParserError> {
    let result = sqlx::query(
        r#"
        DELETE FROM posts
        WHERE id = $1
        "#,
    )
        .bind(id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ParserError::PostNotFound);
    }

    Ok(())
}

pub async fn update(
    pool: &PgPool,
    id: &str,
    title: &str,
    content: &str,
    updated_at: i64,
) -> Result<(), ParserError> {
    let result = sqlx::query(
        r#"
        UPDATE posts
        SET title = $1, content = $2, updated_at = $3
        WHERE id = $4
        "#,
    )
    .bind(title)
    .bind(content)
    .bind(updated_at)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ParserError::PostNotFound);
    }

    Ok(())
}


pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Post>, ParserError> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, content, author_id, created_at, updated_at
        FROM posts
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|r| {
        Post::new(
            r.get("id"),
            r.get("title"),
            r.get("content"),
            r.get("author_id"),
            r.get::<i64, _>("created_at") as u64,
            r.get::<i64, _>("updated_at") as u64,
        )
    }).collect())
}

pub async fn count(pool: &PgPool) -> Result<i64, ParserError> {
    let row = sqlx::query("SELECT COUNT(*) FROM posts")
        .fetch_one(pool)
        .await?;
    
    Ok(row.get(0))
}

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
        VALUES ($1, $2, $3, $4, $5, $6)
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

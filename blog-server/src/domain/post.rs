use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub created_at: u64,
    pub updated_at: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePost {
    title: String,
    content: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePost {
    title: String,
    content: String,
}

impl Post {
    pub fn new(id: String, title: String, content: String, author_id: String, created_at: u64, updated_at: u64) -> Self {
        Self { id, title, content, author_id, created_at, updated_at }
    }
}

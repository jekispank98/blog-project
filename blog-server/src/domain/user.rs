use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    id: String,
    username: String,
    email: String,
    password_hash: i32,
    created_at: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegisterRequest {
    username: String,
    email: String,
    password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginRequest {
    username: String,
    password: String
}
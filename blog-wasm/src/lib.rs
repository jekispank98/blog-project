#[warn(missing_docs)]
use wasm_bindgen::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::window;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostListResponse {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[wasm_bindgen]
pub struct BlogApp {
    base_url: String,
    token: Option<String>,
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: String) -> BlogApp {
        let token = Self::get_token_from_storage();
        BlogApp { base_url, token }
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    pub async fn register(&mut self, username: String, email: String, password: String) -> Result<JsValue, JsValue> {
        let url = format!("{}/api/register", self.base_url);
        let body = serde_json::json!({
            "username": username,
            "email": email,
            "password": password
        });

        let response = Request::post(&url)
            .json(&body)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(JsValue::from_str(&error_text));
        }

        let auth_res: AuthResponse = response.json().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.set_token(Some(auth_res.token.clone()));
        
        serde_wasm_bindgen::to_value(&auth_res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn login(&mut self, email: String, password: String) -> Result<JsValue, JsValue> {
        let url = format!("{}/api/login", self.base_url);
        let body = serde_json::json!({
            "email": email,
            "password": password
        });

        let response = Request::post(&url)
            .json(&body)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(JsValue::from_str(&error_text));
        }

        let auth_res: AuthResponse = response.json().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.set_token(Some(auth_res.token.clone()));

        serde_wasm_bindgen::to_value(&auth_res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn logout(&mut self) {
        self.set_token(None);
    }

    pub async fn load_posts(&self, limit: i64, offset: i64) -> Result<JsValue, JsValue> {
        let url = format!("{}/api/posts?limit={}&offset={}", self.base_url, limit, offset);
        
        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str("Failed to load posts"));
        }

        let posts: PostListResponse = response.json().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        serde_wasm_bindgen::to_value(&posts).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn create_post(&self, title: String, content: String) -> Result<JsValue, JsValue> {
        let url = format!("{}/api/posts", self.base_url);
        let token = self.token.as_ref().ok_or_else(|| JsValue::from_str("Unauthorized"))?;

        let body = serde_json::json!({
            "title": title,
            "content": content
        });

        let response = Request::post(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .json(&body)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(JsValue::from_str(&error_text));
        }

        let post: Post = response.json().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        serde_wasm_bindgen::to_value(&post).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn delete_post(&self, id: String) -> Result<JsValue, JsValue> {
        let url = format!("{}/api/posts/{}", self.base_url, id);
        let token = self.token.as_ref().ok_or_else(|| JsValue::from_str("Unauthorized"))?;

        let response = Request::delete(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(JsValue::from_str(&error_text));
        }

        Ok(JsValue::TRUE)
    }

    // Внутренние методы
    fn set_token(&mut self, token: Option<String>) {
        self.token = token.clone();
        if let Some(t) = token {
            Self::save_token_to_storage(&t);
        } else {
            Self::remove_token_from_storage();
        }
    }

    fn save_token_to_storage(token: &str) {
        if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item("blog_token", token);
        }
    }

    fn get_token_from_storage() -> Option<String> {
        window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("blog_token").ok().flatten())
    }

    fn remove_token_from_storage() {
        if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.remove_item("blog_token");
        }
    }
}

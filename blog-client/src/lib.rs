pub mod error;
pub mod http_client;
pub mod grpc_client;

use error::BlogClientError;
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;
use crate::blog::blog_service_client::BlogServiceClient;
use tonic::Request;

pub mod blog {
    tonic::include_proto!("blog");
}

#[derive(Debug, Clone)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostListResponse {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

pub struct BlogClient {
    transport: Transport,
    http_client: Option<reqwest::Client>,
    grpc_client: Option<BlogServiceClient<Channel>>,
    token: Option<String>,
}

impl BlogClient {
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        let mut http_client = None;
        let mut grpc_client = None;

        match &transport {
            Transport::Http(_) => {
                http_client = Some(reqwest::Client::new());
            }
            Transport::Grpc(addr) => {
                let channel = Channel::from_shared(addr.clone())?
                    .connect()
                    .await?;
                grpc_client = Some(BlogServiceClient::new(channel));
            }
        }

        Ok(Self {
            transport,
            http_client,
            grpc_client,
            token: None,
        })
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    // Auth methods
    pub async fn register(&mut self, username: String, email: String, password: String) -> Result<AuthResponse, BlogClientError> {
        match &self.transport {
            Transport::Http(base_url) => {
                let url = format!("{}/api/register", base_url);
                let body = serde_json::json!({
                    "username": username,
                    "email": email,
                    "password": password
                });
                let resp = self.http_client.as_ref().expect("HTTP client not initialized")
                    .post(&url)
                    .json(&body)
                    .send()
                    .await?;

                if resp.status().is_success() {
                    let auth_resp: AuthResponse = resp.json().await?;
                    self.set_token(auth_resp.token.clone());
                    Ok(auth_resp)
                } else if resp.status() == reqwest::StatusCode::CONFLICT {
                    Err(BlogClientError::InvalidRequest("User already exists".into()))
                } else {
                    Err(BlogClientError::InvalidRequest(format!("HTTP error: {}", resp.status())))
                }
            }
            Transport::Grpc(_) => {
                Err(BlogClientError::InvalidRequest("Register not supported via gRPC yet".into()))
            }
        }
    }

    pub async fn login(&mut self, email: String, password: String) -> Result<AuthResponse, BlogClientError> {
        match &self.transport {
            Transport::Http(base_url) => {
                let url = format!("{}/api/login", base_url);
                let body = serde_json::json!({
                    "email": email,
                    "password": password
                });
                let resp = self.http_client.as_ref().expect("HTTP client not initialized")
                    .post(&url)
                    .json(&body)
                    .send()
                    .await?;

                if resp.status().is_success() {
                    let auth_resp: AuthResponse = resp.json().await?;
                    self.set_token(auth_resp.token.clone());
                    Ok(auth_resp)
                } else if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
                    Err(BlogClientError::Unauthorized)
                } else {
                    Err(BlogClientError::InvalidRequest(format!("HTTP error: {}", resp.status())))
                }
            }
            Transport::Grpc(_) => {
                Err(BlogClientError::InvalidRequest("Login not supported via gRPC yet".into()))
            }
        }
    }

    // Post methods
    pub async fn create_post(&self, title: String, content: String) -> Result<Post, BlogClientError> {
        match &self.transport {
            Transport::Http(base_url) => {
                let url = format!("{}/api/posts", base_url);
                let body = serde_json::json!({
                    "title": title,
                    "content": content
                });
                let mut req = self.http_client.as_ref().expect("HTTP client not initialized")
                    .post(&url)
                    .json(&body);
                
                if let Some(token) = &self.token {
                    req = req.bearer_auth(token);
                }

                let resp = req.send().await?;
                if resp.status().is_success() {
                    Ok(resp.json().await?)
                } else if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
                    Err(BlogClientError::Unauthorized)
                } else {
                    Err(BlogClientError::InvalidRequest(format!("HTTP error: {}", resp.status())))
                }
            }
            Transport::Grpc(_) => {
                let mut client = self.grpc_client.as_ref().expect("gRPC client not initialized").clone();
                let mut req = Request::new(blog::CreatePostRequest {
                    title,
                    content,
                    user_id: "".to_string(),
                });

                if let Some(token) = &self.token {
                    let auth_header = format!("Bearer {}", token);
                    req.metadata_mut().insert("authorization", auth_header.parse().map_err(|_| BlogClientError::Other("Invalid token format".into()))?);
                }

                let resp = client.create_post(req).await?.into_inner();
                let post = resp.post.ok_or(BlogClientError::Other("Empty response".into()))?;
                
                Ok(Post {
                    id: post.post_id,
                    title: post.title,
                    content: post.content,
                    author_id: post.user_id,
                    created_at: post.created_at.parse().unwrap_or(0),
                    updated_at: post.created_at.parse().unwrap_or(0),
                })
            }
        }
    }

    pub async fn get_post(&self, id: String) -> Result<Post, BlogClientError> {
        match &self.transport {
            Transport::Http(base_url) => {
                let url = format!("{}/api/posts/{}", base_url, id);
                let resp = self.http_client.as_ref().expect("HTTP client not initialized")
                    .get(&url)
                    .send()
                    .await?;

                if resp.status().is_success() {
                    Ok(resp.json().await?)
                } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                    Err(BlogClientError::NotFound)
                } else {
                    Err(BlogClientError::InvalidRequest(format!("HTTP error: {}", resp.status())))
                }
            }
            Transport::Grpc(_) => {
                let mut client = self.grpc_client.as_ref().expect("gRPC client not initialized").clone();
                let req = Request::new(blog::GetPostRequest {
                    post_id: id,
                });

                let resp = client.get_post(req).await?.into_inner();
                let post = resp.post.ok_or(BlogClientError::NotFound)?;

                Ok(Post {
                    id: post.post_id,
                    title: post.title,
                    content: post.content,
                    author_id: post.user_id,
                    created_at: post.created_at.parse().unwrap_or(0),
                    updated_at: post.created_at.parse().unwrap_or(0),
                })
            }
        }
    }

    pub async fn update_post(&self, id: String, title: String, content: String) -> Result<Post, BlogClientError> {
        match &self.transport {
            Transport::Http(base_url) => {
                let url = format!("{}/api/posts/{}", base_url, id);
                let body = serde_json::json!({
                    "title": title,
                    "content": content
                });
                let mut req = self.http_client.as_ref().expect("HTTP client not initialized")
                    .put(&url)
                    .json(&body);
                
                if let Some(token) = &self.token {
                    req = req.bearer_auth(token);
                }

                let resp = req.send().await?;
                if resp.status().is_success() {
                    Ok(resp.json().await?)
                } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                    Err(BlogClientError::NotFound)
                } else if resp.status() == reqwest::StatusCode::FORBIDDEN {
                    Err(BlogClientError::InvalidRequest("Forbidden".into()))
                } else {
                    Err(BlogClientError::InvalidRequest(format!("HTTP error: {}", resp.status())))
                }
            }
            Transport::Grpc(_) => {
                let mut client = self.grpc_client.as_ref().expect("gRPC client not initialized").clone();
                let mut req = Request::new(blog::UpdatePostRequest {
                    post_id: id,
                    title,
                    content,
                });

                if let Some(token) = &self.token {
                    let auth_header = format!("Bearer {}", token);
                    req.metadata_mut().insert("authorization", auth_header.parse().map_err(|_| BlogClientError::Other("Invalid token format".into()))?);
                }

                let resp = client.update_post(req).await?.into_inner();
                let post = resp.post.ok_or(BlogClientError::NotFound)?;

                Ok(Post {
                    id: post.post_id,
                    title: post.title,
                    content: post.content,
                    author_id: post.user_id,
                    created_at: post.created_at.parse().unwrap_or(0),
                    updated_at: post.created_at.parse().unwrap_or(0),
                })
            }
        }
    }

    pub async fn delete_post(&self, id: String) -> Result<(), BlogClientError> {
        match &self.transport {
            Transport::Http(base_url) => {
                let url = format!("{}/api/posts/{}", base_url, id);
                let mut req = self.http_client.as_ref().expect("HTTP client not initialized")
                    .delete(&url);
                
                if let Some(token) = &self.token {
                    req = req.bearer_auth(token);
                }

                let resp = req.send().await?;
                if resp.status().is_success() {
                    Ok(())
                } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                    Err(BlogClientError::NotFound)
                } else {
                    Err(BlogClientError::InvalidRequest(format!("HTTP error: {}", resp.status())))
                }
            }
            Transport::Grpc(_) => {
                let mut client = self.grpc_client.as_ref().expect("gRPC client not initialized").clone();
                let mut req = Request::new(blog::DeletePostRequest {
                    post_id: id,
                });

                if let Some(token) = &self.token {
                    let auth_header = format!("Bearer {}", token);
                    req.metadata_mut().insert("authorization", auth_header.parse().map_err(|_| BlogClientError::Other("Invalid token format".into()))?);
                }

                client.delete_post(req).await?;
                Ok(())
            }
        }
    }

    pub async fn list_posts(&self, limit: i64, offset: i64) -> Result<PostListResponse, BlogClientError> {
        match &self.transport {
            Transport::Http(base_url) => {
                let url = format!("{}/api/posts?limit={}&offset={}", base_url, limit, offset);
                let resp = self.http_client.as_ref().expect("HTTP client not initialized")
                    .get(&url)
                    .send()
                    .await?;

                if resp.status().is_success() {
                    Ok(resp.json().await?)
                } else {
                    Err(BlogClientError::InvalidRequest(format!("HTTP error: {}", resp.status())))
                }
            }
            Transport::Grpc(_) => {
                let mut client = self.grpc_client.as_ref().expect("gRPC client not initialized").clone();
                let page = (offset / limit) as i32 + 1;
                let req = Request::new(blog::ListPostsRequest {
                    page,
                    page_size: limit as i32,
                });

                let resp = client.list_posts(req).await?.into_inner();
                let posts = resp.posts.into_iter().map(|post| Post {
                    id: post.post_id,
                    title: post.title,
                    content: post.content,
                    author_id: post.user_id,
                    created_at: post.created_at.parse().unwrap_or(0),
                    updated_at: post.created_at.parse().unwrap_or(0),
                }).collect();

                Ok(PostListResponse {
                    posts,
                    total: resp.total_count as i64,
                    limit,
                    offset,
                })
            }
        }
    }
}

use std::future::{ready, Ready};
use crate::data::{post_repository, user_repository};
use crate::domain::error::ParserError;
use actix_web::{post, web, FromRequest, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use actix_web::dev::Payload;
use actix_web::error::ErrorUnauthorized;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use log::{error, info, warn};
use crate::infrastructure::jwt::Jwt;
use crate::domain::post::Post as DomainPost;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RegisterDto {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LoginDto {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserDto,
}

#[derive(Serialize)]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct CreatePostDto {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
}

fn default_limit() -> i64 { 10 }
fn default_offset() -> i64 { 0 }

#[derive(Debug, Serialize)]
pub struct PostListResponse {
    pub posts: Vec<DomainPost>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[post("/register")]
pub async fn register(
    body: web::Json<RegisterDto>,
    auth_service: web::Data<Arc<AuthService>>
) -> impl Responder {
    match auth_service.register(&body.email, &body.username, &body.password).await {
        Ok(auth_data) => {
            HttpResponse::Created().json(auth_data)
        }
        Err(err) => match err {
            ParserError::UserAlreadyExists => {
                HttpResponse::Conflict().body("User with this email already exists")
            }
            ParserError::DatabaseError(e) => {
                log::error!("Database error during registration: {}", e);
                HttpResponse::InternalServerError().finish()
            }
            ParserError::InternalError(e) => {
                log::error!("Internal error: {}", e);
                HttpResponse::InternalServerError().finish()
            }
            _ => HttpResponse::BadRequest().finish(),
        },
    }
}

#[post("/login")]
pub async fn login(
    body: web::Json<LoginDto>,
    auth_service: web::Data<Arc<AuthService>>
) -> impl Responder {
    match auth_service.login(&body.username, &body.password).await {
        Ok(auth_data) => {
            HttpResponse::Ok().json(auth_data)
        }
        Err(err) => match err {
            ParserError::InvalidCredentials => {
                HttpResponse::Unauthorized().json("Invalid username or password")
            }
            _ => HttpResponse::InternalServerError().finish(),
        },
    }
}
pub struct AuthService {
    pool: PgPool,
    jwt_service: Arc<Jwt>
}
impl AuthService {
    pub fn new(pool: PgPool, jwt_service: Arc<Jwt>) -> Self {
        Self { pool, jwt_service }
    }
    pub async fn register(
        &self,
        email: &str,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, ParserError> {
        let existing_user = user_repository::find_by_email(&self.pool, email)
            .await
            .map_err(|e| ParserError::DatabaseError(e))?;

        if existing_user.is_some() {
            return Err(ParserError::UserAlreadyExists);
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| ParserError::InternalError(format!("Hashing error: {}", e)))?
            .to_string();

        let user_id = uuid::Uuid::new_v4();

        user_repository::create_user(&self.pool, user_id, email, &password_hash)
            .await
            .map_err(|e| ParserError::DatabaseError(e))?;

        let token = self
            .jwt_service
            .generate_token(user_id.to_string(), username.to_string())
            .map_err(|e| ParserError::InternalError(format!("JWT generation failed: {}", e)))?;

        Ok(AuthResponse {
            token,
            user: UserDto {
                id: user_id.to_string(),
                username: username.to_string(),
                email: email.to_string(),
            },
        })
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, ParserError> {
        info!("Attempting login for user: {}", email);

        // 1. Поиск пользователя в БД
        let user = user_repository::find_by_email(&self.pool, email)
            .await
            .map_err(|e| {
                error!("Database error during login for {}: {}", email, e);
                ParserError::DatabaseError(e)
            })?
            .ok_or_else(|| {
                warn!("Login failed: user {} not found", email);
                ParserError::InvalidCredentials
            })?;

        // 2. Проверка пароля (сравнение входящего пароля с хешем из БД)
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| {
                error!("Invalid password hash format in DB for user {}: {}", email, e);
                ParserError::InternalError("Corrupted password hash".to_string())
            })?;

        let is_valid = Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();

        if !is_valid {
            warn!("Login failed: invalid password for user {}", email);
            return Err(ParserError::InvalidCredentials);
        }

        // 3. Генерация токена
        let token = self.jwt_service
            .generate_token(user.id.to_string().clone(), user.email.clone())
            .map_err(|e| {
                error!("Token generation failed for {}: {}", email, e);
                ParserError::InternalError("Failed to generate token".to_string())
            })?;

        info!("User {} successfully logged in", email);

        Ok(AuthResponse {
            token,
            user: UserDto {
                id: user.id.to_string(),
                username: user.email.clone(),
                email: user.email,
            },
        })
    }
}
impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        match req.extensions().get::<AuthenticatedUser>() {
            Some(user) => ready(Ok(user.clone())),
            None => {
                log::error!("Middleware failed to provide AuthenticatedUser extension");
                ready(Err(ErrorUnauthorized("Unauthorized: Missing user context")))
            }
        }
    }
}

pub struct BlogService {
    pool: PgPool,
}

impl BlogService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_post(
        &self,
        title: String,
        content: String,
        author_id: String
    ) -> Result<DomainPost, ParserError> {

        let post_id = uuid::Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        post_repository::create(
            &self.pool,
            &post_id,
            &title,
            &content,
            &author_id,
            now
        ).await?;

        Ok(DomainPost::new(
            post_id,
            title,
            content,
            author_id,
            now as u64,
            now as u64
        ))
    }

    pub async fn get_post(&self, id: String) -> Result<DomainPost, ParserError> {
        let post = post_repository::find_by_id(&self.pool, &id).await?;
        post.ok_or(ParserError::PostNotFound)
    }

    pub async fn update_post(
        &self,
        id: String,
        title: String,
        content: String,
        user_id: String
    ) -> Result<DomainPost, ParserError> {
        let post = post_repository::find_by_id(&self.pool, &id).await?;
        
        match post {
            Some(p) => {
                if p.author_id != user_id {
                    return Err(ParserError::Forbidden);
                }
                
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                
                post_repository::update(&self.pool, &id, &title, &content, now).await?;
                
                Ok(DomainPost::new(
                    id,
                    title,
                    content,
                    user_id,
                    p.created_at,
                    now as u64
                ))
            },
            None => Err(ParserError::PostNotFound)
        }
    }

    pub async fn delete_post(&self, id: String, user_id: String) -> Result<(), ParserError> {
        let post = post_repository::find_by_id(&self.pool, &id).await?;
        
        match post {
            Some(p) => {
                if p.author_id != user_id {
                    return Err(ParserError::Forbidden);
                }
                post_repository::delete_post(self.pool.clone(), &id).await?;
                Ok(())
            },
            None => Err(ParserError::PostNotFound)
        }
    }

    pub async fn list_posts(&self, limit: i64, offset: i64) -> Result<PostListResponse, ParserError> {
        let posts = post_repository::list(&self.pool, limit, offset).await?;
        let total = post_repository::count(&self.pool).await?;

        Ok(PostListResponse {
            posts,
            total,
            limit,
            offset,
        })
    }
}

pub async fn create_post_handler(
    service: web::Data<Arc<BlogService>>,
    user: AuthenticatedUser,
    body: web::Json<CreatePostDto>,
) -> impl Responder {

    match service.create_post(body.title.clone(), body.content.clone(), user.user_id.clone()).await {
        Ok(post) => HttpResponse::Created().json(post),
        Err(e) => {
            log::error!("Failed to create post: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_post_handler(
    service: web::Data<Arc<BlogService>>,
    path: web::Path<String>,
) -> Result<impl Responder, ParserError> {
    let post_id = path.into_inner();

    let post = service.get_post(post_id).await?;
    Ok(HttpResponse::Ok().json(post))
}

pub async fn delete_post_handler(
    service: web::Data<Arc<BlogService>>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<impl Responder, ParserError> {
    let post_id = path.into_inner();
    service.delete_post(post_id, user.user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_posts_handler(
    service: web::Data<Arc<BlogService>>,
    query: web::Query<ListQuery>,
) -> Result<impl Responder, ParserError> {
    let response = service.list_posts(query.limit, query.offset).await?;
    Ok(HttpResponse::Ok().json(response))
}

use crate::presentation::middleware::{jwt_validator, AuthenticatedUser};
use actix_web_httpauth::middleware::HttpAuthentication;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(jwt_validator);

    cfg.service(
        web::scope("/api")
            .service(register)
            .service(login)
            .service(
                web::scope("/posts")
                    .route("", web::get().to(list_posts_handler))
                    .route("/{id}", web::get().to(get_post_handler))
                    .service(
                        web::scope("")
                            .wrap(auth)
                            .route("", web::post().to(create_post_handler))
                            .route("/{id}", web::delete().to(delete_post_handler))
                    )
            )
    );
}

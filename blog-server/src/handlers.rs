use crate::data::user_repository;
use crate::domain::error::ParserError;
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use log::{error, info, warn};
use crate::infrastructure::jwt::Jwt;

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
            .map_err(|e| ParserError::DatabaseError(e.to_string()))?;

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

        let new_user = user_repository::create_user(&self.pool, user_id, email, &password_hash)
            .await
            .map_err(|e| ParserError::DatabaseError(e.to_string()))?;

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
                ParserError::DatabaseError(e.to_string())
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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(register).service(login);
}

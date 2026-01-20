use actix_web::Result;
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, errors::Error as JwtError};
use std::time::{SystemTime, UNIX_EPOCH};
use std::env;

pub struct Claims {
    user_id: String,
    username: String,
    exp: usize
}

#[derive(Clone, Debug)]
pub struct Jwt {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl Jwt {
    pub fn new() -> Jwt {
        let secret = dotenvy::var("JWT_SECRET")
            .expect("JWT_SECRET must be set in environment variables");

        Jwt {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn generate_token(&self, user_id: String, username: String) -> Result<String, jsonwebtoken::errors::Error> {
        // Текущее время в секундах с Unix эпохи
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as usize;

        // 24 часа в секундах = 24 * 60 * 60 = 86400
        let expiration = now + 86400;

        let claims = Claims {
            user_id: user_id.clone(),
            username: username.clone(),
            exp: expiration,
        };

        // Кодируем токен
        encode(&Header::default(), &claims, &self.encoding_key)
    }
    pub fn verify_token(&self, token: &str) -> Result<Claims, JwtError> {
        // Создаем валидацию с настройками по умолчанию
        let validation = Validation::default();

        // Декодируем и проверяем токен
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;

        // Возвращаем claims
        Ok(token_data.claims)
    }
}
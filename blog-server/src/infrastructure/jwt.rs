use actix_web::Result;
use jsonwebtoken::{
    decode, encode, errors::Error as JwtError, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub username: String,
    pub exp: usize,
}

#[derive(Clone, Debug)]
pub struct Jwt {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl Jwt {
    pub fn new() -> Jwt {
        let secret =
            dotenvy::var("JWT_SECRET").expect("JWT_SECRET must be set in environment variables");

        Jwt {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn generate_token(
        &self,
        user_id: String,
        username: String,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as usize;

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
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}

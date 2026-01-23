use actix_web::{dev::ServiceRequest, Error, web, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use crate::infrastructure::jwt::Jwt;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use actix_web::error::ErrorUnauthorized;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub username: String,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    auth: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_service = match req.app_data::<web::Data<Arc<Jwt>>>() {
        Some(jwt) => jwt,
        None => {
            log::error!("Jwt service not found in app_data");
            return Err((ErrorUnauthorized("Internal Server Error"), req));
        }
    };

    match jwt_service.verify_token(auth.token()) {
        Ok(claims) => {
            req.extensions_mut().insert(AuthenticatedUser {
                user_id: claims.user_id,
                username: claims.username,
            });
            Ok(req)
        }
        Err(e) => {
            log::debug!("JWT validation failed: {:?}", e);
            Err((ErrorUnauthorized("Invalid or expired token"), req))
        }
    }
}

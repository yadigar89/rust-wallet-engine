use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{header, Request, StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::json;
use uuid::Uuid;

use crate::models::Claims;

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Token tapılmadı"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Keçərsiz və ya vaxtı bitmiş token"),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AuthError::MissingToken)?;

        if auth_header.starts_with("Bearer ") {
            let token = auth_header.trim_start_matches("Bearer ");
            let claims = verify_token(token).map_err(|_| AuthError::InvalidToken)?;
            Ok(AuthenticatedUser { user_id: claims.sub })
        } else {
            Err(AuthError::MissingToken)
        }
    }
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
   let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub async fn auth_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    if let Some(token) = token {
        if let Ok(claims) = verify_token(token) {
            req.extensions_mut().insert(claims);
            return Ok(next.run(req).await);
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
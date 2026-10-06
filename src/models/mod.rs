use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive (Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub currency: String,
    pub balance: i64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Transaction{
    pub id: Uuid,
    pub from_wallet_id: Uuid,
    pub to_wallet_id: Uuid,
    pub amount: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub currency: String,
}

#[derive(Debug, Deserialize)]
pub struct DepositRequest{
    pub wallet_id: Uuid,
    pub amount: i64,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from_wallet_id: Uuid,
    pub to_wallet_id: Uuid,
    pub amount: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
}
#[derive(Deserialize)]
pub struct WithdrawRequest {
    pub wallet_id: Uuid,
    pub amount: i64,
}    
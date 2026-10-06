use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::models::{
    AuthResponse, CreateWalletRequest, DepositRequest, LoginRequest,
    RegisterUserRequest, TransferRequest, WithdrawRequest,
};
use crate::services::{UserService, WalletService};

pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "success",
        "message": "Payment & Wallet Engine isleyir!"
    }))
}

pub async fn create_wallet(
    auth_user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateWalletRequest>,
) -> impl IntoResponse {
    
    match WalletService::create_wallet(&pool, auth_user.user_id, payload.currency, 0).await {
        Ok(wallet) => (StatusCode::CREATED, Json(wallet)).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn get_wallet(
    State(pool): State<PgPool>,
    Path(wallet_id): Path<Uuid>,
) -> impl IntoResponse {
    match WalletService::get_wallet_by_id(&pool, wallet_id).await {
        Ok(wallet) => (StatusCode::OK, Json(wallet)).into_response(),
        Err(err) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn get_wallet_by_user_id(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    match WalletService::get_wallet_by_user_id(&pool, user_id).await {
        Ok(wallet) => (StatusCode::OK, Json(wallet)).into_response(),
        Err(err) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn transfer_funds(
    State(pool): State<PgPool>,
    Json(payload): Json<TransferRequest>,
) -> impl IntoResponse {
    match WalletService::transfer_funds(
        &pool,
        payload.from_wallet_id,
        payload.to_wallet_id,
        payload.amount,
    )
    .await
    {
        Ok(transaction) => (StatusCode::OK, Json(transaction)).into_response(),
        Err(err_msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": err_msg })),
        )
            .into_response(),
    }
}

pub async fn get_wallet_history(
    State(pool): State<PgPool>,
    Path(wallet_id): Path<Uuid>,
) -> impl IntoResponse {
    match WalletService::get_transactions_by_wallet_id(&pool, wallet_id).await {
        Ok(transactions) => (StatusCode::OK, Json(transactions)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn register(State(pool): State<PgPool>,
Json(payload): Json<RegisterUserRequest>) ->impl IntoResponse {

    match UserService::register_user(&pool, payload.username, 
        payload.email, payload.password).await {
        Ok(user) =>(StatusCode::CREATED, Json(user)).into_response(),
        Err(err_msg) =>(StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error":err_msg}))).into_response(),
    }
}
pub async fn login(State(pool): State<PgPool>,
Json(payload): Json<LoginRequest>) -> impl IntoResponse {

   match UserService::verify_user(&pool, payload.email, payload.password).await {
    Ok(user) => match UserService::generate_token(user.id).await {
        Ok(token) =>(StatusCode::OK, Json(AuthResponse{token, token_type:"Bearer".to_string()})).into_response(),
        Err(err_msg) =>(StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": err_msg}))).into_response(),
    },
    Err(err_msg)=>(StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":err_msg}))).into_response(),
   }
}
pub async fn get_my_wallets(auth_user: AuthenticatedUser,
State(pool): State<PgPool>) ->impl IntoResponse {

    match WalletService::get_user_wallets(&pool, auth_user.user_id).await {

        Ok(wallets) =>(StatusCode::OK, Json(wallets)).into_response(),
        Err(err)=> (StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": err.to_string()}))).into_response(),
    }
}
pub async fn deposit(auth_user: AuthenticatedUser,
State(pool): State<PgPool>,
Json(payload): Json<DepositRequest>) ->impl IntoResponse {

    match WalletService::deposit_funds(&pool, auth_user.user_id, payload.wallet_id, payload.amount).await{
        Ok(wallet) =>(StatusCode::OK, Json(wallet)).into_response(),
        Err(err_msg) =>(StatusCode::BAD_REQUEST, 
        Json(serde_json::json!({"error": err_msg}))).into_response(),
    }
}
pub async fn withdraw(
    State(pool): State<PgPool>,
    Json(payload): Json<WithdrawRequest>,
) -> impl IntoResponse {
    match WalletService::withdraw_funds(&pool, payload.wallet_id, payload.amount).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "message": "Məxaric əməliyyatı uğurla icra olundu." })),
        )
            .into_response(),
        Err(err_msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": err_msg })),
        )
            .into_response(),
    }
}
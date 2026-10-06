mod auth;
mod errors;
mod handlers;
mod models;
mod services;

use axum::{
    http::Method,
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL .env faylinda tapilmadi!");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Verilenler bazasina baglanmaq mumkun olmadi!");

    let cors = CorsLayer::new()
       .allow_origin(Any)
       .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
       .allow_headers(Any);

    let public_routes = Router::new()
       .route("/health", get(handlers::health_check))
       .route("/auth/register", post(handlers::register))
       .route("/auth/login",post(handlers::login));

    let protected_routes = Router::new()
       .route("/wallets/user/{user_id}",get(handlers::get_wallet_by_user_id))
       .route("/wallets", post(handlers::create_wallet).get(handlers::get_my_wallets))
       .route("/wallets/transfer", post(handlers::transfer_funds))
       .route("/wallets/history/{id}", get(handlers::get_wallet_history))
       .route("/wallets/{id}",get(handlers::get_wallet))
       .route("/wallets/deposit", post(handlers::deposit))
       .route("/wallets/withdraw", post(handlers::withdraw))
       .route_layer(middleware::from_fn(auth::auth_middleware));
       

    let app = Router::new()
       .merge(public_routes)
       .merge(protected_routes)
       .layer(cors)
       .with_state(pool);

    

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server http://{} unvani ise dusdu.", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
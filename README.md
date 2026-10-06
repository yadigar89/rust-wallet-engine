# 💸 Production-Ready Rust Wallet & Payment Engine

A high-performance, asynchronous RESTful Payment and Wallet API written in Rust using Axum, SQLx, and PostgreSQL.

## 🛠️ Tech Stack
- **Language:** Rust (2021 Edition)
- **Framework:** [Axum](https://github.com/tokio-rs/axum)
- **Async Runtime:** [Tokio](https://tokio.rs/)
- **Database Access:** [SQLx](https://github.com/launchbadge/sqlx) (PostgreSQL)
- **Data Serialization:** Serde
- **Identifiers:** UUID v4

## 🌟 Key Features
- 💳 **Wallet Management:** Create and query user wallets with multi-currency support.
- ⚡ **Atomic Financial Transactions:** Safe balance transfers with PostgreSQL transactions ensuring ACID compliance and race-condition safety.
- 🛡️ **Robust Error Handling:** Unified structured JSON error responses.

## 🚀 Getting Started

### Prerequisites
- Rust 1.75+
- PostgreSQL server running locally or via Docker

### Environment Setup
Create a `.env` file in the root directory:
```env
DATABASE_URL=postgres://postgres:password@localhost:5432/rust_wallet_db
SERVER_PORT=3000
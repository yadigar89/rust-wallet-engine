use crate::models::{Claims, Transaction, User, Wallet};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use uuid::Uuid;
pub struct WalletService;

impl WalletService {
   pub async fn withdraw_funds(
    pool: &PgPool,
    wallet_id: Uuid,
    amount: i64,
) -> Result<(), String> {
    if amount <= 0 {
        return Err("Çıxarılacaq məbləğ 0-dan böyük olmalıdır!".to_string());
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| e.to_string())?;

    // 1. Cüzdanı tapıb balansı yoxlayırıq
    let wallet = sqlx::query_as::<_, (i64,)>(
        "SELECT balance FROM wallets WHERE id = $1 FOR UPDATE"
    )
    .bind(wallet_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Cüzdan tapılmadı!".to_string())?;

    if wallet.0 < amount {
        return Err("Kifayət qədər balans yoxdur!".to_string());
    }

    // 2. Balansı azaldırıq
    sqlx::query(
        "UPDATE wallets SET balance = balance - $1 WHERE id = $2"
    )
    .bind(amount)
    .bind(wallet_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 3. Əməliyyat tarixçəsini yazırıq
    sqlx::query(
        "INSERT INTO transactions (id, from_wallet_id, to_wallet_id, amount, status, created_at)
         VALUES ($1, $2, NULL, $3, 'COMPLETED', NOW())"
    )
    .bind(Uuid::new_v4())
    .bind(wallet_id)
    .bind(amount)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn create_wallet(
        pool: &PgPool,
        user_id: Uuid,
        currency: String,
        balance: i64,
    ) -> Result<Wallet, sqlx::Error> {
        let wallet = sqlx::query_as::<_, Wallet>(
            r#"
            INSERT INTO wallets (id, user_id, currency, balance, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, currency, balance::BIGINT as balance, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(currency)
        .bind(balance)
        .bind(chrono::Utc::now())
        .fetch_one(pool)
        .await?;

        Ok(wallet)
    }

    pub async fn deposit_funds(pool: &PgPool,user_id: Uuid,
    wallet_id: Uuid,amount: i64) ->Result<Wallet, String> {

        if amount <= 0 {
            return Err("Mebleg 0-dan boyuk olmalidir".to_string());
        }
        let wallet = sqlx::query_as::<_, Wallet>(
            r#"
            UPDATE wallets
            SET balance = balance + $1
            WHERE id = $2 AND user_id = $3
            RETURNING id, user_id, currency, balance::BIGINT as balance, created_at "#,
        )
        .bind(amount)
        .bind(wallet_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        match wallet {
            Some(w) =>Ok(w),
            None => Err("Cuzdan tapilmadi veya cuxdana giris icazesi yoxdur".to_string()),
        }
    }

    pub async fn get_user_wallets(pool: &PgPool, user_id: Uuid) ->Result<Vec<Wallet>,sqlx::Error> {

        let wallets = sqlx::query_as::<_,Wallet>(
            r#"
            SELECT id, user_id, currency, balance::BIGINT as balance, created_at
            FROM wallets WHERE user_id = $1 "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(wallets)
    }

    pub async fn get_wallet_by_id(
        pool: &PgPool,
        wallet_id: Uuid,
    ) -> Result<Wallet, sqlx::Error> {
        let wallet = sqlx::query_as::<_, Wallet>(
            r#"
            SELECT id, user_id, currency, balance::BIGINT as balance, created_at
            FROM wallets
            WHERE id = $1
            "#,
        )
        .bind(wallet_id)
        .fetch_one(pool)
        .await?;

        Ok(wallet)
    }

    pub async fn get_wallet_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Wallet, sqlx::Error> {
        let wallet = sqlx::query_as::<_, Wallet>(
            r#"
            SELECT id, user_id, currency, balance::BIGINT as balance, created_at
            FROM wallets
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(wallet)
    }

   pub async fn transfer_funds(pool: &PgPool, from_wallet_id: Uuid,
to_wallet_id: Uuid, amount: i64) ->Result<Transaction, String>{

    if amount < 0{
        return Err("Transfer meblegi 0-dan boyuk olmalidir!".to_string());
    }
    if from_wallet_id == to_wallet_id {
        return Err("Eyni cuzdana transfer etmek olmaz!".to_string());
    }
    let mut tx = pool.begin().await
           .map_err(|e| format!("Tranzaksiya baslanila bilmedir: {}",e))?;

    let sender_wallet = sqlx::query_as::<_, Wallet>(
        r#"
        SELECT id, user_id, currency, balance::BIGINT as balance, created_at
        FROM wallets WHERE id = $1 "#,
    )    
    .bind(from_wallet_id)
    .fetch_optional(&mut * tx)
    .await
    .map_err(|e| format!("Baza xetasi: {}",e))?;

    let sender = match sender_wallet {
        Some(w) => w,
        None => return Err("Gonderen cuzdan tapilmadi!".to_string()),
    };
    if sender.balance < amount {
        return Err("Cuzdanda kifayet qeder vesait yoxdur (Insufficient Funds)!".to_string());
    }
    sqlx::query("UPDATE wallets SET balance = balance - $1 WHERE id = $2")
        .bind(amount)
        .bind(from_wallet_id)
        .execute(&mut * tx)
        .await
        .map_err(|e| format!("Cixis xetasi: {}",e))?;

    sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
        .bind(amount)
        .bind(to_wallet_id)
        .execute(&mut * tx)
        .await
        .map_err(|e| format!("Medaxil xetasi: {}",e))?;

    let transaction_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let transaction = sqlx::query_as::<_,Transaction>(
        r#"
        INSERT INTO transactions (id, from_wallet_id, to_wallet_id, amount, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, from_wallet_id, to_wallet_id, amount::BIGINT as amount, status, created_at "#,
    )
    .bind(transaction_id)
    .bind(from_wallet_id)
    .bind(to_wallet_id)
    .bind(amount)
    .bind("COMPLETED")
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("Tarixce yazilarken xeta: {}",e))?;

    tx.commit()
       .await
       .map_err(|e| format!("Tranzaksiya tesdiqlene bilmedi: {}", e))?;

    Ok(transaction)
}
pub async fn get_transactions_by_wallet_id(pool: &PgPool, wallet_id: Uuid) ->Result<Vec<Transaction>, sqlx::Error> {

    let transactions = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT id, from_wallet_id, to_wallet_id, amount::BIGINT as amount, status, created_at
        FROM transactions
        WHERE from_wallet_id = $1 OR to_wallet_id = $1
        ORDER BY created_at DESC "#,
    )
    .bind(wallet_id)
    .fetch_all(pool)
    .await?;

    Ok(transactions)
}
}
pub struct UserService;

impl UserService{
    pub async fn register_user(pool: &PgPool, username: String,
    email: String, password: String) ->Result<User, String>{

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
           .hash_password(password.as_bytes(),&salt)
           .map_err(|e|format!("Hash-lanarken xeta bas verdi: {}",e))?
           .to_string();

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at)
            VALUES($1, $2, $3, $4, $5)
            RETURNING id, username, email, password_hash, created_at "#,
        )
        .bind(Uuid::new_v4())
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(chrono::Utc::now())
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Istifadeci qeyde alina bilmedi: {}",e))?;

        Ok(user)

    }

    pub async fn verify_user(pool: &PgPool, email: String, password: String) ->Result<User, String>{

        let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, created_at
        FROM users WHERE email = $1 "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Baza xetasi: {}",e))?
        .ok_or_else(||"Istifadeci tapilmadi!".to_string())?;

        let parsed_hash = PasswordHash::new(&user.password_hash)
           .map_err(|e| format!("Hash oxunmadi: {}",e))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| "Yalnis parol".to_string())?;

        Ok(user)
    }
    pub async fn generate_token(user_id: Uuid) ->Result<String, String> {

        let expiration = chrono::Utc::now()
           .checked_add_signed(chrono::Duration::hours(24))
           .expect("Tarix hesablana bilmedi")
           .timestamp() as usize;

        let claims = Claims {
            sub: user_id,
            exp: expiration,
        };
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e|format!("token yaradila bilmedi: {}",e))
    }
}
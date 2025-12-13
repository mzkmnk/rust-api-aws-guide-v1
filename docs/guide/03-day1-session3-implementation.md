# Day 1: セッション 3 - 実装実演

[← 前へ: セッション 2](./02-day1-session2-architecture.md) | [概要](./00-overview.md)

---

## 3.1 プロジェクトセットアップ

### プロジェクト作成

```bash
cargo new user-api
cd user-api
```

### Cargo.toml の設定

各依存クレートの役割を理解しましょう：

```toml
[package]
name = "user-api"
version = "0.1.0"
edition = "2021"

[dependencies]
# Webフレームワーク - 型安全で高速なAPIを構築
axum = "0.7"

# 非同期ランタイム - async/awaitを動かす基盤
tokio = { version = "1", features = ["full"] }

# データベース - コンパイル時にSQLを検証する型安全なクエリ
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-native-tls", "postgres"] }

# シリアライズ/デシリアライズ - JSON変換に必須
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# エラーハンドリング - derive マクロでエラー型を簡潔に定義
thiserror = "1.0"

# 非同期トレイト - async fn をトレイトで使用可能に
async-trait = "0.1"

# 環境変数 - .envファイルから設定を読み込み
dotenvy = "0.15"

# HTTPミドルウェア - CORS、ロギング等
tower-http = { version = "0.5", features = ["cors", "trace"] }

# ロギング - 構造化ログ出力
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### ディレクトリ構造の作成

セッション2で学んだクリーンアーキテクチャに基づいてディレクトリを作成します：

```bash
mkdir -p src/{api,domain,application,infrastructure}
touch src/{api,domain,application,infrastructure}/mod.rs
touch src/lib.rs
```

最終的な構造：

```
src/
├── main.rs              # エントリーポイント
├── lib.rs               # モジュール公開
├── api/                 # プレゼンテーション層
│   ├── mod.rs
│   ├── handlers.rs      # HTTPハンドラー
│   └── errors.rs        # APIエラー変換
├── domain/              # ドメイン層
│   ├── mod.rs
│   ├── user.rs          # Userエンティティ
│   └── errors.rs        # ドメインエラー
├── application/         # アプリケーション層
│   ├── mod.rs
│   ├── dto.rs           # データ転送オブジェクト
│   └── services.rs      # ビジネスロジック
└── infrastructure/      # インフラストラクチャ層
    ├── mod.rs
    ├── config.rs        # 設定管理
    ├── database.rs      # DB接続
    └── repositories.rs  # データアクセス
```

---

## 3.2 ドメイン層の実装

ドメイン層はビジネスロジックの中心です。外部依存を持たず、純粋なRustコードで構成します。

### src/domain/errors.rs - ドメインエラー

```rust
use thiserror::Error;

/// ドメイン層で発生するエラー
/// ビジネスルール違反を表現する
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("名前は1文字以上100文字以下である必要があります")]
    InvalidName,

    #[error("無効なメールアドレス形式です")]
    InvalidEmail,

    #[error("ユーザーが見つかりません: {0}")]
    UserNotFound(i32),
}
```

**学習ポイント**:
- `thiserror::Error` を derive することで `std::error::Error` トレイトが自動実装される
- `#[error("...")]` でエラーメッセージを定義
- エラーは具体的で、ユーザーに何が問題かを伝える

### src/domain/user.rs - Userエンティティ

```rust
use serde::{Deserialize, Serialize};
use super::errors::DomainError;

/// ユーザーエンティティ
/// ドメインの中心となるデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

impl User {
    /// 新しいユーザーを作成（バリデーション付き）
    ///
    /// # Arguments
    /// * `name` - ユーザー名（1-100文字）
    /// * `email` - メールアドレス（@を含む）
    ///
    /// # Returns
    /// * `Ok(User)` - バリデーション成功
    /// * `Err(DomainError)` - バリデーション失敗
    pub fn new(name: String, email: String) -> Result<Self, DomainError> {
        // 名前のバリデーション
        if name.is_empty() || name.len() > 100 {
            return Err(DomainError::InvalidName);
        }

        // メールのバリデーション（簡易版）
        if !email.contains('@') || email.len() < 3 {
            return Err(DomainError::InvalidEmail);
        }

        Ok(Self {
            id: 0, // DBで自動採番されるため仮の値
            name,
            email,
        })
    }

    /// IDを設定（リポジトリでDB保存後に使用）
    pub fn with_id(mut self, id: i32) -> Self {
        self.id = id;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_user() {
        let user = User::new(
            "田中太郎".to_string(),
            "tanaka@example.com".to_string()
        );
        assert!(user.is_ok());
    }

    #[test]
    fn test_empty_name() {
        let user = User::new(
            "".to_string(),
            "test@example.com".to_string()
        );
        assert!(matches!(user, Err(DomainError::InvalidName)));
    }

    #[test]
    fn test_invalid_email() {
        let user = User::new(
            "テスト".to_string(),
            "invalid-email".to_string()
        );
        assert!(matches!(user, Err(DomainError::InvalidEmail)));
    }
}
```

**学習ポイント**:
- エンティティは自身のバリデーションロジックを持つ
- `new()` でバリデーションを強制し、不正なデータの生成を防ぐ
- テストでビジネスルールを検証

### src/domain/mod.rs

```rust
pub mod errors;
pub mod user;

pub use errors::DomainError;
pub use user::User;
```

---

## 3.3 アプリケーション層の実装

アプリケーション層はユースケースを実装します。ドメイン層とインフラ層を橋渡しします。

### src/application/dto.rs - データ転送オブジェクト

```rust
use serde::{Deserialize, Serialize};

/// ユーザー作成リクエスト
/// APIから受け取るデータ構造
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

/// ユーザーレスポンス
/// APIで返すデータ構造
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
}

impl From<crate::domain::User> for UserResponse {
    fn from(user: crate::domain::User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
        }
    }
}
```

**学習ポイント**:
- DTOはAPIの入出力を定義する
- `Deserialize` はリクエスト用、`Serialize` はレスポンス用
- `From` トレイトで型変換を明示的に定義

### src/application/services.rs - ユーザーサービス

```rust
use std::sync::Arc;
use crate::domain::{User, DomainError};
use crate::infrastructure::repositories::UserRepository;
use super::dto::{CreateUserRequest, UserResponse};

/// アプリケーションエラー
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("ドメインエラー: {0}")]
    Domain(#[from] DomainError),

    #[error("データベースエラー: {0}")]
    Database(#[from] sqlx::Error),

    #[error("ユーザーが見つかりません")]
    NotFound,
}

/// ユーザーサービス
/// ユースケースを実装する
pub struct UserService {
    repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    /// ユーザーを作成する
    ///
    /// # Flow
    /// 1. DTOからドメインエンティティを生成（バリデーション）
    /// 2. リポジトリで永続化
    /// 3. レスポンスDTOに変換して返却
    pub async fn create_user(&self, req: CreateUserRequest) -> Result<UserResponse, AppError> {
        // 1. ドメインエンティティを生成（ここでバリデーション）
        let user = User::new(req.name, req.email)?;

        // 2. リポジトリで保存
        let saved_user = self.repository.save(user).await?;

        // 3. レスポンスに変換
        Ok(UserResponse::from(saved_user))
    }

    /// IDでユーザーを取得する
    pub async fn get_user(&self, id: i32) -> Result<UserResponse, AppError> {
        let user = self.repository
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        Ok(UserResponse::from(user))
    }

    /// 全ユーザーを取得する
    pub async fn list_users(&self) -> Result<Vec<UserResponse>, AppError> {
        let users = self.repository.find_all().await?;
        Ok(users.into_iter().map(UserResponse::from).collect())
    }

    /// ユーザーを削除する
    pub async fn delete_user(&self, id: i32) -> Result<(), AppError> {
        // 存在確認
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.repository.delete(id).await?;
        Ok(())
    }
}
```

**学習ポイント**:
- サービスはリポジトリを `Arc<dyn Trait>` で受け取る（依存性注入）
- `#[from]` でエラーの自動変換を実現
- 各メソッドは1つのユースケースを表現

### src/application/mod.rs

```rust
pub mod dto;
pub mod services;

pub use dto::{CreateUserRequest, UserResponse};
pub use services::{AppError, UserService};
```

---

## 3.4 インフラストラクチャ層の実装

インフラ層は外部システム（DB、外部API等）との接続を担当します。

### src/infrastructure/config.rs - 設定管理

```rust
use std::env;

/// アプリケーション設定
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    /// 環境変数から設定を読み込む
    ///
    /// # Required Environment Variables
    /// - `DATABASE_URL`: PostgreSQL接続文字列
    ///
    /// # Optional Environment Variables
    /// - `SERVER_HOST`: サーバーホスト（デフォルト: 0.0.0.0）
    /// - `SERVER_PORT`: サーバーポート（デフォルト: 3000）
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
        })
    }

    /// サーバーアドレスを取得
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
```

### src/infrastructure/database.rs - DB接続

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// データベース接続プールを作成
///
/// # Arguments
/// * `database_url` - PostgreSQL接続文字列
///   例: "postgresql://user:password@localhost:5432/dbname"
///
/// # Connection Pool
/// - 最大接続数: 5（開発環境向け）
/// - 本番環境では環境に応じて調整
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}
```

### src/infrastructure/repositories.rs - リポジトリ実装

```rust
use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::User;

/// ユーザーリポジトリトレイト
/// データアクセスの抽象化
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: User) -> Result<User, sqlx::Error>;
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<(), sqlx::Error>;
}

/// PostgreSQL用リポジトリ実装
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    /// ユーザーを保存し、採番されたIDを含むUserを返す
    async fn save(&self, user: User) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as::<_, (i32,)>(
            r#"
            INSERT INTO users (name, email)
            VALUES ($1, $2)
            RETURNING id
            "#
        )
        .bind(&user.name)
        .bind(&user.email)
        .fetch_one(&self.pool)
        .await?;

        Ok(user.with_id(row.0))
    }

    /// IDでユーザーを検索
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, name, email
            FROM users
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// 全ユーザーを取得
    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, name, email
            FROM users
            ORDER BY id
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    /// ユーザーを削除
    async fn delete(&self, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
```

**学習ポイント**:
- `#[async_trait]` で非同期メソッドをトレイトで定義可能に
- `Send + Sync` は複数スレッドで安全に共有するために必要
- `query_as` でSQLの結果を直接構造体にマッピング

### src/infrastructure/mod.rs

```rust
pub mod config;
pub mod database;
pub mod repositories;

pub use config::Config;
pub use database::create_pool;
pub use repositories::{PostgresUserRepository, UserRepository};
```

---

## 3.5 API層の実装

API層はHTTPリクエスト/レスポンスを処理します。

### src/api/errors.rs - APIエラー

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use crate::application::AppError;

/// APIエラーレスポンス
/// アプリケーションエラーをHTTPレスポンスに変換
pub struct ApiError(pub AppError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self.0 {
            AppError::Domain(e) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                e.to_string(),
            ),
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "データベースエラーが発生しました".to_string(),
            ),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "リソースが見つかりません".to_string(),
            ),
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}

/// AppError から ApiError への変換
impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        ApiError(err)
    }
}
```

**学習ポイント**:
- `IntoResponse` トレイトでカスタムエラーレスポンスを定義
- 内部エラー詳細は隠蔽し、ユーザー向けメッセージを返す
- エラーコードでクライアント側の処理を容易に

### src/api/handlers.rs - HTTPハンドラー

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::application::{CreateUserRequest, UserResponse, UserService};
use super::errors::ApiError;

/// アプリケーション状態
/// 各ハンドラーで共有するデータ
#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
}

/// ユーザー作成
/// POST /api/users
///
/// # Request Body
/// ```json
/// {
///   "name": "田中太郎",
///   "email": "tanaka@example.com"
/// }
/// ```
///
/// # Response
/// - 201 Created: ユーザー作成成功
/// - 400 Bad Request: バリデーションエラー
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    let user = state.user_service.create_user(payload).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// ユーザー取得
/// GET /api/users/:id
///
/// # Path Parameters
/// - `id`: ユーザーID
///
/// # Response
/// - 200 OK: ユーザー情報
/// - 404 Not Found: ユーザーが存在しない
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state.user_service.get_user(id).await?;
    Ok(Json(user))
}

/// ユーザー一覧取得
/// GET /api/users
///
/// # Response
/// - 200 OK: ユーザー一覧
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, ApiError> {
    let users = state.user_service.list_users().await?;
    Ok(Json(users))
}

/// ユーザー削除
/// DELETE /api/users/:id
///
/// # Response
/// - 204 No Content: 削除成功
/// - 404 Not Found: ユーザーが存在しない
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    state.user_service.delete_user(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// ヘルスチェック
/// GET /health
pub async fn health_check() -> &'static str {
    "OK"
}
```

**学習ポイント**:
- `State<T>` でアプリケーション状態を注入
- `Json<T>` でリクエストボディを自動デシリアライズ
- `Path<T>` でURLパラメータを抽出
- `Result<T, ApiError>` でエラーハンドリング

### src/api/mod.rs

```rust
pub mod errors;
pub mod handlers;

pub use handlers::{
    AppState,
    create_user,
    get_user,
    list_users,
    delete_user,
    health_check,
};
```

---

## 3.6 エントリーポイントの実装

### src/lib.rs - モジュール公開

```rust
pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
```

### src/main.rs - アプリケーション起動

```rust
use axum::{routing::{delete, get, post}, Router};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use user_api::api::{self, AppState};
use user_api::application::UserService;
use user_api::infrastructure::{Config, PostgresUserRepository, create_pool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. ロギング初期化
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // 2. 環境変数読み込み
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    tracing::info!("Starting server...");

    // 3. データベース接続
    let pool = create_pool(&config.database_url).await?;
    tracing::info!("Database connected");

    // 4. 依存性の組み立て（DI）
    let user_repo = Arc::new(PostgresUserRepository::new(pool));
    let user_service = Arc::new(UserService::new(user_repo));
    let state = AppState { user_service };

    // 5. ルーター構築
    let app = Router::new()
        // ヘルスチェック
        .route("/health", get(api::health_check))
        // ユーザーAPI
        .route("/api/users", get(api::list_users).post(api::create_user))
        .route("/api/users/:id", get(api::get_user).delete(api::delete_user))
        // ミドルウェア
        .layer(TraceLayer::new_for_http())
        // 状態注入
        .with_state(state);

    // 6. サーバー起動
    let addr = config.server_addr();
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
```

**学習ポイント**:
- 起動順序: ログ → 設定 → DB → DI → ルーター → サーバー
- `TraceLayer` でHTTPリクエストを自動ログ
- `with_state()` で全ハンドラーに状態を共有

---

## 3.7 データベースセットアップ

### スキーマ作成（schema.sql）

```sql
-- ユーザーテーブル
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- メールアドレス検索用インデックス
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- 作成日時検索用インデックス
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);
```

### .env ファイル

```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/user_api
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=info,user_api=debug
```

### Docker Compose でローカルDB起動

```yaml
# docker-compose.yml
version: '3.8'
services:
  db:
    image: postgres:15
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
      POSTGRES_DB: user_api
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./schema.sql:/docker-entrypoint-initdb.d/schema.sql

volumes:
  postgres_data:
```

```bash
# DB起動
docker compose up -d

# アプリケーション起動
cargo run
```

---

## 3.8 動作確認

### curlでAPIテスト

```bash
# ヘルスチェック
curl http://localhost:3000/health

# ユーザー作成
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "田中太郎", "email": "tanaka@example.com"}'

# ユーザー一覧取得
curl http://localhost:3000/api/users

# 特定ユーザー取得
curl http://localhost:3000/api/users/1

# ユーザー削除
curl -X DELETE http://localhost:3000/api/users/1
```

### 期待されるレスポンス

```json
// POST /api/users - 201 Created
{
  "id": 1,
  "name": "田中太郎",
  "email": "tanaka@example.com"
}

// GET /api/users - 200 OK
[
  {
    "id": 1,
    "name": "田中太郎",
    "email": "tanaka@example.com"
  }
]

// バリデーションエラー - 400 Bad Request
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "無効なメールアドレス形式です"
  }
}
```

---

## セッション 3 チェックリスト

- [ ] プロジェクト構造が作成されている
- [ ] 全モジュールが正しく接続されている
- [ ] `cargo build` が成功する
- [ ] `cargo test` でドメインテストが通る
- [ ] Docker Compose でDBが起動する
- [ ] `cargo run` でサーバーが起動する
- [ ] curl で全エンドポイントが動作する

---

[次へ: Day2 セッション 1 - AWS アーキテクチャ設計 →](./04-day2-session1-aws-architecture.md)

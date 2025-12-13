# Day 1: セッション 3 - 実装実演

[← 前へ: セッション 2](./02-day1-session2-architecture.md) | [概要](./00-overview.md)

---

## 3.1 初期セットアップコマンド

```bash
# プロジェクト作成
cargo new user-api
cd user-api

# Cargo.tomlに依存を追加
# [dependencies]
# axum = "0.7"
# tokio = { version = "1", features = ["full"] }
# sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres"] }
# serde = { version = "1.0", features = ["derive"] }
# serde_json = "1.0"
# thiserror = "1.0"
# async-trait = "0.1"
# dotenv = "0.15"
# tower-http = { version = "0.5", features = ["cors", "trace"] }
# tracing = "0.1"
# tracing-subscriber = "0.3"
```

---

## 3.2 実装の流れ

### ステップ 1: main.rs のスケルトン

```rust
use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ログ初期化
    tracing_subscriber::fmt::init();

    // 環境変数から設定を読み込み
    let config = infrastructure::config::Config::from_env()?;

    // DB接続を初期化
    let pool = infrastructure::database::create_pool(&config.database_url).await?;

    // リポジトリを初期化
    let user_repo = Arc::new(
        infrastructure::repositories::PostgresUserRepository::new(pool)
    );

    // サービスレイヤーを初期化
    let user_service = Arc::new(
        application::services::UserService::new(user_repo)
    );

    // アプリケーション状態
    #[derive(Clone)]
    struct AppState {
        user_service: Arc<application::services::UserService>,
    }

    let state = AppState { user_service };

    // ルータを構築
    let app = Router::new()
        .route("/api/users", post(api::handlers::create_user))
        .route("/api/users/:id", get(api::handlers::get_user))
        .with_state(state);

    // サーバー起動
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;
    Ok(())
}
```

### ステップ 2: handlers.rs の実装

Axum を使った RESTful API のハンドラーを実装します：
- POST /api/users で CreateUserRequest を受け取り、User を返す
- GET /api/users/:id で User を返す
- エラーは ApiError に変換する
- エラーハンドリングは thiserror を使用

### ステップ 3: エラー型の定義

thiserror crate を使って以下のエラーを定義します：
- ValidationError
- DatabaseError
- NotFoundError
- それぞれを HTTP StatusCode に変換する IntoResponse を実装

### ステップ 4: データベーススキーマ作成

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);
```

---

## セッション 3 チェックリスト（実装確認）

- [ ] プロジェクトが作成され、依存が解決している
- [ ] `cargo build` が成功している
- [ ] `cargo test` が実行できる状態
- [ ] main.rs、handlers.rs、entities.rs が実装済み
- [ ] ローカル環境で API 起動確認（`cargo run`）
- [ ] 簡単な curl テスト成功：`curl http://localhost:3000/api/users`

---

[次へ: Day2 セッション 1 - AWS アーキテクチャ設計 →](./04-day2-session1-aws-architecture.md)

# Day 1: セッション 2 - クリーンアーキテクチャ実装（1 時間）

[← 前へ: セッション 1](./01-day1-session1-design.md) | [概要](./00-overview.md)

---

## 2.1 推奨プロジェクト構造

```
user-api/
├── src/
│   ├── main.rs                    # エントリーポイント（AI補助）
│   ├── lib.rs                     # ライブラリ公開API
│   ├── api/                       # 🔴 プレゼンテーション層
│   │   ├── mod.rs
│   │   ├── routes.rs              # ルート定義
│   │   ├── handlers.rs            # リクエストハンドラー
│   │   └── errors.rs              # HTTPエラー変換
│   ├── domain/                    # 🟡 ドメイン層（ビジネスロジック）
│   │   ├── mod.rs
│   │   ├── entities.rs            # User, Email等
│   │   └── errors.rs              # ドメインエラー
│   ├── application/               # 🟢 アプリケーション層
│   │   ├── mod.rs
│   │   ├── dto.rs                 # CreateUserRequest等
│   │   └── services.rs            # UserService
│   └── infrastructure/            # 🔵 インフラストラクチャ層
│       ├── mod.rs
│       ├── database.rs            # DB接続初期化
│       ├── config.rs              # Config構造体
│       └── repositories.rs        # PostgresUserRepository
├── tests/
│   └── integration_tests.rs
├── Cargo.toml
└── .env.example
```

---

## 2.2 各層の実装パターン

### 🔴 API 層: handlers.rs

```rust
use axum::{extract::Path, http::StatusCode, Json};
use crate::domain::User;
use crate::application::dto::CreateUserRequest;

pub async fn create_user(
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    // ここでのpayloadは既にバリデーション済みと仮定
    // アプリケーション層を呼び出し
    let user = todo!("service.create_user(payload)");
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn get_user(
    Path(id): Path<u32>,
) -> Result<Json<User>, ApiError> {
    let user = todo!("service.get_user(id)");
    Ok(Json(user))
}
```

### 🟡 ドメイン層: entities.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(name: String, email: String) -> Result<Self, DomainError> {
        // バリデーション
        if name.is_empty() {
            return Err(DomainError::InvalidName);
        }
        if !email.contains('@') {
            return Err(DomainError::InvalidEmail);
        }
        Ok(Self { id: 0, name, email })
    }
}
```

### 🟢 アプリケーション層: services.rs

```rust
use crate::domain::User;
use crate::infrastructure::repositories::UserRepository;

pub struct UserService {
    repository: std::sync::Arc<dyn UserRepository>,
}

impl UserService {
    pub async fn create_user(&self, req: CreateUserRequest) -> Result<User, AppError> {
        // ドメイン層で検証
        let user = User::new(req.name, req.email)?;
        // インフラストラクチャ層で永続化
        self.repository.save(&user).await?;
        Ok(user)
    }

    pub async fn get_user(&self, id: u32) -> Result<User, AppError> {
        self.repository.get_by_id(id).await
    }
}
```

### 🔵 インフラストラクチャ層: repositories.rs

```rust
use sqlx::PgPool;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: &User) -> Result<(), AppError>;
    async fn get_by_id(&self, id: u32) -> Result<User, AppError>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn save(&self, user: &User) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id"
        )
        .bind(&user.name)
        .bind(&user.email)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_by_id(&self, id: u32) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, email FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }
}
```

---

[次へ: セッション 3 - 実装実演 →](./03-day1-session3-implementation.md)

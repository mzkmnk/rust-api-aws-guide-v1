# Day 3: セッション 2 - JWT 認証の実装

[← 前へ: セッション 1](./09-day3-session1-update.md) | [概要](./00-overview.md) | [次へ: セッション 3 →](./11-day3-session3-cicd.md)

---

## 2.1 学習目標

- JWT（JSON Web Token）の仕組みを理解する
- Axum のミドルウェアで認証を実装する
- 環境変数でシークレットを管理する

---

## 2.2 JWT の基礎

### JWT の構造

```
ヘッダー.ペイロード.署名
eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwiZXhwIjoxNzM0NTAwMDAwfQ.xxxxx
```

| 部分 | 内容 |
|------|------|
| ヘッダー | アルゴリズム情報（HS256 など） |
| ペイロード | ユーザー情報、有効期限 |
| 署名 | 改ざん検知用のハッシュ |

---

## 2.3 実装タスク

### Task 1: 依存関係の追加

**やること**: `Cargo.toml` に JWT 関連のクレートを追加

**要件**:
- JWT のエンコード/デコード用クレート
- 日時操作用クレート（有効期限計算）

<details>
<summary>ヒント</summary>

- `jsonwebtoken` クレートが Rust で一般的
- `chrono` クレートで日時操作（`serde` feature が必要）

</details>

<details>
<summary>コード例</summary>

```toml
[dependencies]
# 既存の依存関係に追加
jsonwebtoken = "9"
chrono = { version = "0.4", features = ["serde"] }
```

追加後:
```bash
cargo build
```

</details>

---

### Task 2: JWT モジュール作成

**やること**: `src/infrastructure/jwt.rs` を新規作成

**要件**:
- `Claims` 構造体: `sub`（ユーザーID）と `exp`（有効期限）を持つ
- `JwtService` 構造体: シークレットキーを保持
- `generate_token`: ユーザーIDからトークンを生成（有効期限24時間）
- `verify_token`: トークンを検証して Claims を返す

<details>
<summary>ヒント</summary>

- `Claims` は `Serialize` と `Deserialize` を derive
- `exp` は Unix timestamp（`i64`）
- `chrono::Utc::now() + TimeDelta::hours(24)` で24時間後
- `EncodingKey::from_secret()` と `DecodingKey::from_secret()` を使用
- 注意: chrono 0.4.35 以降、`Duration` は `TimeDelta` に名前変更（`Duration` はエイリアス）

</details>

<details>
<summary>コード例</summary>

```rust
use chrono::{TimeDelta, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // ユーザーID
    pub exp: i64,     // 有効期限（Unix timestamp）
}

pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(&self, user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
        let expiration = Utc::now() + TimeDelta::hours(24);
        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }
}
```

`src/infrastructure/mod.rs` に追加:
```rust
pub mod jwt;
```

</details>

---

### Task 3: 認証ミドルウェア作成

**やること**: `src/api/middleware.rs` を新規作成

**要件**:
- `Authorization` ヘッダーから Bearer トークンを取得
- トークンがなければ `401 Unauthorized`
- トークンが無効なら `401 Unauthorized`
- 有効なら次のハンドラーへ進む

<details>
<summary>ヒント</summary>

- `request.headers().get(http::header::AUTHORIZATION)` でヘッダー取得
- Bearer トークンは `"Bearer "` で始まる（7文字目以降がトークン）
- `next.run(request).await` で次のハンドラーへ
- 環境変数 `JWT_SECRET` からシークレットを取得

</details>

<details>
<summary>コード例</summary>

```rust
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::infrastructure::jwt::JwtService;

pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".to_string());
    let jwt_service = JwtService::new(jwt_secret);

    match jwt_service.verify_token(token) {
        Ok(_claims) => Ok(next.run(request).await),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
```

`src/api/mod.rs` に追加:
```rust
pub mod middleware;
```

</details>

---

### Task 4: ログインエンドポイント作成

**やること**: `src/api/handler.rs` にログイン機能を追加

**要件**:
- `LoginRequest`: `email` と `password` を受け取る
- `LoginResponse`: `token` を返す
- メールでユーザーを検索し、存在すればトークン発行
- ユーザーが見つからなければ `Unauthorized` エラー

**注意**: 本番ではパスワードハッシュ比較が必要（今回は簡易実装）

<details>
<summary>ヒント</summary>

- `list_users` で全ユーザー取得 → `iter().find()` で検索
- `ApiError` に `Unauthorized` と `Internal` を追加する必要あり
- `JwtService` をインポートして使用

</details>

<details>
<summary>handler.rs への追加</summary>

```rust
use crate::infrastructure::jwt::JwtService;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let users = state.user_service.list_users().await?;
    let user = users
        .iter()
        .find(|u| u.email == payload.email)
        .ok_or(ApiError::Unauthorized)?;

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".to_string());
    let jwt_service = JwtService::new(jwt_secret);

    let token = jwt_service
        .generate_token(user.id)
        .map_err(|_| ApiError::Internal)?;

    Ok(Json(LoginResponse { token }))
}
```

</details>

<details>
<summary>errors.rs への追加</summary>

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    // 既存のエラーに追加
    #[error("認証エラー")]
    Unauthorized,

    #[error("内部エラー")]
    Internal,
}

// IntoResponse の match に追加
Self::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "認証が必要です".to_string()),
Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "内部エラー".to_string()),
```

</details>

---

### Task 5: ルーティング設定

**やること**: `src/main.rs` でミドルウェアを適用

**要件**:
- `/health` と `/api/login` は認証不要
- `/api/users` と `/api/users/{id}` は認証必要

<details>
<summary>ヒント</summary>

- `use axum::middleware;` を追加
- `.layer(middleware::from_fn(auth_middleware))` でミドルウェア適用
- 認証が必要なルートにのみ layer を追加

</details>

<details>
<summary>コード例</summary>

```rust
use axum::{middleware, routing::post};
use crate::api::middleware::auth_middleware;

let app = Router::new()
    // 認証不要
    .route("/health", get(api::handler::health_check))
    .route("/api/login", post(api::handler::login))
    // 認証必要
    .route(
        "/api/users",
        get(api::handler::list_users)
            .post(api::handler::create_user)
            .layer(middleware::from_fn(auth_middleware)),
    )
    .route(
        "/api/users/{id}",
        get(api::handler::get_user)
            .patch(api::handler::update_user)
            .delete(api::handler::delete_user)
            .layer(middleware::from_fn(auth_middleware)),
    )
    .layer(TraceLayer::new_for_http())
    .with_state(state);
```

</details>

---

### Task 6: 環境変数設定

**やること**: `.env` に JWT シークレットを追加

<details>
<summary>コード例</summary>

```
DATABASE_URL=postgres://postgres:password@localhost:5432/user_api
JWT_SECRET=your-super-secret-key-change-in-production
```

</details>

---

## 2.4 動作確認

```bash
# サーバー起動
cargo run

# 認証なしでアクセス（401 エラー）
curl http://localhost:3000/api/users

# ログイン（事前にユーザーを作成しておく）
curl -X POST http://localhost:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "dummy"}'
# {"token":"eyJhbGciOiJIUzI1NiJ9..."}

# トークンを使ってアクセス
TOKEN="eyJhbGciOiJIUzI1NiJ9..."
curl http://localhost:3000/api/users \
  -H "Authorization: Bearer $TOKEN"
```

---

## 2.5 学習ポイント

<details>
<summary>ミドルウェアの仕組み</summary>

```
リクエスト → ミドルウェア → ハンドラー → レスポンス
              ↓
         認証チェック
         ・成功 → 次へ進む
         ・失敗 → 401 を返す
```

</details>

<details>
<summary>Bearer トークン</summary>

```
Authorization: Bearer eyJhbGciOiJIUzI1NiJ9...
               ~~~~~~ ~~~~~~~~~~~~~~~~~~~~~~~~
               スキーム      トークン本体
```

</details>

<details>
<summary>セキュリティ注意点</summary>

| 項目 | 本番環境での対応 |
|------|-----------------|
| JWT_SECRET | 長いランダム文字列を使用 |
| パスワード | bcrypt/argon2 でハッシュ化 |
| HTTPS | 必須（トークン盗聴防止） |
| 有効期限 | 短めに設定（1時間など） |

</details>

---

## 2.6 完了チェックリスト

- [ ] `jsonwebtoken` と `chrono` を追加
- [ ] `JwtService` を実装
- [ ] 認証ミドルウェアを実装
- [ ] ログインエンドポイントを追加
- [ ] 保護されたルートにミドルウェアを適用
- [ ] 認証なしで 401 が返ることを確認
- [ ] トークンありでアクセスできることを確認

---

[次へ: セッション 3 - CI/CD →](./11-day3-session3-cicd.md)

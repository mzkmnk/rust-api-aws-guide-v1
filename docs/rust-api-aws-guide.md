# 2日で学べるRustのAPI設計とアーキテクチャ
## AI モブプログラミング + AWSデプロイ実践ガイド

---

## 📋 資料概要

**対象者**
- Rustの基礎構文を理解している開発者
- フロントエンド経験者でバックエンド学習中の方
- AIを活用した実装に興味のある方

**想定学習時間**: 総合5時間（AI モブプロ含む）
- **Day 1**: 設計原則とAPI実装（2.5時間）
- **Day 2**: アーキテクチャとAWSデプロイ（2.5時間）

**最終成果物**: AWSにデプロイされたRust REST API
- ECS Fargate上で実行
- RDSで永続化
- API Gatewayで公開

---

## 🎯 全体アーキテクチャ

```
┌─────────────────────────────────────────────┐
│           Internet                           │
└──────────────────┬──────────────────────────┘
                   │
        ┌──────────▼──────────┐
        │   API Gateway       │
        │  (REST Endpoint)    │
        └──────────┬──────────┘
                   │
        ┌──────────▼──────────────────┐
        │  Application Load Balancer  │
        └──────────┬──────────────────┘
                   │
    ┌──────────────┼──────────────────┐
    │              │                  │
┌───▼───┐      ┌───▼───┐         ┌───▼───┐
│ ECS   │      │ ECS   │    ...  │ ECS   │
│Task 1 │      │Task 2 │         │Task N │
│(Rust) │      │(Rust) │         │(Rust) │
└───┬───┘      └───┬───┘         └───┬───┘
    │              │                  │
    └──────────────┼──────────────────┘
                   │
        ┌──────────▼──────────┐
        │   RDS PostgreSQL    │
        │   (データ永続化)     │
        └─────────────────────┘
```

---

## Day 1: 設計原則とAPI実装（2.5時間）

### セッション 1: クイック設計概論（30分）

#### 1.1 RustでエレガントなAPI設計の9つのルール

**ルール 1-5: 基本原則**
```rust
// ✅ ルール 1: ユーザー視点の設計（ビルダーパターン）
pub struct HttpClientBuilder {
    base_url: String,
    timeout: Option<Duration>,
    headers: HashMap<String, String>,
}

impl HttpClientBuilder {
    pub fn new(url: impl Into<String>) -> Self { /* ... */ }
    pub fn timeout(mut self, d: Duration) -> Self { 
        self.timeout = Some(d); self 
    }
    pub fn build(self) -> Result<HttpClient> { /* ... */ }
}

// ✅ ルール 2: トレイト境界で型柔軟性
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let path = path.as_ref();
    // パス操作
}

// ✅ ルール 3: 内部で自分たちのAPIを使用
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_api_as_user() {
        let client = HttpClientBuilder::new("http://localhost:3000")
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build client");
    }
}

// ✅ ルール 4: ドキュメンテーション
/// ユーザー作成リクエスト
/// 
/// # Examples
/// ```
/// let req = CreateUserRequest { 
///     name: "Alice".to_string(),
///     email: "alice@example.com".to_string()
/// };
/// ```
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

// ✅ ルール 5: エラーの構造化
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
```

#### 1.2 Axumフレームワーク選定理由

2025年標準選択の理由：
- ✅ **フレームワーク非依存**: ビジネスロジックがフレームワークに依存しない
- ✅ **Tower統合**: 豊富なミドルウェアエコシステム
- ✅ **学習曲線が低い**: 関数ベースで直感的
- ✅ **AWSとの相性**: Lambda + RDSパターンに最適

---

### セッション 2: クリーンアーキテクチャ実装（1時間）

#### 2.1 推奨プロジェクト構造

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

#### 2.2 各層の実装パターン

**🔴 API層: handlers.rs**
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

**🟡 ドメイン層: entities.rs**
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

**🟢 アプリケーション層: services.rs**
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

**🔵 インフラストラクチャ層: repositories.rs**
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

### セッション 3: 実装実演（1時間 - AIモブプロ）

#### 3.1 初期セットアップコマンド

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

#### 3.2 AIモブプロで実装する流れ

**ステップ1: main.rsのスケルトン（AI補助）**
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

**ステップ2: AIに「handlers.rsを書いて」と依頼**
```
プロンプト例：
"Axumを使ったRESTful APIのハンドラーを書いてください。
- POST /api/users で CreateUserRequest を受け取り、User を返す
- GET /api/users/:id で User を返す
- エラーは ApiError に変換する
エラーハンドリングはthiserrorを使用"
```

**ステップ3: AIに「エラー型を定義して」と依頼**
```
プロンプト例：
"thiserror crateを使って以下のエラーを定義してください：
- ValidationError
- DatabaseError
- NotFoundError
- それぞれをHTTP StatusCodeに変換するIntoResponseを実装"
```

**ステップ4: データベーススキーマ作成**
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

### セッション 3 チェックリスト（実装確認）

- [ ] プロジェクトが作成され、依存が解決している
- [ ] `cargo build` が成功している
- [ ] `cargo test` が実行できる状態
- [ ] main.rs、handlers.rs、entities.rs が実装済み
- [ ] ローカル環境でAPI起動確認（`cargo run`）
- [ ] 簡単なcurlテスト成功：`curl http://localhost:3000/api/users`

---

## Day 2: AWSアーキテクチャとデプロイ（2.5時間）

### セッション 1: AWSアーキテクチャ設計（30分）

#### 1.1 本番環境構成

```
┌─────────────────────────────────────────┐
│         AWS Cloud                        │
├─────────────────────────────────────────┤
│ ┌──────────────────────────────────────┐│
│ │  Route 53 (DNS)                      ││
│ │  api.example.com                     ││
│ └────────────┬─────────────────────────┘│
│              │                           │
│ ┌────────────▼─────────────────────────┐│
│ │  API Gateway (REST)                  ││
│ │  - Authentication (API Key)           ││
│ │  - Rate Limiting                      ││
│ │  - CORS設定                           ││
│ └────────────┬─────────────────────────┘│
│              │                           │
│ ┌────────────▼─────────────────────────┐│
│ │ ECS Cluster (Fargate)                ││
│ │ ┌────────────────────────────────────┤│
│ │ │ Task Definition (Rust Binary)      ││
│ │ │ - CPU: 256 (0.25 vCPU)             ││
│ │ │ - Memory: 512 MB                   ││
│ │ │ - ContainerPort: 3000               ││
│ │ │ - Environment Variables: DB_URL     ││
│ │ └────────────────────────────────────┤│
│ │ ┌────────────────────────────────────┤│
│ │ │ Desired Count: 2 (Auto Scaling)    ││
│ │ └────────────────────────────────────┤│
│ └────────────┬─────────────────────────┘│
│              │                           │
│ ┌────────────▼─────────────────────────┐│
│ │  RDS (PostgreSQL 15)                 ││
│ │ - Multi-AZ (本番要件)                 ││
│ │ - Performance Insights有効            ││
│ │ - Automated Backups (7日)             ││
│ └──────────────────────────────────────┘│
│ ┌──────────────────────────────────────┐│
│ │  CloudWatch Logs                     ││
│ │ - ECS Container Logs                 ││
│ │ - Lambda Logs (必要に応じて)          ││
│ └──────────────────────────────────────┘│
└─────────────────────────────────────────┘
```

#### 1.2 コスト最適化のポイント

| リソース | スペック | 月額コスト見積 |
|---------|---------|----------|
| ECS Fargate | 0.25 vCPU × 512MB × 2 tasks | ~$15 |
| RDS PostgreSQL | db.t4g.micro (1年契約) | ~$25 |
| API Gateway | 1M requests/月 | ~$3.50 |
| CloudWatch | Logs retention 7日 | ~$2 |
| **合計** | | **~$45/月** |

---

### セッション 2: Dockerコンテナ化（45分 - AIモブプロ）

#### 2.1 Dockerfileの最適化

```dockerfile
# ========================
# Stage 1: Builder
# ========================
FROM rust:1.75 as builder

WORKDIR /app

# キャッシュレイヤーの最適化：依存だけ先にビルド
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
RUN rm -rf src

# 実装ソースをコピー
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ========================
# Stage 2: Runtime
# ========================
FROM debian:bookworm-slim

# 必要なランタイムライブラリをインストール
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# ビルダーから実行バイナリをコピー
COPY --from=builder /app/target/release/user-api /usr/local/bin/

EXPOSE 3000

CMD ["user-api"]
```

**イメージサイズ最適化**:
- マルチステージビルド使用
- デバッグシンボル削除
- ランタイム最小化
- 最終イメージサイズ: ~100MB

#### 2.2 .dockerignore

```
.git
.gitignore
target/
Cargo.lock
.env
.env.local
.vscode
.idea
README.md
```

#### 2.3 ローカルでのテスト

```bash
# Dockerイメージビルド
docker build -t user-api:latest .

# ローカルPostgreSQLで実行
docker run -e DATABASE_URL=postgresql://user:pass@host.docker.internal/userdb \
  -p 3000:3000 \
  user-api:latest
```

---

### セッション 3: AWS ECS/Fargateへのデプロイ（1時間）

#### 3.1 Amazon ECR（イメージレジストリ）へのプッシュ

```bash
# AWS CLIで認証
aws ecr get-login-password --region ap-northeast-1 | \
  docker login --username AWS --password-stdin 123456789.dkr.ecr.ap-northeast-1.amazonaws.com

# リポジトリ作成
aws ecr create-repository --repository-name user-api --region ap-northeast-1

# イメージタグ付与
docker tag user-api:latest 123456789.dkr.ecr.ap-northeast-1.amazonaws.com/user-api:latest

# プッシュ
docker push 123456789.dkr.ecr.ap-northeast-1.amazonaws.com/user-api:latest
```

#### 3.2 RDS PostgreSQL セットアップ

```bash
# AWS CLI でRDSインスタンス作成
aws rds create-db-instance \
  --db-instance-identifier user-api-db \
  --db-instance-class db.t4g.micro \
  --engine postgres \
  --master-username postgres \
  --master-user-password "GenerateStrongPassword123!" \
  --allocated-storage 20 \
  --vpc-security-group-ids sg-xxxxxxxx \
  --db-subnet-group-name default \
  --publicly-accessible false \
  --region ap-northeast-1

# RDS作成完了後、初期スキーマを流す
psql -h user-api-db.cxxxxxxx.ap-northeast-1.rds.amazonaws.com \
  -U postgres \
  < schema.sql
```

#### 3.3 ECS クラスター・タスク定義

```bash
# クラスター作成
aws ecs create-cluster --cluster-name user-api-cluster --region ap-northeast-1

# タスク定義登録（task-definition.json から）
aws ecs register-task-definition \
  --cli-input-json file://task-definition.json \
  --region ap-northeast-1

# サービス作成
aws ecs create-service \
  --cluster user-api-cluster \
  --service-name user-api-service \
  --task-definition user-api-task:1 \
  --desired-count 2 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[subnet-xxx],securityGroups=[sg-xxx],assignPublicIp=ENABLED}" \
  --region ap-northeast-1
```

**task-definition.json テンプレート**
```json
{
  "family": "user-api-task",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "256",
  "memory": "512",
  "containerDefinitions": [
    {
      "name": "user-api",
      "image": "123456789.dkr.ecr.ap-northeast-1.amazonaws.com/user-api:latest",
      "portMappings": [
        {
          "containerPort": 3000,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "DATABASE_URL",
          "value": "postgresql://postgres:password@user-api-db.cxxxxxxx.ap-northeast-1.rds.amazonaws.com/userdb"
        },
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/user-api",
          "awslogs-region": "ap-northeast-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

#### 3.4 API Gateway 設定（オプション）

```bash
# REST API作成
aws apigateway create-rest-api \
  --name user-api \
  --description "User Management API" \
  --region ap-northeast-1

# リソース・メソッド設定は AWSコンソール or Terraform推奨
```

---

### セッション 4: デプロイ検証と監視（30分）

#### 4.1 デプロイ確認コマンド

```bash
# ECSサービスのステータス確認
aws ecs describe-services \
  --cluster user-api-cluster \
  --services user-api-service \
  --region ap-northeast-1

# タスク一覧確認
aws ecs list-tasks \
  --cluster user-api-cluster \
  --region ap-northeast-1

# ログ確認
aws logs tail /ecs/user-api --follow --region ap-northeast-1

# CloudWatch メトリクス確認
aws cloudwatch get-metric-statistics \
  --namespace AWS/ECS \
  --metric-name CPUUtilization \
  --dimensions Name=ServiceName,Value=user-api-service \
  --statistics Average \
  --start-time 2025-12-13T00:00:00Z \
  --end-time 2025-12-14T00:00:00Z \
  --period 300 \
  --region ap-northeast-1
```

#### 4.2 ヘルスチェック設定

```bash
# ALB（Application Load Balancer）経由でのヘルスチェック
# ヘルスチェック用エンドポイント追加（main.rs）
app = app.route("/health", get(|| async { "OK" }))

# ALB ターゲットグループ設定
aws elbv2 create-target-group \
  --name user-api-targets \
  --protocol HTTP \
  --port 3000 \
  --vpc-id vpc-xxxxxxxx \
  --health-check-path /health \
  --health-check-interval-seconds 30 \
  --health-check-timeout-seconds 5 \
  --healthy-threshold-count 2 \
  --unhealthy-threshold-count 2 \
  --region ap-northeast-1
```

#### 4.3 本番運用チェックリスト

- [ ] ECSタスクが両方起動している
- [ ] CloudWatch Logsにエラーがない
- [ ] RDSコネクション数が正常
- [ ] API Gatewayが正常に動作している
- [ ] ヘルスチェック（/health）が通っている
- [ ] 定期的なバックアップが有効
- [ ] CloudWatch アラーム設定済み
- [ ] 本番環境用の.env設定完了

---

## 📚 実装リファレンス

### Cargo.toml 最終版

```toml
[package]
name = "user-api"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
async-trait = "0.1"
dotenv = "0.15"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### エラーハンドリング実装例

```rust
use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("validation error: {0}")]
    Validation(String),
    
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("user not found")]
    NotFound,
    
    #[error("internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database error occurred".to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "Resource not found".to_string()),
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}
```

---

## 🎓 AIモブプロ活用ヒント

### 効果的なプロンプト例

**1. 構造体を定義する場合**
```
「ユーザーを表すRust構造体を定義してください。
フィールド: id (u32), name (String), email (String), created_at (DateTime<Utc>)
Serde, Debug, Clone derivを付与
DTOとドメインエンティティに分離したバージョンも作成」
```

**2. エラーハンドリング実装時**
```
「thiserror crateを使用して、以下のエラーを定義してください:
- ValidationError(String)
- DatabaseError(sqlx::Error)
- NotFoundError
- UnauthorizedError
それぞれをaxumのIntoResponseに変換する実装も含める」
```

**3. データベースクエリ**
```
「SQLxを使ってユーザーを取得するクエリ関数を書いてください。
- get_user_by_id(pool: &PgPool, id: u32) -> Result<User>
- list_users(pool: &PgPool) -> Result<Vec<User>>
バッチ操作も考慮してください」
```

---

## 🚀 デプロイ後のメンテナンス

### CI/CDパイプライン（GitHub Actions 例）

```yaml
name: Deploy to AWS ECS

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Docker image
        run: docker build -t user-api:${{ github.sha }} .
      
      - name: Push to ECR
        env:
          AWS_REGION: ap-northeast-1
        run: |
          aws ecr get-login-password | docker login --username AWS --password-stdin $ECR_URI
          docker tag user-api:${{ github.sha }} $ECR_URI/user-api:latest
          docker push $ECR_URI/user-api:latest
      
      - name: Update ECS service
        run: |
          aws ecs update-service \
            --cluster user-api-cluster \
            --service user-api-service \
            --force-new-deployment
```

---

## ✅ 最終チェックリスト

### Day 1 終了時
- [ ] プロジェクト構造が完成
- [ ] handlers, entities, services が実装済み
- [ ] `cargo build --release` 成功
- [ ] ローカル環境で API 起動確認
- [ ] curl でエンドポイント動作確認

### Day 2 終了時
- [ ] Docker イメージビルド成功
- [ ] ECR にプッシュ完了
- [ ] RDS インスタンス作成済み
- [ ] ECS タスク定義登録完了
- [ ] ECS サービス稼働確認
- [ ] 本番環境で API へリクエスト可能
- [ ] CloudWatch ログ確認可能
- [ ] ヘルスチェック通過

---

## 📖 参考リソース

**Rust API 設計**
- [Nine Rules for Elegant Rust Library APIs](https://towardsdatascience.com/nine-rules-for-elegant-rust-library-apis-9b986a465247)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)

**AWS デプロイ**
- [AWS ECS Fargate ドキュメント](https://docs.aws.amazon.com/ja_jp/AmazonECS/latest/developerguide/what-is-amazon-ecs.html)
- [AWS RDS PostgreSQL ベストプラクティス](https://docs.aws.amazon.com/ja_jp/AmazonRDS/latest/UserGuide/CHAP_PostgreSQL.html)

**AI 支援開発**
- Claude Code + Cursor IDE 推奨
- factory.ai の Droid CLI との組み合わせ

---

**作成日**: 2025年12月13日  
**推奨実施期間**: 2日間（各日2.5時間）  
**AI支援**: Claude Code / Cursor IDE / Droid CLI

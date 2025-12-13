# 📚 実装リファレンス

[← 前へ: セッション 4](./07-day2-session4-monitoring.md) | [概要](./00-overview.md)

---

## Cargo.toml 最終版

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

---

## エラーハンドリング実装例

```rust
use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
```

#[derive(Error, Debug)]
pub enum AppError { #[error("validation error: {0}")]
Validation(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("user not found")]
    NotFound,

    #[error("internal server error")]
    Internal,

}

impl IntoResponse for AppError {
fn into*response(self) -> Response {
let (status, error_code, message) = match self {
Self::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
Self::Database(*) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database error occurred".to_string()),
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

## 🚀 CI/CD パイプライン（GitHub Actions 例）

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

---

**作成日**: 2025 年 12 月 13 日  
**推奨実施期間**: 2 日間（各日 2.5 時間）

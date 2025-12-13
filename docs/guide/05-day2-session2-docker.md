# Day 2: セッション 2 - Docker コンテナ化

[← 前へ: セッション 1](./04-day2-session1-aws-architecture.md) | [概要](./00-overview.md)

---

## 2.1 Dockerfile の最適化

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

---

## 2.2 .dockerignore

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

---

## 2.3 ローカルでのテスト

```bash
# Dockerイメージビルド
docker build -t user-api:latest .

# ローカルPostgreSQLで実行
docker run -e DATABASE_URL=postgresql://user:pass@host.docker.internal/userdb \
  -p 3000:3000 \
  user-api:latest
```

---

[次へ: セッション 3 - AWS ECS/Fargate デプロイ →](./06-day2-session3-ecs-deploy.md)

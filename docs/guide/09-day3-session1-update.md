# Day 3: セッション 1 - Update 機能の追加

[← 前へ: リファレンス](./08-reference.md) | [概要](./00-overview.md) | [次へ: セッション 2 →](./10-day3-session2-auth.md)

---

## 1.1 学習目標

- CRUD の「U」（Update）を実装する
- 部分更新（PATCH）と全体更新（PUT）の違いを理解する
- SQLx での UPDATE クエリを学ぶ

---

## 1.2 設計方針

REST API における更新操作:

| メソッド | 用途 | 特徴 |
|---------|------|------|
| PUT | 全体置換 | 全フィールド必須 |
| PATCH | 部分更新 | 変更フィールドのみ |

今回は **PATCH**（部分更新）を実装します。実務でより一般的なパターンです。

---

## 1.3 実装タスク

### Task 1: DTO の追加

**やること**: `src/application/dto.rs` に更新用リクエスト構造体を追加

**要件**:
- `name` と `email` を持つ
- どちらも「あってもなくてもいい」（部分更新のため）

<details>
<summary>ヒント</summary>

- Rust で「値があるかもしれない」を表現する型は？
- `Option<T>` を使うと `Some(値)` または `None` を表現できる
- `#[derive(Deserialize)]` でJSONからデシリアライズ可能にする

</details>

<details>
<summary>コード例</summary>

```rust
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}
```

</details>

---

### Task 2: Repository に update メソッド追加

**やること**: 
1. `UserRepository` トレイトに `update` メソッドを追加
2. `PostgresUserRepository` に実装を追加

**要件**:
- `id`, `name`, `email` を受け取る
- `name` が `None` なら既存値を維持
- `email` が `None` なら既存値を維持
- 更新後の `User` を返す

<details>
<summary>ヒント</summary>

- SQL の `COALESCE(a, b)` は「a が NULL なら b を返す」
- `RETURNING` 句で更新後のレコードを取得できる
- `Option<String>` は SQLx で自動的に NULL として扱われる

</details>

<details>
<summary>トレイト定義</summary>

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    // 既存メソッド...
    async fn update(&self, id: i32, name: Option<String>, email: Option<String>) -> Result<User, sqlx::Error>;
}
```

</details>

<details>
<summary>実装例</summary>

```rust
async fn update(&self, id: i32, name: Option<String>, email: Option<String>) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET name = COALESCE($2, name),
            email = COALESCE($3, email)
        WHERE id = $1
        RETURNING id, name, email
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(email)
    .fetch_one(&self.pool)
    .await
}
```

</details>

---

### Task 3: Service に update_user メソッド追加

**やること**: `src/application/services.rs` に `update_user` メソッドを追加

**要件**:
- ユーザーが存在しなければ `NotFound` エラー
- `name` が `Some` の場合のみバリデーション（空文字 or 100文字超でエラー）
- `email` が `Some` の場合のみバリデーション（`@` なし or 3文字未満でエラー）
- Repository の `update` を呼び出す

<details>
<summary>ヒント</summary>

- `if let Some(ref value) = option` で値がある場合のみ処理
- `ref` を使うと所有権を移動せずに参照できる
- 存在確認は `find_by_id` → `ok_or(AppError::NotFound)?`

</details>

<details>
<summary>コード例</summary>

```rust
pub async fn update_user(&self, id: i32, req: UpdateUserRequest) -> Result<UserResponse, AppError> {
    // 存在確認
    self.repository
        .find_by_id(id)
        .await?
        .ok_or(AppError::NotFound)?;

    // バリデーション（値がある場合のみ）
    if let Some(ref name) = req.name {
        if name.is_empty() || name.len() > 100 {
            return Err(AppError::Domain(DomainError::InvalidName));
        }
    }
    if let Some(ref email) = req.email {
        if !email.contains('@') || email.len() < 3 {
            return Err(AppError::Domain(DomainError::InvalidEmail));
        }
    }

    let updated = self.repository.update(id, req.name, req.email).await?;
    Ok(UserResponse::from(updated))
}
```

</details>

---

### Task 4: Handler 追加

**やること**: `src/api/handler.rs` に `update_user` ハンドラーを追加

**要件**:
- パスから `id` を取得
- ボディから `UpdateUserRequest` を取得
- 成功時は `200 OK` と更新後のユーザーを返す

<details>
<summary>ヒント</summary>

- `Path(id): Path<i32>` でパスパラメータを取得
- `Json(payload): Json<UpdateUserRequest>` でボディを取得
- `use` に `UpdateUserRequest` を追加するのを忘れずに

</details>

<details>
<summary>コード例</summary>

```rust
use crate::application::dto::{CreateUserRequest, UpdateUserRequest, UserResponse};

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    let user = state.user_service.update_user(id, payload).await?;
    Ok((StatusCode::OK, Json(user)))
}
```

</details>

---

### Task 5: ルーティング追加

**やること**: `src/main.rs` のルーティングに PATCH メソッドを追加

**要件**:
- `/api/users/{id}` に PATCH メソッドを追加
- 既存の GET, DELETE と同じルートに追加

<details>
<summary>ヒント</summary>

- `.patch(handler)` でPATCHメソッドを追加
- メソッドチェーンで複数のHTTPメソッドを同じパスに設定できる

</details>

<details>
<summary>コード例</summary>

```rust
.route(
    "/api/users/{id}",
    get(api::handler::get_user)
        .patch(api::handler::update_user)
        .delete(api::handler::delete_user),
)
```

</details>

---

## 1.4 動作確認

```bash
# ローカルで起動
cargo run

# ユーザー作成
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Test User", "email": "test@example.com"}'

# 名前のみ更新
curl -X PATCH http://localhost:3000/api/users/1 \
  -H "Content-Type: application/json" \
  -d '{"name": "Updated Name"}'

# メールのみ更新
curl -X PATCH http://localhost:3000/api/users/1 \
  -H "Content-Type: application/json" \
  -d '{"email": "new@example.com"}'

# 両方更新
curl -X PATCH http://localhost:3000/api/users/1 \
  -H "Content-Type: application/json" \
  -d '{"name": "New Name", "email": "new@example.com"}'

# 存在しないユーザー（404 エラー）
curl -X PATCH http://localhost:3000/api/users/999 \
  -H "Content-Type: application/json" \
  -d '{"name": "Test"}'
```

---

## 1.5 学習ポイント

<details>
<summary>Option 型の活用</summary>

```rust
// Option<T> は「値があるかもしれない」を表現
pub struct UpdateUserRequest {
    pub name: Option<String>,   // Some("新しい名前") or None
    pub email: Option<String>,  // Some("new@example.com") or None
}

// if let で値がある場合のみ処理
if let Some(ref name) = req.name {
    // name を使った処理
}
```

</details>

<details>
<summary>COALESCE の動作</summary>

```sql
-- COALESCE(a, b): a が NULL なら b を返す
UPDATE users
SET name = COALESCE(NULL, name)  -- 既存値を維持
SET name = COALESCE('新名前', name)  -- '新名前' に更新
```

</details>

---

## 1.6 完了チェックリスト

- [ ] `UpdateUserRequest` DTO を追加
- [ ] Repository に `update` メソッドを追加
- [ ] Service に `update_user` メソッドを追加
- [ ] Handler に `update_user` を追加
- [ ] ルーティングに `.patch()` を追加
- [ ] curl で部分更新が動作することを確認
- [ ] 存在しないユーザーで 404 が返ることを確認

---

[次へ: セッション 2 - JWT 認証 →](./10-day3-session2-auth.md)

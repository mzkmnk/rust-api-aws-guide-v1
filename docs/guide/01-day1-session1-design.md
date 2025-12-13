# Day 1: セッション 1 - クイック設計概論

[← 概要に戻る](./00-overview.md)

---

## 1.1 Rust でエレガントな API 設計の 9 つのルール
◊
### ルール 1-5: 基本原則

````rust
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
````

---

## 1.2 Axum フレームワーク選定理由

2025 年標準選択の理由：

- ✅ **フレームワーク非依存**: ビジネスロジックがフレームワークに依存しない
- ✅ **Tower 統合**: 豊富なミドルウェアエコシステム
- ✅ **学習曲線が低い**: 関数ベースで直感的
- ✅ **AWS との相性**: Lambda + RDS パターンに最適

---

[次へ: セッション 2 - クリーンアーキテクチャ実装 →](./02-day1-session2-architecture.md)

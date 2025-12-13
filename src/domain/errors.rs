use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("名前は1文字以上100文字未満で文字未満である必要があります。")]
    InvalidName,
    #[error("無効なメールアドレス形式です")]
    InvalidEmail,
}

use std::sync::Arc;

use crate::{
    application::dto::{CreateUserRequest, UserResponse},
    domain::{errors::DomainError, user::User},
    infrastructure::repositories::UserRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("ドメインエラー: {0}")]
    Domain(#[from] DomainError),

    #[error("データベースエラー: {0}")]
    Database(#[from] sqlx::Error),

    #[error("ユーザが見つかりません。")]
    NotFound,
}

pub struct UserService {
    repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> Result<UserResponse, AppError> {
        let user = User::new(req.name, req.email)?;

        let saved_user = self.repository.save(user).await?;

        Ok(UserResponse::from(saved_user))
    }

    pub async fn get_user(&self, id: i32) -> Result<UserResponse, AppError> {
        let user = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        Ok(UserResponse::from(user))
    }

    pub async fn list_users(&self) -> Result<Vec<UserResponse>, AppError> {
        let users = self.repository.find_all().await?;

        Ok(users.into_iter().map(UserResponse::from).collect())
    }

    pub async fn delete_user(&self, id: i32) -> Result<(), AppError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.repository.delete(id).await?;

        Ok(())
    }
}

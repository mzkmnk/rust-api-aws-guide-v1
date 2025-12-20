use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    api::errors::ApiError,
    application::{
        dto::{CreateUserRequest, UserResponse},
        services::UserService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    let user = state.user_service.create_user(payload).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    let user = state.user_service.get_user(id).await?;
    Ok((StatusCode::OK, Json(user)))
}

pub async fn list_users(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<UserResponse>>), ApiError> {
    let users = state.user_service.list_users().await?;
    Ok((StatusCode::OK, Json(users)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    state.user_service.delete_user(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn health_check() -> &'static str {
    r#"
  ██████╗ ██╗  ██╗
 ██╔═══██╗██║ ██╔╝
 ██║   ██║█████╔╝ 
 ██║   ██║██╔═██╗ 
 ╚██████╔╝██║  ██╗
  ╚═════╝ ╚═╝  ╚═╝
"#
}

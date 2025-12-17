use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
}

impl From<crate::domain::user::User> for UserResponse {
    fn from(user: crate::domain::user::User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
        }
    }
}

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(name: String, email: String) -> Result<Self, DomainError> {
        if name.is_empty() || name.len() > 100 {
            return Err(DomainError::InvalidName);
        }

        if !email.contains("@") || email.len() < 3 {
            return Err(DomainError::InvalidEmail);
        }

        Ok(Self { id: 0, name, email })
    }

    pub fn with_id(mut self, id: i32) -> Self {
        self.id = id;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_user() {
        let user = User::new("John Doe".to_string(), "john.doe@example.com".to_string());
        assert!(user.is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let user = User::new("John Doe".to_string(), "invalid-email".to_string());
        assert!(matches!(user, Err(DomainError::InvalidEmail)));
    }
}

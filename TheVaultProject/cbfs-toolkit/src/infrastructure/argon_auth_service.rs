use crate::domain::user::{User, UserRepository, AuthService};
use crate::errors::{AppError, AppResult};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString, PasswordHash, PasswordVerifier, PasswordHasher},
    Argon2,
};
use std::sync::Arc;

pub struct ArgonAuthService {
    user_repo: Arc<dyn UserRepository + Send + Sync>,
}

impl ArgonAuthService {
    pub fn new(user_repo: Arc<dyn UserRepository + Send + Sync>) -> Self {
        Self { user_repo }
    }
}

impl AuthService for ArgonAuthService {
    fn hash_password(&self, password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Auth(format!("Hashing failed: {}", e)))?
            .to_string())
    }

    fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
         let parsed_hash = PasswordHash::new(hash)
             .map_err(|e| AppError::Auth(format!("Password hash error: {}", e)))?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    fn authenticate(&self, username: &str, password: &str) -> AppResult<Option<User>> {
        // 1. Get user
        if let Some(user) = self.user_repo.find_by_username(username)? {
            // 2. Get stored hash
            if let Some(hash) = self.user_repo.get_password_hash(username)? {
                // 3. Verify
                if self.verify_password(password, &hash)? {
                    return Ok(Some(user));
                }
            }
        }
        Ok(None)
    }
}

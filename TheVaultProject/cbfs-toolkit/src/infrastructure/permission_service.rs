use crate::domain::user::{User, UserRepository, PermissionService};
use crate::errors::AppResult;
use std::sync::Arc;

/// Concrete implementation of PermissionService
/// Delegates to UserRepository for permission checks
pub struct StandardPermissionService {
    user_repo: Arc<dyn UserRepository + Send + Sync>,
}

impl StandardPermissionService {
    pub fn new(user_repo: Arc<dyn UserRepository + Send + Sync>) -> Self {
        Self { user_repo }
    }
}

impl PermissionService for StandardPermissionService {
    fn check_permission(&self, user: &User, permission: &str) -> AppResult<bool> {
        // Get explicit permission from repository
        let has_explicit = self.user_repo.has_permission(user.id, permission)?;
        
        // Delegate to Domain entity for business rule
        Ok(user.can_perform(has_explicit))
    }
}

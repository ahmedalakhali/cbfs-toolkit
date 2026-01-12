use crate::errors::AppResult;

// Permission Constants
pub const PERM_CREATE_USER: &str = "CreateUser";
pub const PERM_CREATE_VAULT: &str = "CreateVault";
pub const PERM_CLOUD_WRITE: &str = "CloudWrite";

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub is_admin: bool,
}

#[allow(dead_code)]
impl User {
    pub fn new(id: i32, username: String, is_admin: bool) -> Self {
        Self {
            id,
            username,
            is_admin,
        }
    }

    /// Domain logic to check if a user has a specific permission.
    /// This might rely on the repository if permissions are stored separately and not loaded eagerly.
    /// However, for clean architecture, the entity should ideally be self-contained or the service checks it.
    /// In the legacy code, `has_permission` queried the DB directly. 
    /// To keep it clean, we might want to load permissions into the User object or have a PermissionService.
    /// For now, we'll separate the data access from the entity. 
    /// We will add a method that delegates to a repository closer to the use case, or we make `has_permission`
    /// take a list of permissions the user *has*.
    /// 
    /// But to adapt to the existing `has_permission` logic easier, we might need to inject dependencies? 
    /// Entities shouldn't have services injected.
    /// 
    /// Alternative: The `User` struct could hold its permissions if they are few.
    /// The DB schema has a `permissions` table.
    /// 
    /// Let's change the design slightly: The `User` entity will hold `permissions: Vec<String>` if feasible.
    /// If that's too expensive (many permissions?), we keep it separate. 
    /// Given the simple use case, loading permissions with the user is better for "Aggregate Root" design.
    /// 
    /// However, existing code does lazy loaded check `SELECT count(*) ...`. 
    /// I will define a helper trait or service for checking permissions if not loaded.
    pub fn has_admin_privileges(&self) -> bool {
        self.is_admin
    }

    /// Core business rule: Check if user can perform an action.
    /// Admins can perform ANY action. Regular users need explicit permission.
    /// 
    /// # Arguments
    /// * `has_explicit_permission` - Whether the user has been granted this specific permission
    /// 
    /// # Returns
    /// true if user is admin OR has explicit permission
    pub fn can_perform(&self, has_explicit_permission: bool) -> bool {
        self.is_admin || has_explicit_permission
    }
}

/// Interface for User data access
pub trait UserRepository {
    fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
    fn create(&self, username: &str, password_hash: &str, is_admin: bool) -> AppResult<()>;
    fn list_all(&self) -> AppResult<Vec<User>>;
    fn make_admin(&self, username: &str) -> AppResult<()>;
    fn get_password_hash(&self, username: &str) -> AppResult<Option<String>>;
    
    // Permission management
    fn grant_permission(&self, username: &str, permission: &str) -> AppResult<()>;
    fn has_permission(&self, user_id: i32, permission: &str) -> AppResult<bool>;
    fn revoke_permission(&self, username: &str, permission: &str) -> AppResult<()>;
    fn list_permissions(&self, user_id: i32) -> AppResult<Vec<String>>;
    
    // User management
    fn delete(&self, user_id: i32) -> AppResult<()>;
    fn update_password(&self, user_id: i32, password_hash: &str) -> AppResult<()>;
    
    // Initialization
    fn init(&self) -> AppResult<bool>; // Returns true if admin exists
}

/// Interface for Authentication
pub trait AuthService: Send + Sync {
    fn authenticate(&self, username: &str, password: &str) -> AppResult<Option<User>>;
    fn hash_password(&self, password: &str) -> AppResult<String>;
    fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool>;
}

/// Interface for Permission Checking
/// Centralizes all permission logic following Single Responsibility Principle
pub trait PermissionService {
    /// Check if a user has a specific permission
    /// Returns true if user is admin OR has the specific permission
    fn check_permission(&self, user: &User, permission: &str) -> AppResult<bool>;
}

use crate::domain::user::{User, UserRepository};
use crate::errors::{AppError, AppResult};
use rusqlite::{params, Connection};
use chrono::Utc;

pub struct SQLiteUserRepository {
    db_path: String,
}

impl SQLiteUserRepository {
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
        }
    }

    fn connect(&self) -> AppResult<Connection> {
        Connection::open(&self.db_path).map_err(|e| AppError::General(format!("DB Connection error: {}", e)))
    }
}

impl UserRepository for SQLiteUserRepository {
    fn init(&self) -> AppResult<bool> {
        let conn = self.connect()?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                is_admin BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )",
            [],
        ).map_err(|e| AppError::General(e.to_string()))?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS permissions (
                user_id INTEGER NOT NULL,
                permission TEXT NOT NULL,
                PRIMARY KEY (user_id, permission),
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            [],
        ).map_err(|e| AppError::General(e.to_string()))?;

        // Check if any admin exists
        let count: i32 = conn.query_row("SELECT count(*) FROM users WHERE is_admin = 1", [], |row| row.get(0))
            .map_err(|e| AppError::General(e.to_string()))?;
        
        Ok(count > 0)
    }

    fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT id, username, is_admin FROM users WHERE username = ?1")
            .map_err(|e| AppError::General(e.to_string()))?;
            
        let mut rows = stmt.query(params![username])
            .map_err(|e| AppError::General(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| AppError::General(e.to_string()))? {
            Ok(Some(User {
                id: row.get(0).map_err(|e| AppError::General(e.to_string()))?,
                username: row.get(1).map_err(|e| AppError::General(e.to_string()))?,
                is_admin: row.get(2).map_err(|e| AppError::General(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }
    
    // Helper to get password hash (not in Domain User entity, internal concern)
    // But AuthService needs it. 
    // Wait, AuthService needs to retrieve the user's password hash to verify it.
    // The `User` entity doesn't have the password hash.
    // So `AuthService` might need access to `UserRepository` OR `UserRepository` needs to expose `authenticate`?
    // In strict Clean Architecture, `AuthService` implements `authenticate`. It would call `UserRepository` to get the stored hash?
    // Or `UserRepository` returns a model that includes the hash?
    // Ideally, `UserRepository` should handle fetching the data.
    // Let's add `get_password_hash` to `UserRepository` or just `find_user_with_credentials`?
    // Let's modify `verify_password` logic.
    // Actually, to keep it simple, `authenticate` usually belongs in the Service.
    // The service gets the user data (including hash) from the repo.
    // But `User` entity shouldn't probably leak the hash if we want to be strict, but for practical reasons it might exist in a DTO or specific method.
    // I will add `get_password_hash` to the `UserRepository` implementation (as a public method or internal helper if I move auth logic here).
    // Better: let `AuthService` depend on `UserRepository`.
    // But `UserRepository` interface needs to expose the hash for this to work, OR `UserRepository` has a method `get_login_data` returning `(User, String)`.
    
    fn create(&self, username: &str, password_hash: &str, is_admin: bool) -> AppResult<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO users (username, password, is_admin, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![username, password_hash, is_admin, Utc::now().to_rfc3339()],
        ).map_err(|e| AppError::General(e.to_string()))?;
        Ok(())
    }

    fn list_all(&self) -> AppResult<Vec<User>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT id, username, is_admin FROM users")
            .map_err(|e| AppError::General(e.to_string()))?;
            
        let rows = stmt.query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                is_admin: row.get(2)?,
            })
        }).map_err(|e| AppError::General(e.to_string()))?;

        let mut users = Vec::new();
        for user in rows {
            users.push(user.map_err(|e| AppError::General(e.to_string()))?);
        }
        Ok(users)
    }

    fn make_admin(&self, username: &str) -> AppResult<()> {
        let conn = self.connect()?;
        conn.execute(
            "UPDATE users SET is_admin = 1 WHERE username = ?1",
            params![username],
        ).map_err(|e| AppError::General(e.to_string()))?;
        Ok(())
    }

    fn grant_permission(&self, username: &str, permission: &str) -> AppResult<()> {
        let conn = self.connect()?;
        // Get user ID
        let user_id: i32 = conn.query_row(
            "SELECT id FROM users WHERE username = ?1",
            params![username],
            |row| row.get(0),
        ).map_err(|_| AppError::General("User not found".to_string()))?;

        conn.execute(
            "INSERT OR IGNORE INTO permissions (user_id, permission) VALUES (?1, ?2)",
            params![user_id, permission],
        ).map_err(|e| AppError::General(e.to_string()))?;
        Ok(())
    }

    fn has_permission(&self, user_id: i32, permission: &str) -> AppResult<bool> {
        let conn = self.connect()?;
        let count: i32 = conn.query_row(
            "SELECT count(*) FROM permissions WHERE user_id = ?1 AND permission = ?2",
            params![user_id, permission],
            |row| row.get(0),
        ).map_err(|e| AppError::General(e.to_string()))?;
        Ok(count > 0)
    }
    fn get_password_hash(&self, username: &str) -> AppResult<Option<String>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT password FROM users WHERE username = ?1")
            .map_err(|e| AppError::General(e.to_string()))?;
        
        let mut rows = stmt.query(params![username])
            .map_err(|e| AppError::General(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| AppError::General(e.to_string()))? {
            Ok(Some(row.get(0).map_err(|e| AppError::General(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    fn revoke_permission(&self, username: &str, permission: &str) -> AppResult<()> {
        let conn = self.connect()?;
        // Get user ID first
        let user_id: i32 = conn.query_row(
            "SELECT id FROM users WHERE username = ?1",
            params![username],
            |row| row.get(0),
        ).map_err(|_| AppError::General("User not found".to_string()))?;

        conn.execute(
            "DELETE FROM permissions WHERE user_id = ?1 AND permission = ?2",
            params![user_id, permission],
        ).map_err(|e| AppError::General(e.to_string()))?;
        Ok(())
    }

    fn list_permissions(&self, user_id: i32) -> AppResult<Vec<String>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT permission FROM permissions WHERE user_id = ?1")
            .map_err(|e| AppError::General(e.to_string()))?;
        
        let rows = stmt.query_map(params![user_id], |row| row.get(0))
            .map_err(|e| AppError::General(e.to_string()))?;

        let mut permissions = Vec::new();
        for perm in rows {
            permissions.push(perm.map_err(|e| AppError::General(e.to_string()))?);
        }
        Ok(permissions)
    }

    fn delete(&self, user_id: i32) -> AppResult<()> {
        let conn = self.connect()?;
        // Delete permissions first (though CASCADE should handle it)
        conn.execute(
            "DELETE FROM permissions WHERE user_id = ?1",
            params![user_id],
        ).map_err(|e| AppError::General(e.to_string()))?;
        
        // Delete user
        conn.execute(
            "DELETE FROM users WHERE id = ?1",
            params![user_id],
        ).map_err(|e| AppError::General(e.to_string()))?;
        Ok(())
    }

    fn update_password(&self, user_id: i32, password_hash: &str) -> AppResult<()> {
        let conn = self.connect()?;
        conn.execute(
            "UPDATE users SET password = ?1 WHERE id = ?2",
            params![password_hash, user_id],
        ).map_err(|e| AppError::General(e.to_string()))?;
        Ok(())
    }
}

//! Secure Credential Storage Module

//! Stores Google OAuth credentials in Windows Credential Manager instead of plain text files.
//! This provides OS-level security - credentials are encrypted and tied to the current user.

use keyring::Entry;

const SERVICE_NAME: &str = "VaultConsole-GDrive";
const CLIENT_SECRET_USER: &str = "ClientSecret";
const OAUTH_TOKEN_USER: &str = "OAuthToken";

/// Store the client secret JSON in Windows Credential Manager
pub fn store_client_secret(data: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, CLIENT_SECRET_USER)
        .map_err(|e| format!("Failed to create credential entry: {}", e))?;
    entry.set_password(data)
        .map_err(|e| format!("Failed to store client secret: {}", e))
}

/// Retrieve the client secret JSON from Windows Credential Manager
pub fn get_client_secret() -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, CLIENT_SECRET_USER)
        .map_err(|e| format!("Failed to create credential entry: {}", e))?;
    
    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to retrieve client secret: {}", e)),
    }
}

/// Store the OAuth token JSON in Windows Credential Manager
pub fn store_oauth_token(data: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, OAUTH_TOKEN_USER)
        .map_err(|e| format!("Failed to create credential entry: {}", e))?;
    entry.set_password(data)
        .map_err(|e| format!("Failed to store OAuth token: {}", e))
}

/// Retrieve the OAuth token JSON from Windows Credential Manager
pub fn get_oauth_token() -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, OAUTH_TOKEN_USER)
        .map_err(|e| format!("Failed to create credential entry: {}", e))?;
    
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to retrieve OAuth token: {}", e)),
    }
}

/// Delete all stored credentials
#[allow(dead_code)]
pub fn delete_all_credentials() -> Result<(), String> {
    let secret_entry = Entry::new(SERVICE_NAME, CLIENT_SECRET_USER)
        .map_err(|e| format!("Failed to access credential entry: {}", e))?;
    let token_entry = Entry::new(SERVICE_NAME, OAUTH_TOKEN_USER)
        .map_err(|e| format!("Failed to access credential entry: {}", e))?;
    
    // Ignore NoEntry errors since the credential might not exist
    let _ = secret_entry.delete_password();
    let _ = token_entry.delete_password();
    
    Ok(())
}

/// Migrate existing plain-text credential files to Windows Credential Manager
/// Returns true if any migration occurred
pub fn migrate_from_files() -> Result<bool, String> {
    let mut migrated = false;
    
    // Migrate client_secret.json
    let secret_paths = ["client_secret.json", "../client_secret.json"];
    for path in secret_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            // Verify it's already stored or store it
            if get_client_secret()?.is_none() {
                store_client_secret(&content)?;
                println!("[INFO] Migrated client secret from {} to Windows Credential Manager", path);
                migrated = true;
            }
            // Delete the plain text file after successful storage
            if let Err(e) = std::fs::remove_file(path) {
                println!("[WARN] Could not delete {}: {} (please delete manually for security)", path, e);
            } else {
                println!("[INFO] Deleted plain-text file: {}", path);
            }
        }
    }
    
    // Migrate gdrive_fuse_token.json
    let token_paths = ["gdrive_fuse_token.json", "../gdrive_fuse_token.json"];
    for path in token_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if get_oauth_token()?.is_none() {
                store_oauth_token(&content)?;
                println!("[INFO] Migrated OAuth token from {} to Windows Credential Manager", path);
                migrated = true;
            }
            if let Err(e) = std::fs::remove_file(path) {
                println!("[WARN] Could not delete {}: {} (please delete manually for security)", path, e);
            } else {
                println!("[INFO] Deleted plain-text file: {}", path);
            }
        }
    }
    
    Ok(migrated)
}

// ============================================================================
// Token file helpers for yup-oauth2 integration  
// Since TokenStorage trait has complex lifetime requirements, we use a temp file approach
// ============================================================================

use std::path::PathBuf;

/// Get a secure temp path for the token file
fn get_token_temp_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("vault_console_gdrive_token.json");
    path
}

/// Prepares a token file for yup-oauth2 by:
/// 1. If token exists in Credential Manager, write it to temp file
/// 2. Returns path to temp file
pub fn prepare_token_file() -> PathBuf {
    let temp_path = get_token_temp_path();
    
    // If we have a token stored in Credential Manager, write it to the temp file
    if let Ok(Some(token_json)) = get_oauth_token() {
        let _ = std::fs::write(&temp_path, token_json);
        println!("[INFO] Loaded OAuth token from secure storage");
    }
    
    temp_path
}

/// Save the token from temp file to Credential Manager and clean up
pub fn save_and_cleanup_token_file() {
    let temp_path = get_token_temp_path();
    
    if temp_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&temp_path) {
            // Store in Credential Manager
            if let Err(e) = store_oauth_token(&content) {
                eprintln!("[WARN] Could not save token to secure storage: {}", e);
            }
        }
        
        // Delete the temp file
        let _ = std::fs::remove_file(&temp_path);
    }
}


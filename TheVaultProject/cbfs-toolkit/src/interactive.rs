/*
 * CBFS Vault 2024 Rust Edition - Vault Console
 * Interactive Shell Module with RBAC
 */

use std::io::{self, Write};
use cbfsvault::cbvaultdrive::CBVaultDrive;
use cbfsvault::*;
use cbfsconnect::fuse::FUSE; // Import FUSE
use crate::config::*;
use std::sync::Arc;
use crate::domain::user::{User, UserRepository, AuthService, PermissionService, PERM_CREATE_USER, PERM_CREATE_VAULT, PERM_CLOUD_WRITE};
use crate::domain::vault::{VaultService, FileOpParams as DomainFileOpParams};

// ANSI Color Constants
const COLOR_TITLE: &str = "\x1b[95m";   // Bright Purple
const COLOR_TEXT: &str = "\x1b[96m";    // Bright Cyan
const COLOR_UNIQUE: &str = "\x1b[1;94m";  // Bold Bright Blue
const COLOR_ERROR: &str = "\x1b[91m";   // Bright Red
const COLOR_INPUT: &str = "\x1b[97m";   // Bright White
const COLOR_RESET: &str = "\x1b[0m";    // Reset

fn print_banner() {
    println!("{}", COLOR_TITLE);
    println!(r#"
  ____    _    _     _     ____    _    ____ _  __
 / ___|  / \  | |   | |   | __ )  / \  / ___| |/ /
| |     / _ \ | |   | |   |  _ \ / _ \| |   | ' / 
| |___ / ___ \| |___| |___| |_) / ___ \ |___| . \ 
 \____/_/   \_\_____|_____|____/_/   \_\____|_|\_\
                                                  
 _____ _____ ____ _   _ _   _  ___  _     ___   ____ ___ _____ ____  
|_   _| ____/ ___| | | | \ | |/ _ \| |   / _ \ / ___|_ _| ____/ ___| 
  | | |  _|| |   | |_| |  \| | | | | |  | | | | |  _ | ||  _| \___ \ 
  | | | |__| |___|  _  | |\  | |_| | |__| |_| | |_| || || |___ ___) |
  |_| |_____\____|_| |_|_| \_|\___/|_____\___/ \____|___|_____|____/ 
                                                                     
    "#);
    println!("{}\n", COLOR_RESET);
    println!("      {}═══ ＣＡＬＬＢＡＣＫ  ＦＩＬＥＳＹＳＴＥＭ  ＣＯＭＰＯＮＥＮＴＳ ═══{}", COLOR_TEXT, COLOR_RESET);
    println!("{}", COLOR_RESET);
}


/// Session state for interactive mode
pub struct Session {
    vault_service: Box<dyn VaultService>,
    drive: Option<&'static mut CBVaultDrive>, // Drive logic kept separate for now (Infrastructure limit)
    vault_path: String,
    password: String,
    mount_point: String,
    is_mounted: bool,
    use_sqlite: bool,
    user: User,
    permission_service: Arc<dyn PermissionService>,
}

impl Session {
    pub fn new(user: User, use_sqlite: bool, permission_service: Arc<dyn PermissionService>, vault_service: Box<dyn VaultService>) -> Self {
        Session {
            vault_service,
            drive: None,
            vault_path: String::new(),
            password: String::new(),
            mount_point: String::new(),
            is_mounted: false,
            use_sqlite,
            user,
            permission_service,
        }
    }
}

/// Parse command line into command and arguments
fn parse_command(input: &str) -> (String, Vec<String>) {
    let trimmed = input.trim();
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    
    for c in trimmed.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    
    if !current.is_empty() {
        parts.push(current);
    }
    
    if parts.is_empty() {
        return (String::new(), Vec::new());
    }
    
    let cmd = parts.remove(0).to_lowercase();
    (cmd, parts)
}

/// Display help information
fn show_help() {
    println!("\n{}Available Commands:{}", COLOR_TITLE, COLOR_RESET);
    
    println!("  {}open <vault>           {}Open an existing vault file", COLOR_INPUT, COLOR_TEXT);
    println!("  {}create <vault>         {}Create a new vault file (Requires Permission)", COLOR_INPUT, COLOR_TEXT);
    println!("  {}close                  {}Close current vault", COLOR_INPUT, COLOR_TEXT);
    println!("  {}password <pw>          {}Set password for vault operations", COLOR_INPUT, COLOR_TEXT);
    println!();
    println!("  {}add <files...>         {}Add files to the vault", COLOR_INPUT, COLOR_TEXT);
    println!("  {}list [pattern]         {}List vault contents (default: *)", COLOR_INPUT, COLOR_TEXT);
    println!("  {}extract <pattern> [out]{} Extract files (default output: current dir)", COLOR_INPUT, COLOR_TEXT);
    println!("  {}delete <pattern>       {}Delete files from vault", COLOR_INPUT, COLOR_TEXT);
    println!();
    println!("  {}mount <drive>          {}Mount vault as drive (e.g., Z:)", COLOR_INPUT, COLOR_TEXT);
    println!("                         {}NOTE: SQLite vaults cannot be mounted directly.", COLOR_TEXT);
    println!("                         {}Use 'convert-to-file' first to create a mountable vault.", COLOR_TEXT);
    println!("  {}unmount                {}Unmount the mounted drive", COLOR_INPUT, COLOR_TEXT);
    println!();
    println!("  {}convert-to-file <out>  {}Convert SQLite vault to regular vault file (for mounting)", COLOR_INPUT, COLOR_TEXT);
    println!();
    println!("  {}info                   {}Show current session info", COLOR_INPUT, COLOR_TEXT);
    println!("  {}help                   {}Show this help message", COLOR_INPUT, COLOR_TEXT);
    println!("  {}exit, quit             {}Return to Main Menu", COLOR_INPUT, COLOR_TEXT);
    println!("{}", COLOR_RESET);
}

/// Show session info
fn show_info(session: &Session) {
    println!("\n{}--- Session Info ---{}", COLOR_TITLE, COLOR_RESET);
    println!("{}", COLOR_TEXT);
    println!("User: {} ({})", session.user.username, if session.user.is_admin { "Admin" } else { "User" });
    if !session.vault_path.is_empty() {
        println!("Vault: {}", session.vault_path);
        println!("Status: {}", if session.vault_service.is_open() { "Open" } else { "Closed" });
    } else {
        println!("Vault: (none)");
    }
    
    if !session.password.is_empty() {
        println!("Password: (set)");
    } else {
        println!("Password: (not set)");
    }
    
    if session.is_mounted {
        println!("Mounted at: {}", session.mount_point);
    } else {
        println!("Mount: (not mounted)");
    }
    println!("--------------------\n");
}

/// Open a vault
fn cmd_open(session: &mut Session, args: &[String]) -> bool {
    if args.is_empty() {
        eprintln!("Usage: open <vault_path>");
        return false;
    }
    
    // Close existing vault if open
    if session.vault_service.is_open() {
        cmd_close(session);
    }
    
    let vault_path = &args[0];
    
    match session.vault_service.open(vault_path, &session.password) {
        Ok(_) => {
            session.vault_path = vault_path.clone();
            println!("Vault opened: {}", vault_path);
            true
        },
        Err(e) => {
            eprintln!("{}Failed to open vault: {}{}", COLOR_ERROR, e, COLOR_RESET);
            false
        }
    }
}

/// Create a new vault
fn cmd_create(session: &mut Session, args: &[String]) -> bool {
    // Permission Check - now using centralized PermissionService
    let has_perm = session.permission_service
        .check_permission(&session.user, PERM_CREATE_VAULT)
        .unwrap_or(false);

    if !has_perm {
        eprintln!("{}[ERROR] Access Denied: You do not have permission to create vaults.{}", COLOR_ERROR, COLOR_RESET);
        return false;
    }

    if args.is_empty() {
        eprintln!("Usage: create <vault_path>");
        return false;
    }
    
    // Close existing vault if open
    if session.vault_service.is_open() {
        cmd_close(session);
    }
    
    let vault_path = &args[0];

    match session.vault_service.create(vault_path, &session.password, session.use_sqlite) {
        Ok(_) => {
            session.vault_path = vault_path.clone();
            println!("Vault created: {}", vault_path);
            true
        },
        Err(e) => {
            eprintln!("{}Failed to create vault: {}{}", COLOR_ERROR, e, COLOR_RESET);
            false
        }
    }
}

/// Close current vault
fn cmd_close(session: &mut Session) -> bool {
    if session.is_mounted {
        cmd_unmount(session);
    }
    
    if session.vault_service.is_open() {
        let _ = session.vault_service.close();
        println!("Vault closed: {}", session.vault_path);
    }
    session.vault_path.clear();
    true
}

/// Set password
fn cmd_password(session: &mut Session, args: &[String]) -> bool {
    if args.is_empty() {
        session.password.clear();
        println!("Password cleared");
    } else {
        session.password = args[0].clone();
        println!("Password set");
    }
    true
}

/// Add files to vault
fn cmd_add(session: &mut Session, args: &[String]) -> bool {
    if !session.vault_service.is_open() {
        eprintln!("No vault is open. Use 'open' or 'create' first.");
        return false;
    }
    
    if args.is_empty() {
        eprintln!("Usage: add [-comp <0-9>] [-r] [-del] <file1> [file2] ...");
        return false;
    }
    
    let mut files: Vec<String> = Vec::new();
    let mut params = DomainFileOpParams::new();
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with("-") {
            match arg.as_str() {
                "-r" => params.recursive = true,
                "-del" => params.delete_after_archiving = true,
                "-y" => params.always_yes = true,
                "-lw" => params.lowercase = true,
                "-u" => params.uppercase = true,
                "-ad" => params.add_empty_dirs = true,
                val if val.starts_with("-comp") => {
                    // Handle -comp 9 or -comp9
                    let level = if val.len() > 5 {
                        // -comp9 format
                        val[5..].parse::<i32>().ok()
                    } else {
                        // -comp 9 format
                        if i + 1 < args.len() {
                            i += 1;
                            args[i].parse::<i32>().ok()
                        } else {
                            None
                        }
                    };
                    
                    if let Some(l) = level {
                        params.compression_level = l;
                    } else {
                        eprintln!("Invalid compression level");
                        return false;
                    }
                },
                _ => {
                    eprintln!("Unknown flag: {}", arg);
                    return false;
                }
            }
        } else {
            files.push(arg.clone());
        }
        i += 1;
    }
    
    if files.is_empty() {
        eprintln!("No files specified to add.");
        return false;
    }

    if let Err(e) = session.vault_service.add_files(&files, &mut params) {
        eprintln!("Failed to add files: {}", e);
        return false;
    }
    true
}

/// List vault contents
fn cmd_list(session: &Session, args: &[String]) -> bool {
    if !session.vault_service.is_open() {
        eprintln!("No vault is open. Use 'open' or 'create' first.");
        return false;
    }
    
    let pattern = if args.is_empty() { "*" } else { &args[0] };
    
    match session.vault_service.list_files(pattern) {
        Ok(entries) => {
             for entry in entries {
                 if entry.is_directory {
                     println!("{:<50} <DIR>", entry.name);
                 } else {
                     println!("{:<50} {:>15} bytes", entry.name, entry.size);
                 }
             }
        }
        Err(e) => {
             eprintln!("Failed to list files: {}", e);
             return false;
        }
    }
    true
}

/// Extract files from vault
fn cmd_extract(session: &Session, args: &[String]) -> bool {
    if !session.vault_service.is_open() {
        eprintln!("No vault is open. Use 'open' or 'create' first.");
        return false;
    }
    
    let pattern = if args.is_empty() { "*" } else { &args[0] };
    let output = if args.len() > 1 { &args[1] } else { "." };
    
    let params = DomainFileOpParams::new(); // Use domain params
    if let Err(e) = session.vault_service.extract_files(pattern, output, &params) {
        eprintln!("Failed to extract files: {}", e);
        return false;
    }
    true
}

/// Delete files from vault
fn cmd_delete(session: &Session, args: &[String]) -> bool {
    if !session.vault_service.is_open() {
        eprintln!("No vault is open. Use 'open' or 'create' first.");
        return false;
    }
    
    if args.is_empty() {
        eprintln!("Usage: delete <pattern>");
        return false;
    }
    
    if let Err(e) = session.vault_service.delete_files(&args[0], false) {
        eprintln!("Failed to delete files: {}", e);
        return false;
    }
    true
}


/// Convert SQLite vault to regular vault file for mounting
fn cmd_convert_to_file(session: &Session, args: &[String]) -> bool {
    if !session.use_sqlite {
        eprintln!("Current vault is not a SQLite vault. This command only works with SQLite vaults.");
        return false;
    }
    
    if !session.vault_service.is_open() {
        eprintln!("No vault is open. Use 'open' first.");
        return false;
    }

    if args.is_empty() {
        eprintln!("Usage: convert-to-file <output_vault_file>");
        return false;
    }
    
    let output_path = &args[0];
    if let Err(e) = session.vault_service.convert_to_file(output_path) {
        eprintln!("Conversion failed: {}", e);
        return false;
    }
    println!("Conversion complete.");
    true
}

/// Mount vault as drive
fn cmd_mount(session: &mut Session, args: &[String]) -> bool {
    if !session.vault_service.is_open() {
        eprintln!("No vault is open. Use 'open' or 'create' first.");
        return false;
    }
    
    if args.is_empty() {
        eprintln!("Usage: mount <drive_letter>");
        return false;
    }
    
    if session.is_mounted {
        eprintln!("Already mounted at {}. Use 'unmount' first.", session.mount_point);
        return false;
    }
    
    // Close the CBVault (Service) and use CBVaultDrive for mounting
    // Mounting requires the vault file to be CLOSED.
    if session.vault_service.is_open() {
        let _ = session.vault_service.close();
    }
    
    let drive = CBVaultDrive::new();

    if drive.initialize(GUID).is_err() {
        eprintln!("{}Failed to initialize drive{}", COLOR_ERROR, COLOR_RESET);
        drive.dispose();
        cmd_open(session, &[session.vault_path.clone()]);
        return false;
    }
    
    let vault_file_path = if session.use_sqlite {
        let db_path = if session.vault_path.ends_with(".db") {
            session.vault_path.clone()
        } else {
            format!("{}.db", session.vault_path.trim_end_matches(".vlt").trim_end_matches(".vault"))
        };
        crate::sqlite_vault::setup_drive_callbacks(drive, Some(&db_path));
        db_path
    } else {
        session.vault_path.clone()
    }; // Note: Ignoring absolute path conversion for brevity, assume path is valid

    let _ = drive.set_vault_file(&vault_file_path);
    if !session.password.is_empty() {
        let _ = drive.set_vault_password(&session.password);
    }
    
    let journal_mode = if session.use_sqlite { VAULT_JM_NONE } else { VAULT_JM_METADATA };

    if drive.open_vault(VAULT_OM_OPEN_EXISTING, journal_mode).is_err() {
        eprintln!("Failed to open vault for mounting");
        drive.dispose();
        cmd_open(session, &[session.vault_path.clone()]);
        return false;
    }
    
    let mount_point = &args[0];
    if drive.add_mounting_point(mount_point, STGMP_SIMPLE, 0).is_err() {
        eprintln!("Failed to mount");
        let _ = drive.close_vault(false);
        drive.dispose();
        cmd_open(session, &[session.vault_path.clone()]);
        return false;
    }
    
    session.drive = Some(drive);
    session.mount_point = mount_point.clone();
    session.is_mounted = true;
    println!("Vault mounted at {}", mount_point);
    true
}

/// Unmount drive
fn cmd_unmount(session: &mut Session) -> bool {
    if !session.is_mounted {
        eprintln!("Vault is not mounted.");
        return false;
    }
    
    if let Some(drive) = session.drive.take() {
        let _ = drive.remove_mounting_point(0, &session.mount_point, STGMP_SIMPLE, 0);
        let _ = drive.close_vault(false);
        drive.dispose();
    }
    
    println!("Vault unmounted from {}", session.mount_point);
    session.mount_point.clear();
    session.is_mounted = false;
    
    if !session.vault_path.is_empty() {
        let vault_path = session.vault_path.clone();
        cmd_open(session, &[vault_path]);
    }
    
    true
}

/// Login authentication
fn login_loop(auth_service: &dyn AuthService) -> Option<User> {
    print_banner();
    println!("\n{}╔════════════════════════════════════════╗", COLOR_TITLE);
    println!("║      CBFS Vault Console - Login        ║");
    println!("╚════════════════════════════════════════╝{}", COLOR_RESET);
    println!("\n{}(Type '{}{}{}' to quit the application)\n{}", COLOR_TEXT, COLOR_TITLE, "exit", COLOR_TEXT, COLOR_RESET);
    
    loop {
        print!("{}Username: {}", COLOR_TEXT, COLOR_INPUT);
        let _ = io::stdout().flush();
        let mut username = String::new();
        // Reset color if we return or fail, but mostly we rely on the loop to set it back for prompts
        if io::stdin().read_line(&mut username).is_err() { return None; }
        
        // Restore interface color immediately after input
        print!("{}", COLOR_TEXT);
        
        let username = username.trim();
        if username.is_empty() { continue; }

        if username.eq_ignore_ascii_case("exit") || username.eq_ignore_ascii_case("quit") {
            return None;
        }
        
        print!("{}Password: {}", COLOR_TEXT, COLOR_INPUT);
        let _ = io::stdout().flush();
        let mut password = String::new();
        if io::stdin().read_line(&mut password).is_err() { return None; }
        
        // Restore interface color
        print!("{}", COLOR_TEXT);
        
        let password = password.trim();
        
        
        match auth_service.authenticate(username, password) {
            Ok(Some(user)) => {
                // Success: Green message, Username in Input color (White) for distinction
                println!("\n{}✓ Login successful! Welcome, {}{}.{}", COLOR_UNIQUE, COLOR_INPUT, user.username, COLOR_RESET);
                println!("{}", COLOR_RESET);
                return Some(user);
            }
            Ok(None) => println!("{}✗ Invalid credentials. Try again.{}\n", COLOR_ERROR, COLOR_RESET),
            Err(e) => {
                eprintln!("{}Database error: {}{}", COLOR_ERROR, e, COLOR_RESET);
                return None;
            }
        }
    }
}

fn input_line(prompt: &str) -> String {
    print!("{}{}{}", COLOR_TEXT, prompt, COLOR_INPUT);
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    print!("{}", COLOR_TEXT); // Switch back to text color
    input.trim().to_string()
}

enum MenuResult {
    Stay,
    Logout,
    ExitApp,
    StartVaultLoop(bool),
}

/// User Management Submenu for Admins
fn user_management_submenu(current_user: &User, user_repo: &dyn UserRepository, auth_service: &dyn AuthService) {
    loop {
        println!("\n{}=== User Accounts Management ==={}", COLOR_TITLE, COLOR_RESET);
        println!("{}", COLOR_TEXT);
        println!("1. Create New User");
        println!("2. Delete User");
        println!("3. List User Permissions");
        println!("4. Revoke Permission");
        println!("5. Reset User Password");
        println!("6. Back to Main Menu\n");
        
        let prompt = format!("{}Select an option: ", COLOR_UNIQUE);
        let choice = input_line(&prompt);
        
        match choice.as_str() {
            "1" => {
                // Create New User
                let username = input_line("New Username: ");
                let password = input_line("New Password: ");
                if !username.is_empty() && !password.is_empty() {
                    match auth_service.hash_password(&password) {
                        Ok(hash) => {
                            match user_repo.create(&username, &hash, false) {
                                Ok(_) => println!("[SUCCESS] User '{}' created successfully.", username),
                                Err(e) => eprintln!("[ERROR] Failed to create user: {}", e),
                            }
                        },
                        Err(e) => eprintln!("[ERROR] Password hashing failed: {}", e),
                    }
                } else {
                    println!("[ERROR] Username and password cannot be empty.");
                }
            }
            "2" => {
                // Delete User
                let username = input_line("Username to delete: ");
                if username.is_empty() {
                    println!("[ERROR] Username cannot be empty.");
                    continue;
                }
                
                // Check if user exists
                match user_repo.find_by_username(&username) {
                    Ok(Some(target_user)) => {
                        // Safety: Cannot delete yourself
                        if target_user.id == current_user.id {
                            println!("[ERROR] You cannot delete your own account!");
                            continue;
                        }
                        
                        // Safety: Cannot delete admins (optional but recommended)
                        if target_user.is_admin {
                            println!("[ERROR] Cannot delete admin accounts. Demote first if needed.");
                            continue;
                        }
                        
                        // Confirmation
                        let confirm = input_line(&format!("Are you sure you want to delete user '{}'? (yes/no): ", username));
                        if confirm.to_lowercase() == "yes" {
                            match user_repo.delete(target_user.id) {
                                Ok(_) => println!("[SUCCESS] User '{}' has been deleted.", username),
                                Err(e) => eprintln!("[ERROR] Failed to delete user: {}", e),
                            }
                        } else {
                            println!("[INFO] Deletion cancelled.");
                        }
                    }
                    Ok(None) => println!("[ERROR] User '{}' not found.", username),
                    Err(e) => eprintln!("[ERROR] Failed to look up user: {}", e),
                }
            }
            "3" => {
                // List User Permissions
                let username = input_line("Username: ");
                if username.is_empty() {
                    println!("[ERROR] Username cannot be empty.");
                    continue;
                }
                
                match user_repo.find_by_username(&username) {
                    Ok(Some(target_user)) => {
                        if target_user.is_admin {
                            println!("[INFO] User '{}' is an Admin (has ALL permissions).", username);
                        } else {
                            match user_repo.list_permissions(target_user.id) {
                                Ok(perms) => {
                                    if perms.is_empty() {
                                        println!("[INFO] User '{}' has no explicit permissions.", username);
                                    } else {
                                        println!("\n+---------------------------+");
                                        println!("| Permissions for '{}'", username);
                                        println!("+---------------------------+");
                                        for perm in perms {
                                            println!("| - {}", perm);
                                        }
                                        println!("+---------------------------+\n");
                                    }
                                }
                                Err(e) => eprintln!("[ERROR] Failed to list permissions: {}", e),
                            }
                        }
                    }
                    Ok(None) => println!("[ERROR] User '{}' not found.", username),
                    Err(e) => eprintln!("[ERROR] Failed to look up user: {}", e),
                }
            }
            "4" => {
                // Revoke Permission
                let username = input_line("Username: ");
                if username.is_empty() {
                    println!("[ERROR] Username cannot be empty.");
                    continue;
                }
                
                match user_repo.find_by_username(&username) {
                    Ok(Some(target_user)) => {
                        if target_user.is_admin {
                            println!("[INFO] User '{}' is an Admin. Cannot revoke permissions from admins.", username);
                        } else {
                            // Show current permissions first
                            match user_repo.list_permissions(target_user.id) {
                                Ok(perms) => {
                                    if perms.is_empty() {
                                        println!("[INFO] User '{}' has no permissions to revoke.", username);
                                    } else {
                                        println!("[INFO] Current permissions: {:?}", perms);
                                        let perm = input_line("Permission to revoke: ");
                                        if perm.is_empty() {
                                            println!("[ERROR] Permission cannot be empty.");
                                        } else if !perms.contains(&perm) {
                                            println!("[INFO] User '{}' does not have '{}' permission.", username, perm);
                                        } else {
                                            match user_repo.revoke_permission(&username, &perm) {
                                                Ok(_) => println!("[SUCCESS] Permission '{}' revoked from user '{}'.", perm, username),
                                                Err(e) => eprintln!("[ERROR] Failed to revoke permission: {}", e),
                                            }
                                        }
                                    }
                                }
                                Err(e) => eprintln!("[ERROR] Failed to list permissions: {}", e),
                            }
                        }
                    }
                    Ok(None) => println!("[ERROR] User '{}' not found.", username),
                    Err(e) => eprintln!("[ERROR] Failed to look up user: {}", e),
                }
            }
            "5" => {
                // Reset User Password
                let username = input_line("Username: ");
                if username.is_empty() {
                    println!("[ERROR] Username cannot be empty.");
                    continue;
                }
                
                match user_repo.find_by_username(&username) {
                    Ok(Some(target_user)) => {
                        let new_password = input_line("New Password: ");
                        if new_password.is_empty() {
                            println!("[ERROR] Password cannot be empty.");
                            continue;
                        }
                        
                        let confirm_password = input_line("Confirm New Password: ");
                        if new_password != confirm_password {
                            println!("[ERROR] Passwords do not match.");
                            continue;
                        }
                        
                        match auth_service.hash_password(&new_password) {
                            Ok(hash) => {
                                match user_repo.update_password(target_user.id, &hash) {
                                    Ok(_) => println!("[SUCCESS] Password for '{}' has been reset.", username),
                                    Err(e) => eprintln!("[ERROR] Failed to update password: {}", e),
                                }
                            }
                            Err(e) => eprintln!("[ERROR] Password hashing failed: {}", e),
                        }
                    }
                    Ok(None) => println!("[ERROR] User '{}' not found.", username),
                    Err(e) => eprintln!("[ERROR] Failed to look up user: {}", e),
                }
            }
            "6" | "back" | "exit" | "q" => {
                println!("[INFO] Returning to Main Menu...");
                break;
            }
            _ => println!("Invalid option."),
        }
    }
}

fn main_menu(user: &User, user_repo: &dyn UserRepository, permission_service: &dyn PermissionService, auth_service: &dyn AuthService, cloud_drive: &mut Option<&'static mut FUSE>) -> MenuResult {
    println!("\n{}Welcome to the Callback Virtual Filesystem, {}{}{}.{}", COLOR_TITLE, COLOR_ERROR, if user.is_admin { "Admin" } else { &user.username }, COLOR_TITLE, COLOR_RESET);
    println!("\n{}Instructions:{}", COLOR_TITLE, COLOR_RESET);
    println!("{}", COLOR_TEXT);
    println!("1. List Users");
    if permission_service.check_permission(user, PERM_CREATE_USER).unwrap_or(false) {
        println!("2. User Accounts Management (Requires {} permission)", PERM_CREATE_USER);
    } else {
        println!("2. Create User (Locked)");
    }
    
    if user.is_admin {
        println!("3. Grant Permission (Requires Admin)");
        println!("4. Make User Admin (Requires Admin)");
    }
    println!("5. Start Up and Choose CBFS Mode");
    
    // Check if cloud drive is mounted
    let is_cloud_mounted = cloud_drive.is_some();
    if is_cloud_mounted {
        println!("   [INFO] Google Drive is currently MOUNTED.");
    }
    
    println!("6. Logout");
    println!("7. Exit Program\n");
    
    // Apply unique color to this specific prompt by manually constructing the string with the color code
    // The input_line helper will prepend COLOR_TEXT, but our specific colored string will override it for the text part.
    let prompt_text = format!("{}Select an option: ", COLOR_UNIQUE);
    let choice = input_line(&prompt_text);
    
    match choice.as_str() {
        "1" => {
            match user_repo.list_all() {
                Ok(users) => {
                    println!("\n+----+----------------------+-------+");
                    println!("| ID | Username             | Admin |");
                    println!("+----+----------------------+-------+");
                    for u in users {
                        println!("| {:<2} | {}{:<20}{} | {:<5} |", u.id, COLOR_INPUT, u.username, COLOR_TEXT, u.is_admin);
                    }
                    println!("+----+----------------------+-------+\n");
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "2" => {
            if !permission_service.check_permission(user, PERM_CREATE_USER).unwrap_or(false) {
                println!("Access Denied.");
            } else {
                user_management_submenu(user, user_repo, auth_service);
            }
        }
        "3" => {
            if !user.is_admin {
                println!("Access Denied.");
            } else {
                let username = input_line("Username: ");
                if username.is_empty() {
                    println!("[ERROR] Username cannot be empty.");
                } else {
                    // First check if user exists
                    match user_repo.find_by_username(&username) {
                        Ok(Some(target_user)) => {
                            // Check if target user is admin - admins have all permissions
                            if target_user.is_admin {
                                println!("[INFO] User '{}' is an Admin and already has all permissions.", username);
                            } else {
                                let perm = input_line("Permission (e.g., CreateUser, CreateVault, CloudWrite): ");
                                if perm.is_empty() {
                                    println!("[ERROR] Permission cannot be empty.");
                                } else {
                                    // Check if user already has this permission
                                    match user_repo.has_permission(target_user.id, &perm) {
                                        Ok(true) => {
                                            println!("[INFO] User '{}' already has the '{}' permission.", username, perm);
                                        }
                                        Ok(false) => {
                                            // Grant permission
                                            match user_repo.grant_permission(&username, &perm) {
                                                Ok(_) => println!("[SUCCESS] Permission '{}' granted to user '{}'.", perm, username),
                                                Err(e) => eprintln!("[ERROR] Failed to grant permission: {}", e),
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("[ERROR] Failed to check permissions: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        Ok(None) => {
                            println!("[ERROR] User '{}' does not exist.", username);
                        }
                        Err(e) => {
                            eprintln!("[ERROR] Failed to look up user: {}", e);
                        }
                    }
                }
            }
        }
        "4" => {
            if !user.is_admin {
                println!("Access Denied.");
            } else {
                let username = input_line("Username: ");
                if username.is_empty() {
                    println!("[ERROR] Username cannot be empty.");
                } else {
                    // First check if user exists
                    match user_repo.find_by_username(&username) {
                        Ok(Some(target_user)) => {
                            // Check if already admin
                            if target_user.is_admin {
                                println!("[INFO] User '{}' is already an Admin.", username);
                            } else {
                                // Make admin
                                match user_repo.make_admin(&username) {
                                    Ok(_) => println!("[SUCCESS] User '{}' is now an Admin.", username),
                                    Err(e) => eprintln!("[ERROR] Failed to make user admin: {}", e),
                                }
                            }
                        }
                        Ok(None) => {
                            println!("[ERROR] User '{}' does not exist.", username);
                        }
                        Err(e) => {
                            eprintln!("[ERROR] Failed to look up user: {}", e);
                        }
                    }
                }
            }
        }
        "5" => {
            println!("{}=== Select Vault Mode ==={}", COLOR_INPUT, COLOR_TEXT);
            println!("1. Standard Vault (Local File)");
            println!("2. SQLite Backend (Callback Mode)");
            
            if is_cloud_mounted {
                println!("3. Unmount Google Drive Connection.");
            } else {
                println!("3. Use Google Drive Cloud as A Drive Connection Virtual Disk.");
            }
            
            let mode_prompt = format!("{}Select mode [1-3]: ", COLOR_UNIQUE);
            let mode = input_line(&mode_prompt);
            if mode == "3" {
                if is_cloud_mounted {
                    // Unmount logic
                    if let Some(fuse) = cloud_drive.take() {
                         println!("[INFO] Unmounting Google Drive...");
                         // Save OAuth token to Credential Manager before unmount
                         crate::credential_store::save_and_cleanup_token_file();
                         if let Err(e) = fuse.unmount() {
                             eprintln!("[ERROR] Unmount failed: {}", e.get_code());
                             // Put it back? Or dispose? better dispose and warn.
                             fuse.dispose();
                         } else {
                             println!("[INFO] Unmount successful.");
                             fuse.dispose();
                         }
                    }
                } else {
                    println!("[INFO] Initializing Google Drive (FUSE-based)...");
                
                // First check if client secret is stored in Windows Credential Manager
                let mut secret_path = String::new();
                let secret_from_credential_manager = match crate::credential_store::get_client_secret() {
                    Ok(Some(_)) => {
                        println!("[INFO] Client secret loaded from Windows Credential Manager");
                        true
                    }
                    _ => false
                };
                
                if !secret_from_credential_manager {
                    // Auto-detect client_secret.json using a flexible search
                    let search_paths = [".", ".."];
                    for path in search_paths {
                        if let Ok(entries) = std::fs::read_dir(path) {
                            for entry in entries.flatten() {
                                let name = entry.file_name().to_string_lossy().to_string();
                                if name.starts_with("client_secret") && name.ends_with(".json") {
                                    secret_path = entry.path().to_string_lossy().to_string();
                                    break;
                                }
                            }
                        }
                        if !secret_path.is_empty() { break; }
                    }

                    if !secret_path.is_empty() {
                        println!("[INFO] Found client secret file: {}", secret_path);
                        // Read the file and store in Credential Manager
                        match std::fs::read_to_string(&secret_path) {
                            Ok(content) => {
                                if let Err(e) = crate::credential_store::store_client_secret(&content) {
                                    eprintln!("[WARN] Could not store in Credential Manager: {}", e);
                                } else {
                                    println!("[INFO] Client secret stored securely in Windows Credential Manager");
                                    // Delete the plain-text file
                                    if let Err(e) = std::fs::remove_file(&secret_path) {
                                        println!("[WARN] Could not delete {}: {} (please delete manually)", secret_path, e);
                                    } else {
                                        println!("[INFO] Deleted plain-text file: {}", secret_path);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("[ERROR] Failed to read client secret: {}", e);
                                secret_path.clear();
                            }
                        }
                    } else {
                        // Prompt user for the path
                        secret_path = input_line("Enter path to client_secret.json: ");
                        if !secret_path.is_empty() {
                            match std::fs::read_to_string(&secret_path) {
                                Ok(content) => {
                                    if let Err(e) = crate::credential_store::store_client_secret(&content) {
                                        eprintln!("[WARN] Could not store in Credential Manager: {}", e);
                                    } else {
                                        println!("[INFO] Client secret stored securely in Windows Credential Manager");
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[ERROR] Failed to read file: {}", e);
                                    secret_path.clear();
                                }
                            }
                        }
                    }
                }

                // Check if we have credentials (either from CM or just loaded)
                let have_credentials = secret_from_credential_manager || 
                    crate::credential_store::get_client_secret().ok().flatten().is_some();
                
                if !have_credentials {
                    println!("[ERROR] No client secret available. Cannot connect to Google Drive.");
                } else {
                    #[cfg(target_os = "windows")]
                    let default_mount = "Z:";
                    #[cfg(not(target_os = "windows"))]
                    let default_mount = "/mnt/gdrive";

                    let prompt = format!("Enter mount point [Default: {}]: ", default_mount);
                    let mut mount_point = input_line(&prompt);
                    
                    if mount_point.is_empty() {
                        mount_point = default_mount.to_string();
                        println!("[INFO] Using default mount point: {}", mount_point);
                    }

                    // Check if user has write permission
                    let allow_write = permission_service.check_permission(user, PERM_CLOUD_WRITE).unwrap_or(false);
                    if allow_write {
                        println!("[INFO] You have WRITE access to Google Drive");
                    } else {
                        println!("[INFO] READ-ONLY access (no CloudWrite permission)");
                    }
                    
                    // We need to pass a credential file path for now - create a temp file from CM
                    // For now, use a special marker path that fuse_gdrive will understand
                    let credential_source = if secret_from_credential_manager || secret_path.is_empty() {
                        "__CREDENTIAL_MANAGER__".to_string()
                    } else {
                        secret_path.clone()
                    };
                    
                    // Call non-blocking mount
                    match crate::fuse_gdrive::mount_fuse_gdrive(&credential_source, &mount_point, allow_write) {
                        Ok(fuse) => {
                             *cloud_drive = Some(fuse);
                             println!("Check Explorer/Terminal for access.");
                             // Return to menu
                        },
                        Err(e) => {
                             eprintln!("Failed to mount Google Drive. Error code: {}", e);
                        }
                    }
                }

                }
            } else {
                let use_sqlite = mode == "2";
                if use_sqlite {
                    println!("[INFO] Initializing SQLite Backend...");
                } else {
                    println!("[INFO] Standard Mode Selected.");
                }
                return MenuResult::StartVaultLoop(use_sqlite);
            }
        }
        "6" => return MenuResult::Logout,
        "7" => return MenuResult::ExitApp,
        _ => println!("Invalid option."),
    }
    
    MenuResult::Stay
}

/// Main vault shell loop
fn vault_shell_loop(user: User, use_sqlite: bool, permission_service: Arc<dyn PermissionService>, vault_service: Box<dyn VaultService>) {
    println!("\n{}Interactive Mode - Type '{}{}{}' for commands, '{}{}{}' to quit{}", 
             COLOR_INPUT, COLOR_TITLE, "help", COLOR_INPUT, COLOR_TITLE, "exit", COLOR_INPUT, COLOR_RESET);
    if use_sqlite {
        println!("(SQLite Backend Enabled)\n");
    }
    
    let mut session = Session::new(user, use_sqlite, permission_service, vault_service);
    let mut input = String::new();
    
    loop {
        // Print prompt
        let prompt = if !session.vault_path.is_empty() {
            let name = std::path::Path::new(&session.vault_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| session.vault_path.clone());
            format!("vault [{}]> ", name)
        } else {
            "vault> ".to_string()
        };
        
        // Use prompt + input color
        print!("{}{}{}", COLOR_UNIQUE, prompt, COLOR_INPUT);
        let _ = io::stdout().flush();
        
        // Read input
        input.clear();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        print!("{}", COLOR_TEXT); // Restore text color
        
        let (cmd, args) = parse_command(&input);
        
        if cmd.is_empty() {
            continue;
        }
        
        match cmd.as_str() {
            "help" | "h" | "?" => show_help(),
            "info" => show_info(&session),
            "open" => { cmd_open(&mut session, &args); }
            "create" | "new" => { cmd_create(&mut session, &args); }
            "close" => { cmd_close(&mut session); }
            "password" | "pw" => { cmd_password(&mut session, &args); }
            "add" => { cmd_add(&mut session, &args); }
            "list" | "ls" | "dir" => { cmd_list(&session, &args); }
            "extract" | "x" => { cmd_extract(&session, &args); }
            "delete" | "del" | "rm" => { cmd_delete(&session, &args); }
            "convert-to-file" | "convert" => { cmd_convert_to_file(&session, &args); }
            "mount" => { cmd_mount(&mut session, &args); }
            "unmount" | "umount" => { cmd_unmount(&mut session); }
            "exit" | "quit" | "q" => {
                // Cleanup
                if session.is_mounted {
                    cmd_unmount(&mut session);
                }
                if session.vault_service.is_open() {
                    cmd_close(&mut session);
                }
                println!("Returning to Main Menu...");
                break;
            }
            "clear" | "cls" => {
                print!("\x1B[2J\x1B[1;1H");
                let _ = io::stdout().flush();
            }
            _ => {
                eprintln!("Unknown command: '{}'. Type 'help' for available commands.", cmd);
            }
        }
    }
}


/// Main entry point for interactive module


pub fn run(
    auth_service: Arc<dyn AuthService>, 
    user_repo: Arc<dyn UserRepository>,
    permission_service: Arc<dyn PermissionService>,
    vault_factory: Box<dyn Fn() -> Box<dyn VaultService>>
) {
    // Set initial text color
    print!("{}", COLOR_TEXT);

    // Initialize DB and Check for Admin
    let admin_exists = match user_repo.init() {
        Ok(exists) => exists,
        Err(e) => {
            eprintln!("Fatal Error: Could not initialize user database: {}", e);
            return;
        }
    };
    
    if !admin_exists {
        println!("{}\n", COLOR_RESET);
        println!("      {}═══ ＣＡＬＬＢＡＣＫ  ＦＩＬＥＳＹＳＴＥＭ  ＣＯＭＰＯＮＥＮＴＳ ═══{}", COLOR_TEXT, COLOR_RESET);
        println!("{}", COLOR_RESET);
        println!("\n╔════════════════════════════════════════╗");
        println!("║             Admin Creation             ║");
        println!("╚════════════════════════════════════════╝\n");
        println!("No admin account found. Please create an admin account now.\n");
        
        loop {
            let username = input_line("Enter Admin Username: ");
            if username.is_empty() { continue; }
            let password = input_line("Enter Admin Password: ");
            if password.is_empty() { continue; }
            
            // Hash password using auth_service
            let password_hash = match auth_service.hash_password(&password) {
                Ok(hash) => hash,
                Err(e) => {
                    eprintln!("Hashing failed: {}", e);
                    continue;
                }
            };
             
            if let Err(e) = user_repo.create(&username, &password_hash, true) {
                 eprintln!("Failed to create admin: {}", e);
            } else {
                println!("Admin account created successfully.\n");
                break;
            }
        }
    }

    loop {
        // Cloud Drive session persistence
        let mut cloud_drive: Option<&'static mut FUSE> = None;
        
        if let Some(user) = login_loop(auth_service.as_ref()) {
            loop {
                match main_menu(&user, user_repo.as_ref(), permission_service.as_ref(), auth_service.as_ref(), &mut cloud_drive) {
                    MenuResult::Stay => continue,
                    MenuResult::Logout => {
                        // Ensure unmount on logout
                        if let Some(fuse) = cloud_drive.take() {
                             println!("[INFO] Auto-unmounting Google Drive...");
                             let _ = fuse.unmount();
                             fuse.dispose();
                        }
                        break;
                    }, 
                    MenuResult::ExitApp => {
                         // Ensure unmount on exit
                        if let Some(fuse) = cloud_drive.take() {
                             println!("[INFO] Auto-unmounting Google Drive...");
                             let _ = fuse.unmount();
                             fuse.dispose();
                        }
                        return;
                    },
                    MenuResult::StartVaultLoop(use_sqlite) => {
                         let vault_service = vault_factory();
                         vault_shell_loop(user.clone(), use_sqlite, permission_service.clone(), vault_service);
                    }
                }
            }
        } else {
            break;
        }
    }
}


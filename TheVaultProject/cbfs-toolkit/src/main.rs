/*
 * CBFS Vault 2024 Rust Edition - Vault Console
 * Professional command-line vault management application
 *
 * This application integrates CBFS Vault SDK to provide comprehensive
 * vault management including creation, mounting, and file operations.
 *
 * www.callback.com/cbfsvault
 */

mod config;
mod utils;
mod vault_manager;
mod mount_manager;
mod file_ops;
mod interactive;
// mod memory_vault; // Removed

mod sqlite_vault;
mod fuse_common;
mod fuse_gdrive;
mod errors;
mod credential_store;

mod domain;
mod infrastructure;

use std::env;
use std::sync::Arc;
use cbfsvault::*;
use config::*;
use crate::domain::vault::{VaultService, FileOpParams};
use crate::infrastructure::sqlite_user_repo::SQLiteUserRepository;
use crate::infrastructure::argon_auth_service::ArgonAuthService;
use crate::infrastructure::cbfs_vault_service::CbfsVaultService;
use crate::infrastructure::permission_service::StandardPermissionService;
use utils::*;
use vault_manager::*;
use mount_manager::*;
use file_ops::*;

/// Application parameters
struct AppParams {
    // Vault parameters
    vault_path: String,
    create_new: bool,
    overwrite: bool,
    page_size: i32,
    password: String,
    fixed_size: i64,
    
    // Mount parameters
    mount_point: String,
    readonly: bool,
    local: bool,
    network: bool,
    icon_path: String,
    logo: String,
    unmount_on_terminate: bool,
    proc_name: String,
    proc_id: i32,
    
    // File operation parameters
    operation: char, // 'a'=add, 'd'=delete, 'e'=extract, 'x'=extract with path, 's'=list, 'v'=test
    files: Vec<String>,
    recursive: bool,
    compression: i32,
    delete_after: bool,
    lowercase: bool,
    uppercase: bool,
    overwrite_files: bool,
    always_yes: bool,
    output_path: String,
    add_empty_dirs: bool,
    
    // Driver operations
    driver_path: String,
    check_driver: bool,
    
    // SQLite mode
    use_sqlite: bool,
}

impl AppParams {
    fn new() -> Self {
        AppParams {
            vault_path: String::new(),
            create_new: false,
            overwrite: false,
            page_size: DEFAULT_PAGE_SIZE,
            password: String::new(),
            fixed_size: 0,
            
            mount_point: String::new(),
            readonly: false,
            local: false,
            network: false,
            icon_path: String::new(),
            logo: String::new(),
            unmount_on_terminate: false,
            proc_name: String::new(),
            proc_id: -1,
            
            operation: '\0',
            files: Vec::new(),
            recursive: false,
            compression: 0,
            delete_after: false,
            lowercase: false,
            uppercase: false,
            overwrite_files: false,
            always_yes: false,
            output_path: String::new(),
            add_empty_dirs: false,
            
            driver_path: String::new(),
            check_driver: false,
            
            use_sqlite: false,
        }
    }
}

/// Parse command line arguments
fn parse_arguments(args: &Vec<String>, params: &mut AppParams) -> bool {
    let mut i = 1;
    
    while i < args.len() {
        let arg = &args[i];
        
        if arg.starts_with('-') {
            let switch = &arg[1..];
            
            match switch {
                "c" => params.create_new = true,
                "o" => params.overwrite = true,
                "ro" => params.readonly = true,
                "lc" => params.local = true,
                "n" => params.network = true,
                "t" => params.unmount_on_terminate = true,
                "r" => params.recursive = true,
                "u" => params.uppercase = true,
                "lw" => params.lowercase = true,
                "y" => params.always_yes = true,
                "del" => params.delete_after = true,
                "chk" => params.check_driver = true,
                "w+" => params.overwrite_files = true,
                "w-" => params.overwrite_files = false,
                "sqlite" | "-sqlite" => params.use_sqlite = true,
                
                s if s.starts_with("pg") => {
                    let value = if s.len() > 2 {
                        &s[2..]
                    } else {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("Missing value for -pg");
                            return false;
                        }
                        &args[i]
                    };
                    params.page_size = value.parse().unwrap_or(DEFAULT_PAGE_SIZE);
                }
                
                s if s.starts_with("pw") => {
                    let value = if s.len() > 2 {
                        &s[2..]
                    } else {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("Missing value for -pw");
                            return false;
                        }
                        &args[i]
                    };
                    params.password = value.to_string();
                }
                
                s if s.starts_with("f") => {
                    let value = if s.len() > 1 {
                        &s[1..]
                    } else {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("Missing value for -f");
                            return false;
                        }
                        &args[i]
                    };
                    let mb: i64 = value.parse().unwrap_or(0);
                    params.fixed_size = mb * 1024 * 1024;
                }
                
                s if s.starts_with("m") => {
                    let value = if s.len() > 1 {
                        &s[1..]
                    } else {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("Missing value for -m");
                            return false;
                        }
                        &args[i]
                    };
                    params.mount_point = value.to_string();
                }
                
                s if s.starts_with("i") => {
                    let value = if s.len() > 1 {
                        &s[1..]
                    } else {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("Missing value for -i");
                            return false;
                        }
                        &args[i]
                    };
                    params.icon_path = value.to_string();
                }
                
                s if s.starts_with("l") && s.len() > 1 && s.chars().nth(1).unwrap() != 'c' && s.chars().nth(1).unwrap() != 'w' => {
                    let value = if s.len() > 1 {
                        &s[1..]
                    } else {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("Missing value for -l");
                            return false;
                        }
                        &args[i]
                    };
                    params.logo = value.to_string();
                }
                
                s if s.starts_with("ps") => {
                    let value = if s.len() > 2 {
                        &s[2..]
                    } else {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("Missing value for -ps");
                            return false;
                        }
                        &args[i]
                    };
                    if let Ok(pid) = value.parse::<i32>() {
                        params.proc_id = pid;
                    } else {
                        params.proc_name = value.to_string();
                    }
                }
                
                s if s.starts_with("comp") => {
                    let value = if s.len() > 4 {
                        &s[4..]
                    } else {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("Missing value for -comp");
                            return false;
                        }
                        &args[i]
                    };
                    params.compression = value.parse().unwrap_or(0);
                }
                
                s if s.starts_with("out") => {
                    let value = if s.len() > 3 {
                        &s[3..]
                    } else {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("Missing value for -out");
                            return false;
                        }
                        &args[i]
                    };
                    params.output_path = value.to_string();
                }
                
                s if s.starts_with("drv") => {
                    let value = if s.len() > 3 {
                        &s[3..]
                    } else {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("Missing value for -drv");
                            return false;
                        }
                        &args[i]
                    };
                    params.driver_path = value.to_string();
                }
                
                "a" => {
                    params.operation = 'a';
                    // Collect remaining args as files
                    i += 1;
                    if i >= args.len() {
                        eprintln!("No vault path specified");
                        return false;
                    }
                    params.vault_path = args[i].clone();
                    i += 1;
                    while i < args.len() {
                        params.files.push(args[i].clone());
                        i += 1;
                    }
                    return true;
                }
                
                "d" | "e" | "x" | "s" | "v" => {
                    params.operation = switch.chars().next().unwrap();
                    i += 1;
                    if i >= args.len() {
                        eprintln!("No vault path specified");
                        return false;
                    }
                    params.vault_path = args[i].clone();
                    i += 1;
                    if i < args.len() {
                        params.files.push(args[i].clone());
                    } else {
                        params.files.push("*".to_string());
                    }
                    return true;
                }
                
                _ => {
                    eprintln!("Unknown switch: {}", arg);
                    return false;
                }
            }
        } else {
            // Non-switch argument - vault path
            if params.vault_path.is_empty() {
                params.vault_path = arg.clone();
            }
        }
        
        i += 1;
    }
    
    true
}

fn main() {
    // Enable ANSI color support on Windows
    #[cfg(target_os = "windows")]
    {
        use std::io::IsTerminal;
        
        // Enable Virtual Terminal Processing for colored output
        if std::io::stdout().is_terminal() {
            unsafe {
                #[link(name = "kernel32")]
                extern "system" {
                    fn GetStdHandle(nStdHandle: u32) -> *mut std::ffi::c_void;
                    fn GetConsoleMode(hConsoleHandle: *mut std::ffi::c_void, lpMode: *mut u32) -> i32;
                    fn SetConsoleMode(hConsoleHandle: *mut std::ffi::c_void, dwMode: u32) -> i32;
                }
                
                const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5u32; // -11 as u32
                const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
                
                let handle = GetStdHandle(STD_OUTPUT_HANDLE);
                if !handle.is_null() {
                    let mut mode: u32 = 0;
                    if GetConsoleMode(handle, &mut mode) != 0 {
                        SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
                    }
                }
            }
        }
    }
    
    let args: Vec<String> = env::args().collect();
    
    banner();
    
    // Check driver status
    #[cfg(target_os = "windows")]
    {
        check_driver(MODULE_DRIVER_BLOCK, "Disk");
        check_driver(MODULE_DRIVER_FS, "Filesystem");
    }
    #[cfg(target_os = "macos")]
    {
        check_driver(MODULE_DRIVER_FS, "NFS server");
    }
    #[cfg(target_os = "linux")]
    {
        check_driver(MODULE_DRIVER_FS, "FUSE library");
    }
    
    
    // Default to interactive mode when no arguments OR when -i/--interactive is specified
    if args.len() < 2 || args.iter().any(|a| a == "-i" || a == "--interactive") {
        // Migrate any existing plain-text credentials to Windows Credential Manager
        match credential_store::migrate_from_files() {
            Ok(true) => println!("[SECURITY] Credentials migrated to Windows Credential Manager"),
            Ok(false) => {}  // No migration needed
            Err(e) => eprintln!("[WARN] Credential migration issue: {}", e),
        }
        
        // Composition Root for Interactive Mode
        let user_repo = Arc::new(SQLiteUserRepository::new("users.db"));
        let auth_service = Arc::new(ArgonAuthService::new(user_repo.clone()));
        let permission_service = Arc::new(StandardPermissionService::new(user_repo.clone()));
        
        // Factory for creating fresh VaultService instances
        let vault_factory = Box::new(|| -> Box<dyn VaultService> {
            Box::new(CbfsVaultService::new())
        });

        interactive::run(auth_service, user_repo, permission_service, vault_factory);
        std::process::exit(0);
    }
    

    

    
    let mut params = AppParams::new();
    
    if !parse_arguments(&args, &mut params) {
        eprintln!("\nUse -h or no arguments to see usage information");
        std::process::exit(1);
    }
    
    // Handle driver operations
    #[cfg(target_os = "windows")]
    {
        if !params.driver_path.is_empty() {
            let reboot = install_drivers(&params.driver_path);
            if reboot != 0 {
                std::process::exit(0);
            }
        }
        
        if params.check_driver {
            std::process::exit(0);
        }
        
        if !params.icon_path.is_empty() {
            install_icon(&params.icon_path);
        }
    }
    
    // Handle file operations
    if params.operation != '\0' {
        let vault_params = VaultParams {
            page_size: params.page_size,
            password: params.password.clone(),
            fixed_vault_size: params.fixed_size,
            overwrite_vault: params.overwrite,
            use_sqlite: params.use_sqlite,
        };
        
        let vault = open_vault(&params.vault_path, &vault_params, params.operation != 'a');
        
        match params.operation {
            'a' => {
                let mut file_params = FileOpParams::new();
                file_params.recursive = params.recursive;
                file_params.compression_level = params.compression;
                file_params.delete_after_archiving = params.delete_after;
                file_params.lowercase = params.lowercase;
                file_params.uppercase = params.uppercase;
                file_params.always_yes = params.always_yes;
                file_params.add_empty_dirs = params.add_empty_dirs;
                
                if let Err(e) = add_files(vault, &params.files, &mut file_params) {
                    eprintln!("Error adding files: {}", e);
                    std::process::exit(1);
                }
            }
            's' => {
                let pattern = if params.files.is_empty() {
                    "*"
                } else {
                    &params.files[0]
                };
                if let Err(e) = list_files(vault, pattern) {
                    eprintln!("Error listing files: {}", e);
                    std::process::exit(1);
                }
            }
            'e' | 'x' => {
                let pattern = if params.files.is_empty() {
                    "*"
                } else {
                    &params.files[0]
                };
                let output = if params.output_path.is_empty() {
                    "."
                } else {
                    &params.output_path
                };
                
                let mut file_params = FileOpParams::new();
                file_params.overwrite_files = params.overwrite_files;
                file_params.always_yes = params.always_yes;
                
                if let Err(e) = extract_files(vault, pattern, output, &mut file_params) {
                    eprintln!("Error extracting files: {}", e);
                    std::process::exit(1);
                }
            }
            'd' => {
                let pattern = if params.files.is_empty() {
                    "*"
                } else {
                    &params.files[0]
                };
                if let Err(e) = delete_files(vault, pattern, params.recursive) {
                    eprintln!("Error deleting files: {}", e);
                    std::process::exit(1);
                }

            }
            'v' => {
                let pattern = if params.files.is_empty() {
                    "*"
                } else {
                    &params.files[0]
                };
                let _ = test_vault(vault, pattern);
            }
            _ => {}
        }
        
        if let Err(e) = close_vault(vault) {
            eprintln!("Warning: Failed to close vault properly (error: {}). Changes may not be saved!", e.get_code());
            std::process::exit(e.get_code());
        }
        std::process::exit(0);
    }
    
    // Handle mounting
    if !params.mount_point.is_empty() {
        if params.vault_path.is_empty() {
            eprintln!("Vault path must be specified for mounting");
            std::process::exit(1);
        }
        
        let vault_params = VaultParams {
            page_size: params.page_size,
            password: params.password.clone(),
            fixed_vault_size: params.fixed_size,
            overwrite_vault: params.overwrite,
            use_sqlite: params.use_sqlite,
        };
        
        let mount_params = MountParams {
            mounting_point: params.mount_point.clone(),
            readonly: params.readonly,
            local: params.local,
            network: params.network,
            icon_path: params.icon_path.clone(),
            logo: params.logo.clone(),
            unmount_on_terminate: params.unmount_on_terminate,
            proc_name: params.proc_name.clone(),
            proc_id: params.proc_id,
        };
        
        let ret = mount_vault(&params.vault_path, &mount_params, &vault_params);
        std::process::exit(ret);
    }
    
    // Handle vault creation/info
    if !params.vault_path.is_empty() {
        let vault_params = VaultParams {
            page_size: params.page_size,
            password: params.password.clone(),
            fixed_vault_size: params.fixed_size,
            overwrite_vault: params.overwrite,
            use_sqlite: params.use_sqlite,
        };
        
        let vault = open_vault(&params.vault_path, &vault_params, false);
        println!("Vault opened successfully: {}", params.vault_path);
        let _ = close_vault(vault);
        std::process::exit(0);
    }
    
    // No operation specified
    eprintln!("No operation specified");
    usage();
    std::process::exit(1);
}

/*
 * CBFS Vault 2024 Rust Edition - Vault Console
 * Vault Management Module
 */

use std::path::Path;
use cbfsvault::cbvault::CBVault;
use cbfsvault::*;
use crate::config::*;

/// Parameters for vault operations
#[allow(dead_code)]
pub struct VaultParams {
    pub page_size: i32,
    pub password: String,
    pub fixed_vault_size: i64,
    pub overwrite_vault: bool,
    pub use_sqlite: bool,  // Use SQLite callback mode for storage
}

impl VaultParams {
    #[allow(dead_code)]
    pub fn new() -> Self {
        VaultParams {
            page_size: DEFAULT_PAGE_SIZE,
            password: String::new(),
            fixed_vault_size: 0,
            overwrite_vault: false,
            use_sqlite: false,
        }
    }
}

/// Open or create a vault
pub fn open_vault(vault_path: &str, params: &VaultParams, only_existing: bool) -> &'static CBVault {
    let mut vault = CBVault::new();
    
    // Enable SQLite callback mode if requested
    if params.use_sqlite {
        // Convert vault path to database filename (append .db if needed)
        let db_path = if vault_path.ends_with(".db") {
            vault_path.to_string()
        } else {
            format!("{}.db", vault_path.trim_end_matches(".vlt").trim_end_matches(".vault"))
        };
        crate::sqlite_vault::setup_callbacks(&mut vault, Some(&db_path));
    }
    
    let _ = vault.set_vault_file(vault_path);
    
    let res = if only_existing {
        vault.open_vault(VAULT_OM_OPEN_EXISTING, if params.use_sqlite { VAULT_JM_NONE } else { VAULT_JM_METADATA })
    } else {
        if params.use_sqlite || !Path::new(vault_path).is_file() || params.overwrite_vault {
            // Configure new vault
            if params.page_size != 0 && !params.use_sqlite {
                let res = vault.set_page_size(params.page_size);
                if res.is_err() {
                    let code = res.unwrap_err().get_code();
                    eprintln!(
                        "Error {} occurred while setting page size. Ensure size is power of 2 between 512-65536.",
                        code
                    );
                    std::process::exit(code);
                }
            }
            
            if params.fixed_vault_size != 0 && !params.use_sqlite {
                let res = vault.set_vault_size_max(params.fixed_vault_size);
                if res.is_ok() {
                    let _ = vault.set_vault_size_min(params.fixed_vault_size);
                }
                if res.is_err() {
                    let code = res.unwrap_err().get_code();
                    eprintln!("Error {} occurred while setting fixed vault size.", code);
                    std::process::exit(code);
                }
            }
            
            vault.open_vault(VAULT_OM_CREATE_ALWAYS, if params.use_sqlite { VAULT_JM_NONE } else { VAULT_JM_METADATA })
        } else {
            vault.open_vault(VAULT_OM_OPEN_ALWAYS, if params.use_sqlite { VAULT_JM_NONE } else { VAULT_JM_METADATA })
        }
    };
    
    if res.is_err() {
        let code = res.unwrap_err().get_code();
        eprintln!("Error {} occurred while opening vault.", code);
        std::process::exit(code);
    }
    
    vault
}

/// Close vault safely
pub fn close_vault(vault: &CBVault) -> Result<(), CBFSVaultError> {
    vault.close_vault()
}


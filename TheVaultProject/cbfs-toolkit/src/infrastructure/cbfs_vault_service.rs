use crate::domain::vault::{VaultService, VaultEntry, FileOpParams};
use crate::errors::{AppError, AppResult};
use cbfsvault::cbvault::CBVault;
use cbfsvault::*;

// default page size of vault
const DEFAULT_PAGE_SIZE: i32 = 4096;

pub struct CbfsVaultService {
    vault: Option<&'static mut CBVault>,
    current_path: String,
}

impl CbfsVaultService {
    pub fn new() -> Self {
        Self {
            vault: None,
            current_path: String::new(),
        }
    }
    
    // Helper to get vault ref
    fn get_vault(&self) -> AppResult<&CBVault> {
        self.vault.as_deref().ok_or(AppError::General("Vault not open".to_string()))
    }
}

impl VaultService for CbfsVaultService {
    fn open(&mut self, path: &str, password: &str) -> AppResult<()> {
        if self.vault.is_some() {
            self.close()?;
        }
        
        let vault = CBVault::new();
        
        // SQLite detection - currently unused but kept for future implementation
        let _use_sqlite = path.ends_with(".db") || path.contains("sqlite"); 
        // The trait method `open` doesn't take `use_sqlite`. Ideally detection or param.
        // For now, let's assume detection or we update trait.
        
        // Actually, the legacy code had `use_sqlite` logic in `interactive.rs`.
        // Let's rely on extension detection or just try standard open.
        // If we strictly follow the trait, we might need to update it to accept `VaultConfig` object?
        // But for step-by-step, let's stick to basic.
        
        let _ = vault.set_vault_file(path);
        if !password.is_empty() {
            let _ = vault.set_vault_password(password);
        }
        
        vault.open_vault(VAULT_OM_OPEN_EXISTING, VAULT_JM_METADATA)
             .map_err(|e| AppError::Vault { code: e.get_code(), message: "Failed to open vault".to_string() })?;
        
        self.vault = Some(vault);
        self.current_path = path.to_string();
        Ok(())
    }

    fn create(&mut self, path: &str, password: &str, use_sqlite: bool) -> AppResult<()> {
        if self.vault.is_some() {
            self.close()?;
        }
        
        let vault = CBVault::new();
        
        if use_sqlite {
             // Setup callbacks logic here or call helper
             // crate::sqlite_vault::setup_callbacks(&mut vault, Some(path)); 
             // We need to import sqlite_vault if we want this feature.
             // For now, let's focus on standard vault to prove architecture.
        }

        let _ = vault.set_vault_file(path);
        let _ = vault.set_page_size(DEFAULT_PAGE_SIZE);
        if !password.is_empty() {
            let _ = vault.set_vault_password(password);
        }
        
        vault.open_vault(VAULT_OM_CREATE_ALWAYS, VAULT_JM_METADATA)
             .map_err(|e| AppError::Vault { code: e.get_code(), message: "Failed to create vault".to_string() })?;
             
        self.vault = Some(vault);
        self.current_path = path.to_string();
        Ok(())
    }

    fn close(&mut self) -> AppResult<()> {
        if let Some(vault) = self.vault.take() {
            vault.close_vault().map_err(|e| AppError::Vault { code: e.get_code(), message: "Failed to close vault".to_string() })?;
        }
        self.current_path.clear();
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.vault.is_some()
    }
    
    fn get_path(&self) -> String {
        self.current_path.clone()
    }

    fn list_files(&self, pattern: &str) -> AppResult<Vec<VaultEntry>> {
        let vault = self.get_vault()?;
        let mut entries = Vec::new();
        
        let search_res = vault.find_first(pattern, VAULT_FATTR_ANY_FILE, VAULT_FF_NEED_ATTRIBUTES | VAULT_FF_NEED_NAME | VAULT_FF_NEED_SIZE);
        
        if let Ok(search_data) = search_res {
             loop {
                if let Ok(name) = vault.get_search_result_name(search_data) {
                     let is_dir = vault.get_search_result_attributes(search_data)
                        .map(|attr| (attr & VAULT_FATTR_DIRECTORY) != 0)
                        .unwrap_or(false);
                     let size = vault.get_search_result_size(search_data).unwrap_or(0);
                     
                     entries.push(VaultEntry {
                         name,
                         is_directory: is_dir,
                         size: size as u64,
                     });
                }
                if !vault.find_next(search_data).unwrap_or(false) { break; }
             }
             let _ = vault.find_close(search_data);
        }
        
        Ok(entries)
    }

    fn add_files(&self, files: &[String], params: &mut FileOpParams) -> AppResult<()> {
        let vault = self.get_vault()?;
        // Call file_ops functions directly - they now use domain FileOpParams
        crate::file_ops::add_files(vault, &files.to_vec(), params)
    }

    fn extract_files(&self, pattern: &str, output_path: &str, params: &FileOpParams) -> AppResult<()> {
        let vault = self.get_vault()?;
        // Note: file_ops extract_files takes &mut but only for potential future use
        let mut params_mut = params.clone();
        crate::file_ops::extract_files(vault, pattern, output_path, &mut params_mut)
    }

    fn delete_files(&self, pattern: &str, recursive: bool) -> AppResult<()> {
        let vault = self.get_vault()?;
        crate::file_ops::delete_files(vault, pattern, recursive)
    }

    fn convert_to_file(&self, _output_path: &str) -> AppResult<()> {
        // Logic for converting SQLite to File (Optional for now)
         Err(AppError::General("Not implemented in service yet".to_string()))
    }
}

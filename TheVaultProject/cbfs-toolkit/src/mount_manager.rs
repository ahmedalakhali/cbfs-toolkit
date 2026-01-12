/*
 * CBFS Vault 2024 Rust Edition - Vault Console
 * Mount Management Module
 */

use std::io::{self, Read};
use cbfsvault::cbvaultdrive::*;
use cbfsvault::*;
use crate::config::*;

/// Parameters for mounting operations
#[allow(dead_code)]
pub struct MountParams {
    pub mounting_point: String,
    pub readonly: bool,
    pub local: bool,
    pub network: bool,
    pub icon_path: String,
    pub logo: String,
    pub unmount_on_terminate: bool,
    pub proc_name: String,
    pub proc_id: i32,
}

impl MountParams {
    #[allow(dead_code)]
    pub fn new() -> Self {
        MountParams {
            mounting_point: String::new(),
            readonly: false,
            local: false,
            network: false,
            icon_path: String::new(),
            logo: String::new(),
            unmount_on_terminate: false,
            proc_name: String::new(),
            proc_id: -1,
        }
    }
}

/// Mount a vault as a drive
pub fn mount_vault(
    vault_path: &str,
    params: &MountParams,
    vault_params: &crate::vault_manager::VaultParams,
) -> i32 {
    let drive = CBVaultDrive::new();
    
    // Initialize drive
    let ret_val = drive.initialize(GUID);
    if ret_val.is_err() {
        let ret_code = ret_val.unwrap_err().get_code();
        eprintln!("Initialization of CBVaultDrive failed with error {}", ret_code);
        drive.dispose();
        return ret_code;
    }
    
    let _ = drive.set_vault_file(vault_path);
    let _ = drive.set_unmount_on_termination(params.unmount_on_terminate);
    let _ = drive.set_read_only(params.readonly);
    
    // Open the vault
    let open_mode = if vault_params.overwrite_vault {
        VAULT_OM_CREATE_ALWAYS
    } else {
        VAULT_OM_OPEN_EXISTING
    };
    
    let ret_val = drive.open_vault(open_mode, VAULT_JM_METADATA);
    if ret_val.is_err() {
        let ret_code = ret_val.unwrap_err().get_code();
        eprintln!("Opening of vault failed with error {}", ret_code);
        let _ = drive.close_vault(false);
        drive.dispose();
        return ret_code;
    }
    
    // Set up process restrictions if specified
    if params.proc_id > 0 || !params.proc_name.is_empty() {
        let _ = drive.set_process_restrictions_enabled(true);
        let ret_val = if params.proc_id > 0 {
            drive.add_granted_process("", params.proc_id, true, STG_DACCESS_READWRITE)
        } else {
            drive.add_granted_process(&params.proc_name, -1, true, STG_DACCESS_READWRITE)
        };
        
        if ret_val.is_err() {
            let _ = drive.set_process_restrictions_enabled(false);
            let ret_code = ret_val.unwrap_err().get_code();
            eprintln!(
                "Addition of granted process failed with error {}. Process restrictions not enabled.",
                ret_code
            );
        }
    }
    
    // Determine mounting flags
    let flags = if params.network {
        STGMP_NETWORK | STGMP_NETWORK_CLAIM_SERVER_NAME
    } else if std::env::consts::OS.eq_ignore_ascii_case("windows") && !params.local {
        STGMP_MOUNT_MANAGER
    } else if params.local {
        STGMP_SIMPLE | STGMP_LOCAL
    } else {
        STGMP_SIMPLE
    };
    
    // Add mounting point
    let ret_val = drive.add_mounting_point(&params.mounting_point, flags, 0);
    if ret_val.is_err() {
        let ret_code = ret_val.unwrap_err().get_code();
        eprintln!("Addition of mounting point failed with error {}", ret_code);
        let _ = drive.close_vault(false);
        drive.dispose();
        return ret_code;
    }
    
    // Set icon if registered
    #[cfg(target_os = "windows")]
    {
        let ret_val = drive.is_icon_registered(ICON_ID);
        if ret_val.is_ok() && ret_val.ok().unwrap() {
            let _ = drive.set_icon(ICON_ID);
        }
    }
    
    println!("Vault mounted successfully at {}", params.mounting_point);
    println!("Press Enter to unmount and quit...");
    
    // Wait for user input
    let mut buf: Vec<u8> = Vec::new();
    let _ = io::stdin().read(&mut buf);
    
    // Unmount
    println!("Unmounting vault...");
    let mut ret_val = drive.remove_mounting_point(0, &params.mounting_point, flags, 0);
    if ret_val.is_err() {
        let ret_code = ret_val.unwrap_err().get_code();
        eprintln!("remove_mounting_point() failed with error {}", ret_code);
    }
    
    ret_val = drive.close_vault(false);
    if ret_val.is_err() {
        eprintln!("close_vault() failed with error {}", ret_val.unwrap_err().get_code());
    } else {
        println!("Vault unmounted successfully");
    }
    
    drive.dispose();
    0
}

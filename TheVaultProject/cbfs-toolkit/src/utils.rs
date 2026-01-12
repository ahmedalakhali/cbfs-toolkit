/*
 * CBFS Vault 2024 Rust Edition - Vault Console
 * Utility Functions
 */

use std::io::{self, Read};
use cbfsvault::cbvaultdrive::CBVaultDrive;
use crate::config::*;

/// Display application banner
pub fn banner() {
    println!("CBFS Vault Console Copyright (c) 2017-2025, Callback Technologies, Inc.");
}

/// Display usage information
pub fn usage() {
    println!("\nUsage: vault-console [-<switches>] <vault-file> [<additional-params>]\n");
    
    println!("Modes:");
    println!("  -i, --interactive   Launch interactive shell mode");
    
    println!("Vault Creation & Configuration:");
    println!("  -c              Create new vault file if it doesn't exist");
    println!("  -o              Overwrite existing vault file");
    println!("  -pg <size>      Set page size for new vault (default: 4096)");
    println!("  -pw <password>  Set password for vault");
    println!("  -f <size>       Set fixed vault size in MB");
    
    println!("\nMounting Options:");
    println!("  -m <point>      Mount vault at specified drive letter or path");
    println!("  -ro             Open vault in read-only mode");
    println!("  -lc             Mount for current user only");
    println!("  -n              Mount as network volume");
    #[cfg(target_os = "windows")]
    println!("  -i <path>       Set custom drive icon");
    println!("  -l <label>      Set volume label/logo");
    println!("  -t              Unmount on process termination");
    println!("  -ps <pid|name>  Add process restriction");
    
    println!("\nFile Operations:");
    println!("  -a <files...>   Add files to vault");
    println!("  -d <pattern>    Delete files from vault");
    println!("  -e <pattern>    Extract files from vault");
    println!("  -x <pattern>    Extract files with full path");
    println!("  -s [pattern]    List (show) vault contents");
    println!("  -v [pattern]    Verify/test vault integrity");
    
    println!("\nFile Operation Modifiers:");
    println!("  -r              Recursive operation");
    println!("  -u              Convert names to uppercase");
    println!("  -lw             Convert names to lowercase");
    println!("  -w+             Overwrite existing files during extraction");
    println!("  -w-             Don't overwrite files (default)");
    println!("  -del            Delete files after adding to vault");
    println!("  -y              Answer yes to all prompts");
    println!("  -out <path>     Output directory for extraction");
    println!("  -comp <0-9>     Compression level (0=none, 9=max)");
    
    #[cfg(target_os = "windows")]
    {
        println!("\nDriver Management:");
        println!("  -drv <cab>      Install drivers from CAB file");
        println!("  -chk            Check driver status");
    }
    
    println!("\nExamples:");
    println!("  vault-console -c -pg 4096 -pw secret myvault.vault");
    println!("  vault-console -c -m Z: -pw secret -l \"My Vault\" vault.vault");
    println!("  vault-console -a -pw secret -r vault.vault C:\\\\Documents\\\\*.txt");
    println!("  vault-console -s -pw secret vault.vault");
    println!("  vault-console -x -pw secret -out C:\\\\Extracted vault.vault \"*\"");
}

/// Check driver status and display version information
pub fn check_driver(module: i32, module_name: &str) {
    let drive = CBVaultDrive::new();
    
    let state = drive.get_driver_status(GUID, module);
    if state.is_err() || state.unwrap() != cbfsvault::MODULE_STATUS_RUNNING {
        eprintln!("{} driver is not installed\n", module_name);
    } else {
        let version = drive.get_module_version(GUID, module);
        if version.is_ok() {
            let version_val: i64 = version.unwrap();
            println!(
                "{} driver is installed, version: {}.{}.{}.{}\n",
                module_name,
                (version_val & 0x7FFF000000000000) >> 48,
                (version_val & 0xFFFF00000000) >> 32,
                (version_val & 0xFFFF0000) >> 16,
                version_val & 0xFFFF
            );
        }
    }
    drive.dispose();
}

/// Install drivers from CAB file
pub fn install_drivers(driver_path: &String) -> i32 {
    println!("Installing drivers from {}", driver_path);
    
    let drive = CBVaultDrive::new();
    
    let drv_reboot_res = drive.install(
        driver_path,
        GUID,
        "",
        cbfsvault::MODULE_DRIVER_PNP_BUS
            | cbfsvault::MODULE_DRIVER_BLOCK
            | cbfsvault::MODULE_DRIVER_FS
            | cbfsvault::MODULE_HELPER_DLL,
        cbfsvault::INSTALL_REMOVE_OLD_VERSIONS,
    );
    
    if drv_reboot_res.is_err() {
        let ret_code = drv_reboot_res.err().unwrap().get_code();
        if ret_code == ERROR_PRIVILEGE_NOT_HELD {
            eprintln!("Drivers are not installed due to insufficient privileges. Please run with administrator rights");
        } else {
            eprintln!("Drivers have not been installed because of error {}", ret_code);
        }
        drive.dispose();
        return ret_code;
    }
    
    let drv_reboot: i32 = drv_reboot_res.ok().unwrap();
    drive.dispose();
    
    print!("Drivers installed successfully");
    if drv_reboot != 0 {
        println!(", reboot is required\n");
    } else {
        println!();
    }
    
    drv_reboot
}

/// Install icon for drive
#[cfg(target_os = "windows")]
pub fn install_icon(icon_path: &str) -> bool {
    let drive = CBVaultDrive::new();
    
    let ret_val = drive.register_icon(icon_path, GUID, ICON_ID);
    let result = if ret_val.is_err() {
        let ret_code = ret_val.err().unwrap().get_code();
        eprintln!("Installation of icon failed with error {}", ret_code);
        false
    } else if ret_val.ok().unwrap() {
        println!("Icon installed successfully, reboot required for icon to be displayed");

        true
    } else {
        false
    };
    
    drive.dispose();
    result
}

/// Ask user a yes/no/all question
pub fn ask_yna(question: &str) -> i32 {
    loop {
        println!("{} (y/n/a) ", question);
        let mut buf: Vec<u8> = Vec::new();
        buf.push(0);
        buf.push(0);
        let ret_val = io::stdin().read(&mut buf);
        if ret_val.is_ok() {
            let chars_read = ret_val.unwrap();
            for i in 0..chars_read {
                match char::from(buf[i]) {
                    'y' | 'Y' => return ANSWER_YES,
                    'n' | 'N' => return ANSWER_NO,
                    'a' | 'A' => return ANSWER_ALL,
                    _ => {}
                }
            }
            eprintln!("Invalid input, please try again:");
        } else {
            eprintln!("Error reading input, please try again:");
        }
    }
}

/// Extract file path from full path
pub fn extract_file_path(filepath: &String, separator: &str) -> String {
    let last_res = filepath.rfind(separator);
    if last_res.is_some() {
        let l = last_res.unwrap();
        if l > 0 {
            return filepath[..l].to_string();
        }
    }
    separator.to_string()
}

/// Extract file name from full path
pub fn extract_file_name(filepath: &String, separator: &str) -> String {
    let last_res = filepath.rfind(separator);
    if last_res.is_some() {
        let last = last_res.unwrap();
        if last < filepath.len() - 1 {
            return filepath[last + 1..].to_string();
        } else {
            return String::new();
        }
    } else {
        filepath.clone()
    }
}

/// Check if path contains wildcards
pub fn has_wildcards(path: &String) -> bool {
    path.contains('*') || path.contains('?')
}

/// Split path with mask
pub fn split_path_with_mask(filepath: &String, separator: &str) -> (String, String) {
    if has_wildcards(filepath) {
        (
            extract_file_path(filepath, separator),
            extract_file_name(filepath, separator),
        )
    } else {
        (filepath.clone(), String::new())
    }
}

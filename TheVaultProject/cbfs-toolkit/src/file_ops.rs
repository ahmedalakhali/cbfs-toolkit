/*
 * CBFS Vault 2024 Rust Edition - Vault Console
 * File Operations Module
 */

use std::fs::{self, File};
use std::io::{IoSlice, Read, Write};
use std::path::Path;
use cbfsvault::cbvault::CBVault;
use cbfsvault::*;
use crate::config::*;
use crate::utils::*;
use crate::errors::{AppError, AppResult};
use crate::domain::vault::FileOpParams; // Use domain FileOpParams instead of duplicating

/// Convert OS path to vault path
fn os_path_to_vault_path(ospath: &String, vault: &CBVault, params: &FileOpParams) -> AppResult<String> {
    #[cfg(target_os = "windows")]
    let os_sep_str = "\\";
    #[cfg(not(target_os = "windows"))]
    let os_sep_str = "/";

    let local_ospath: String = if params.recursive {
        #[cfg(target_os = "windows")]
        let starts_with_root = ospath.to_lowercase().starts_with(&params.root_dir);
        #[cfg(not(target_os = "windows"))]
        let starts_with_root = ospath.starts_with(&params.root_dir);

        if starts_with_root {
            ospath[params.root_dir.len()..].to_string()
        } else {
            ospath.clone()
        }
    } else {
        extract_file_name(ospath, os_sep_str)
    };

    let vault_sep = (vault.path_separator().map_err(|e| AppError::Vault { code: e.get_code(), message: "Path separator error".to_string() })? as u8) as char;
    let vault_sep_str = vault_sep.to_string();

    #[cfg(target_os = "windows")]
    let os_sep = '\\';
    #[cfg(not(target_os = "windows"))]
    let os_sep = '/';

    let mut result = if vault_sep != os_sep {
        local_ospath.replace(os_sep, &vault_sep_str)
    } else {
        local_ospath.clone()
    };

    // Remove drive letter on Windows
    #[cfg(target_os = "windows")]
    if result.len() >= 2 && result.chars().nth(1).unwrap_or('\0') == ':' {
        result = result[2..].to_string();
    }

    let mut buf = String::new();
    for part in result.split(&vault_sep_str) {
        if !part.is_empty() && part != "." && part != ".." {
            buf.push_str(&vault_sep_str);
            if params.lowercase {
                buf.push_str(&part.to_lowercase());
            } else if params.uppercase {
                buf.push_str(&part.to_uppercase());
            } else {
                buf.push_str(part);
            }
        }
    }

    if buf.is_empty() {
        Ok(vault_sep_str)
    } else {
        Ok(buf)
    }
}

/// Add a file to the vault
fn add_file(vault: &CBVault, filepath: &String, params: &mut FileOpParams) -> AppResult<()> {
    let vault_sep = (vault.path_separator().map_err(|e| AppError::Vault { code: e.get_code(), message: "Path separator error".to_string() })? as u8) as char;
    let vault_sep_str = vault_sep.to_string();

    let vault_file_name = os_path_to_vault_path(filepath, vault, params)?;
    let vault_file_path = extract_file_path(&vault_file_name, &vault_sep_str);

    // Check if directory exists, create if not
    let exists = vault.file_exists(&vault_file_path).unwrap_or(false);

    if !exists {
        vault.create_directory(&vault_file_path, true)?;
    }

    // Check if file exists
    let exists = vault.file_exists(&vault_file_name).unwrap_or(false);

    if exists {
        let add_file = if params.always_yes {
            true
        } else if params.overwrite_files_set {
            if params.overwrite_files {
                true
            } else {
                eprintln!("Target file '{}' already exists, skipping", vault_file_name);
                false
            }
        } else {
            let q = format!("'{}' already exists. Overwrite it?", vault_file_name);
            match ask_yna(&q) {
                ANSWER_YES => true,
                ANSWER_ALL => {
                    params.overwrite_files = true;
                    params.overwrite_files_set = true;
                    true
                }
                _ => {
                    println!("Skipped '{}'", vault_file_name);
                    false
                }
            }
        };

        if !add_file {
            return Ok(());
        }
    }

    print!("Adding {}", filepath);

    // Open source file
    let mut in_file = File::open(filepath).map_err(AppError::Io)?;

    // Open vault file
    let mut out_file = vault.open_file_ex(
        &vault_file_name,
        VAULT_FOM_CREATE_ALWAYS,
        false,
        true,
        true,
        true,
        VAULT_EM_NONE,
        "",
        if params.compression_level == 0 {
            VAULT_CM_NONE
        } else {
            VAULT_CM_ZLIB
        },
        params.compression_level,
        16,
    )?;

    let _ = out_file.set_len(0);

    // Copy data
    let mut buf: Vec<u8> = vec![0; BUFFER_SIZE];
    loop {
        let block_read = match in_file.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("\nFailed to read input file");
                let _ = out_file.close();
                let _ = vault.delete_file(&vault_file_name);
                return Err(AppError::Io(e));
            }
        };

        if block_read == 0 {
            break;
        }

        if let Err(e) = out_file.write_vectored(&[IoSlice::new(&buf[0..block_read])]) {
            eprintln!("\nFailed to write to vault file");
            let _ = out_file.close();
            let _ = vault.delete_file(&vault_file_name);
            return Err(AppError::Io(e));
        }
    }

    let _ = out_file.close();
    println!(" succeeded");
    Ok(())
}

/// Add a directory to the vault
fn add_directory(vault: &CBVault, dirpath: &String, params: &FileOpParams) -> AppResult<()> {
    let vault_dir_name = os_path_to_vault_path(dirpath, vault, params)?;
    
    match vault.create_directory(&vault_dir_name, true) {
        Ok(_) => Ok(()),
        Err(e) => {
            let code = e.get_code();
            if code != VAULT_ERR_FILE_ALREADY_EXISTS {
                eprintln!("Failed to create directory '{}': error {}", vault_dir_name, code);
                Err(AppError::Vault { code, message: format!("Failed to create directory '{}'", vault_dir_name) })
            } else {
                Ok(())
            }
        }
    }
}

/// Enumerate items in a directory
fn enum_items(
    vault: &CBVault,
    dir: &String,
    mask: &String,
    include_files: bool,
    include_dirs: bool,
    recursive: bool,
) -> Vec<String> {
    int_enum_items(vault, &Path::new(dir), mask, include_files, include_dirs, recursive)
}

fn int_enum_items(
    vault: &CBVault,
    dir: &Path,
    mask: &String,
    include_files: bool,
    include_dirs: bool,
    recursive: bool,
) -> Vec<String> {
    let mut entries = Vec::new();

    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() && include_files {
                let filename = path.to_string_lossy().into_owned();
                let add_file = if !mask.is_empty() {
                    vault.file_matches_mask(mask, &filename, false).unwrap_or(false)
                } else {
                    true
                };
                if add_file {
                    entries.push(filename);
                }
            } else if path.is_dir() {
                if include_dirs {
                    let filename = path.to_string_lossy().into_owned();
                    let add_dir = if !recursive && !mask.is_empty() {
                        vault.file_matches_mask(mask, &filename, false).unwrap_or(false)
                    } else {
                        true
                    };
                    if add_dir {
                        entries.push(filename);
                    }
                }
                if recursive {
                    entries.extend(int_enum_items(vault, &path, mask, include_files, include_dirs, true));
                }
            }
        }
    }

    entries
}

/// Iterate over files and apply function
fn iterate_files<F>(
    filepath: &String,
    func: F,
    recursive: bool,
    vault: &CBVault,
    params: &mut FileOpParams,
) -> AppResult<()>
where
    F: Fn(&CBVault, &String, bool, &mut FileOpParams) -> AppResult<()> + Copy,
{
    #[cfg(target_os = "windows")]
    let sep_char = "\\";
    #[cfg(not(target_os = "windows"))]
    let sep_char = "/";

    let mut path = filepath.clone();
    #[cfg(target_os = "windows")]
    {
        path = path.replace("/", sep_char);
    }

    let (mut root_dir, mut mask) = split_path_with_mask(&path, sep_char);

    if params.root_dir.is_empty() {
        #[cfg(target_os = "windows")]
        {
            params.root_dir = root_dir.to_lowercase();
        }
        #[cfg(not(target_os = "windows"))]
        {
            params.root_dir = root_dir.clone();
        }
    }

    if root_dir.is_empty() {
        root_dir = ".".to_string();
    }

    let is_dir = if mask.is_empty() {
        let is_dir = Path::new(&path).is_dir();
        if is_dir {
            mask = "*".to_string();
        }
        is_dir
    } else {
        Path::new(&root_dir).is_dir()
    };

    if is_dir {
        let dirs = if params.recursive {
            enum_items(vault, &root_dir, &String::new(), false, true, false)
        } else {
            enum_items(vault, &root_dir, &mask, false, true, false)
        };

        for dir in &dirs {
            func(vault, dir, true, params)?;
            
            if recursive {
                let combined = format!("{}{}{}", dir, sep_char, mask);
                iterate_files(&combined, func, recursive, vault, params)?;
            }
        }

        let files = enum_items(vault, &root_dir, &mask, true, false, false);
        for file in &files {
            func(vault, file, false, params)?;
        }
    } else {
        func(vault, &path, false, params)?;
    }

    Ok(())
}

/// Add files to vault
pub fn add_files(vault: &CBVault, files: &Vec<String>, params: &mut FileOpParams) -> AppResult<()> {
    for filepath in files {
        iterate_files(
            filepath,
            |v, f, is_dir, p| {
                if is_dir && !p.add_empty_dirs {
                    return Ok(());
                }
                if is_dir {
                    add_directory(v, f, p)
                } else {
                    add_file(v, f, p)
                }
            },
            params.recursive,
            vault,
            params,
        )?;
    }
    Ok(())
}

/// List vault contents
pub fn list_files(vault: &CBVault, pattern: &str) -> AppResult<()> {
    let search_res = vault.find_first(pattern, VAULT_FATTR_ANY_FILE, VAULT_FF_NEED_ATTRIBUTES | VAULT_FF_NEED_NAME | VAULT_FF_NEED_SIZE);
    
    // We treat "no files found" not as a hard error but just empty output, unless it's a real failure
    if let Err(_e) = search_res {
        // Decide if we want to print "No files found" or just return.
        // For CLI, printing is nice.
        println!("No files found");
        return Ok(()); 
    }

    let search_data = search_res?;
    
    loop {
        if let Ok(filename) = vault.get_search_result_name(search_data) {
            let is_dir = vault.get_search_result_attributes(search_data)
                .map(|attr| (attr & VAULT_FATTR_DIRECTORY) != 0)
                .unwrap_or(false);
            
            let size = vault.get_search_result_size(search_data).unwrap_or(0);
            
            if is_dir {
                println!("{:<50} <DIR>", filename);
            } else {
                println!("{:<50} {:>15} bytes", filename, size);
            }
        }
        
        if !vault.find_next(search_data).unwrap_or(false) {
            break;
        }
    }
    
    let _ = vault.find_close(search_data);
    Ok(())
}

/// Extract file from vault
pub fn extract_files(vault: &CBVault, pattern: &str, output_path: &str, _params: &mut FileOpParams) -> AppResult<()> {
    let sep = (vault.path_separator().map_err(|e| AppError::Vault { code: e.get_code(), message: "Path separator error".to_string() })? as u8) as char;
    
    let search_res = vault.find_first(pattern, VAULT_FATTR_ANY_FILE, VAULT_FF_NEED_ATTRIBUTES | VAULT_FF_NEED_NAME | VAULT_FF_NEED_SIZE);
    if search_res.is_err() {
        println!("No files found");
        return Ok(());
    }

    let search_data = search_res?;
    
    loop {
        if let Ok(filename) = vault.get_search_result_name(search_data) {
            let is_dir = vault.get_search_result_attributes(search_data)
                .map(|attr| (attr & VAULT_FATTR_DIRECTORY) != 0)
                .unwrap_or(false);
            
            if is_dir {
                // Create directory
                let out_path = format!("{}{}{}", output_path, std::path::MAIN_SEPARATOR, filename.replace(sep, &std::path::MAIN_SEPARATOR.to_string()));
                fs::create_dir_all(&out_path).map_err(AppError::Io)?;
                println!("Created directory: {}", out_path);
            } else {
                // Extract file
                print!("Extracting {}", filename);
                
                let vault_file_res = vault.open_file_ex(
                    &filename,
                    VAULT_FOM_OPEN_EXISTING,
                    true,
                    false,
                    false,
                    false,
                    VAULT_EM_NONE,
                    "",
                    VAULT_CM_NONE,
                    0,
                    0,
                );
                
                match vault_file_res {
                    Ok(mut in_file) => {
                        let out_path = format!("{}{}{}", output_path, std::path::MAIN_SEPARATOR, filename.replace(sep, &std::path::MAIN_SEPARATOR.to_string()));
                        
                        // Create parent directory
                        if let Some(parent) = Path::new(&out_path).parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        
                        match File::create(&out_path) {
                            Ok(mut out_file) => {
                                let mut buf: Vec<u8> = vec![0; BUFFER_SIZE];
                                loop {
                                    match in_file.read(&mut buf) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            if let Err(_) = out_file.write_all(&buf[0..n]) {
                                                 println!(" failed to write output");
                                                 break;
                                            }
                                        }
                                        Err(_) => {
                                            println!(" failed to read from vault");
                                            break;
                                        }
                                    }
                                }
                                println!(" succeeded");
                            }
                            Err(_) => println!(" failed to create output file"),
                        }
                        let _ = in_file.close();
                    }
                    Err(_) => println!(" failed to open vault file"),
                }
            }
        }
        
        if !vault.find_next(search_data).unwrap_or(false) {
            break;
        }
    }
    
    let _ = vault.find_close(search_data);
    Ok(())
}

/// Delete files from vault
pub fn delete_files(vault: &CBVault, pattern: &str, recursive: bool) -> AppResult<()> {
    let flags = if recursive {
        VAULT_FF_NEED_NAME | VAULT_FF_NEED_ATTRIBUTES | VAULT_FF_RECURSIVE
    } else {
        VAULT_FF_NEED_NAME | VAULT_FF_NEED_ATTRIBUTES
    };
    
    let search_res = vault.find_first(pattern, VAULT_FATTR_ANY_FILE, flags);
    if let Err(e) = search_res {
        println!("No files found matching '{}' (error: {})", pattern, e.get_code());
        return Ok(());
    }

    let mut files_to_delete: Vec<(String, bool)> = Vec::new(); // (name, is_directory)
    let search_data = search_res?;
    
    loop {
        if let Ok(filename) = vault.get_search_result_name(search_data) {
            let is_dir = vault.get_search_result_attributes(search_data)
                .map(|attr| (attr & VAULT_FATTR_DIRECTORY) != 0)
                .unwrap_or(false);
            files_to_delete.push((filename, is_dir));
        }
        
        if !vault.find_next(search_data).unwrap_or(false) {
            break;
        }
    }
    
    let _ = vault.find_close(search_data);
    
    if files_to_delete.is_empty() {
        println!("No files found matching '{}'", pattern);
        return Ok(());
    }
    
    println!("Found {} items to delete", files_to_delete.len());
    
    // Sort by path depth (deepest first)
    files_to_delete.sort_by(|a, b| {
        let depth_a = a.0.matches('\\').count() + a.0.matches('/').count();
        let depth_b = b.0.matches('\\').count() + b.0.matches('/').count();
        
        match depth_b.cmp(&depth_a) {
            std::cmp::Ordering::Equal => {
                match (a.1, b.1) {
                    (false, true) => std::cmp::Ordering::Less,
                    (true, false) => std::cmp::Ordering::Greater,
                    _ => a.0.cmp(&b.0),
                }
            }
            other => other,
        }
    });
    
    let mut deleted = 0;
    let mut failed = 0;
    
    for (filename, is_dir) in files_to_delete {
        let item_type = if is_dir { "directory" } else { "file" };
        print!("Deleting {} '{}'", item_type, filename);
        
        match vault.delete_file(&filename) {
            Ok(_) => {
                match vault.file_exists(&filename) {
                    Ok(exists) => {
                        if exists {
                            println!(" WARNING: delete reported success but file still exists!");
                            failed += 1;
                        } else {
                            println!(" succeeded");
                            deleted += 1;
                        }
                    }
                    Err(_) => {
                        println!(" succeeded");
                        deleted += 1;
                    }
                }
            }
            Err(e) => {
                println!(" failed (error: {})", e.get_code());
                failed += 1;
            }
        }
    }
    
    println!("\nDeleted: {}, Failed: {}", deleted, failed);
    
    if deleted > 0 {
        print!("Compacting vault to commit changes...");
        match vault.compact_vault() {
            Ok(_) => println!(" done"),
            Err(e) => println!(" warning: compact failed (error: {})", e.get_code()),
        }
    }
    Ok(())
}

/// Test vault integrity
pub fn test_vault(vault: &CBVault, pattern: &str) -> AppResult<()> {
    let search_res = vault.find_first(pattern, VAULT_FATTR_FILE, VAULT_FF_NEED_NAME | VAULT_FF_RECURSIVE);
    if search_res.is_err() {
        println!("No files found");
        return Ok(());
    }

    let search_data = search_res?;
    let mut tested = 0;
    let mut failed = 0;
    
    loop {
        if let Ok(filename) = vault.get_search_result_name(search_data) {
            print!("Testing {}", filename);
            
            let vault_file_res = vault.open_file_ex(
                &filename,
                VAULT_FOM_OPEN_EXISTING,
                false,
                true,
                false,
                false,
                VAULT_EM_NONE,
                "",
                VAULT_CM_NONE,
                0,
                0,
            );
            
            if let Ok(mut in_file) = vault_file_res {
                let mut buf: Vec<u8> = vec![0; BUFFER_SIZE];
                let mut ok = true;
                
                loop {
                    if let Ok(block_read) = in_file.read(&mut buf) {
                        if block_read == 0 {
                            break;
                        }
                    } else {
                        ok = false;
                        break;
                    }
                }
                
                let _ = in_file.close();
                
                if ok {
                    println!(" OK");
                    tested += 1;
                } else {
                    println!(" FAILED");
                    failed += 1;
                }
            } else {
                println!(" FAILED to open");
                failed += 1;
            }
        }
        
        if !vault.find_next(search_data).unwrap_or(false) {
            break;
        }
    }
    
    let _ = vault.find_close(search_data);
    
    println!("\nTested: {}, Failed: {}", tested, failed);
    Ok(())
}

/*
 * SQLite Vault - Database-Backed Callback Mode Implementation
 *
 * This module demonstrates CBFS Vault's callback mode by storing vault
 * data in an SQLite database using fixed-size chunks (blocks).
 *
 * Benefits:
 *   - Scalability: Supports massive vaults with constant memory usage.
 *   - Efficiency: Reads/Writes only touch relevant 1MB blocks.
 *   - Persistence: Data survives app restarts.
 */

use std::sync::Mutex;
use std::cmp::{min, max};
use rusqlite::{Connection, params, OptionalExtension};
use cbfsvault::cbvault::{
    CBVault,
    CBVaultVaultOpenEventArgs,
    CBVaultVaultCloseEventArgs,
    CBVaultVaultReadEventArgs,
    CBVaultVaultWriteEventArgs,
    CBVaultVaultGetSizeEventArgs,
    CBVaultVaultSetSizeEventArgs,
    CBVaultVaultFlushEventArgs,
};
use cbfsvault::cbvaultdrive::{
    CBVaultDrive,
    CBVaultDriveVaultOpenEventArgs,
    CBVaultDriveVaultCloseEventArgs,
    CBVaultDriveVaultReadEventArgs,
    CBVaultDriveVaultWriteEventArgs,
    CBVaultDriveVaultGetSizeEventArgs,
    CBVaultDriveVaultSetSizeEventArgs,
    CBVaultDriveVaultFlushEventArgs,
};


// ============================================================================
// CONSTANTS & GLOBALS
// ============================================================================

const BLOCK_SIZE: i64 = 1024 * 1024; // 1MB blocks

/// Thread-safe SQLite connection for vault storage.
static DB_CONNECTION: once_cell::sync::Lazy<Mutex<Option<Connection>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

/// Transaction state tracking
static IN_TRANSACTION: once_cell::sync::Lazy<Mutex<bool>> =
    once_cell::sync::Lazy::new(|| Mutex::new(false));

/// Cached size of the vault data to avoid DB roundtrips on every GetSize
static VAULT_SIZE_CACHE: once_cell::sync::Lazy<Mutex<i64>> =
    once_cell::sync::Lazy::new(|| Mutex::new(0));

/// Default database filename
static DB_FILENAME: once_cell::sync::Lazy<Mutex<String>> =
    once_cell::sync::Lazy::new(|| Mutex::new(String::from("vault_storage.db")));

// ============================================================================
// PUBLIC API
// ============================================================================

/// Sets up SQLite callback mode on the given vault.
pub fn setup_callbacks(vault: &mut CBVault, db_path: Option<&str>) {
    if let Some(path) = db_path {
        *DB_FILENAME.lock().unwrap() = path.to_string();
    }
    
    if let Err(e) = vault.set_callback_mode(true) {
        eprintln!("[SQLiteVault] Failed to enable callback mode: {:?}", e);
        return;
    }
    
    vault.set_on_vault_open(Some(on_vault_open));
    vault.set_on_vault_close(Some(on_vault_close));
    vault.set_on_vault_read(Some(on_vault_read));
    vault.set_on_vault_write(Some(on_vault_write));
    vault.set_on_vault_flush(Some(on_vault_flush));
    vault.set_on_vault_get_size(Some(on_vault_get_size));
    vault.set_on_vault_set_size(Some(on_vault_set_size));
    
    println!("[SQLiteVault] Callback mode enabled - data will be stored in: {}", 
             DB_FILENAME.lock().unwrap());
}

/// Sets up SQLite callback mode on the given vault drive.
pub fn setup_drive_callbacks(drive: &mut CBVaultDrive, db_path: Option<&str>) {
    if let Some(path) = db_path {
        *DB_FILENAME.lock().unwrap() = path.to_string();
    }
    
    if let Err(e) = drive.set_callback_mode(true) {
        eprintln!("[SQLiteVault] Failed to enable callback mode for drive: {:?}", e);
        return;
    }
    
    drive.set_on_vault_open(Some(on_drive_vault_open));
    drive.set_on_vault_close(Some(on_drive_vault_close));
    drive.set_on_vault_read(Some(on_drive_vault_read));
    drive.set_on_vault_write(Some(on_drive_vault_write));
    drive.set_on_vault_flush(Some(on_drive_vault_flush));
    drive.set_on_vault_get_size(Some(on_drive_vault_get_size));
    drive.set_on_vault_set_size(Some(on_drive_vault_set_size));
    
    println!("[SQLiteVault] Drive callback mode enabled - data will be stored in: {}", 
             DB_FILENAME.lock().unwrap());
}



// ============================================================================
// DATABASE LOGIC
// ============================================================================

fn init_database() -> Result<(), rusqlite::Error> {
    let db_file = DB_FILENAME.lock().unwrap().clone();
    let conn = Connection::open(&db_file)?;
    
    // Performance tuning
    conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
    ")?;

    // Create tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS vault_metadata (
            key TEXT PRIMARY KEY,
            value INTEGER
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS vault_blocks (
            block_id INTEGER PRIMARY KEY,
            data BLOB
        )",
        [],
    )?;

    // Initialize size if not present
    conn.execute(
        "INSERT OR IGNORE INTO vault_metadata (key, value) VALUES ('total_size', 0)",
        [],
    )?;
    
    // Load current size into cache
    let existing_size: i64 = conn.query_row(
        "SELECT value FROM vault_metadata WHERE key = 'total_size'",
        [],
        |row| row.get(0)
    ).unwrap_or(0);
    
    *VAULT_SIZE_CACHE.lock().unwrap() = existing_size;
    *DB_CONNECTION.lock().unwrap() = Some(conn);
    *IN_TRANSACTION.lock().unwrap() = false;
    
    println!("[SQLiteVault] Database initialized. Current Size: {} bytes. Block Size: {} bytes", existing_size, BLOCK_SIZE);
    
    Ok(())
}

/// Start a transaction if one isn't already active
fn ensure_transaction(conn: &Connection) -> Result<(), rusqlite::Error> {
    let mut flag = IN_TRANSACTION.lock().unwrap();
    if !*flag {
        conn.execute("BEGIN IMMEDIATE", [])?;
        *flag = true;
    }
    Ok(())
}

/// Commit the current transaction if active
fn commit_transaction() -> bool {
    let mut conn_guard = DB_CONNECTION.lock().unwrap();
    if let Some(conn) = conn_guard.as_mut() {
        let mut flag = IN_TRANSACTION.lock().unwrap();
        if *flag {
            if let Err(e) = conn.execute("COMMIT", []) {
                eprintln!("[SQLiteVault] Commit failed: {:?}", e);
                return false;
            }
            *flag = false;
        }
    }
    true
}

fn read_range(offset: i64, count: usize, buffer_ptr: *mut u8) -> bool {
    let conn_guard = DB_CONNECTION.lock().unwrap();
    let conn = match conn_guard.as_ref() {
        Some(c) => c,
        None => return false,
    };

    let start_block = offset / BLOCK_SIZE;
    let end_offset = offset + count as i64;
    let end_block = (end_offset - 1) / BLOCK_SIZE;

    let mut bytes_read = 0;
    
    // Prepare statement once for the loop
    let mut stmt = match conn.prepare_cached("SELECT data FROM vault_blocks WHERE block_id = ?") {
        Ok(s) => s,
        Err(_) => return false,
    };

    for block_id in start_block..=end_block {
        let block_start_pos = block_id * BLOCK_SIZE;
        
        // Determine overlap between this block and the requested range
        let read_start = max(offset, block_start_pos);
        let read_end = min(end_offset, block_start_pos + BLOCK_SIZE);
        let read_len = (read_end - read_start) as usize;

        if read_len == 0 { continue; }

        let block_offset = (read_start - block_start_pos) as usize; // Where to read inside the block
        let buffer_offset = bytes_read; // Where to write inside user buffer

        // Fetch block
        let block_data_result: Result<Vec<u8>, _> = stmt.query_row(params![block_id], |row| row.get(0));

        match block_data_result {
            Ok(data) => {
                // If we found data, copy the slice
                if block_offset < data.len() {
                    let available = min(read_len, data.len() - block_offset);
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            data.as_ptr().add(block_offset),
                            buffer_ptr.add(buffer_offset),
                            available
                        );
                    }
                    if available < read_len {
                         unsafe {
                            std::ptr::write_bytes(buffer_ptr.add(buffer_offset + available), 0, read_len - available);
                        }
                    }
                } else {
                    // Block exists but data is shorter than this offset?? Zero fill.
                     unsafe {
                        std::ptr::write_bytes(buffer_ptr.add(buffer_offset), 0, read_len);
                    }
                }
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Block missing (sparse), fill with zeros
                unsafe {
                    std::ptr::write_bytes(buffer_ptr.add(buffer_offset), 0, read_len);
                }
            },
            Err(e) => {
                eprintln!("[SQLiteVault] Read Error block {}: {:?}", block_id, e);
                return false;
            }
        }
        
        bytes_read += read_len;
    }

    true
}

fn write_range(offset: i64, count: usize, buffer_ptr: *const u8) -> bool {
    let mut conn_guard = DB_CONNECTION.lock().unwrap();
    let conn = match conn_guard.as_mut() {
        Some(c) => c,
        None => return false,
    };
    
    // Ensure we are in a transaction
    if let Err(e) = ensure_transaction(conn) {
        eprintln!("[SQLiteVault] Failed to start transaction: {:?}", e);
        return false;
    }

    let start_block = offset / BLOCK_SIZE;
    let end_offset = offset + count as i64;
    let end_block = (end_offset - 1) / BLOCK_SIZE;

    let mut bytes_written = 0;

    for block_id in start_block..=end_block {
        let block_start_pos = block_id * BLOCK_SIZE;
        
        // Determine overlap
        let write_start = max(offset, block_start_pos);
        let write_end = min(end_offset, block_start_pos + BLOCK_SIZE);
        let write_len = (write_end - write_start) as usize;

        if write_len == 0 { continue; }
        
        let block_offset = (write_start - block_start_pos) as usize; // Where to write inside the block
        let buffer_offset = bytes_written; // Where to read from user buffer

        // Optimization: If writing a FULL block, we don't need to read the old one.
        if write_len == BLOCK_SIZE as usize {
            let mut new_block = vec![0u8; BLOCK_SIZE as usize];
            unsafe {
                std::ptr::copy_nonoverlapping(
                    buffer_ptr.add(buffer_offset),
                    new_block.as_mut_ptr(),
                    BLOCK_SIZE as usize
                );
            }
            if let Err(e) = conn.execute(
                "INSERT OR REPLACE INTO vault_blocks (block_id, data) VALUES (?1, ?2)",
                params![block_id, new_block]
            ) {
                eprintln!("[SQLiteVault] Write full block {} failed: {:?}", block_id, e);
                return false;
            }
        } else {
            // Partial write: Read-Modify-Write
            let current_data: Option<Vec<u8>> = conn.query_row(
                "SELECT data FROM vault_blocks WHERE block_id = ?",
                params![block_id],
                |row| row.get(0)
            ).optional().unwrap_or(None);

            let mut block_data = current_data.unwrap_or_else(|| vec![0u8; BLOCK_SIZE as usize]);
            
            if block_data.len() < BLOCK_SIZE as usize {
                block_data.resize(BLOCK_SIZE as usize, 0);
            }

            // Apply patch
            unsafe {
                std::ptr::copy_nonoverlapping(
                    buffer_ptr.add(buffer_offset),
                    block_data.as_mut_ptr().add(block_offset),
                    write_len
                );
            }

            if let Err(e) = conn.execute(
                "INSERT OR REPLACE INTO vault_blocks (block_id, data) VALUES (?1, ?2)",
                params![block_id, block_data]
            ) {
                 eprintln!("[SQLiteVault] Write partial block {} failed: {:?}", block_id, e);
                 return false;
            }
        }

        bytes_written += write_len;
    }
    
    // Update Size if we grew
    let current_size = *VAULT_SIZE_CACHE.lock().unwrap();
    if end_offset > current_size {
        if let Err(e) = conn.execute(
            "UPDATE vault_metadata SET value = ? WHERE key = 'total_size'",
            params![end_offset]
        ) {
             eprintln!("[SQLiteVault] Update size failed: {:?}", e);
             return false;
        }
        *VAULT_SIZE_CACHE.lock().unwrap() = end_offset;
    }
    
    // NOTE: We do NOT commit here anymore. We wait for on_vault_flush or on_vault_close.

    true
}

fn set_size_internal(new_size: i64) -> bool {
    let mut conn_guard = DB_CONNECTION.lock().unwrap();
    let conn = match conn_guard.as_mut() {
        Some(c) => c,
        None => return false,
    };
    
    if let Err(e) = ensure_transaction(conn) {
        eprintln!("[SQLiteVault] Failed to start transaction in set_size: {:?}", e);
        return false;
    }

    // Update metadata
    if let Err(_) = conn.execute(
        "UPDATE vault_metadata SET value = ? WHERE key = 'total_size'",
        params![new_size]
    ) { return false; }

    // Calc last needed block
    let last_block_id = if new_size == 0 { -1 } else { (new_size - 1) / BLOCK_SIZE };
    
    // 1. Delete blocks beyond the new end
    if let Err(_) = conn.execute("DELETE FROM vault_blocks WHERE block_id > ?", params![last_block_id]) {
        return false;
    }

    // 2. Truncate the last block if it exists and new_size is inside it
    if new_size > 0 {
        let last_bytes = (new_size % BLOCK_SIZE) as usize;
        
        if last_bytes > 0 {
            // Read last block
             let current_data: Option<Vec<u8>> = conn.query_row(
                "SELECT data FROM vault_blocks WHERE block_id = ?",
                params![last_block_id],
                |row| row.get(0)
            ).optional().unwrap_or(None);

            if let Some(mut data) = current_data {
                if data.len() > last_bytes {
                    data.truncate(last_bytes);
                    
                    if let Err(_) = conn.execute(
                        "UPDATE vault_blocks SET data = ? WHERE block_id = ?",
                        params![data, last_block_id]
                    ) { return false; }
                }
            }
        }
    }

    // NOTE: No commit here.
    
    // Update cache
    *VAULT_SIZE_CACHE.lock().unwrap() = new_size;
    
    true
}

// ============================================================================
// CALLBACK WRAPPERS (BOILERPLATE)
// ============================================================================

pub fn on_vault_open(_sender: &CBVault, e: &mut CBVaultVaultOpenEventArgs) {
    if let Err(err) = init_database() {
        eprintln!("[SQLiteVault] Failed to init database: {:?}", err);
        e.set_result_code(-1);
        return;
    }
    e.set_vault_handle(1);
    e.set_result_code(0);
}

pub fn on_vault_close(_sender: &CBVault, e: &mut CBVaultVaultCloseEventArgs) {
    let _ = commit_transaction();
    let mut conn_guard = DB_CONNECTION.lock().unwrap();
    *conn_guard = None; // Close DB
    e.set_result_code(0);
}

pub fn on_vault_read(_sender: &CBVault, e: &mut CBVaultVaultReadEventArgs) {
    if read_range(e.offset(), e.count() as usize, e.buffer()) {
        e.set_result_code(0);
    } else {
        e.set_result_code(-1);
    }
}

pub fn on_vault_write(_sender: &CBVault, e: &mut CBVaultVaultWriteEventArgs) {
    if write_range(e.offset(), e.count() as usize, e.buffer()) {
        e.set_result_code(0);
    } else {
        e.set_result_code(-1);
    }
}

pub fn on_vault_flush(_sender: &CBVault, e: &mut CBVaultVaultFlushEventArgs) {
    if commit_transaction() {
         e.set_result_code(0);
    } else {
         e.set_result_code(-1);
    }
}

pub fn on_vault_get_size(_sender: &CBVault, e: &mut CBVaultVaultGetSizeEventArgs) {
    e.set_size(*VAULT_SIZE_CACHE.lock().unwrap());
    e.set_result_code(0);
}

pub fn on_vault_set_size(_sender: &CBVault, e: &mut CBVaultVaultSetSizeEventArgs) {
    if set_size_internal(e.new_size()) {
        e.set_result_code(0);
    } else {
        e.set_result_code(-1);
    }
}

// ============================================================================
// DRIVE CALLBACK WRAPPERS
// ============================================================================

pub fn on_drive_vault_open(_sender: &CBVaultDrive, e: &mut CBVaultDriveVaultOpenEventArgs) {
    if let Err(err) = init_database() {
        eprintln!("[SQLiteVault] Failed to init database: {:?}", err);
        e.set_result_code(-1);
        return;
    }
    e.set_vault_handle(1);
    e.set_result_code(0);
}

pub fn on_drive_vault_close(_sender: &CBVaultDrive, e: &mut CBVaultDriveVaultCloseEventArgs) {
    let _ = commit_transaction();
    let mut conn_guard = DB_CONNECTION.lock().unwrap();
    *conn_guard = None;
    e.set_result_code(0);
}

pub fn on_drive_vault_read(_sender: &CBVaultDrive, e: &mut CBVaultDriveVaultReadEventArgs) {
     if read_range(e.offset(), e.count() as usize, e.buffer()) {
        e.set_result_code(0);
    } else {
        e.set_result_code(-1);
    }
}

pub fn on_drive_vault_write(_sender: &CBVaultDrive, e: &mut CBVaultDriveVaultWriteEventArgs) {
     if write_range(e.offset(), e.count() as usize, e.buffer()) {
        e.set_result_code(0);
    } else {
        e.set_result_code(-1);
    }
}

pub fn on_drive_vault_flush(_sender: &CBVaultDrive, e: &mut CBVaultDriveVaultFlushEventArgs) {
    if commit_transaction() {
         e.set_result_code(0);
    } else {
         e.set_result_code(-1);
    }
}

pub fn on_drive_vault_get_size(_sender: &CBVaultDrive, e: &mut CBVaultDriveVaultGetSizeEventArgs) {
    e.set_size(*VAULT_SIZE_CACHE.lock().unwrap());
    e.set_result_code(0);
}

pub fn on_drive_vault_set_size(_sender: &CBVaultDrive, e: &mut CBVaultDriveVaultSetSizeEventArgs) {
     if set_size_internal(e.new_size()) {
        e.set_result_code(0);
    } else {
        e.set_result_code(-1);
    }
}




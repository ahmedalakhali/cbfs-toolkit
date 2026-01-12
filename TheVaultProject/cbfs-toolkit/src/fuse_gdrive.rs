/*
 * FUSE-based Google Drive Virtual Filesystem
 * Combines FUSE mounting with Google Drive API for cloud storage access
 */
// #![allow(static_mut_refs)] - No longer needed after refactor

use std::io::Cursor;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use cbfsconnect::fuse::*;
use crate::fuse_common::*;
use google_drive3::{api::{File, Scope}, DriveHub};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use tokio::runtime::Runtime;
use once_cell::sync::Lazy;
use chrono::{DateTime, Utc};

// FUSE constants and Error codes are imported from fuse_common

// File mode constants for FUSE
const S_IFDIR: i32 = 0o40000;  // Directory

const S_IFREG: i32 = 0o100000; // Regular file
const S_IRWXU: i32 = 0o700;    // User read/write/execute
const S_IRWXG: i32 = 0o070;    // Group read/write/execute
const S_IRWXO: i32 = 0o007;    // Others read/write/execute

// Type aliases
type PathCache = HashMap<String, String>;     // Path -> Google Drive ID
type FileCache = HashMap<String, File>;       // ID -> File metadata
type ContentCache = HashMap<String, Vec<u8>>; // ID -> File content

/// Virtual file entry for our in-memory representation
#[derive(Clone)]
struct VirtualEntry {
    id: String,
    name: String,
    is_dir: bool,
    size: i64,
    #[allow(dead_code)]
    mime_type: String,
    created: DateTime<Utc>,
    modified: DateTime<Utc>,
}

/// Google Drive Filesystem State
struct GDriveState {
    hub: Option<Arc<DriveHub<HttpsConnector<HttpConnector>>>>,
    path_cache: PathCache,
    file_cache: FileCache,
    content_cache: ContentCache,
    entries: HashMap<String, VirtualEntry>,  // Path -> VirtualEntry
    dirty_files: HashSet<String>,            // Set of IDs that need uploading
    runtime: Arc<Runtime>,
}

impl GDriveState {
    fn new() -> Self {
        GDriveState {
            hub: None,
            path_cache: HashMap::new(),
            file_cache: HashMap::new(),
            content_cache: HashMap::new(),
            entries: HashMap::new(),
            dirty_files: HashSet::new(),
            runtime: Arc::new(Runtime::new().unwrap()),
        }
    }
}

// Global state (thread-safe)
static GDRIVE_STATE: Lazy<Mutex<GDriveState>> = Lazy::new(|| Mutex::new(GDriveState::new()));
static READ_ONLY_MODE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true)); // Default: read-only
static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// Permission denied error code (EACCES) is imported from fuse_common

/// Set read-only mode for cloud access (true = read-only, false = read-write)
pub fn set_read_only_mode(read_only: bool) {
    *READ_ONLY_MODE.lock().unwrap() = read_only;
    println!("[INFO] Cloud access mode: {}", if read_only { "READ-ONLY" } else { "READ-WRITE" });
}

/// Check if write operations should be blocked
fn is_write_blocked() -> bool {
    *READ_ONLY_MODE.lock().unwrap()
}

// ======================
// Authentication & Setup
// ======================

fn authenticate(client_secret_path: &str) -> Result<(), String> {
    let mut state = GDRIVE_STATE.lock().unwrap();
    
    // Create hub
    let hub = state.runtime.block_on(async {
        // Load client secret - from Credential Manager or file
        let secret = if client_secret_path == "__CREDENTIAL_MANAGER__" {
            // Load from Windows Credential Manager
            let secret_json = crate::credential_store::get_client_secret()
                .map_err(|e| format!("Failed to access Credential Manager: {}", e))?
                .ok_or_else(|| "No client secret found in Credential Manager".to_string())?;
            

            
            // robustly parse the secret (handle "installed" or "web" wrapper)
            let val: serde_json::Value = serde_json::from_str(&secret_json)
                .map_err(|e| format!("Invalid JSON in Credential Manager: {}", e))?;
            
            let nested = val.get("installed").or_else(|| val.get("web")).unwrap_or(&val);
            serde_json::from_value(nested.clone())
                .map_err(|e| format!("Failed to parse client secret structure: {}", e))?
        } else {
            // Load from file
            match yup_oauth2::read_application_secret(client_secret_path).await {
                Ok(s) => s,
                Err(e) => return Err(format!("Failed to read client secret: {}", e)),
            }
        };

        // Prepare token file - this loads existing token from Credential Manager to temp file
        let token_path = crate::credential_store::prepare_token_file();
        let token_path_str = token_path.to_string_lossy().to_string();
        
        let auth = match InstalledFlowAuthenticator::builder(
            secret,
            InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk(&token_path_str)
        .build()
        .await {
            Ok(a) => a,
            Err(e) => return Err(format!("Failed to create authenticator: {}", e)),
        };

        let hub = DriveHub::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .build(),
            ),
            auth,
        );

        Ok(hub)
    })?;
    
    state.hub = Some(Arc::new(hub));
    
    // Populate root directory
    populate_directory(&mut state, "root", "/")?;
    
    Ok(())
}

fn populate_directory(state: &mut GDriveState, parent_id: &str, dir_path: &str) -> Result<(), String> {
    let hub = match &state.hub {
        Some(h) => h.clone(),
        None => return Err("Hub not initialized".to_string()),
    };
    
    state.runtime.block_on(async {
        let q = format!("'{}' in parents and trashed = false", parent_id);
        println!("[DEBUG] Querying Google Drive: {}", q);
        
        let result = hub.files().list()
            .q(&q)
            .add_scope(Scope::Full)
            .param("fields", "files(id, name, mimeType, size, createdTime, modifiedTime)")
            .doit()
            .await;
        
        match result {
            Ok((_, list)) => {
                if let Some(files) = list.files {
                    println!("[INFO] Found {} files in '{}'", files.len(), dir_path);
                    
                    for file in files {
                        if let (Some(name), Some(id)) = (&file.name, &file.id) {
                            let full_path = if dir_path == "/" {
                                format!("/{}", name)
                            } else {
                                format!("{}/{}", dir_path, name)
                            };
                            
                            let is_dir = file.mime_type.as_deref() == Some("application/vnd.google-apps.folder");
                            let size = file.size.unwrap_or(0);
                            
                            println!("[DEBUG] Cached: {} ({})", full_path, if is_dir { "dir" } else { "file" });
                            
                            // Create virtual entry
                            let entry = VirtualEntry {
                                id: id.clone(),
                                name: name.clone(),
                                is_dir,
                                size,
                                mime_type: file.mime_type.clone().unwrap_or_default(),
                                created: Utc::now(),
                                modified: Utc::now(),
                            };
                            
                            state.path_cache.insert(full_path.clone(), id.clone());
                            state.file_cache.insert(id.clone(), file);
                            state.entries.insert(full_path, entry);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(format!("Failed to list files: {:?}", e)),
        }
    })
}

fn download_file(state: &mut GDriveState, id: &str) -> Option<Vec<u8>> {
    // Check cache first
    if let Some(content) = state.content_cache.get(id) {
        return Some(content.clone());
    }
    
    let hub = match &state.hub {
        Some(h) => h.clone(),
        None => return None,
    };
    
    let id_clone = id.to_string();
    
    let result = state.runtime.block_on(async {
        let response = hub.files().get(&id_clone)
            .acknowledge_abuse(true)
            .add_scope(Scope::Full)
            .param("alt", "media")
            .doit()
            .await;
        
        match response {
            Ok((mut resp, _)) => {
                match hyper::body::to_bytes(resp.body_mut()).await {
                    Ok(bytes) => Ok(bytes.to_vec()),
                    Err(e) => Err(format!("Failed to read body: {:?}", e)),
                }
            }
            Err(e) => Err(format!("Failed to download: {:?}", e)),
        }
    });
    
    if let Ok(content) = result {
        state.content_cache.insert(id_clone, content.clone());
        return Some(content);
    }
    
    None
}

impl GDriveState {
    // Prepare upload data (inside lock)
    fn prepare_upload(&self, path: &str) -> Result<Option<UploadData>, String> {
        let entry = match self.entries.get(path) {
            Some(e) => e.clone(),
            None => return Err(format!("No entry for path: {}", path)),
        };
        
        if entry.is_dir {
            return Err("Cannot upload a directory as a file".to_string());
        }
        
        let id = entry.id.clone();
        if !self.dirty_files.contains(&id) {
            return Ok(None); // Not modified
        }
        
        let content = match self.content_cache.get(&id) {
            Some(c) => c.clone(),
            None => return Err(format!("No content in cache for ID: {}", id)),
        };
        
        let hub = match &self.hub {
            Some(h) => h.clone(),
            None => return Err("Hub not initialized".to_string()),
        };

        let is_new = id.starts_with("new_");
        let parent_id = if is_new {
            // Resolve parent ID
            let parent_path = if let Some(idx) = path.rfind('/') {
                if idx == 0 { "/" } else { &path[..idx] }
            } else {
                "/"
            };
            
            if parent_path == "/" {
                "root".to_string()
            } else {
                self.path_cache.get(parent_path).cloned().unwrap_or_else(|| "root".to_string())
            }
        } else {
            "root".to_string() // Not needed for update
        };

        Ok(Some(UploadData {
            hub,
            runtime: self.runtime.clone(),
            content,
            entry,
            is_new,
            parent_id,
        }))
    }

    // Finish upload (inside lock)
    fn finish_upload(&mut self, path: &str, result: Result<File, String>) {
        match result {
            Ok(uploaded_file) => {
                if let Some(entry) = self.entries.get_mut(path) {
                    let old_id = entry.id.clone();
                    
                    if let Some(new_id) = uploaded_file.id.clone() {
                        println!("[INFO] Upload finished for: {} (ID: {})", entry.name, new_id);
                        
                        // Update caches
                        self.dirty_files.remove(&old_id);
                        self.dirty_files.remove(&new_id); // Ensure new ID is clean
                        
                        // Replace old ID with real ID
                        self.path_cache.insert(path.to_string(), new_id.clone());
                         entry.id = new_id.clone();
                        
                        if let Some(data) = self.content_cache.remove(&old_id) {
                            self.content_cache.insert(new_id.clone(), data);
                        }
                        self.file_cache.insert(new_id, uploaded_file);
                    }
                } else {
                     // Entry might have been deleted while uploading?
                     println!("[WARN] Entry for {} missing after upload.", path);
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Upload failed for {}: {}", path, e);
                // Keep it dirty to retry later? Or ignore?
            }
        }
    }
}

struct UploadData {
    hub: Arc<DriveHub<HttpsConnector<HttpConnector>>>,
    runtime: Arc<Runtime>,
    content: Vec<u8>,
    entry: VirtualEntry,
    is_new: bool,
    parent_id: String,
}

// Standalone function to perform upload (NO LOCK CODE HERE)
fn perform_upload(data: UploadData) -> Result<File, String> {
    data.runtime.block_on(async {
        let cursor = Cursor::new(data.content);
        let mut file_meta = File::default();

        if data.is_new {
            file_meta.name = Some(data.entry.name);
            file_meta.parents = Some(vec![data.parent_id]);
            
            data.hub.files().create(file_meta)
                .add_scope(Scope::Full)
                .upload(cursor, "application/octet-stream".parse().unwrap())
                .await
                .map(|(_, f)| f)
                .map_err(|e| format!("{:?}", e))
        } else {
            // Update existing
             data.hub.files().update(file_meta, &data.entry.id)
                .add_scope(Scope::Full)
                .upload(cursor, "application/octet-stream".parse().unwrap())
                .await
                .map(|(_, f)| f)
                .map_err(|e| format!("{:?}", e))
        }
    })
}

// ====================
// FUSE Event Handlers
// ====================

fn on_get_attr(sender: &FUSE, e: &mut FUSEGetAttrEventArgs) {
    let filepath = e.path();
    println!("[FUSE] GetAttr: {}", filepath);
    
    let state = GDRIVE_STATE.lock().unwrap();
    
    let uid = sender.get_uid().unwrap_or(0);
    let gid = sender.get_gid().unwrap_or(0);
    
    // Root directory
    if filepath == "/" {
        e.set_ino(1);
        e.set_mode(S_IFDIR | S_IRWXU | S_IRWXG | S_IRWXO);
        e.set_uid(uid);
        e.set_gid(gid);
        e.set_link_count(2);
        e.set_size(512);
        return;
    }
    
    // Look up file
    if let Some(entry) = state.entries.get(filepath) {
        e.set_ino(entry.id.chars().map(|c| c as i64).sum::<i64>().abs() + 2);
        
        if entry.is_dir {
            e.set_mode(S_IFDIR | S_IRWXU | S_IRWXG | S_IRWXO);
            e.set_size(512);
        } else {
            e.set_mode(S_IFREG | S_IRWXU | S_IRWXG | S_IRWXO);
            e.set_size(entry.size);
        }
        
        e.set_uid(uid);
        e.set_gid(gid);
        e.set_link_count(1);
    } else {
        e.set_result(-ENOENT);
    }
}

fn on_readdir(sender: &FUSE, e: &mut FUSEReadDirEventArgs) {
    let filepath = e.path();
    println!("[FUSE] ReadDir: {}", filepath);
    
    let state = GDRIVE_STATE.lock().unwrap();
    
    let uid = sender.get_uid().unwrap_or(0);
    let gid = sender.get_gid().unwrap_or(0);
    
    // Find all children of this directory
    let prefix = if filepath == "/" {
        "/".to_string()
    } else {
        format!("{}/", filepath)
    };
    
    for (path, entry) in &state.entries {
        // Check if this is a direct child
        let is_child = if filepath == "/" {
            path.starts_with("/") && path.matches('/').count() == 1
        } else {
            path.starts_with(&prefix) && path[prefix.len()..].find('/').is_none()
        };
        
        if is_child {
            let mode = if entry.is_dir {
                S_IFDIR | S_IRWXU | S_IRWXG | S_IRWXO
            } else {
                S_IFREG | S_IRWXU | S_IRWXG | S_IRWXO
            };
            
            let ino = entry.id.chars().map(|c| c as i64).sum::<i64>().abs() + 2;
            
            let _ = sender.fill_dir(
                e.filler_context(),
                &entry.name,
                ino,
                mode,
                uid,
                gid,
                1,
                entry.size,
                &entry.created,
                &entry.modified,
                &entry.created,
            );
        }
    }
}

fn on_open(_sender: &FUSE, e: &mut FUSEOpenEventArgs) {
    let filepath = e.path();
    println!("[FUSE] Open: {}", filepath);
    
    let state = GDRIVE_STATE.lock().unwrap();
    
    if state.entries.contains_key(filepath) {
        // File exists - allow open
    } else {
        e.set_result(-ENOENT);
    }
}

fn on_read(_sender: &FUSE, e: &mut FUSEReadEventArgs) {
    let filepath = e.path();
    println!("[FUSE] Read: {} (offset={}, size={})", filepath, e.offset(), e.size());
    
    let mut state = GDRIVE_STATE.lock().unwrap();
    
    if let Some(entry) = state.entries.get(filepath).cloned() {
        let id = entry.id.clone();
        
        if let Some(content) = download_file(&mut state, &id) {
            let offset = e.offset() as usize;
            let len = e.size() as usize;
            
            if offset < content.len() {
                let end = std::cmp::min(offset + len, content.len());
                let slice = &content[offset..end];
                
                unsafe {
                    let buf_ptr = e.buffer();
                    if !buf_ptr.is_null() {
                        std::ptr::copy_nonoverlapping(slice.as_ptr(), buf_ptr, slice.len());
                    }
                }
                e.set_result(slice.len() as i32);
            } else {
                e.set_result(0);
            }
        } else {
            e.set_result(-ENOENT);
        }
    } else {
        e.set_result(-ENOENT);
    }
}

fn on_release(_sender: &FUSE, e: &mut FUSEReleaseEventArgs) {
    let filepath = e.path();
    
    // Step 1: Prepare (Lock)
    let upload_task = {
        let state = GDRIVE_STATE.lock().unwrap();
        match state.prepare_upload(filepath) {
            Ok(opt) => opt,
            Err(err) => {
                 // Only log error if it's strictly a preparing error, 
                 // but "No entry" or "Not dirty" are fine to ignore usually,
                 // except we probably want to know if upload failed.
                 if !err.starts_with("No entry") { // Simple filter
                     // eprintln!("[WARN] Prepare upload failed: {}", err);
                 }
                 None
            }
        }
    };

    // Step 2: Upload (No Lock)
    if let Some(task) = upload_task {
        // println!("[DEBUG] Uploading {} in background...", filepath);
        let result = perform_upload(task);
        
        // Step 3: Finish (Lock)
        let mut state = GDRIVE_STATE.lock().unwrap();
        state.finish_upload(filepath, result);
    }
}

fn on_stat_fs(_sender: &FUSE, e: &mut FUSEStatFSEventArgs) {
    println!("[FUSE] StatFS");
    
    let total_size: i64 = 15 * 1024 * 1024 * 1024; // 15GB (free tier)
    
    e.set_block_size(SECTOR_SIZE);
    e.set_total_blocks(total_size / SECTOR_SIZE);
    e.set_free_blocks(total_size / SECTOR_SIZE / 2); // Show 50% free
    e.set_free_blocks_avail(e.free_blocks());
    e.set_max_filename_length(255);
}

// stub handlers for unimplemented operations
fn is_path_malicious(path: &str) -> bool {
    // Block paths containing the UTF-8 replacement character
    // This character indicates corrupted or malicious naming used by shortcut worms
    path.contains('\u{FFFD}')
}

fn is_extension_allowed(path: &str) -> bool {
    let lower_path = path.to_lowercase();
    // List of disallowed extensions
    let disallowed_extensions = [".exe", ".dll", ".com", ".bat", ".cmd", ".vbs", ".msi", ".scr", ".pif"];
    
    for ext in disallowed_extensions.iter() {
        if lower_path.ends_with(ext) {
            return false;
        }
    }
    true
}

fn on_create(_sender: &FUSE, e: &mut FUSECreateEventArgs) {
    let filepath = e.path();
    println!("[FUSE] Create: {}", filepath);
    
    // Check write permission
    if is_write_blocked() {
        println!("[FUSE] Create denied - read-only mode");
        e.set_result(-EACCES);
        return;
    }
    
    if is_path_malicious(filepath) {
        println!("[WARNING] Blocked creation of malicious path: {}", filepath);
        e.set_result(-EACCES);
        return;
    }

    if !is_extension_allowed(filepath) {
        println!("[WARNING] Blocked creation of restricted file type: {}", filepath);
        e.set_result(-EACCES);
        return;
    }
    
    let mut state = GDRIVE_STATE.lock().unwrap();
    
    if state.entries.contains_key(filepath) {
        e.set_result(-EEXIST);
        return;
    }
    
    let filename = filepath.rsplit('/').next().unwrap_or(filepath).to_string();
    let id = format!("new_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    
    let entry = VirtualEntry {
        id: id.clone(),
        name: filename,
        is_dir: false,
        size: 0,
        mime_type: "application/octet-stream".to_string(),
        created: Utc::now(),
        modified: Utc::now(),
    };
    
    state.path_cache.insert(filepath.to_string(), id.clone());
    state.entries.insert(filepath.to_string(), entry);
    state.content_cache.insert(id.clone(), Vec::new());
    state.dirty_files.insert(id);
    
    e.set_result(0);
}

fn on_mk_dir(_sender: &FUSE, e: &mut FUSEMkDirEventArgs) {
    let filepath = e.path();
    println!("[FUSE] MkDir: {}", filepath);
    
    // Check write permission
    if is_write_blocked() {
        println!("[FUSE] MkDir denied - read-only mode");
        e.set_result(-EACCES);
        return;
    }
    
    if is_path_malicious(filepath) {
        println!("[WARNING] Blocked directory creation of malicious path: {}", filepath);
        e.set_result(-EACCES);
        return;
    }
    
    let mut state = GDRIVE_STATE.lock().unwrap();
    
    // Resolve parent ID
    let parent_path = if let Some(idx) = filepath.rfind('/') {
        if idx == 0 { "/" } else { &filepath[..idx] }
    } else {
        "/"
    };
    
    let parent_id = if parent_path == "/" {
        "root".to_string()
    } else {
        state.path_cache.get(parent_path).cloned().unwrap_or_else(|| "root".to_string())
    };
    
    let filename = filepath.rsplit('/').next().unwrap_or(filepath).to_string();
    let hub = state.hub.clone().unwrap();
    let runtime = &state.runtime;
    
    let result = runtime.block_on(async {
        let mut folder_meta = File::default();
        folder_meta.name = Some(filename.clone());
        folder_meta.mime_type = Some("application/vnd.google-apps.folder".to_string());
        folder_meta.parents = Some(vec![parent_id]);
        
        hub.files().create(folder_meta)
            .add_scope(Scope::Full)
            .upload(Cursor::new(Vec::new()), "application/octet-stream".parse().unwrap())
            .await
            .map(|(_, f)| f)
            .map_err(|err| format!("{:?}", err))
    });
    
    match result {
        Ok(folder) => {
            if let Some(id) = folder.id.clone() {
                println!("[INFO] Created folder: {} with ID: {}", filename, id);
                let entry = VirtualEntry {
                    id: id.clone(),
                    name: filename,
                    is_dir: true,
                    size: 0,
                    mime_type: "application/vnd.google-apps.folder".to_string(),
                    created: Utc::now(),
                    modified: Utc::now(),
                };
                state.path_cache.insert(filepath.to_string(), id.clone());
                state.file_cache.insert(id, folder);
                state.entries.insert(filepath.to_string(), entry);
                e.set_result(0);
            } else {
                e.set_result(-EINVAL);
            }
        }
        Err(err) => {
            eprintln!("[ERROR] MkDir failed: {}", err);
            e.set_result(-EINVAL);
        }
    }
}

fn on_unlink(_sender: &FUSE, e: &mut FUSEUnlinkEventArgs) {
    let filepath = e.path();
    println!("[FUSE] Unlink: {}", filepath);
    
    // Check write permission
    let blocked = is_write_blocked();
    println!("[FUSE] Unlink permission check - read_only_mode={}", blocked);
    if blocked {
        println!("[FUSE] Unlink denied - read-only mode");
        e.set_result(-EACCES);
        return;
    }
    
    let mut state = GDRIVE_STATE.lock().unwrap();
    
    let id = match state.path_cache.get(filepath) {
        Some(i) => i.clone(),
        None => {
            e.set_result(-ENOENT);
            return;
        }
    };
    
    let hub = state.hub.clone().unwrap();
    let runtime = &state.runtime;
    
    let result = runtime.block_on(async {
        hub.files().delete(&id)
            .add_scope(Scope::Full)
            .doit()
            .await
            .map(|_| ())
            .map_err(|err| format!("{:?}", err))
    });
    
    match result {
        Ok(_) => {
            println!("[INFO] Deleted file: {}", filepath);
            state.path_cache.remove(filepath);
            state.entries.remove(filepath);
            state.file_cache.remove(&id);
            state.content_cache.remove(&id);
            state.dirty_files.remove(&id);
            e.set_result(0);
        }
        Err(err) => {
            eprintln!("[ERROR] Unlink failed: {}", err);
            e.set_result(-EINVAL);
        }
    }
}

fn on_rm_dir(_sender: &FUSE, e: &mut FUSERmDirEventArgs) {
    let filepath = e.path();
    println!("[FUSE] RmDir: {}", filepath);
    
    // Check write permission
    if is_write_blocked() {
        println!("[FUSE] RmDir denied - read-only mode");
        e.set_result(-EACCES);
        return;
    }
    
    let mut state = GDRIVE_STATE.lock().unwrap();
    
    let id = match state.path_cache.get(filepath) {
        Some(i) => i.clone(),
        None => {
            e.set_result(-ENOENT);
            return;
        }
    };
    
    let hub = state.hub.clone().unwrap();
    let runtime = &state.runtime;
    
    let result = runtime.block_on(async {
        hub.files().delete(&id)
            .add_scope(Scope::Full)
            .doit()
            .await
            .map(|_| ())
            .map_err(|err| format!("{:?}", err))
    });
    
    match result {
        Ok(_) => {
            println!("[INFO] Deleted directory: {}", filepath);
            state.path_cache.remove(filepath);
            state.entries.remove(filepath);
            state.file_cache.remove(&id);
            e.set_result(0);
        }
        Err(err) => {
            eprintln!("[ERROR] RmDir failed: {}", err);
            e.set_result(-EINVAL);
        }
    }
}

fn on_write(_sender: &FUSE, e: &mut FUSEWriteEventArgs) {
    let filepath = e.path();
    println!("[FUSE] Write: {} (offset={}, size={})", filepath, e.offset(), e.size());
    
    // Check write permission
    if is_write_blocked() {
        println!("[FUSE] Write denied - read-only mode");
        e.set_result(-EACCES);
        return;
    }
    
    let mut state = GDRIVE_STATE.lock().unwrap();
    
    let id = match state.path_cache.get(filepath) {
        Some(i) => i.clone(),
        None => {
            e.set_result(-ENOENT);
            return;
        }
    };
    
    let size_written = {
        let content = state.content_cache.entry(id.clone()).or_insert_with(|| Vec::new());
        let offset = e.offset() as usize;
        let size = e.size() as usize;
        
        if offset + size > content.len() {
            content.resize(offset + size, 0);
        }
        
        unsafe {
            let buf_ptr = e.buffer();
            if !buf_ptr.is_null() {
                std::ptr::copy_nonoverlapping(buf_ptr, content.as_mut_ptr().add(offset), size);
            }
        }
        size
    };
    
    // Update entry size
    let content_len = state.content_cache.get(&id).map(|c| c.len()).unwrap_or(0);
    if let Some(entry) = state.entries.get_mut(filepath) {
        entry.size = content_len as i64;
        entry.modified = Utc::now();
    }
    
    state.dirty_files.insert(id);
    e.set_result(size_written as i32);
}

fn on_truncate(_sender: &FUSE, e: &mut FUSETruncateEventArgs) {
    let filepath = e.path();
    let size = e.size() as usize;
    println!("[FUSE] Truncate: {} to {}", filepath, size);
    
    // Check write permission
    if is_write_blocked() {
        println!("[FUSE] Truncate denied - read-only mode");
        e.set_result(-EACCES);
        return;
    }
    
    let mut state = GDRIVE_STATE.lock().unwrap();
    
    let id = match state.path_cache.get(filepath) {
        Some(i) => i.clone(),
        None => {
            e.set_result(-ENOENT);
            return;
        }
    };
    
    {
        let content = state.content_cache.entry(id.clone()).or_insert_with(|| Vec::new());
        content.resize(size, 0);
    }
    
    if let Some(entry) = state.entries.get_mut(filepath) {
        entry.size = size as i64;
    }
    
    state.dirty_files.insert(id);
    e.set_result(0);
}

fn on_rename(_sender: &FUSE, e: &mut FUSERenameEventArgs) {
    let old_path = e.old_path();
    let new_path = e.new_path();
    println!("[FUSE] Rename: {} -> {}", old_path, new_path);
    
    // Check write permission
    if is_write_blocked() {
        println!("[FUSE] Rename denied - read-only mode");
        e.set_result(-EACCES);
        return;
    }
    
    if is_path_malicious(new_path) {
        println!("[WARNING] Blocked malicious move: {} -> {}", old_path, new_path);
        // We return Access Denied to stop the virus from moving the file
        e.set_result(-EACCES);
        return;
    }

    if !is_extension_allowed(new_path) {
        println!("[WARNING] Blocked rename to restricted file type: {}", new_path);
        e.set_result(-EACCES);
        return;
    }
    
    let mut state = GDRIVE_STATE.lock().unwrap();
    
    let id = match state.path_cache.get(old_path) {
        Some(i) => i.clone(),
        None => {
            e.set_result(-ENOENT);
            return;
        }
    };
    
    let new_name = new_path.rsplit('/').next().unwrap_or(new_path).to_string();
    let hub = state.hub.clone().unwrap();
    let runtime = &state.runtime;
    
    let result = runtime.block_on(async {
        let mut file_meta = File::default();
        file_meta.name = Some(new_name.clone());
        
        hub.files().update(file_meta, &id)
            .add_scope(Scope::Full)
            .upload(Cursor::new(Vec::new()), "application/octet-stream".parse().unwrap())
            .await
            .map(|(_, f)| f)
            .map_err(|err| format!("{:?}", err))
    });
    
    match result {
        Ok(updated_file) => {
            println!("[INFO] Renamed: {} -> {}", old_path, new_path);
            
            // Update caches
            state.path_cache.remove(old_path);
            state.path_cache.insert(new_path.to_string(), id.clone());
            
            if let Some(mut entry) = state.entries.remove(old_path) {
                entry.name = new_name;
                state.entries.insert(new_path.to_string(), entry);
            }
            
            state.file_cache.insert(id, updated_file);
            e.set_result(0);
        }
        Err(err) => {
            eprintln!("[ERROR] Rename failed: {}", err);
            e.set_result(-EINVAL);
        }
    }
}

fn on_utime(_sender: &FUSE, _e: &mut FUSEUTimeEventArgs) {
    // Allow utime to succeed silently
}

fn on_fallocate(_sender: &FUSE, e: &mut FUSEFAllocateEventArgs) {
    println!("[FUSE] FAllocate: {} (not implemented)", e.path());
    e.set_result(-EINVAL);
}

// ==================
// Setup and Mount
// ==================

fn setup_event_handlers(fuse: &mut FUSE) {
    fuse.set_on_get_attr(Some(on_get_attr));
    fuse.set_on_read_dir(Some(on_readdir));
    fuse.set_on_open(Some(on_open));
    fuse.set_on_read(Some(on_read));
    fuse.set_on_release(Some(on_release));
    fuse.set_on_stat_fs(Some(on_stat_fs));
    fuse.set_on_create(Some(on_create));
    fuse.set_on_mk_dir(Some(on_mk_dir));
    fuse.set_on_unlink(Some(on_unlink));
    fuse.set_on_rm_dir(Some(on_rm_dir));
    fuse.set_on_write(Some(on_write));
    fuse.set_on_truncate(Some(on_truncate));
    fuse.set_on_rename(Some(on_rename));
    fuse.set_on_u_time(Some(on_utime));
    fuse.set_on_f_allocate(Some(on_fallocate));
}

fn check_driver() -> bool {
    let fuse = FUSE::new();
    
    let state = fuse.get_driver_status(GUID);
    if state.is_err() || state.unwrap() != cbfsconnect::MODULE_STATUS_RUNNING {
        println!("[ERROR] FUSE driver is not installed");
        fuse.dispose();
        return false;
    }
    
    let version = fuse.get_driver_version(GUID);
    if let Ok(v) = version {
        println!("[INFO] FUSE driver version: {}.{}.{}.{}",
            (v & 0x7FFF000000000000) >> 48,
            (v & 0xFFFF00000000) >> 32,
            (v & 0xFFFF0000) >> 16,
            v & 0xFFFF);
    }
    
    fuse.dispose();
    true
}

/// Main entry point - called from interactive.rs
/// allow_write: if false, blocks all write operations (read-only mode)
/// Returns a Result containing the active FUSE instance on success, or an error code on failure.
pub fn mount_fuse_gdrive(client_secret_path: &str, mount_point: &str, allow_write: bool) -> Result<&'static mut FUSE, i32> {
    println!("[INFO] Starting FUSE-based Google Drive...");
    
    // Set access mode based on user permission
    set_read_only_mode(!allow_write);
    
    // Check driver
    if !check_driver() {
        return Err(1);
    }
    
    // Authenticate and populate cache
    if let Err(e) = authenticate(client_secret_path) {
        eprintln!("[ERROR] Authentication failed: {}", e);
        return Err(1);
    }
    
    // Create and configure FUSE
    let mut fuse = FUSE::new();
    setup_event_handlers(&mut fuse);
    
    // Initialize
    if let Err(e) = fuse.initialize(GUID) {
        eprintln!("[ERROR] FUSE initialization failed: {}", e.get_code());
        fuse.dispose();
        return Err(e.get_code());
    }
    
    // Mount
    if let Err(e) = fuse.mount(mount_point) {
        eprintln!("[ERROR] Mount failed: {}", e.get_code());
        fuse.dispose();
        return Err(e.get_code());
    }
    
    ACTIVE.store(true, std::sync::atomic::Ordering::SeqCst);
    
    println!("[INFO] Google Drive mounted at {}", mount_point);
    // Note: We no longer block here. The FUSE instance is returned to the caller.
    // The caller is responsible for eventually unmounting and disposing.
    
    Ok(fuse)
}

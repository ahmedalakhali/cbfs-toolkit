use crate::errors::AppResult;

#[derive(Debug, Clone)]
pub struct VaultEntry {
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
    // Add other attributes as needed
}

/// Parameters for file operations (Domain Object)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileOpParams {
    pub recursive: bool,
    pub compression_level: i32,
    pub delete_after_archiving: bool,
    pub lowercase: bool,
    pub uppercase: bool,
    pub overwrite_files: bool,
    pub overwrite_files_set: bool, // Track if user explicitly set overwrite behavior
    pub always_yes: bool,
    pub output_path: String,
    pub add_empty_dirs: bool,
    pub root_dir: String,
}

impl FileOpParams {
    pub fn new() -> Self {
        FileOpParams {
            recursive: false,
            compression_level: 0,
            delete_after_archiving: false,
            lowercase: false,
            uppercase: false,
            overwrite_files: false,
            overwrite_files_set: false,
            always_yes: false,
            output_path: String::new(),
            add_empty_dirs: false,
            root_dir: String::new(),
        }
    }
}

pub trait VaultService {
    fn open(&mut self, path: &str, password: &str) -> AppResult<()>;
    fn create(&mut self, path: &str, password: &str, use_sqlite: bool) -> AppResult<()>;
    fn close(&mut self) -> AppResult<()>;
    fn is_open(&self) -> bool;
    #[allow(dead_code)]
    fn get_path(&self) -> String;
    
    // Core Operations
    fn add_files(&self, files: &[String], params: &mut FileOpParams) -> AppResult<()>;
    fn list_files(&self, pattern: &str) -> AppResult<Vec<VaultEntry>>;
    fn extract_files(&self, pattern: &str, output_path: &str, params: &FileOpParams) -> AppResult<()>;
    fn delete_files(&self, pattern: &str, recursive: bool) -> AppResult<()>;
    
    // Additional capabilities
    fn convert_to_file(&self, output_path: &str) -> AppResult<()>;
}

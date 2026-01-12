/*
 * CBFS Vault 2024 Rust Edition - Vault Console
 * Configuration and Constants
 */

// GUID for the vault driver
pub const GUID: &str = "{713CC6CE-B3E2-4fd9-838D-E28F558F6866}";


// Icon ID for drive mounting
pub const ICON_ID: &str = "0_VaultConsole";

// Default page size for new vaults
pub const DEFAULT_PAGE_SIZE: i32 = 4096;

// Buffer size for file operations
pub const BUFFER_SIZE: usize = 1024 * 1024; // 1MB

// Error codes
pub const ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;

// Answer codes for prompts
pub const ANSWER_YES: i32 = 1;
pub const ANSWER_NO: i32 = 2;
pub const ANSWER_ALL: i32 = 3;

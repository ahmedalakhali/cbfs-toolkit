/*
 * CBFS Connect 2024 Rust Edition - Shared FUSE Logic
 * Common constants and helper functions for FUSE drives
 */

// No imports needed after cleanup


// Error codes
pub const ENOENT: i32 = 2;
pub const EACCES: i32 = 13;
pub const EEXIST: i32 = 17;
pub const EINVAL: i32 = 22;

// FUSE constants
pub const SECTOR_SIZE: i64 = 512;
pub const GUID: &str = "{713CC6CE-B3E2-4fd9-838D-E28F558F6866}";

// Windows error codes
// ERROR_PRIVILEGE_NOT_HELD removed


// install_drivers removed as it is handled by main.rs/utils.rs



#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

#![crate_type = "lib"]
#![crate_name = "cbfsvault"]

pub mod cbfsvaultkey;
pub mod cbvault;
pub mod cbvaultdrive;

extern crate libloading as lib;

use ctor::ctor;

use std::{path::PathBuf, env, ffi::{c_void, c_char, c_long, CStr, OsStr,OsString}, fmt::Display, ptr::null_mut, io::SeekFrom, sync::OnceLock};
use lib::{Library, Symbol};
use chrono::*;

//pub(crate) type Int64ArrayType = *mut i64;
pub(crate) type IntPtrArrayType = *mut isize;
pub(crate) type IntArrayType = *mut i32;

//pub(crate) type ConstIntArrayType = *const i32;

//pub(crate) type ConstUInt64ArrayType = *mut u64;
//pub(crate) type UInt64ArrayType = *mut u64;
pub(crate) type UIntPtrArrayType = *mut usize;
//pub(crate) type UIntArrayType = *mut u32;

pub(crate) type ConstUIntPtrArrayType = *const usize;

// typedef LPVOID (CBFSVAULT_CALL CBFSVAULT_EvtStr)(LPVOID lpEvtStr, INT id, LPVOID val, INT opt);
type CBFSVaultEvtStrType = unsafe extern "system" fn(pEvtStr : usize, id : i32, value : *mut c_void, opt : i32) -> *mut c_void;

// typedef LPVOID (CBFSVAULT_CALL CBFSVAULT_EvtStr)(LPVOID lpEvtStr, INT id, LPVOID val, INT opt);
type CBFSVaultEvtStrSetType = unsafe extern "system" fn(pEvtStr : *mut c_char, id : i32, value : *const c_char, opt : i32) -> *mut c_void;

// typedef INT (CBFSVAULT_CALL CBFSVAULT_Stream)(LPVOID lpStream, INT op, LPVOID param[], LPVOID ret_val);
type CBFSVaultStreamType = unsafe extern "system" fn(pStream : usize, op : i32, params: ConstUIntPtrArrayType, ret_val : *const c_void) -> i32; // for Flush and Close

type CBFSVaultStreamRWType = unsafe extern "system" fn(pStream : usize, op : i32, params: ConstUIntPtrArrayType, ret_val : *mut i32) -> i32; // for Read and Write

type CBFSVaultStreamSeekType = unsafe extern "system" fn(pStream : usize, op : i32, params: ConstUIntPtrArrayType, ret_val : *mut u64) -> i32;

type CBFSVaultStreamGetLengthType = unsafe extern "system" fn(pStream : usize, op : i32, params: *const c_void, ret_val : *mut u64) -> i32;

type CBFSVaultStreamSetLengthType = unsafe extern "system" fn(pStream : usize, op : i32, params: ConstUIntPtrArrayType, ret_val : *const c_void) -> i32;

// typedef INT (CBFSVAULT_CALL *LPFNCBFSVAULTEVENT) (LPVOID lpObj, INT event_id, INT cparam, LPVOID param[], INT cbparam[]);
type CBFSVaultSinkDelegateType = unsafe extern "system" fn(pObj : usize, event_id : c_long, cparam : c_long, param : IntPtrArrayType, cbparam : IntArrayType) -> c_long;

static CBFSVault_EvtStr : OnceLock<Symbol<'static, CBFSVaultEvtStrType>> = OnceLock::new();
static CBFSVault_EvtStrSet : OnceLock<Symbol<'static, CBFSVaultEvtStrSetType>> = OnceLock::new();

static CBFSVault_Stream : OnceLock<Symbol<'static, CBFSVaultStreamType>> = OnceLock::new();
static CBFSVault_StreamRW : OnceLock<Symbol<'static, CBFSVaultStreamRWType>> = OnceLock::new();
static CBFSVault_StreamSeek : OnceLock<Symbol<'static, CBFSVaultStreamSeekType>> = OnceLock::new();
static CBFSVault_StreamGetLength : OnceLock<Symbol<'static, CBFSVaultStreamGetLengthType>> = OnceLock::new();
static CBFSVault_StreamSetLength : OnceLock<Symbol<'static, CBFSVaultStreamSetLengthType>> = OnceLock::new();

static LIB_HANDLE : OnceLock<Library> = OnceLock::new();

static LIB_LOADED : OnceLock<bool> = OnceLock::new();

/// Check if the native library has been loaded
#[inline]
pub(crate) fn is_lib_loaded() -> bool {
    LIB_LOADED.get().copied().unwrap_or(false)
}

////////////////////
// Product Constants
////////////////////

    // VAULT_ERR_INVALID_VAULT_FILE: The specified file is not a CBFS Vault vault.
pub const VAULT_ERR_INVALID_VAULT_FILE: i32 = -1;

    // VAULT_ERR_INVALID_PAGE_SIZE: The specified page size is not valid.
pub const VAULT_ERR_INVALID_PAGE_SIZE: i32 = -2;

    // VAULT_ERR_VAULT_CORRUPTED: The vault is corrupted.
pub const VAULT_ERR_VAULT_CORRUPTED: i32 = -3;

    // VAULT_ERR_TOO_MANY_TRANSACTIONS: Too many transactions active.
pub const VAULT_ERR_TOO_MANY_TRANSACTIONS: i32 = -4;

    // VAULT_ERR_FILE_ALREADY_EXISTS: A file, directory, symbolic link, or alternate stream with the specified name already exists.
pub const VAULT_ERR_FILE_ALREADY_EXISTS: i32 = -5;

    // VAULT_ERR_TRANSACTIONS_STILL_ACTIVE: One or more transactions are still active.
pub const VAULT_ERR_TRANSACTIONS_STILL_ACTIVE: i32 = -6;

    // VAULT_ERR_TAG_ALREADY_EXISTS: The specified file tag already exists.
pub const VAULT_ERR_TAG_ALREADY_EXISTS: i32 = -7;

    // VAULT_ERR_FILE_NOT_FOUND: The specified file, directory, symbolic link, or alternate stream was not found.
pub const VAULT_ERR_FILE_NOT_FOUND: i32 = -8;

    // VAULT_ERR_PATH_NOT_FOUND: The specified path was not found.
pub const VAULT_ERR_PATH_NOT_FOUND: i32 = -9;

    // VAULT_ERR_SHARING_VIOLATION: The specified file or alternate stream is already open in an exclusive access mode.
pub const VAULT_ERR_SHARING_VIOLATION: i32 = -10;

    // VAULT_ERR_SEEK_BEYOND_EOF: Cannot seek beyond the end of a file or alternate stream.
pub const VAULT_ERR_SEEK_BEYOND_EOF: i32 = -11;

    // VAULT_ERR_NO_MORE_FILES: No other files, directories, symbolic links, or alternate streams match the search criteria.
pub const VAULT_ERR_NO_MORE_FILES: i32 = -12;

    // VAULT_ERR_INVALID_FILE_NAME: The specified name is not valid.
pub const VAULT_ERR_INVALID_FILE_NAME: i32 = -13;

    // VAULT_ERR_VAULT_ACTIVE: The requested operation cannot be performed while a vault is open.
pub const VAULT_ERR_VAULT_ACTIVE: i32 = -14;

    // VAULT_ERR_VAULT_NOT_ACTIVE: A vault must be open before the requested operation can be performed.
pub const VAULT_ERR_VAULT_NOT_ACTIVE: i32 = -15;

    // VAULT_ERR_INVALID_PASSWORD: The specified password is incorrect.
pub const VAULT_ERR_INVALID_PASSWORD: i32 = -16;

    // VAULT_ERR_VAULT_READ_ONLY: The requested operation cannot be performed; the vault is open in read-only mode.
pub const VAULT_ERR_VAULT_READ_ONLY: i32 = -17;

    // VAULT_ERR_NO_ENCRYPTION_HANDLERS: Cannot use custom encryption; no custom encryption event handlers provided.
pub const VAULT_ERR_NO_ENCRYPTION_HANDLERS: i32 = -18;

    // VAULT_ERR_OUT_OF_MEMORY: Out of memory.
pub const VAULT_ERR_OUT_OF_MEMORY: i32 = -19;

    // VAULT_ERR_SYMLINK_DESTINATION_NOT_FOUND: A symbolic link's destination file could not be found.
pub const VAULT_ERR_SYMLINK_DESTINATION_NOT_FOUND: i32 = -20;

    // VAULT_ERR_FILE_IS_NOT_SYMLINK: The specified file is not a symbolic link.
pub const VAULT_ERR_FILE_IS_NOT_SYMLINK: i32 = -21;

    // VAULT_ERR_BUFFER_TOO_SMALL: The specified buffer is too small to hold the requested value.
pub const VAULT_ERR_BUFFER_TOO_SMALL: i32 = -22;

    // VAULT_ERR_BAD_COMPRESSED_DATA: Decompression failed (possibly due to corruption).
pub const VAULT_ERR_BAD_COMPRESSED_DATA: i32 = -23;

    // VAULT_ERR_INVALID_PARAMETER: Invalid parameter.
pub const VAULT_ERR_INVALID_PARAMETER: i32 = -24;

    // VAULT_ERR_VAULT_FULL: The vault is full (and cannot be automatically resized).
pub const VAULT_ERR_VAULT_FULL: i32 = -25;

    // VAULT_ERR_INTERRUPTED_BY_USER: Operation interrupted by user.
pub const VAULT_ERR_INTERRUPTED_BY_USER: i32 = -26;

    // VAULT_ERR_TAG_NOT_FOUND: The specified file tag was not found.
pub const VAULT_ERR_TAG_NOT_FOUND: i32 = -27;

    // VAULT_ERR_DIRECTORY_NOT_EMPTY: The specified directory is not empty.
pub const VAULT_ERR_DIRECTORY_NOT_EMPTY: i32 = -28;

    // VAULT_ERR_HANDLE_CLOSED: The file or alternate stream was closed unexpectedly; the handle is no longer valid.
pub const VAULT_ERR_HANDLE_CLOSED: i32 = -29;

    // VAULT_ERR_INVALID_STREAM_HANDLE: Invalid file or alternate stream handle.
pub const VAULT_ERR_INVALID_STREAM_HANDLE: i32 = -30;

    // VAULT_ERR_FILE_ACCESS_DENIED: Access denied.
pub const VAULT_ERR_FILE_ACCESS_DENIED: i32 = -31;

    // VAULT_ERR_NO_COMPRESSION_HANDLERS: Cannot use custom compression; no custom compression event handlers provided.
pub const VAULT_ERR_NO_COMPRESSION_HANDLERS: i32 = -32;

    // VAULT_ERR_NOT_IMPLEMENTED: Not implemented in this version of CBFS Vault.
pub const VAULT_ERR_NOT_IMPLEMENTED: i32 = -33;

    // VAULT_ERR_DRIVER_NOT_INSTALLED: The CBFS Vault system driver has not been installed.
pub const VAULT_ERR_DRIVER_NOT_INSTALLED: i32 = -35;

    // VAULT_ERR_NEW_VAULT_VERSION: The specified vault cannot be opened, it was created using a newer version of CBFS Vault.
pub const VAULT_ERR_NEW_VAULT_VERSION: i32 = -37;

    // VAULT_ERR_FILE_IS_NOT_DIRECTORY: The specified file is not a directory.
pub const VAULT_ERR_FILE_IS_NOT_DIRECTORY: i32 = -38;

    // VAULT_ERR_INVALID_TAG_DATA_TYPE: The specified file tag data type is not valid.
pub const VAULT_ERR_INVALID_TAG_DATA_TYPE: i32 = -39;

    // VAULT_ERR_VAULT_FILE_DOES_NOT_EXIST: The specified vault storage file does not exist.
pub const VAULT_ERR_VAULT_FILE_DOES_NOT_EXIST: i32 = -40;

    // VAULT_ERR_VAULT_FILE_ALREADY_EXISTS: The specified vault storage file already exists.
pub const VAULT_ERR_VAULT_FILE_ALREADY_EXISTS: i32 = -41;

    // VAULT_ERR_CALLBACK_MODE_FAILURE: Some callback mode event handler has returned an unidentified error.
pub const VAULT_ERR_CALLBACK_MODE_FAILURE: i32 = -42;

    // VAULT_ERR_EXTERNAL_ERROR: External library could not be initialized or used.
pub const VAULT_ERR_EXTERNAL_ERROR: i32 = -43;

    // VAULT_FATTR_FILE: The entry is a file.
pub const VAULT_FATTR_FILE: i32 = 0x00000001;

    // VAULT_FATTR_DIRECTORY: The entry is a directory.
pub const VAULT_FATTR_DIRECTORY: i32 = 0x00000002;

    // VAULT_FATTR_DATA_STREAM: The entry is an alternate data stream.
pub const VAULT_FATTR_DATA_STREAM: i32 = 0x00000004;

    // VAULT_FATTR_COMPRESSED: The file or stream is compressed.
pub const VAULT_FATTR_COMPRESSED: i32 = 0x00000008;

    // VAULT_FATTR_ENCRYPTED: The file or stream is encrypted.
pub const VAULT_FATTR_ENCRYPTED: i32 = 0x00000010;

    // VAULT_FATTR_SYMLINK: The entry is a symbolic link.
pub const VAULT_FATTR_SYMLINK: i32 = 0x00000020;

    // VAULT_FATTR_READONLY: The file is read-only.
pub const VAULT_FATTR_READONLY: i32 = 0x00000040;

    // VAULT_FATTR_ARCHIVE: The file requires archiving.
pub const VAULT_FATTR_ARCHIVE: i32 = 0x00000080;

    // VAULT_FATTR_HIDDEN: The file is hidden.
pub const VAULT_FATTR_HIDDEN: i32 = 0x00000100;

    // VAULT_FATTR_SYSTEM: The file is a system file.
pub const VAULT_FATTR_SYSTEM: i32 = 0x00000200;

    // VAULT_FATTR_TEMPORARY: The file is temporary.
pub const VAULT_FATTR_TEMPORARY: i32 = 0x00000400;

    // VAULT_FATTR_DELETE_ON_CLOSE: The file should be deleted when the last handle to the file is closed.
pub const VAULT_FATTR_DELETE_ON_CLOSE: i32 = 0x00000800;

    // VAULT_FATTR_RESERVED_0: Reserved.
pub const VAULT_FATTR_RESERVED_0: i32 = 0x00001000;

    // VAULT_FATTR_RESERVED_1: Reserved.
pub const VAULT_FATTR_RESERVED_1: i32 = 0x00002000;

    // VAULT_FATTR_RESERVED_2: Reserved.
pub const VAULT_FATTR_RESERVED_2: i32 = 0x00004000;

    // VAULT_FATTR_RESERVED_3: Reserved.
pub const VAULT_FATTR_RESERVED_3: i32 = 0x00008000;

    // VAULT_FATTR_NO_USER_CHANGE: A mask that includes all attributes that cannot be changed.
pub const VAULT_FATTR_NO_USER_CHANGE: i32 = 0x0000F03F;

    // VAULT_FATTR_USER_DEFINED: A mask for application-defined attributes.
pub const VAULT_FATTR_USER_DEFINED: i32 = 0x7FF00000;

    // VAULT_FATTR_ANY_FILE: A mask that includes any and all attributes.
pub const VAULT_FATTR_ANY_FILE: i32 = 0x7FFFFFFF;

    // VAULT_CR_CHECK_ONLY: Check only, do not attempt any repairs.
pub const VAULT_CR_CHECK_ONLY: i32 = 0x00000001;

    // VAULT_CR_CHECK_ALL_PAGES: Check all vault pages, including empty ones.
pub const VAULT_CR_CHECK_ALL_PAGES: i32 = 0x00000002;

    // VAULT_FMF_FAST_FORMAT: Perform a fast format; only initialize the pages necessary for storing the filesystem structure.
pub const VAULT_FMF_FAST_FORMAT: i32 = 0x00000001;

    // VAULT_JM_NONE: No journaling is used.
pub const VAULT_JM_NONE: i32 = 0;

    // VAULT_JM_METADATA: Journaling is used only for metadata (filesystem structure and directory contents).
pub const VAULT_JM_METADATA: i32 = 1;

    // VAULT_JM_FULL: Journaling is used for both filesystem structure and file data and metadata.
pub const VAULT_JM_FULL: i32 = 2;

    // VAULT_FF_NEED_NAME: Include entry names (without paths) when returning search results.
pub const VAULT_FF_NEED_NAME: i32 = 0x00000001;

    // VAULT_FF_NEED_FULL_NAME: Include fully qualified entry names when returning search results.
pub const VAULT_FF_NEED_FULL_NAME: i32 = 0x00000002;

    // VAULT_FF_NEED_ATTRIBUTES: Include entry attributes when returning search results.
pub const VAULT_FF_NEED_ATTRIBUTES: i32 = 0x00000004;

    // VAULT_FF_NEED_SIZE: Include entry sizes when returning search results.
pub const VAULT_FF_NEED_SIZE: i32 = 0x00000008;

    // VAULT_FF_NEED_METADATA_SIZE: Include entry metadata sizes when returning search results.
pub const VAULT_FF_NEED_METADATA_SIZE: i32 = 0x00000010;

    // VAULT_FF_NEED_TIMES: Include entry times when returning search results.
pub const VAULT_FF_NEED_TIMES: i32 = 0x00000020;

    // VAULT_FF_NEED_LINK_DEST: Include symbolic link destinations when returning search results.
pub const VAULT_FF_NEED_LINK_DEST: i32 = 0x00000040;

    // VAULT_FF_DONT_NEED_ENCRYPTION: If attributes are requested, exclude encryption information when returning search results .
pub const VAULT_FF_DONT_NEED_ENCRYPTION: i32 = 0x00000200;

    // VAULT_FF_DONT_NEED_COMPRESSION: If attributes are requested, exclude compression information when returning search results.
pub const VAULT_FF_DONT_NEED_COMPRESSION: i32 = 0x00000400;

    // VAULT_FF_EMULATE_FAT: Inserts . and .. pseudo-entries into search results for all directories except the root one.
pub const VAULT_FF_EMULATE_FAT: i32 = 0x00001000;

    // VAULT_FF_RECURSIVE: Search recursively in all subdirectories.
pub const VAULT_FF_RECURSIVE: i32 = 0x00002000;

    // VAULT_FF_CASE_INSENSITIVE: Forces case-insensitive search, even if the vault is case-sensitive.
pub const VAULT_FF_CASE_INSENSITIVE: i32 = 0x00004000;

    // VAULT_OM_CREATE_NEW: Creates a new vault if possible, failing if one already exists.
pub const VAULT_OM_CREATE_NEW: i32 = 0;

    // VAULT_OM_CREATE_ALWAYS: Creates a new vault, overwriting an existing one if necessary.
pub const VAULT_OM_CREATE_ALWAYS: i32 = 1;

    // VAULT_OM_OPEN_EXISTING: Opens a vault if it exists; fails otherwise.
pub const VAULT_OM_OPEN_EXISTING: i32 = 2;

    // VAULT_OM_OPEN_ALWAYS: Opens a vault if it exists; creates a new one otherwise.
pub const VAULT_OM_OPEN_ALWAYS: i32 = 3;

    // VAULT_ST_FIXED_SIZE: The vault is a fixed size.
pub const VAULT_ST_FIXED_SIZE: i32 = 0x00000001;

    // VAULT_ST_READ_ONLY: The vault was opened in read-only mode.
pub const VAULT_ST_READ_ONLY: i32 = 0x00000002;

    // VAULT_ST_CORRUPTED: The vault is corrupted.
pub const VAULT_ST_CORRUPTED: i32 = 0x00000004;

    // VAULT_ST_TRANSACTIONS_USED: The vault was opened in journaling mode.
pub const VAULT_ST_TRANSACTIONS_USED: i32 = 0x00000008;

    // VAULT_ST_ACCESS_TIME_USED: Last access times are being tracked.
pub const VAULT_ST_ACCESS_TIME_USED: i32 = 0x00000010;

    // VAULT_ST_ENCRYPTED: The vault is encrypted with whole-vault encryption.
pub const VAULT_ST_ENCRYPTED: i32 = 0x00000020;

    // VAULT_ST_VALID_PASSWORD_SET: The correct whole-vault encryption password has been provided.
pub const VAULT_ST_VALID_PASSWORD_SET: i32 = 0x00000040;

    // VAULT_ST_PHYSICAL_VOLUME: The vault is backed by a storage volume or partition formatted with the CBFS Vault filesystem.
pub const VAULT_ST_PHYSICAL_VOLUME: i32 = 0x00000080;

    // VAULT_ST_PARTED: The vault's contents are split across multiple files on disk.
pub const VAULT_ST_PARTED: i32 = 0x00000100;

    // VAULT_TDT_RAWDATA: The tag is untyped and must be addressed by Id.
pub const VAULT_TDT_RAWDATA: i32 = 0x0;

    // VAULT_TDT_BOOLEAN: The tag contains Boolean data and must be addressed by name.
pub const VAULT_TDT_BOOLEAN: i32 = 0x1;

    // VAULT_TDT_STRING: The tag contains String (UTF-16LE) data and must be addressed by name.
pub const VAULT_TDT_STRING: i32 = 0x2;

    // VAULT_TDT_DATETIME: The tag contains DateTime data and must be addressed by name.
pub const VAULT_TDT_DATETIME: i32 = 0x3;

    // VAULT_TDT_NUMBER: The tag contains numeric (signed 64-bit) data and must be addressed by name.
pub const VAULT_TDT_NUMBER: i32 = 0x4;

    // VAULT_TDT_ANSISTRING: The tag contains AnsiString (8-bit string) data and must be addressed by name.
pub const VAULT_TDT_ANSISTRING: i32 = 0x5;

    // VAULT_PSC_BACKSLASH: Backslash ('\\').
pub const VAULT_PSC_BACKSLASH: i32 = 92;

    // VAULT_PSC_SLASH: Forward slash ('/').
pub const VAULT_PSC_SLASH: i32 = 47;

    // VAULT_CM_NONE: Do not use compression.
pub const VAULT_CM_NONE: i32 = 0;

    // VAULT_CM_DEFAULT: Use default compression (zlib).
pub const VAULT_CM_DEFAULT: i32 = 1;

    // VAULT_CM_CUSTOM: Use event-based custom compression.
pub const VAULT_CM_CUSTOM: i32 = 2;

    // VAULT_CM_ZLIB: Use zlib compression.
pub const VAULT_CM_ZLIB: i32 = 3;

    // VAULT_CM_RLE: Use RLE compression.
pub const VAULT_CM_RLE: i32 = 4;

    // VAULT_EM_NONE: Do not use encryption.
pub const VAULT_EM_NONE: i32 = 0x0;

    // VAULT_EM_DEFAULT: Use default encryption (VAULT_EM_XTS_AES256_PBKDF2_HMAC_SHA256).
pub const VAULT_EM_DEFAULT: i32 = 0x1;

    // VAULT_EM_XTS_AES256_PBKDF2_HMAC_SHA256: Use AES256 encryption with PBKDF2 key derivation based on a HMAC_SHA256 key hash.
pub const VAULT_EM_XTS_AES256_PBKDF2_HMAC_SHA256: i32 = 0x2;

    // VAULT_EM_CUSTOM256_PBKDF2_HMAC_SHA256: Use event-based custom 256-bit encryption with PBKDF2 key derivation based on a HMAC_SHA256 key hash.
pub const VAULT_EM_CUSTOM256_PBKDF2_HMAC_SHA256: i32 = 0x3;

    // VAULT_EM_CUSTOM512_PBKDF2_HMAC_SHA256: Use event-based custom 512-bit encryption with PBKDF2 key derivation based on a HMAC_SHA256 key hash.
pub const VAULT_EM_CUSTOM512_PBKDF2_HMAC_SHA256: i32 = 0x4;

    // VAULT_EM_CUSTOM1024_PBKDF2_HMAC_SHA256: Use event-based custom 1024-bit encryption with PBKDF2 key derivation based on a HMAC_SHA256 key hash.
pub const VAULT_EM_CUSTOM1024_PBKDF2_HMAC_SHA256: i32 = 0x5;

    // VAULT_EM_CUSTOM256_CUSTOM_KEY_DERIVE: Use event-based custom 256-bit encryption with custom key derivation.
pub const VAULT_EM_CUSTOM256_CUSTOM_KEY_DERIVE: i32 = 0x23;

    // VAULT_EM_CUSTOM512_CUSTOM_KEY_DERIVE: Use event-based custom 512-bit encryption with custom key derivation.
pub const VAULT_EM_CUSTOM512_CUSTOM_KEY_DERIVE: i32 = 0x24;

    // VAULT_EM_CUSTOM1024_CUSTOM_KEY_DERIVE: Use event-based custom 1024-bit encryption with custom key derivation.
pub const VAULT_EM_CUSTOM1024_CUSTOM_KEY_DERIVE: i32 = 0x25;

    // VAULT_EM_CUSTOM256_DIRECT_KEY: Use event-based custom 256-bit encryption with no key derivation.
pub const VAULT_EM_CUSTOM256_DIRECT_KEY: i32 = 0x43;

    // VAULT_EM_CUSTOM512_DIRECT_KEY: Use event-based custom 512-bit encryption with no key derivation.
pub const VAULT_EM_CUSTOM512_DIRECT_KEY: i32 = 0x44;

    // VAULT_EM_CUSTOM1024_DIRECT_KEY: Use event-based custom 1024-bit encryption with no key derivation.
pub const VAULT_EM_CUSTOM1024_DIRECT_KEY: i32 = 0x45;

    // VAULT_EM_UNKNOWN: Unidentified or unknown encryption.
pub const VAULT_EM_UNKNOWN: i32 = 0xFF;

    // VAULT_FOM_CREATE_NEW: Creates a new file or alternate stream if possible, failing if one already exists.
pub const VAULT_FOM_CREATE_NEW: i32 = 0;

    // VAULT_FOM_CREATE_ALWAYS: Creates a new file or stream, overwriting an existing one if necessary.
pub const VAULT_FOM_CREATE_ALWAYS: i32 = 1;

    // VAULT_FOM_OPEN_EXISTING: Opens a file or stream if it exists; fails otherwise.
pub const VAULT_FOM_OPEN_EXISTING: i32 = 2;

    // VAULT_FOM_OPEN_ALWAYS: Opens a file or stream if it exists; creates a new one otherwise.
pub const VAULT_FOM_OPEN_ALWAYS: i32 = 3;

    // VAULT_PO_FORMATTING: Formatting a vault.
pub const VAULT_PO_FORMATTING: i32 = 0;

    // VAULT_PO_CHECKING_1: Checking a vault (stage 1).
pub const VAULT_PO_CHECKING_1: i32 = 1;

    // VAULT_PO_CHECKING_2: Checking a vault (stage 2).
pub const VAULT_PO_CHECKING_2: i32 = 2;

    // VAULT_PO_CHECKING_3: Checking a vault (stage 3).
pub const VAULT_PO_CHECKING_3: i32 = 3;

    // VAULT_PO_CHECKING_4: Checking a vault (stage 4).
pub const VAULT_PO_CHECKING_4: i32 = 4;

    // VAULT_PO_CHECKING_5: Checking a vault (stage 5).
pub const VAULT_PO_CHECKING_5: i32 = 5;

    // VAULT_PO_PAGE_CORRUPTED: Processing a corrupted vault page.
pub const VAULT_PO_PAGE_CORRUPTED: i32 = 8;

    // VAULT_PO_PAGE_ORPHANED: Processing an orphaned vault page.
pub const VAULT_PO_PAGE_ORPHANED: i32 = 9;

    // VAULT_PO_COMPRESSING: Compressing a file or alternate stream.
pub const VAULT_PO_COMPRESSING: i32 = 10;

    // VAULT_PO_DECOMPRESSING: Decompressing a file or alternate stream.
pub const VAULT_PO_DECOMPRESSING: i32 = 11;

    // VAULT_PO_ENCRYPTING: Encrypting a vault, file, or alternate stream.
pub const VAULT_PO_ENCRYPTING: i32 = 12;

    // VAULT_PO_DECRYPTING: Decrypting a vault, file, or alternate stream
pub const VAULT_PO_DECRYPTING: i32 = 13;

    // VAULT_PO_COMPACTING: Compacting a vault.
pub const VAULT_PO_COMPACTING: i32 = 14;

    // VAULT_PO_RESIZING: Resizing a vault.
pub const VAULT_PO_RESIZING: i32 = 15;

    // VAULT_PO_CALCULATING_SIZE: Calculating a vault's size.
pub const VAULT_PO_CALCULATING_SIZE: i32 = 16;

    // VAULT_PO_COPYING_FILES_TO_VAULT: Copying files to a vault.
pub const VAULT_PO_COPYING_FILES_TO_VAULT: i32 = 17;

    // VAULT_PO_COPYING_FILES_FROM_VAULT: Copying files from a vault.
pub const VAULT_PO_COPYING_FILES_FROM_VAULT: i32 = 18;

    // VAULT_CFF_OVERWRITE_NONE: Never overwrite destination files.
pub const VAULT_CFF_OVERWRITE_NONE: i32 = 0x00000000;

    // VAULT_CFF_OVERWRITE_IF_NEWER: Overwrite a destination file only if the source file is newer.
pub const VAULT_CFF_OVERWRITE_IF_NEWER: i32 = 0x00000001;

    // VAULT_CFF_OVERWRITE_ALL: Always overwrite destination files.
pub const VAULT_CFF_OVERWRITE_ALL: i32 = 0x00000002;

    // VAULT_CFF_INCLUDE_SUBDIRS_WITH_CONTENTS: Include all subdirectories in source directory, and their contents, recursively.
pub const VAULT_CFF_INCLUDE_SUBDIRS_WITH_CONTENTS: i32 = 0x00010000;

    // VAULT_CFF_INCLUDE_SUBDIRS_NO_CONTENTS: Include all subdirectories in the source directory, without their contents.
pub const VAULT_CFF_INCLUDE_SUBDIRS_NO_CONTENTS: i32 = 0x00020000;

    // VAULT_CFF_COPY_DIRS_STRUCTURE: Include all subdirectories in the source directory, without their contents.
pub const VAULT_CFF_COPY_DIRS_STRUCTURE: i32 = 0x00040000;

    // VAULT_CFF_COPY_STRUCTURE: Include all subdirectories in source directory, and their contents, recursively, but without file content.
pub const VAULT_CFF_COPY_STRUCTURE: i32 = 0x00080000;

    // VAULT_CFF_FIRE_COPY_EVENTS: Fire events related to file copying.
pub const VAULT_CFF_FIRE_COPY_EVENTS: i32 = 0x40000000;

    // MODULE_DRIVER_PNP_BUS: PnP Bus Driver (.sys file).
pub const MODULE_DRIVER_PNP_BUS: i32 = 0x00000001;

    // MODULE_DRIVER_BLOCK: Virtual disk driver (.sys file).
pub const MODULE_DRIVER_BLOCK: i32 = 0x00000002;

    // MODULE_DRIVER_FS: Filesystem driver (.sys file).
pub const MODULE_DRIVER_FS: i32 = 0x00000004;

    // MODULE_HELPER_DLL: Shell Helper DLL (CBVaultShellHelper2024.dll)
pub const MODULE_HELPER_DLL: i32 = 0x00010000;

    // STG_DACCESS_READ: Grant/deny read access.
pub const STG_DACCESS_READ: i32 = 0x00000001;

    // STG_DACCESS_WRITE: Grant/deny write access.
pub const STG_DACCESS_WRITE: i32 = 0x00000002;

    // STG_DACCESS_READWRITE: Grant/deny read and write access.
pub const STG_DACCESS_READWRITE: i32 = 0x00000003;

    // INSTALL_REMOVE_OLD_VERSIONS: Uninstall drivers and helper DLLs from previous component versions (e.g., 2017).
pub const INSTALL_REMOVE_OLD_VERSIONS: i32 = 0x00000001;

    // INSTALL_KEEP_START_TYPE: Keep the driver's current start type setting in the registry.
pub const INSTALL_KEEP_START_TYPE: i32 = 0x00000002;

    // INSTALL_OVERWRITE_SAME_VERSION: Install files when their version is the same as the version of already installed files.
pub const INSTALL_OVERWRITE_SAME_VERSION: i32 = 0x00000004;

    // UNINSTALL_VERSION_PREVIOUS: Uninstall modules from previous product versions.
pub const UNINSTALL_VERSION_PREVIOUS: i32 = 0x00000001;

    // UNINSTALL_VERSION_CURRENT: Uninstall modules from the current product version.
pub const UNINSTALL_VERSION_CURRENT: i32 = 0x00000002;

    // UNINSTALL_VERSION_ALL: Uninstall modules from all product versions.
pub const UNINSTALL_VERSION_ALL: i32 = 0x00000003;

    // MODULE_STATUS_NOT_PRESENT: The specified module is not present on the system.
pub const MODULE_STATUS_NOT_PRESENT: i32 = 0x00000000;

    // MODULE_STATUS_STOPPED: The specified module is in the Stopped state.
pub const MODULE_STATUS_STOPPED: i32 = 0x00000001;

    // MODULE_STATUS_RUNNING: The specified module is loaded and running.
pub const MODULE_STATUS_RUNNING: i32 = 0x00000004;

    // STGMP_SIMPLE: Create a simple mounting point.
pub const STGMP_SIMPLE: i32 = 0x00010000;

    // STGMP_MOUNT_MANAGER: Create a mounting point that appears to the system as a physical device.
pub const STGMP_MOUNT_MANAGER: i32 = 0x00020000;

    // STGMP_NETWORK: Create a network mounting point.
pub const STGMP_NETWORK: i32 = 0x00040000;

    // STGMP_LOCAL: Specifies that a local mounting point should be created.
pub const STGMP_LOCAL: i32 = 0x10000000;

    // STGMP_NETWORK_ALLOW_MAP_AS_DRIVE: Indicates that users may assign a drive letter to the share (e.g., using the 'Map network drive...' context menu item in Windows File Explorer).
pub const STGMP_NETWORK_ALLOW_MAP_AS_DRIVE: i32 = 0x00000001;

    // STGMP_NETWORK_HIDDEN_SHARE: Indicates that the share should be skipped during enumeration.
pub const STGMP_NETWORK_HIDDEN_SHARE: i32 = 0x00000002;

    // STGMP_NETWORK_READ_ACCESS: Makes a read-only share available for the mounting point.
pub const STGMP_NETWORK_READ_ACCESS: i32 = 0x00000004;

    // STGMP_NETWORK_WRITE_ACCESS: Makes a read/write share available for the mounting point.
pub const STGMP_NETWORK_WRITE_ACCESS: i32 = 0x00000008;

    // STGMP_NETWORK_CLAIM_SERVER_NAME: Specifies that the server name is unique.
pub const STGMP_NETWORK_CLAIM_SERVER_NAME: i32 = 0x00000010;

    // STGMP_DRIVE_LETTER_NOTIFY_ASYNC: Causes the method to return immediately without waiting for mounting notifications to be sent to the system.
pub const STGMP_DRIVE_LETTER_NOTIFY_ASYNC: i32 = 0x20000000;

    // STGMP_AUTOCREATE_DRIVE_LETTER: Tells the component that it should assign the drive letter automatically.
pub const STGMP_AUTOCREATE_DRIVE_LETTER: i32 = 0x40000000;

    // STGMP_LOCAL_FUSE: Creates a mounting point, accessible only for current user.
pub const STGMP_LOCAL_FUSE: i32 = 0x10000000;

    // STGMP_SYMLINK_DEBUG: Prints debug messages to stderr
pub const STGMP_SYMLINK_DEBUG: i32 = 0x40000000;

    // STGMP_SYMLINK_SYSTEM_DEBUG: Prints debug messages generated by the FUSE library to stderr
pub const STGMP_SYMLINK_SYSTEM_DEBUG: i32 = 0x20000000;

    // STGT_DISK: Create a regular disk device.
pub const STGT_DISK: i32 = 0x00000000;

    // STGT_CDROM: Create a CD-ROM or DVD device.
pub const STGT_CDROM: i32 = 0x00000001;

    // STGT_DISK_PNP: Create a plug-and-play storage device.
pub const STGT_DISK_PNP: i32 = 0x00000003;

    // STGC_FLOPPY_DISKETTE: The storage is a floppy disk device.
pub const STGC_FLOPPY_DISKETTE: i32 = 0x00000001;

    // STGC_READONLY_DEVICE: The storage is a read-only device.
pub const STGC_READONLY_DEVICE: i32 = 0x00000002;

    // STGC_WRITE_ONCE_MEDIA: The storage device's media can only be written to once.
pub const STGC_WRITE_ONCE_MEDIA: i32 = 0x00000008;

    // STGC_REMOVABLE_MEDIA: The storage device's media is removable.
pub const STGC_REMOVABLE_MEDIA: i32 = 0x00000010;

    // STGC_AUTOCREATE_DRIVE_LETTER: The system should automatically create a drive letter for the storage device.
pub const STGC_AUTOCREATE_DRIVE_LETTER: i32 = 0x00002000;

    // STGC_SHOW_IN_EJECTION_TRAY: The storage device should be shown in the 'Safely Remove Hardware and Eject Media' menu in the system notification area (system tray).
pub const STGC_SHOW_IN_EJECTION_TRAY: i32 = 0x00004000;

    // STGC_ALLOW_EJECTION: The storage device can be ejected.
pub const STGC_ALLOW_EJECTION: i32 = 0x00008000;

    // STGC_RESERVED_1: Reserved, do not use.
pub const STGC_RESERVED_1: i32 = 0x00010000;

    // STGC_RESERVED_2: Reserved, do not use.
pub const STGC_RESERVED_2: i32 = 0x00020000;

    // FILE_SYS_SHARE_READ: Enables subsequent open operations on a file to request read access.
pub const FILE_SYS_SHARE_READ: i32 = 0x00000001;

    // FILE_SYS_SHARE_WRITE: Enables subsequent open operations on a file to request write access.
pub const FILE_SYS_SHARE_WRITE: i32 = 0x00000002;

    // FILE_SYS_SHARE_DELETE: Enables subsequent open operations on a file to request delete access.
pub const FILE_SYS_SHARE_DELETE: i32 = 0x00000004;


///////////////////
// Helper Functions
///////////////////

fn charptr_to_string(raw_ptr: *const c_char) -> Result<String, std::str::Utf8Error> 
{
    unsafe 
    {
        if raw_ptr == std::ptr::null() 
        {
            return Result::Ok(String::default());
        }
        // Create a CStr from the raw pointer
        let c_str = CStr::from_ptr(raw_ptr);

        // Convert the CStr to a &str, checking for valid UTF-8
        let str_slice = c_str.to_str()?;

        // Convert the &str to a String
        Ok(str_slice.to_owned())
    }
}

const HNANO_PER_SEC : i64 = 10_000_000;
const SECONDS_FROM_1601_TO_1970: i64 = 11644473600;

//const TICKS_PER_MSEC : i64 = 10_000;
const TICKS_PER_MCSEC : i64 = 10;
//const MSECONDS_OFFSET : i64 = 11644473600000;
const MCSECONDS_OFFSET : i64 = 11644473600000000;

// file_time_to_chrono_time converts Windows FileTime to Chrono::DateTime
pub fn file_time_to_chrono_time(time : i64) -> DateTime<Utc>
{
    let total_sec: i64;
    let sec : i64;
    let nsec : u32;
    if time >= 0
    {
        total_sec = time / HNANO_PER_SEC;
        nsec = ((time % HNANO_PER_SEC) * 100) as u32;
        sec = total_sec - SECONDS_FROM_1601_TO_1970;
    }
    else
    {
        total_sec = (time / HNANO_PER_SEC) - 1;
        nsec = (((HNANO_PER_SEC - (-time % HNANO_PER_SEC)) * 100)) as u32;
        sec = total_sec - SECONDS_FROM_1601_TO_1970;
    }
    DateTime::from_timestamp(sec, nsec).unwrap()
}

// chrono_time_to_file_time converts Chrono::DateTime to Windows FileTime
#[inline]
pub fn chrono_time_to_file_time(time : &chrono::DateTime<Utc>) -> i64
{
    ((time.timestamp_micros() + MCSECONDS_OFFSET) * TICKS_PER_MCSEC) as i64
}

// ######################################## Base exception class ########################################

#[derive(Debug)]
pub struct CBFSVaultError 
{
    m_code : i32,
    m_message : String,
}

impl std::error::Error for CBFSVaultError 
{

}

impl Display for CBFSVaultError 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "CBFSVaultError {} ('{}')", &self.m_code, &self.m_message)
    }
}

/*
impl std::fmt::Debug for CBFSVaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "CBFSVaultError {} ('{}')", &self.m_code, &self.m_message)
        std::fmt::Debug::fmt(&self, f)
    }
}
*/

impl From<i32> for CBFSVaultError 
{
    #[inline]
    fn from(code: i32) -> CBFSVaultError 
    {
        CBFSVaultError { m_code: code, m_message : String::from("") }
    }
}

impl From<String> for CBFSVaultError
{
    #[inline]
    fn from(message : String) -> CBFSVaultError  
    {
        CBFSVaultError { m_code: -1, m_message : String::from(message) }
    }
}

impl CBFSVaultError
{
    pub fn new(code: i32, message: &str) -> CBFSVaultError
    {
        CBFSVaultError { m_code: code, m_message : String::from(message) }
    }

    pub fn from_code(code: i32) -> CBFSVaultError
    {
        CBFSVaultError { m_code: code, m_message : String::from("") }
    }

    #[inline]
    pub fn get_code(&self) -> i32 {
        return self.m_code;
    }

    #[inline]
    pub fn get_message(&self) -> String {
        return self.m_message.to_string();
    }
}

// ######################################### Stream Classes #########################################

pub(crate) const STREAM_OP_READ : i32 = 500;
pub(crate) const STREAM_OP_WRITE : i32 = 501;
pub(crate) const STREAM_OP_SEEK : i32 = 502;
pub(crate) const STREAM_OP_FLUSH : i32 = 503;
pub(crate) const STREAM_OP_CLOSE : i32 = 504;
// pub(crate) const STREAM_OP_CHECK : i32 = 505;
pub(crate) const STREAM_OP_GET_LENGTH : i32 = 506;
pub(crate) const STREAM_OP_SET_LENGTH : i32 = 507;
// pub(crate) const STREAM_OP_LAST_ERROR : i32 = 508;
// pub(crate) const STREAM_OP_LAST_ERROR_CODE : i32 = 509;

pub(crate) const STREAM_SEEK_FROM_BEGIN : i32 = 0;
pub(crate) const STREAM_SEEK_FROM_CURRENT : i32 = 1;
pub(crate) const STREAM_SEEK_FROM_END : i32 = 2;

#[cfg(target_os = "windows")]
pub(crate) const ERROR_HANDLE_EOF : i32 = 38;

/*
pub(crate) const STREAM_CAN_READ : i32 = 0;
pub(crate) const STREAM_CAN_WRITE : i32 = 1;
pub(crate) const STREAM_CAN_SEEK : i32 = 2;

pub(crate) const STREAM_OP_READ : i32 = 500;
pub(crate) const STREAM_OP_WRITE : i32 = 501;
pub(crate) const STREAM_OP_SEEK : i32 = 502;
pub(crate) const STREAM_OP_FLUSH : i32 = 503;
pub(crate) const STREAM_OP_CLOSE : i32 = 504;
pub(crate) const STREAM_OP_CHECK : i32 = 505;
pub(crate) const STREAM_OP_GET_LENGTH : i32 = 506;
pub(crate) const STREAM_OP_SET_LENGTH : i32 = 507;
pub(crate) const STREAM_OP_LAST_ERROR : i32 = 508;
pub(crate) const STREAM_OP_LAST_ERROR_CODE : i32 = 509;

pub(crate) const STREAM_SEEK_FROM_BEGIN : i32 = 0;
pub(crate) const STREAM_SEEK_FROM_CURRENT : i32 = 1;
pub(crate) const STREAM_SEEK_FROM_END : i32 = 2;

pub(crate) const STREAM_CAN_READ : i32 = 0;
pub(crate) const STREAM_CAN_WRITE : i32 = 1;
pub(crate) const STREAM_CAN_SEEK : i32 = 2;
*/

pub struct CBFSVaultStream 
{
    m_file : usize,
    m_closed : bool
}

impl CBFSVaultStream 
{

    pub fn new(file : usize) -> CBFSVaultStream
    {
        let result = CBFSVaultStream { m_file : file, m_closed : false };
        return result;
    }

    fn panic_if_closed(&self)
    {
        if self.m_closed || self.m_file == 0 
        {
            panic!("Stream already closed")
        }
    }

    pub fn close(&mut self)
    {
        if self.m_closed || self.m_file == 0 
        {
            return;
        }
        unsafe
        {
            let func = CBFSVault_Stream.get().unwrap();
            func(self.m_file, STREAM_OP_CLOSE, std::ptr::null(), std::ptr::null());
        }
        self.m_closed = true;
        self.m_file = 0;
    }

    fn check_result_usize(&self, ret_val : usize, rescode : i32) -> std::io::Result<usize>
    {
        if rescode == 0 
        {
            return Ok(ret_val);
        } 
        else 
        {
              let error = CBFSVaultError::new(rescode,  "");
              return Err(std::io::Error::new(std::io::ErrorKind::Other, error));
        }
    }

    fn check_result_u64(&self, ret_val : u64, rescode : i32) -> std::io::Result<u64> 
    {
        if rescode == 0 
        {
            return Ok(ret_val);
        } 
        else
        {
              let error = CBFSVaultError::new(rescode,  "");
              return Err(std::io::Error::new(std::io::ErrorKind::Other, error));
        }
    }

    fn check_result(&self, rescode : i32) -> std::io::Result<()>
    {
        if rescode == 0
        {
            return Ok(());
        } 
        else 
        {
              let error = CBFSVaultError::new(rescode, "");
              return Err(std::io::Error::new(std::io::ErrorKind::Other, error));
        }
    }

    pub fn set_len(&self, size : u64) -> std::io::Result<()> 
    {
        self.panic_if_closed();

        let mut params : [usize; 1] = [0];
        let tp : *const u64 = &size;
        params[0] = tp as usize;

        unsafe 
        {
            let func = CBFSVault_StreamSetLength.get().unwrap();
            func(self.m_file, STREAM_OP_SET_LENGTH, (&params).as_ptr(), null_mut());            
        }
        return std::io::Result::Ok(());
    }

    pub fn get_len(&self) -> std::io::Result<u64>
    {
        self.panic_if_closed();

        let mut ret_val : u64 = 0;

        unsafe
        {
            let func = CBFSVault_StreamGetLength.get().unwrap();
            func(self.m_file, STREAM_OP_GET_LENGTH, std::ptr::null(), (&mut ret_val) as *mut u64);
        }
        return std::io::Result::Ok(ret_val);
    }
    
    pub fn get_position(&self) -> std::io::Result<u64>
    {
        self.panic_if_closed();

        let mut params : [usize; 2] = [0; 2];
        let mut ret_val : u64 = 0;
        let rescode : i32;
        let orig : i32 = STREAM_SEEK_FROM_CURRENT;
        let position : i64 = 0;

        unsafe
        {
            let tp : *const i64 = &position;
            params[0] = tp as usize;
            params[1] = (orig as isize) as usize;

            let func = CBFSVault_StreamSeek.get().unwrap();
            rescode = func(self.m_file, STREAM_OP_SEEK, (&params).as_ptr(), (&mut ret_val) as *mut u64);
        }
        return self.check_result_u64(ret_val, rescode);
    }

    fn get_pos(&self)  -> u64
    {
        let pos_res = self.get_position();
        if pos_res.is_err()
        {
            return 0;
        }
        else
        {
            return pos_res.unwrap();
        }
    }
}

impl Drop for CBFSVaultStream 
{
    fn drop(&mut self) 
    {
        self.close();
    }
}

impl std::fmt::Debug for CBFSVaultStream 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
        .debug_struct("CBFSVaultStream")
        .field("handle", &self.m_file)
        .field("closed", &self.m_closed)
        .field("position", &self.get_pos())
        .field("length", &self.get_len())
        .finish()
    }
}

impl std::io::Read for CBFSVaultStream 
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>
    {
        self.panic_if_closed();

        let to_read : usize = buf.len();
        if to_read == 0
        {
            return Ok(0);
        }
        let mut params : [usize; 2] = [0; 2];
        let mut ret_val : i32 = 0;
        let rescode : i32;
        unsafe 
        {
            params[0] = buf.as_ptr() as usize;
            params[1] = to_read as usize;

            let func = CBFSVault_StreamRW.get().unwrap();
            
            rescode = func(self.m_file, STREAM_OP_READ, (&params).as_ptr(), (&mut ret_val) as *mut i32);
            #[cfg(target_os = "windows")]
            if rescode == ERROR_HANDLE_EOF
            {
                #[cfg(target_os = "windows")]
                return Ok(0);
            }
        
        }

        return self.check_result_usize(ret_val as usize, rescode);
    }
}

impl std::io::Write for CBFSVaultStream 
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> 
    {
        self.panic_if_closed();

        let to_write : usize = buf.len();
        if to_write == 0
        {
            return Ok(0);
        }
        let mut params : [usize; 2] = [0; 2];
        let mut ret_val : i32 = 0;
        let rescode : i32;
        unsafe 
        {
            params[0] = buf.as_ptr() as usize;
            params[1] = to_write as usize;

            let func = CBFSVault_StreamRW.get().unwrap();
            rescode = func(self.m_file, STREAM_OP_WRITE, (&params).as_ptr(), (&mut ret_val) as *mut i32);
        }
        return self.check_result_usize(ret_val as usize, rescode);
    }

    fn flush(&mut self) -> std::io::Result<()> 
    {
        self.panic_if_closed();
        let rescode : i32;
        unsafe 
        {
            let func = CBFSVault_Stream.get().unwrap();
            rescode = func(self.m_file, STREAM_OP_FLUSH, std::ptr::null(), std::ptr::null());
        }
        return self.check_result(rescode);
    }
}

impl std::io::Seek for CBFSVaultStream
{

    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> 
    {
        self.panic_if_closed();

        let mut params : [usize; 2] = [0; 2];
        let mut ret_val : u64 = 0;
        let rescode : i32;
        let orig : i32;
        let position : i64;

        match pos 
        {
            SeekFrom::Start(n) => 
            {
                position = n as i64;
                orig = STREAM_SEEK_FROM_BEGIN;
            }
            SeekFrom::Current(n) =>
            {
                position = n;
                orig = STREAM_SEEK_FROM_CURRENT;
            }
            SeekFrom::End(n) => 
            {
                position = n;
                orig = STREAM_SEEK_FROM_END;
            }
        }

        unsafe 
        {
            let tp : *const i64 = &position;
            params[0] = tp as usize;
            params[1] = (orig as isize) as usize;

            let func = CBFSVault_StreamSeek.get().unwrap();
            rescode = func(self.m_file, STREAM_OP_SEEK, (&params).as_ptr(), (&mut ret_val) as *mut u64);
        }

        return self.check_result_u64(ret_val, rescode);
    }
    
    fn rewind(&mut self) -> std::io::Result<()> 
    {
        let res : std::io::Result<u64> = self.seek(SeekFrom::Start(0));
        if res.is_err() 
        {
          return std::io::Result::Err(res.unwrap_err());
        }
        else 
        {
            return std::io::Result::Ok(());
        }
    }
    
    /* Uncomment when the API stabilizes. Until then, call get_len() directly.
    fn stream_len(&mut self) -> std::io::Result<u64> {
        return self.get_len();
    }
    */
    
    fn stream_position(&mut self) -> std::io::Result<u64> 
    {
        return self.seek(SeekFrom::Current(0));
    }
    
}



pub(crate) fn get_lib_funcs(lib_hand : &'static Library) -> bool
{
    // CBFSVault_EvtStr
    let func_ptr_res: Result<Symbol<'static, CBFSVaultEvtStrType>, _> = unsafe { lib_hand.get(b"CBFSVault_EvtStr") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_EvtStr.set(func);
    } else {
        return false;
    }
    // CBFSVault_EvtStrSet
    let func_ptr_res: Result<Symbol<'static, CBFSVaultEvtStrSetType>, _> = unsafe { lib_hand.get(b"CBFSVault_EvtStr") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_EvtStrSet.set(func);
    } else {
        return false;
    }

    // CBFSVault_Stream
    let func_ptr_res: Result<Symbol<'static, CBFSVaultStreamType>, _> = unsafe { lib_hand.get(b"CBFSVault_Stream") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_Stream.set(func);
    } else {
        return false;
    }
    // CBFSVault_StreamRW
    let func_ptr_res: Result<Symbol<'static, CBFSVaultStreamRWType>, _> = unsafe { lib_hand.get(b"CBFSVault_Stream") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_StreamRW.set(func);
    } else {
        return false;
    }
    // CBFSVault_StreamSeek
    let func_ptr_res: Result<Symbol<'static, CBFSVaultStreamSeekType>, _> = unsafe { lib_hand.get(b"CBFSVault_Stream") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_StreamSeek.set(func);
    } else {
        return false;
    }
    // CBFSVault_StreamGetLength
    let func_ptr_res: Result<Symbol<'static, CBFSVaultStreamGetLengthType>, _> = unsafe { lib_hand.get(b"CBFSVault_Stream") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_StreamGetLength.set(func);
    } else {
        return false;
    }
    // CBFSVault_StreamSetLength
    let func_ptr_res: Result<Symbol<'static, CBFSVaultStreamSetLengthType>, _> = unsafe { lib_hand.get(b"CBFSVault_Stream") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_StreamSetLength.set(func);
    } else {
        return false;
    }

    return true;
}


#[cfg(target_os = "windows")]
pub fn get_lib_path() -> OsString
{

    #[cfg(target_arch = "x86_64")]
    return OsString::from("..\\lib64\\windows\\");
    #[cfg(target_arch = "x86")]
    return OsString::from("..\\lib\\windows\\");
    #[cfg(target_arch = "aarch64")]
    return OsString::from("..\\libarm64\\windows\\");
}

#[cfg(target_os = "linux")]
pub fn get_lib_path() -> OsString
{
    #[cfg(target_arch = "aarch64")]
    return OsString::from("../libarm64/linux/");
    #[cfg(target_arch = "x86_64")]
    return OsString::from("../lib64/linux/");
    #[cfg(target_arch = "x86")]
    return OsString::from("../lib/linux/");
}

#[cfg(target_os = "macos")]
pub fn get_lib_path() -> OsString
{
    #[cfg(target_arch = "aarch64")]
    return OsString::from("../lib64/mac/");
    #[cfg(target_arch = "x86_64")]
    return OsString::from("../lib64/mac/");
    #[cfg(target_arch = "x86")]
    return OsString::from("../lib/mac/");
}

#[cfg(target_os = "windows")]
fn get_lib_name() -> &'static OsStr
{
    return OsStr::new("rustcbfsvault24.dll");
}

#[cfg(target_os = "linux")]
fn get_lib_name() -> &'static OsStr
{
    return OsStr::new("librustcbfsvault.so.24.0");
}

#[cfg(target_os = "macos")]
fn get_lib_name() -> &'static OsStr
{
    return OsStr::new("librustcbfsvault.24.0.dylib");
}

#[ctor]
fn init()
{
}

pub(crate) fn init_on_demand()
{
    // Already initialized?
    if LIB_LOADED.get().is_some() {
        return;
    }

    let lib_name : Option<&OsStr> = Some(get_lib_name());

    // try loading by just name        
    let mut load_res = unsafe { libloading::Library::new(lib_name.unwrap()) };
    if !(load_res.is_ok())
    {
        // current directory

        let mut tmp_path_buf : PathBuf = PathBuf::from(".");
        tmp_path_buf.push(lib_name.unwrap());

        load_res = unsafe { libloading::Library::new(tmp_path_buf.into_os_string()) };
    }
    
    if !(load_res.is_ok())
    {
        // relative path to library directory
        let mut tmp_path_buf : PathBuf = PathBuf::from(get_lib_path());
        tmp_path_buf.push(lib_name.unwrap());
        load_res = unsafe { libloading::Library::new(tmp_path_buf.into_os_string()) };
    }
    
    if !(load_res.is_ok())
    {
        // path of the executable
        let exe_full_path = env::current_exe();
        match exe_full_path 
        {
            Ok(mut exe_path_val) => 
            {
                if exe_path_val.pop()
                {
                    let mut tmp_path_buf : PathBuf = exe_path_val;
                    tmp_path_buf.push(lib_name.unwrap());
                    load_res = unsafe { libloading::Library::new(tmp_path_buf.into_os_string()) };
                }
            }
        
            Err(_e) => 
            {                     
                // do nothing
            }
        }
    }

    if !load_res.is_ok() 
    {
        panic!("Failed to load the required library '{}'.", lib_name.unwrap().to_str().unwrap());
    }

    // Store library in OnceLock
    if let Ok(lib) = load_res {
        let _ = LIB_HANDLE.set(lib);
    }
      
    if let Some(lib_ref) = LIB_HANDLE.get()
    {
        // Get a 'static reference by converting through raw pointer
        // This is safe because OnceLock guarantees the library stays alive for 'static
        let static_lib_ref: &'static Library = unsafe { &*(lib_ref as *const Library) };
        
        if (!get_lib_funcs(static_lib_ref)) 
        || (!cbvault::get_lib_funcs(static_lib_ref))
        || (!cbvaultdrive::get_lib_funcs(static_lib_ref))
        {
            panic!("Failed to load required functions from '{}'.", lib_name.unwrap().to_str().unwrap());
        }
        let _ = LIB_LOADED.set(true);
    }
}


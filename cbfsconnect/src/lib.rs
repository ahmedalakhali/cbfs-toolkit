
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(static_mut_refs)]

#![crate_type = "lib"]
#![crate_name = "cbfsconnect"]

pub mod cbfsconnectkey;
pub mod cbcache;
pub mod cbfs;
pub mod fuse;
pub mod nfs;

extern crate libloading as lib;

use ctor::ctor;

use std::{path::PathBuf, env, ffi::{c_void, c_char, c_long, CStr, OsStr,OsString}, fmt::Display, ptr::null_mut, io::SeekFrom};
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

// typedef LPVOID (CBFSCONNECT_CALL CBFSCONNECT_EvtStr)(LPVOID lpEvtStr, INT id, LPVOID val, INT opt);
type CBFSConnectEvtStrType = unsafe extern "system" fn(pEvtStr : usize, id : i32, value : *mut c_void, opt : i32) -> *mut c_void;

// typedef LPVOID (CBFSCONNECT_CALL CBFSCONNECT_EvtStr)(LPVOID lpEvtStr, INT id, LPVOID val, INT opt);
type CBFSConnectEvtStrSetType = unsafe extern "system" fn(pEvtStr : *mut c_char, id : i32, value : *const c_char, opt : i32) -> *mut c_void;

// typedef INT (CBFSCONNECT_CALL CBFSCONNECT_Stream)(LPVOID lpStream, INT op, LPVOID param[], LPVOID ret_val);
type CBFSConnectStreamType = unsafe extern "system" fn(pStream : usize, op : i32, params: ConstUIntPtrArrayType, ret_val : *const c_void) -> i32; // for Flush and Close

type CBFSConnectStreamRWType = unsafe extern "system" fn(pStream : usize, op : i32, params: ConstUIntPtrArrayType, ret_val : *mut i32) -> i32; // for Read and Write

type CBFSConnectStreamSeekType = unsafe extern "system" fn(pStream : usize, op : i32, params: ConstUIntPtrArrayType, ret_val : *mut u64) -> i32;

type CBFSConnectStreamGetLengthType = unsafe extern "system" fn(pStream : usize, op : i32, params: *const c_void, ret_val : *mut u64) -> i32;

type CBFSConnectStreamSetLengthType = unsafe extern "system" fn(pStream : usize, op : i32, params: ConstUIntPtrArrayType, ret_val : *const c_void) -> i32;

// typedef INT (CBFSCONNECT_CALL *LPFNCBFSCONNECTEVENT) (LPVOID lpObj, INT event_id, INT cparam, LPVOID param[], INT cbparam[]);
type CBFSConnectSinkDelegateType = unsafe extern "system" fn(pObj : usize, event_id : c_long, cparam : c_long, param : IntPtrArrayType, cbparam : IntArrayType) -> c_long;

static mut CBFSConnect_EvtStr : Option<Symbol<CBFSConnectEvtStrType>> = None;
static mut CBFSConnect_EvtStrSet : Option<Symbol<CBFSConnectEvtStrSetType>> = None;

static mut CBFSConnect_Stream : Option<Symbol<CBFSConnectStreamType>> = None;
static mut CBFSConnect_StreamRW : Option<Symbol<CBFSConnectStreamRWType>> = None;
static mut CBFSConnect_StreamSeek : Option<Symbol<CBFSConnectStreamSeekType>> = None;
static mut CBFSConnect_StreamGetLength : Option<Symbol<CBFSConnectStreamGetLengthType>> = None;
static mut CBFSConnect_StreamSetLength : Option<Symbol<CBFSConnectStreamSetLengthType>> = None;

static mut lib_handle : Option<Library> = None;

static mut lib_loaded : bool = false;

////////////////////
// Product Constants
////////////////////

    // CBFS_ROUTE_FILE_READ_ONLY: Causes files to be treated as read-only; all write operations will be automatically denied.
pub const CBFS_ROUTE_FILE_READ_ONLY: i32 = 0x00000001;

    // CBFS_ROUTE_OPEN_EVENT: Prevents 'open' requests from being routed automatically.
pub const CBFS_ROUTE_OPEN_EVENT: i32 = 0x00000020;

    // CBFS_ROUTE_CLEANUP_EVENT: Prevents 'cleanup' requests from being routed automatically.
pub const CBFS_ROUTE_CLEANUP_EVENT: i32 = 0x00000040;

    // CBFS_ROUTE_CLOSE_EVENT: Prevents 'close' requests from being routed automatically.
pub const CBFS_ROUTE_CLOSE_EVENT: i32 = 0x00000080;

    // CBFS_ROUTE_ENUMERATE_DIRECTORY_EVENT: Prevents 'enumerate directory' requests from being routed automatically.
pub const CBFS_ROUTE_ENUMERATE_DIRECTORY_EVENT: i32 = 0x00000200;

    // CBFS_ROUTE_SET_SECURITY_EVENT: Prevents 'set security' requests from being routed automatically.
pub const CBFS_ROUTE_SET_SECURITY_EVENT: i32 = 0x00000400;

    // CBFS_ROUTE_GET_SECURITY_EVENT: Prevents 'get security' requests from being routed automatically.
pub const CBFS_ROUTE_GET_SECURITY_EVENT: i32 = 0x00000800;

    // CBFS_ROUTE_SET_FILE_ATTRIBUTES_EVENT: Prevents 'set file attributes' requests from being routed automatically.
pub const CBFS_ROUTE_SET_FILE_ATTRIBUTES_EVENT: i32 = 0x00002000;

    // CBFS_ROUTE_SET_FILE_SIZES_EVENT: Prevents 'set file size' requests from being routed automatically.
pub const CBFS_ROUTE_SET_FILE_SIZES_EVENT: i32 = 0x00004000;

    // CBFS_ROUTE_SET_VALID_DATA_LENGTH_EVENT: Prevents 'set valid data length' requests from being routed automatically.
pub const CBFS_ROUTE_SET_VALID_DATA_LENGTH_EVENT: i32 = 0x00008000;

    // CBFS_ROUTE_CREATE_HARD_LINK_EVENT: Prevents 'create hard link' requests from being routed automatically.
pub const CBFS_ROUTE_CREATE_HARD_LINK_EVENT: i32 = 0x00020000;

    // CBFS_ROUTE_QUERY_QUOTA_EVENT: Prevents 'query quota' requests from being routed automatically.
pub const CBFS_ROUTE_QUERY_QUOTA_EVENT: i32 = 0x00040000;

    // CBFS_ROUTE_SET_QUOTA_EVENT: Prevents 'set quota' requests from being routed automatically.
pub const CBFS_ROUTE_SET_QUOTA_EVENT: i32 = 0x00080000;

    // CBFS_ROUTE_CAN_FILE_BE_DELETED_EVENT: Prevents 'can file be deleted' requests from being routed automatically.
pub const CBFS_ROUTE_CAN_FILE_BE_DELETED_EVENT: i32 = 0x00200000;

    // CBFS_ROUTE_IS_DIRECTORY_EMPTY_EVENT: Prevents 'is directory empty' requests from being routed automatically.
pub const CBFS_ROUTE_IS_DIRECTORY_EMPTY_EVENT: i32 = 0x00400000;

    // CBFS_ROUTE_RENAME_EVENT: Prevents 'rename/move' requests from being routed automatically.
pub const CBFS_ROUTE_RENAME_EVENT: i32 = 0x00800000;

    // CBFS_ROUTE_SET_EA_EVENT: Prevents 'set ea' requests from being routed automatically.
pub const CBFS_ROUTE_SET_EA_EVENT: i32 = 0x01000000;

    // CBFS_ROUTE_QUERY_EA_EVENT: Prevents 'query ea' requests from being routed automatically.
pub const CBFS_ROUTE_QUERY_EA_EVENT: i32 = 0x02000000;

    // CBFS_NOTIFY_FLAG_ADDED: The specified file or directory has been created.
pub const CBFS_NOTIFY_FLAG_ADDED: i32 = 0x00000001;

    // CBFS_NOTIFY_FLAG_REMOVED: The specified file or directory has been removed.
pub const CBFS_NOTIFY_FLAG_REMOVED: i32 = 0x00000002;

    // CBFS_NOTIFY_FLAG_MODIFIED: The specified file or directory has been modified.
pub const CBFS_NOTIFY_FLAG_MODIFIED: i32 = 0x00000003;

    // CBFS_NOTIFY_FLAG_METADATA_MODIFIED: The timestamp or other attributes of the specified file or directory have been modified.
pub const CBFS_NOTIFY_FLAG_METADATA_MODIFIED: i32 = 0x00000004;

    // CBFS_NOTIFY_FLAG_ALLOCATION_SIZE_MODIFIED: The allocation size for the specified file or directory has been modified.
pub const CBFS_NOTIFY_FLAG_ALLOCATION_SIZE_MODIFIED: i32 = 0x00000005;

    // CBFS_NOTIFY_FLAG_MODIFIED_NOT_INVALIDATE: The specified file or directory has been modified, but handles should not be invalidated.
pub const CBFS_NOTIFY_FLAG_MODIFIED_NOT_INVALIDATE: i32 = 0x00000006;

    // CBFS_NOTIFY_FLAG_RENAMED: The specified file or directory has been renamed.
pub const CBFS_NOTIFY_FLAG_RENAMED: i32 = 0x00000007;

    // CBFS_NOTIFY_FLAG_SKIP_LOCKED_FILE: Ignore the request if the file has locked ranges.
pub const CBFS_NOTIFY_FLAG_SKIP_LOCKED_FILE: i32 = 0x00001000;

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

    // MODULE_PNP_BUS: PnP Bus Driver (.sys file).
pub const MODULE_PNP_BUS: i32 = 0x00000001;

    // MODULE_DRIVER: Core Product Driver (.sys file).
pub const MODULE_DRIVER: i32 = 0x00000002;

    // MODULE_HELPER_DLL: Shell Helper DLL (CBFSShellHelper24.dll)
pub const MODULE_HELPER_DLL: i32 = 0x00010000;

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

    // FILEINFO_EA_SIZE: Ea size is requested.
pub const FILEINFO_EA_SIZE: i32 = 0x0400;

    // FILEINFO_REAL_NAME: File's actual name is requested.
pub const FILEINFO_REAL_NAME: i32 = 0x0040;

    // FILEINFO_SHORT_NAME: Short name is requested.
pub const FILEINFO_SHORT_NAME: i32 = 0x0200;

    // FILEINFO_TIME: File times are requested.
pub const FILEINFO_TIME: i32 = 0x0004;

    // FILEINFO_SIZE: File size is requested.
pub const FILEINFO_SIZE: i32 = 0x0008;

    // FILEINFO_ATTR: File attributes are requested.
pub const FILEINFO_ATTR: i32 = 0x0010;

    // FILEINFO_FILEID: File Id is requested.
pub const FILEINFO_FILEID: i32 = 0x0020;

    // FILEINFO_REPARSE_TAG: File Reparse Tag is requested.
pub const FILEINFO_REPARSE_TAG: i32 = 0x0080;

    // STG_DACCESS_READ: Grant/deny read access.
pub const STG_DACCESS_READ: i32 = 0x00000001;

    // STG_DACCESS_WRITE: Grant/deny write access.
pub const STG_DACCESS_WRITE: i32 = 0x00000002;

    // STG_DACCESS_READWRITE: Grant/deny read and write access.
pub const STG_DACCESS_READWRITE: i32 = 0x00000003;

    // FILE_SYS_ATTR_READ_ONLY: The file is read-only.
pub const FILE_SYS_ATTR_READ_ONLY: i32 = 0x00000001;

    // FILE_SYS_ATTR_HIDDEN: The file or directory is hidden.
pub const FILE_SYS_ATTR_HIDDEN: i32 = 0x00000002;

    // FILE_SYS_ATTR_SYSTEM: A file or directory that the operating system uses a part of, or uses exclusively.
pub const FILE_SYS_ATTR_SYSTEM: i32 = 0x00000004;

    // FILE_SYS_ATTR_DIRECTORY: The entry is a directory.
pub const FILE_SYS_ATTR_DIRECTORY: i32 = 0x00000010;

    // FILE_SYS_ATTR_ARCHIVE: The entry is an archive file or directory.
pub const FILE_SYS_ATTR_ARCHIVE: i32 = 0x00000020;

    // FILE_SYS_ATTR_NORMAL: A file doesn't have other attributes set.
pub const FILE_SYS_ATTR_NORMAL: i32 = 0x00000080;

    // FILE_SYS_ATTR_TEMPORARY: A file that is being used for temporary storage.
pub const FILE_SYS_ATTR_TEMPORARY: i32 = 0x00000100;

    // FILE_SYS_ATTR_SPARSE_FILE: A file that is a sparse file.
pub const FILE_SYS_ATTR_SPARSE_FILE: i32 = 0x00000200;

    // FILE_SYS_ATTR_REPARSE_POINT: A file that is a reparse point or a symbolic link.
pub const FILE_SYS_ATTR_REPARSE_POINT: i32 = 0x00000400;

    // FILE_SYS_ATTR_COMPRESSED: A file or directory that is compressed.
pub const FILE_SYS_ATTR_COMPRESSED: i32 = 0x00000800;

    // FILE_SYS_ATTR_OFFLINE: The data of a file are not available immediately.
pub const FILE_SYS_ATTR_OFFLINE: i32 = 0x00001000;

    // FILE_SYS_ATTR_NOT_CONTENT_INDEXED: The file or directory is not to be indexed by the content indexing service.
pub const FILE_SYS_ATTR_NOT_CONTENT_INDEXED: i32 = 0x00002000;

    // FILE_SYS_ATTR_ENCRYPTED: A file or directory that is encrypted.
pub const FILE_SYS_ATTR_ENCRYPTED: i32 = 0x00004000;

    // FILE_SYS_ATTR_VIRTUAL: Reserved.
pub const FILE_SYS_ATTR_VIRTUAL: i32 = 0x00010000;

    // FILE_SYS_ATTR_RECALL_ON_OPEN: The file or directory has no physical representation on the local system; the item is virtual.
pub const FILE_SYS_ATTR_RECALL_ON_OPEN: i32 = 0x00040000;

    // DESIRED_ACCESS_FILE_LIST_DIRECTORY: For a directory, the right to list the contents of the directory.
pub const DESIRED_ACCESS_FILE_LIST_DIRECTORY: i32 = 0x00000001;

    // DESIRED_ACCESS_FILE_READ_DATA: For a file object, the right to read the corresponding file data.
pub const DESIRED_ACCESS_FILE_READ_DATA: i32 = 0x00000001;

    // DESIRED_ACCESS_FILE_ADD_FILE: For a directory, the right to create a file in the directory.
pub const DESIRED_ACCESS_FILE_ADD_FILE: i32 = 0x00000002;

    // DESIRED_ACCESS_FILE_WRITE_DATA: For a file object, the right to write data to the file.
pub const DESIRED_ACCESS_FILE_WRITE_DATA: i32 = 0x00000002;

    // DESIRED_ACCESS_FILE_ADD_SUBDIRECTORY: For a directory, the right to create a subdirectory.
pub const DESIRED_ACCESS_FILE_ADD_SUBDIRECTORY: i32 = 0x00000004;

    // DESIRED_ACCESS_FILE_APPEND_DATA: For a file object, the right to append data to the file.
pub const DESIRED_ACCESS_FILE_APPEND_DATA: i32 = 0x00000004;

    // DESIRED_ACCESS_FILE_READ_EA: The right to read extended file attributes.
pub const DESIRED_ACCESS_FILE_READ_EA: i32 = 0x00000008;

    // DESIRED_ACCESS_FILE_WRITE_EA: The right to write extended file attributes.
pub const DESIRED_ACCESS_FILE_WRITE_EA: i32 = 0x00000010;

    // DESIRED_ACCESS_FILE_EXECUTE: For a native code file, the right to execute the file.
pub const DESIRED_ACCESS_FILE_EXECUTE: i32 = 0x00000020;

    // DESIRED_ACCESS_FILE_DELETE_CHILD: For a directory, the right to delete a directory and all the files it contains, including read-only files.
pub const DESIRED_ACCESS_FILE_DELETE_CHILD: i32 = 0x00000040;

    // DESIRED_ACCESS_FILE_READ_ATTRIBUTES: The right to read file attributes.
pub const DESIRED_ACCESS_FILE_READ_ATTRIBUTES: i32 = 0x00000080;

    // DESIRED_ACCESS_FILE_WRITE_ATTRIBUTES: The right to write file attributes.
pub const DESIRED_ACCESS_FILE_WRITE_ATTRIBUTES: i32 = 0x00000100;

    // DESIRED_ACCESS_READ_CONTROL: The right to read the information in the file or directory object's security descriptor.
pub const DESIRED_ACCESS_READ_CONTROL: i32 = 0x00020000;

    // DESIRED_ACCESS_STANDARD_RIGHTS_READ: Includes READ_CONTROL, which is the right to read the information in the file or directory object's security descriptor.
pub const DESIRED_ACCESS_STANDARD_RIGHTS_READ: i32 = 0x00020000;

    // DESIRED_ACCESS_STANDARD_RIGHTS_WRITE: Same as STANDARD_RIGHTS_READ
pub const DESIRED_ACCESS_STANDARD_RIGHTS_WRITE: i32 = 0x00020000;

    // DESIRED_ACCESS_STANDARD_RIGHTS_EXECUTE: Same as STANDARD_RIGHTS_READ
pub const DESIRED_ACCESS_STANDARD_RIGHTS_EXECUTE: i32 = 0x00020000;

    // DESIRED_ACCESS_SYNCHRONIZE: The right to use the object for synchronization.
pub const DESIRED_ACCESS_SYNCHRONIZE: i32 = 0x00100000;

    // DESIRED_ACCESS_FILE_ALL_ACCESS: All possible access rights for a file.
pub const DESIRED_ACCESS_FILE_ALL_ACCESS: i32 = 0x001F01FF;

    // DESIRED_ACCESS_FILE_GENERIC_READ: A combinarion of flags that allow reading of the file.
pub const DESIRED_ACCESS_FILE_GENERIC_READ: i32 = 0x00120089;

    // DESIRED_ACCESS_FILE_GENERIC_WRITE: A combinarion of flags that allow modifications to the file.
pub const DESIRED_ACCESS_FILE_GENERIC_WRITE: i32 = 0x00120116;

    // DESIRED_ACCESS_FILE_GENERIC_EXECUTE: A combinarion of flags that allow execution of the file.
pub const DESIRED_ACCESS_FILE_GENERIC_EXECUTE: i32 = 0x001200A0;

    // FILE_DISPOSITION_CREATE_NEW: Creates a new file, only if it does not already exist.
pub const FILE_DISPOSITION_CREATE_NEW: i32 = 0x00000001;

    // FILE_DISPOSITION_CREATE_ALWAYS: Creates a new file, always.
pub const FILE_DISPOSITION_CREATE_ALWAYS: i32 = 0x00000002;

    // FILE_DISPOSITION_OPEN_EXISTING: Opens a file, only if it exists
pub const FILE_DISPOSITION_OPEN_EXISTING: i32 = 0x00000003;

    // FILE_DISPOSITION_OPEN_ALWAYS: Opens a file, always.
pub const FILE_DISPOSITION_OPEN_ALWAYS: i32 = 0x00000004;

    // FILE_DISPOSITION_TRUNCATE_EXISTING: Opens a file and truncates it so that its size is zero bytes, only if it exists.
pub const FILE_DISPOSITION_TRUNCATE_EXISTING: i32 = 0x00000005;

    // FILE_SYS_SHARE_READ: Enables subsequent open operations on a file to request read access.
pub const FILE_SYS_SHARE_READ: i32 = 0x00000001;

    // FILE_SYS_SHARE_WRITE: Enables subsequent open operations on a file to request write access.
pub const FILE_SYS_SHARE_WRITE: i32 = 0x00000002;

    // FILE_SYS_SHARE_DELETE: Enables subsequent open operations on a file to request delete access.
pub const FILE_SYS_SHARE_DELETE: i32 = 0x00000004;

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

    // SETFA_ORIGIN_CREATE_FILE: The attributes are set as a part of a file create/open operation.
pub const SETFA_ORIGIN_CREATE_FILE: i32 = 0x0000;

    // SETFA_ORIGIN_SET_INFORMATION: The attributes are set via the SetFileAttributes or NtSetInformationFile(FileBasicInformation) function.
pub const SETFA_ORIGIN_SET_INFORMATION: i32 = 0x0006;

    // SETFA_ORIGIN_SET_SIZE: The attributes are set via the SetEndOfFile or NtSetInformationFile(FileEndOfFileInformation) function.
pub const SETFA_ORIGIN_SET_SIZE: i32 = 0x0106;

    // SETFA_ORIGIN_FSCTL: A file's Compressed state is changed using one of the *DeviceIOControl functions.
pub const SETFA_ORIGIN_FSCTL: i32 = 0x000d;

    // SETFA_ORIGIN_CLEANUP: A handle to the file is being closed and its attributes are updated .
pub const SETFA_ORIGIN_CLEANUP: i32 = 0x0012;

    // FO_O_RDONLY: The file is to be opened in read-only mode.
pub const FO_O_RDONLY: i32 = 0;

    // FO_O_WRONLY: The file is to be opened in write-only mode.
pub const FO_O_WRONLY: i32 = 0x00000001;

    // FO_O_RDWR: The file is to be opened in read-write mode.
pub const FO_O_RDWR: i32 = 0x00000002;

    // FO_O_CREAT: If the file does not exist, it should be created.
pub const FO_O_CREAT: i32 = 0x00000040;

    // FO_O_EXCL: Fail if the file is to be created but already exists.
pub const FO_O_EXCL: i32 = 0x00000080;

    // FUSE_GETLK: The lock owner is to be retrieved.
pub const FUSE_GETLK: i32 = 0x0005;

    // FUSE_SETLK: Set lock.
pub const FUSE_SETLK: i32 = 0x0006;

    // FUSE_SETLKW: Set lock and wait.
pub const FUSE_SETLKW: i32 = 0x0007;

    // FUSE_RDLCK: Read lock should be acquired.
pub const FUSE_RDLCK: i32 = 0x0000;

    // FUSE_WRLCK: Write lock should be acquired.
pub const FUSE_WRLCK: i32 = 0x0001;

    // FUSE_UNLCK: The lock should be released.
pub const FUSE_UNLCK: i32 = 0x0002;

    // CACHE_CHECK_NO_RECREATE_VAULT: If the vault is damaged beyond repair, report an error.
pub const CACHE_CHECK_NO_RECREATE_VAULT: i32 = 0x00000000;

    // CACHE_CHECK_RECREATE_VAULT: If the vault is damaged beyond repair, recreate the vault automatically.
pub const CACHE_CHECK_RECREATE_VAULT: i32 = 0x00000001;

    // CACHE_OP_IDLE: The cache is idle.
pub const CACHE_OP_IDLE: i32 = 0;

    // CACHE_OP_CLEANUP: Cache cleanup is being performed.
pub const CACHE_OP_CLEANUP: i32 = 2;

    // CACHE_OP_COMPACT: The cache's storage file is being compacted.
pub const CACHE_OP_COMPACT: i32 = 3;

    // CACHE_OP_READ: The cache is reading some file's data.
pub const CACHE_OP_READ: i32 = 4;

    // CACHE_OP_WRITE: The cache is writing some file's data.
pub const CACHE_OP_WRITE: i32 = 5;

    // ENUM_MODE_ALL: Enumerate all files.
pub const ENUM_MODE_ALL: i32 = 0;

    // ENUM_MODE_CHANGED: Enumerate files with changed blocks.
pub const ENUM_MODE_CHANGED: i32 = 1;

    // ENUM_MODE_UNCHANGED: Enumerate files with no changed blocks.
pub const ENUM_MODE_UNCHANGED: i32 = 2;

    // ENUM_MODE_LOCAL: Enumerate local files.
pub const ENUM_MODE_LOCAL: i32 = 4;

    // ENUM_MODE_ORPHAN: Enumerate orphan files.
pub const ENUM_MODE_ORPHAN: i32 = 8;

    // ENUM_MODE_PINNED: Enumerate pinned files.
pub const ENUM_MODE_PINNED: i32 = 16;

    // FILE_STATUS_CHANGED: Retrieves the value of the Changed state of the file.
pub const FILE_STATUS_CHANGED: i32 = 1;

    // FILE_STATUS_UNCHANGED: Retrieves the value of the Unchanged state of the file.
pub const FILE_STATUS_UNCHANGED: i32 = 2;

    // FILE_STATUS_LOCAL: Retrieves the value that indicates whether the file is Local.
pub const FILE_STATUS_LOCAL: i32 = 3;

    // FILE_STATUS_ORPHAN: Retrieves the value that indicates whether the file is Orphan.
pub const FILE_STATUS_ORPHAN: i32 = 4;

    // FILE_STATUS_OPEN: Retrieves the value of the Open state of the file.
pub const FILE_STATUS_OPEN: i32 = 5;

    // FLUSH_DELAYED: Flush as usual, taking into account any specified delay.
pub const FLUSH_DELAYED: i32 = 0;

    // FLUSH_NONE: Do not flush anything.
pub const FLUSH_NONE: i32 = 1;

    // FLUSH_IMMEDIATE: Flush immediately.
pub const FLUSH_IMMEDIATE: i32 = 2;

    // FLUSH_MODE_SYNC: Flush file data synchronously, blocking until finished.
pub const FLUSH_MODE_SYNC: i32 = 0;

    // FLUSH_MODE_ASYNC: Flush file data in the background; do not wait until the flush operation completes.
pub const FLUSH_MODE_ASYNC: i32 = 1;

    // FLUSH_MODE_TEST: Check to see if file data exists and needs to be flushed, but do not actually flush anything.
pub const FLUSH_MODE_TEST: i32 = 2;

    // FLUSH_RESULT_NOTHING: The file has no data that need flushing.
pub const FLUSH_RESULT_NOTHING: i32 = 0;

    // FLUSH_RESULT_DIRTY: Data in the file need flushing, but no flush operation is running yet.
pub const FLUSH_RESULT_DIRTY: i32 = 1;

    // FLUSH_RESULT_RUNNING: Data in the file are in the process of being flushed.
pub const FLUSH_RESULT_RUNNING: i32 = 2;

    // FLUSH_RESULT_SUCCESS: Data in the file have been flushed successfully.
pub const FLUSH_RESULT_SUCCESS: i32 = 3;

    // PREFETCH_NOTHING: Do not prefetch any file data.
pub const PREFETCH_NOTHING: i64 = 0;

    // PREFETCH_ALL: Prefetch all file data.
pub const PREFETCH_ALL: i64 = -1;

    // CACHE_PO_CLEANUP: Cache cleanup
pub const CACHE_PO_CLEANUP: i32 = 1;

    // PURGE_DELAYED: Purge as usual, taking into account any specified delay.
pub const PURGE_DELAYED: i32 = 0;

    // PURGE_NONE: Do not purge anything.
pub const PURGE_NONE: i32 = 1;

    // PURGE_IMMEDIATE: Purge immediately.
pub const PURGE_IMMEDIATE: i32 = 2;

    // RWEVENT_IN_PROGRESS: A file's data are being transferred into or out of the cache.
pub const RWEVENT_IN_PROGRESS: i32 = 0x00000001;

    // RWEVENT_CANCELED: A data transfer has been canceled for some reason.
pub const RWEVENT_CANCELED: i32 = 0x00000002;

    // RWEVENT_WHOLE_FILE: The entire file is being transferred.
pub const RWEVENT_WHOLE_FILE: i32 = 0x00000004;

    // RWEVENT_CONTINUOUS_STARTED: A continuous transfer has started.
pub const RWEVENT_CONTINUOUS_STARTED: i32 = 0x00000010;

    // RWEVENT_CONTINUOUS_RESTARTED: A continuous transfer has restarted from the beginning.
pub const RWEVENT_CONTINUOUS_RESTARTED: i32 = 0x00000020;

    // RWEVENT_CONTINUOUS_FINISHED: A continuous transfer is finishing (i.e., the current block is the final one).
pub const RWEVENT_CONTINUOUS_FINISHED: i32 = 0x00000040;

    // RWEVENT_RANDOM_STARTED: A random-access transfer has started.
pub const RWEVENT_RANDOM_STARTED: i32 = 0x00000100;

    // RWEVENT_RANDOM_FINISHED: A random-access transfer is finishing (i.e., the current block is the final one).
pub const RWEVENT_RANDOM_FINISHED: i32 = 0x00000200;

    // RWEVENT_BEFORE_END: The current transfer will finish before the end of the file.
pub const RWEVENT_BEFORE_END: i32 = 0x00001000;

    // RWEVENT_TIL_END: The current transfer will last until the end of the file.
pub const RWEVENT_TIL_END: i32 = 0x00002000;

    // RWCAP_POS_SEQUENTIAL_FROM_BOF: No random access allowed; read/write operations must start at the beginning of a file.
pub const RWCAP_POS_SEQUENTIAL_FROM_BOF: i32 = 0x00000001;

    // RWCAP_POS_BLOCK_MULTIPLE: Reading/writing is possible at positions that are multiple of the corresponding block size.
pub const RWCAP_POS_BLOCK_MULTIPLE: i32 = 0x00000004;

    // RWCAP_POS_RANDOM: Reading/writing is possible at any position.
pub const RWCAP_POS_RANDOM: i32 = 0x00000008;

    // RWCAP_POS_SUPPL_ONLY_WITHIN_FILE: File position must remain in the range: 0 <= FilePos < FileSize
pub const RWCAP_POS_SUPPL_ONLY_WITHIN_FILE: i32 = 0x00000010;

    // RWCAP_SIZE_WHOLE_FILE: Only whole-file reads/writes are possible; partial reads/writes are not supported.
pub const RWCAP_SIZE_WHOLE_FILE: i32 = 0x00000100;

    // RWCAP_SIZE_FIXED_BLOCKS_WITH_TAIL: Reads/writes must be done in blocks of a fixed size, except for the last block, which may be of any size.
pub const RWCAP_SIZE_FIXED_BLOCKS_WITH_TAIL: i32 = 0x00000200;

    // RWCAP_SIZE_FIXED_BLOCKS_NO_TAIL: Reads/writes must be done in blocks of a fixed size, including the last block.
pub const RWCAP_SIZE_FIXED_BLOCKS_NO_TAIL: i32 = 0x00000400;

    // RWCAP_SIZE_ANY: Reads/writes may be done in blocks of any size.
pub const RWCAP_SIZE_ANY: i32 = 0x00000800;

    // RWCAP_WRITE_APPEND_ONLY: Append only; write operations must start at the end of a file.
pub const RWCAP_WRITE_APPEND_ONLY: i32 = 0x00000002;

    // RWCAP_WRITE_SUPPL_NOT_BEYOND_EOF: File position remain in the range: 0 <= FilePos <= FileSize
pub const RWCAP_WRITE_SUPPL_NOT_BEYOND_EOF: i32 = 0x00000020;

    // RWCAP_WRITE_TRUNCATES_FILE: When writes occur, the last data written becomes the new end of the file.
pub const RWCAP_WRITE_TRUNCATES_FILE: i32 = 0x00001000;

    // RWCAP_WRITE_KEEPS_FILESIZE: Normal writing behavior (i.e., writes do not alter a file's size, except possibly to expand it).
pub const RWCAP_WRITE_KEEPS_FILESIZE: i32 = 0x00002000;

    // RWCAP_WRITE_NOT_BEYOND_EOF: Writes may not extend past the end of a file's current size (i.e., cannot cause a file to grow).
pub const RWCAP_WRITE_NOT_BEYOND_EOF: i32 = 0x00004000;

    // RSZCAP_GROW_TO_ANY: Files can grow to any size.
pub const RSZCAP_GROW_TO_ANY: i32 = 0x00000001;

    // RSZCAP_SHRINK_TO_ANY: Files can shrink to any size.
pub const RSZCAP_SHRINK_TO_ANY: i32 = 0x00000002;

    // RSZCAP_GROW_TO_BLOCK_MULTIPLE: Files can grow to sizes that are a multiple of the block size.
pub const RSZCAP_GROW_TO_BLOCK_MULTIPLE: i32 = 0x00000004;

    // RSZCAP_SHRINK_TO_BLOCK_MULTIPLE: Files can shrink to sizes that are a multiple of the block size.
pub const RSZCAP_SHRINK_TO_BLOCK_MULTIPLE: i32 = 0x00000008;

    // RSZCAP_TRUNCATE_ON_OPEN: The external storage (or application itself) supports truncating a file when it is opened for writing.
pub const RSZCAP_TRUNCATE_ON_OPEN: i32 = 0x00000010;

    // RSZCAP_TRUNCATE_AT_ZERO: The external storage (or application itself) supports truncating a file to zero at any time (not just when it is being opened).
pub const RSZCAP_TRUNCATE_AT_ZERO: i32 = 0x00000020;

    // RWRESULT_SUCCESS: Success.
pub const RWRESULT_SUCCESS: i32 = 0x00000000;

    // RWRESULT_PARTIAL: Partial success.
pub const RWRESULT_PARTIAL: i32 = 0x00000080;

    // RWRESULT_RANGE_BEYOND_EOF: Specified range is beyond the end of the file (EOF).
pub const RWRESULT_RANGE_BEYOND_EOF: i32 = 0x00000081;

    // RWRESULT_FILE_MODIFIED_EXTERNALLY: The specified file's size was modified externally.
pub const RWRESULT_FILE_MODIFIED_EXTERNALLY: i32 = 0x00000082;

    // RWRESULT_FILE_MODIFIED_LOCALLY: The requested file's size was modified locally.
pub const RWRESULT_FILE_MODIFIED_LOCALLY: i32 = 0x00000083;

    // RWRESULT_FAILURE: The operation failed for some transient reason.
pub const RWRESULT_FAILURE: i32 = 0x0000008D;

    // RWRESULT_FILE_FAILURE: The operation failed for some external-file-related reason.
pub const RWRESULT_FILE_FAILURE: i32 = 0x0000008E;

    // RWRESULT_PERMANENT_FAILURE: The operation failed for some external-storage-related reason.
pub const RWRESULT_PERMANENT_FAILURE: i32 = 0x0000008F;

    // ACCESS4_READ: Read data from a file or read a directory.
pub const ACCESS4_READ: i32 = 0x00000001;

    // ACCESS4_LOOKUP: Look up a name in a directory (no meaning for non-directory objects).
pub const ACCESS4_LOOKUP: i32 = 0x00000002;

    // ACCESS4_MODIFY: Rewrite existing file data or modify existing directory entries.
pub const ACCESS4_MODIFY: i32 = 0x00000004;

    // ACCESS4_EXTEND: Write new data or add directory entries.
pub const ACCESS4_EXTEND: i32 = 0x00000008;

    // ACCESS4_DELETE: Delete an existing directory entry.
pub const ACCESS4_DELETE: i32 = 0x00000010;

    // ACCESS4_EXECUTE: Execute a file (no meaning for a directory).
pub const ACCESS4_EXECUTE: i32 = 0x00000020;

    // OPEN4_NOCREATE: Indicates the file should be opened if it exists, but if the file does not exist, the file should not be created. In this case, the operation should fail with the error <var>NFS4ERR_NOENT</var>.
pub const OPEN4_NOCREATE: i32 = 0;

    // OPEN4_CREATE: Indicates the file should be created using the method specified by the <var>CreateMode</var> parameter.
pub const OPEN4_CREATE: i32 = 1;

    // UNCHECKED4: Indicates the file should be created without checking for the existence of a duplicate file in the associated directory.
pub const UNCHECKED4: i32 = 0;

    // GUARDED4: Indicates the file should be created, but the server should check for the presence of a duplicate file before doing so. If the file exists, the open operation should fail with the error <var>NFS4ERR_EXIST</var>.
pub const GUARDED4: i32 = 1;

    // EXCLUSIVE4: Indicates the file should be exclusively created, or created with the condition that no other client should be concurrently creating or opening a file with the same name. If a file with the same name does exist, the operation should fail with the error <var>NFS4ERR_EXIST</var>.
pub const EXCLUSIVE4: i32 = 2;

    // OPEN4_SHARE_ACCESS_READ: Indicates the client desires read-only access to the file.
pub const OPEN4_SHARE_ACCESS_READ: i32 = 0x00000001;

    // OPEN4_SHARE_ACCESS_WRITE: Indicates the client desires write-only access to the file.
pub const OPEN4_SHARE_ACCESS_WRITE: i32 = 0x00000002;

    // OPEN4_SHARE_ACCESS_BOTH: Indicates the client desires both read and write access to the file.
pub const OPEN4_SHARE_ACCESS_BOTH: i32 = 0x00000003;

    // OPEN4_SHARE_DENY_NONE: Indicates no denial of share access to other clients while the file is open.
pub const OPEN4_SHARE_DENY_NONE: i32 = 0x00000000;

    // OPEN4_SHARE_DENY_READ: Indicates denial of read access to other clients while the file is open.
pub const OPEN4_SHARE_DENY_READ: i32 = 0x00000001;

    // OPEN4_SHARE_DENY_WRITE: Indicates denial of write access to other clients while the file is open.
pub const OPEN4_SHARE_DENY_WRITE: i32 = 0x00000002;

    // OPEN4_SHARE_DENY_BOTH: Indicates denial of both read and write access to other clients while the file is open.
pub const OPEN4_SHARE_DENY_BOTH: i32 = 0x00000003;

    // READ_LT: Indicates a non-blocking read lock.
pub const READ_LT: i32 = 1;

    // WRITE_LT: Indicates a non-blocking write lock.
pub const WRITE_LT: i32 = 2;

    // READW_LT: Indicates a blocking read lock.
pub const READW_LT: i32 = 3;

    // WRITEW_LT: Indicates a blocking write lock.
pub const WRITEW_LT: i32 = 4;

    // UNSTABLE4: Indicates the application is free to commit any part of the data written and filesystem metadata before returning any results.
pub const UNSTABLE4: i32 = 0;

    // DATA_SYNC4: Indicates the application <b>must</b> commit all of the data to stable storage and enough filesystem metadata to retrieve the data before returning any results.
pub const DATA_SYNC4: i32 = 1;

    // FILE_SYNC4: Indicates the application <b>must</b> commit the data written plus all filesystem metadata to stable storage before returning any results.
pub const FILE_SYNC4: i32 = 2;

    // NFS4_OK: Indicates successful completion of the operation, in that all constituent operations completed without error.
pub const NFS4_OK: i32 = 0;

    // NFS4ERR_ACCESS: Indicates that permission has been denied. The client does not have the correct permission to perform the requested operation.
pub const NFS4ERR_ACCESS: i32 = 13;

    // NFS4ERR_ADMIN_REVOKED: A locking state of any type has been revoked due to administrative interaction, possibly while the lease is valid, or because a delegation was revoked because of failure to return it, while the lease was valid.
pub const NFS4ERR_ADMIN_REVOKED: i32 = 10047;

    // NFS4ERR_ATTRNOTSUPP: An attribute specified by the client is not supported by the server. This error <b>must not</b> be returned in the <var>GetAttr</var> event.
pub const NFS4ERR_ATTRNOTSUPP: i32 = 10032;

    // NFS4ERR_BADCHAR: A UTF-8 string contains a character that is not supported by the server in the context in which it is being used.
pub const NFS4ERR_BADCHAR: i32 = 10040;

    // NFS4ERR_BADNAME: A name string in a request consisted of valid UTF-8 characters supported by the server, but the name is not supported by the server as a valid name for current operation.
pub const NFS4ERR_BADNAME: i32 = 10041;

    // NFS4ERR_BADOWNER: This error is returned when an owner or owner_group attribute value cannot be translated to a local representation.
pub const NFS4ERR_BADOWNER: i32 = 10039;

    // NFS4ERR_BADTYPE: An attempt was made to create an object with an inappropriate type specified.
pub const NFS4ERR_BADTYPE: i32 = 10007;

    // NFS4ERR_BAD_COOKIE: This error should be returned if the client provides an invalid or unusable cookie in the <var>ReadDir</var> event.
pub const NFS4ERR_BAD_COOKIE: i32 = 10003;

    // NFS4ERR_BAD_RANGE: The range specified in the <var>Lock</var> event is not appropriate to the allowable range of offsets for the server.
pub const NFS4ERR_BAD_RANGE: i32 = 10042;

    // NFS4ERR_DEADLOCK: The server has been able to determine a file locking deadlock condition for a blocking lock request.
pub const NFS4ERR_DEADLOCK: i32 = 10045;

    // NFS4ERR_DELAY: For some reason, the application could not process this operation in what was deemed a reasonable time.
pub const NFS4ERR_DELAY: i32 = 10008;

    // NFS4ERR_DENIED: An attempt to lock a file is denied. Note this may be a temporary condition.
pub const NFS4ERR_DENIED: i32 = 10010;

    // NFS4ERR_DQUOT: Either the resource (quota) hard limit has been exceeded for the server, or the client's resource limit on the server has been exceeded.
pub const NFS4ERR_DQUOT: i32 = 69;

    // NFS4ERR_EXIST: A file system object of the specified target name already exists. Applicable when creating or renaming a file during the <var>Open</var> or <var>Rename</var> operations.
pub const NFS4ERR_EXIST: i32 = 17;

    // NFS4ERR_EXPIRED: This error indicates a locking state of any type has been revoked or released due to cancellation of the client's lease, either immediately upon lease expiration, or following a later request for a conflicting lock.
pub const NFS4ERR_EXPIRED: i32 = 10011;

    // NFS4ERR_FBIG: The file system object is too large. The operation would have caused a file system object to grow beyond the server's limit.
pub const NFS4ERR_FBIG: i32 = 27;

    // NFS4ERR_FILE_OPEN: The operation is not allowed because a file system object involved in the operation is currently open. Servers may disallow removing or renaming open file system objects.
pub const NFS4ERR_FILE_OPEN: i32 = 10046;

    // NFS4ERR_GRACE: The server is in its recovery or grace period, which should at least match the lease period of the server. A locking request other than a reclaim could not be granted during that period.
pub const NFS4ERR_GRACE: i32 = 10013;

    // NFS4ERR_INVAL: The arguments for this operation are determined to be invalid by the application.
pub const NFS4ERR_INVAL: i32 = 22;

    // NFS4ERR_IO: This indicates an unrecoverable I/O error has occurred for the file system.
pub const NFS4ERR_IO: i32 = 5;

    // NFS4ERR_ISDIR: The current object designates a directory when the current operation does not allow a directory to be accepted as the target of this operation.
pub const NFS4ERR_ISDIR: i32 = 21;

    // NFS4ERR_LEASE_MOVED: A lease being renewed is associated with a file system that has been migrated to a new server.
pub const NFS4ERR_LEASE_MOVED: i32 = 10031;

    // NFS4ERR_LOCKED: A READ or WRITE operation was attempted on a file where there was a conflict between the I/O and an existing lock.
pub const NFS4ERR_LOCKED: i32 = 10012;

    // NFS4ERR_LOCKS_HELD: An operation was prevented due to the existence of a lock on the target object.
pub const NFS4ERR_LOCKS_HELD: i32 = 10037;

    // NFS4ERR_LOCK_NOTSUPP: A locking request was attempted that would require the upgrade or downgrade of a lock range already held by the owner when the server does not support atomic upgrade or downgrade of locks.
pub const NFS4ERR_LOCK_NOTSUPP: i32 = 10043;

    // NFS4ERR_LOCK_RANGE: A lock request is operating on a range that partially overlaps a currently held lock for the current lock-owner and does not precisely match a single such lock, where the server does not support this type of request and thus does not implement POSIX locking semantics.
pub const NFS4ERR_LOCK_RANGE: i32 = 10028;

    // NFS4ERR_MLINK: The request would have caused the server's limit for the number of hard links a file system object may have to be exceeded.
pub const NFS4ERR_MLINK: i32 = 31;

    // NFS4ERR_NAMETOOLONG: This is returned when the filename in an operation exceeds the server's implementation limit.
pub const NFS4ERR_NAMETOOLONG: i32 = 63;

    // NFS4ERR_NOENT: This indicates no such file or directory. The file system object referenced by the name specified does not exist.
pub const NFS4ERR_NOENT: i32 = 2;

    // NFS4ERR_NOSPC: This indicates no space left on the device. The operation would have caused the server's file system to exceed its limit.
pub const NFS4ERR_NOSPC: i32 = 28;

    // NFS4ERR_NOTDIR: Indicates that the current object is not a directory for an operation in which a directory is required.
pub const NFS4ERR_NOTDIR: i32 = 20;

    // NFS4ERR_NOTEMPTY: An attempt was made to remove a directory that was not empty.
pub const NFS4ERR_NOTEMPTY: i32 = 66;

    // NFS4ERR_NOTSUPP: The operation is not supported, either because the operation is an <b>optional</b> one and is not supported by this server or because the operation <b>must not</b> be implemented in the current minor version.
pub const NFS4ERR_NOTSUPP: i32 = 10004;

    // NFS4ERR_NO_GRACE: The server cannot guarantee that it has not granted state to another client that may conflict with this client's state. No further reclaims from this client will succeed.
pub const NFS4ERR_NO_GRACE: i32 = 10033;

    // NFS4ERR_OPENMODE: The client attempted a <var>Read</var>, <var>Write</var>, <var>Lock</var>, or other operation not available to the client (e.g., writing to a file opened only for read).
pub const NFS4ERR_OPENMODE: i32 = 10038;

    // NFS4ERR_PERM: This indicates that the requester is not the owner.  The operation was not allowed because the caller is neither a privileged user nor the owner of the target of the operation.
pub const NFS4ERR_PERM: i32 = 1;

    // NFS4ERR_RECLAIM_BAD: The server cannot guarantee that it has not granted state to another client that may conflict with the requested state.  However, this applies only to the state requested in this call; further reclaims may succeed.
pub const NFS4ERR_RECLAIM_BAD: i32 = 10034;

    // NFS4ERR_RECLAIM_CONFLICT: The reclaim attempted by the client conflicts with a lock already held by another client.  Unlike <var>NFS4ERR_RECLAIM_BAD</var>, this can only occur if one of the clients misbehaved.
pub const NFS4ERR_RECLAIM_CONFLICT: i32 = 10035;

    // NFS4ERR_ROFS: This indicates a read-only file system.  A modifying operation was attempted on a read-only file system.
pub const NFS4ERR_ROFS: i32 = 30;

    // NFS4ERR_SERVERFAULT: An error that does not map to any of the specific legal NFSv4 protocol error values occurred on the server.
pub const NFS4ERR_SERVERFAULT: i32 = 10006;

    // NFS4ERR_SHARE_DENIED: An attempt to open a file with a share reservation has failed due to an existing share conflict.
pub const NFS4ERR_SHARE_DENIED: i32 = 10015;

    // NFS4ERR_SYMLINK: The current object designates a symbolic link when the current operation does not allow a symbolic link as the target.
pub const NFS4ERR_SYMLINK: i32 = 10029;

    // NFS4ERR_TOOSMALL: This error is used where an operation returns a variable amount of data, with a limit specified by the client.  Where the data returned cannot be fitted within the limit specified by the client, this error results.
pub const NFS4ERR_TOOSMALL: i32 = 10005;

    // NFS4ERR_XDEV: This indicates an attempt to perform an operation, such as linking, has inappropriately crossed a boundary (e.g., across a file system boundary).
pub const NFS4ERR_XDEV: i32 = 18;


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
pub struct CBFSConnectError 
{
    m_code : i32,
    m_message : String,
}

impl std::error::Error for CBFSConnectError 
{

}

impl Display for CBFSConnectError 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "CBFSConnectError {} ('{}')", &self.m_code, &self.m_message)
    }
}

/*
impl std::fmt::Debug for CBFSConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "CBFSConnectError {} ('{}')", &self.m_code, &self.m_message)
        std::fmt::Debug::fmt(&self, f)
    }
}
*/

impl From<i32> for CBFSConnectError 
{
    #[inline]
    fn from(code: i32) -> CBFSConnectError 
    {
        CBFSConnectError { m_code: code, m_message : String::from("") }
    }
}

impl From<String> for CBFSConnectError
{
    #[inline]
    fn from(message : String) -> CBFSConnectError  
    {
        CBFSConnectError { m_code: -1, m_message : String::from(message) }
    }
}

impl CBFSConnectError
{
    pub fn new(code: i32, message: &str) -> CBFSConnectError
    {
        CBFSConnectError { m_code: code, m_message : String::from(message) }
    }

    pub fn from_code(code: i32) -> CBFSConnectError
    {
        CBFSConnectError { m_code: code, m_message : String::from("") }
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

pub struct CBFSConnectStream 
{
    m_file : usize,
    m_closed : bool
}

impl CBFSConnectStream 
{

    pub fn new(file : usize) -> CBFSConnectStream
    {
        let result = CBFSConnectStream { m_file : file, m_closed : false };
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
            let func = CBFSConnect_Stream.clone().unwrap();
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
              let error = CBFSConnectError::new(rescode,  "");
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
              let error = CBFSConnectError::new(rescode,  "");
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
              let error = CBFSConnectError::new(rescode, "");
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
            let func = CBFSConnect_StreamSetLength.clone().unwrap();
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
            let func = CBFSConnect_StreamGetLength.clone().unwrap();
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

            let func = CBFSConnect_StreamSeek.clone().unwrap();
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

impl Drop for CBFSConnectStream 
{
    fn drop(&mut self) 
    {
        self.close();
    }
}

impl std::fmt::Debug for CBFSConnectStream 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
        .debug_struct("CBFSConnectStream")
        .field("handle", &self.m_file)
        .field("closed", &self.m_closed)
        .field("position", &self.get_pos())
        .field("length", &self.get_len())
        .finish()
    }
}

impl std::io::Read for CBFSConnectStream 
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

            let func = CBFSConnect_StreamRW.clone().unwrap();
            
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

impl std::io::Write for CBFSConnectStream 
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

            let func = CBFSConnect_StreamRW.clone().unwrap();
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
            let func = CBFSConnect_Stream.clone().unwrap();
            rescode = func(self.m_file, STREAM_OP_FLUSH, std::ptr::null(), std::ptr::null());
        }
        return self.check_result(rescode);
    }
}

impl std::io::Seek for CBFSConnectStream
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

            let func = CBFSConnect_StreamSeek.clone().unwrap();
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
    unsafe
    {
        // CBFSConnect_EvtStr
        let func_ptr_res  = lib_hand.get(b"CBFSConnect_EvtStr");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_EvtStr = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_EvtStrSet
        let func_ptr_res  = lib_hand.get(b"CBFSConnect_EvtStr");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_EvtStrSet = func_ptr_res.ok();
        }
        else 
        {
            return false;
        }

        // CBFSConnect_Stream
        let func_ptr_res  = lib_hand.get(b"CBFSConnect_Stream");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_Stream = func_ptr_res.ok();
        }
        else 
        {
            return false;
        }
        // CBFSConnect_StreamRW
        let func_ptr_res  = lib_hand.get(b"CBFSConnect_Stream");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_StreamRW = func_ptr_res.ok();
        }
        else 
        {
            return false;
        }
        // CBFSConnect_StreamSeek
        let func_ptr_res  = lib_hand.get(b"CBFSConnect_Stream");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_StreamSeek = func_ptr_res.ok();
        }
        else 
        {
            return false;
        }
        // CBFSConnect_StreamGetLength
        let func_ptr_res  = lib_hand.get(b"CBFSConnect_Stream");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_StreamGetLength = func_ptr_res.ok();
        }
        else 
        {
            return false;
        }
        // CBFSConnect_StreamSetLength
        let func_ptr_res  = lib_hand.get(b"CBFSConnect_Stream");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_StreamSetLength = func_ptr_res.ok();
        }
        else 
        {
            return false;
        }

        return true;
    }
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
    return OsStr::new("rustcbfsconnect24.dll");
}

#[cfg(target_os = "linux")]
fn get_lib_name() -> &'static OsStr
{
    return OsStr::new("librustcbfsconnect.so.24.0");
}

#[cfg(target_os = "macos")]
fn get_lib_name() -> &'static OsStr
{
    return OsStr::new("librustcbfsconnect.24.0.dylib");
}

#[ctor]
fn init()
{
}

fn init_on_demand()
{
    let lib_name : Option<&OsStr> = Some(get_lib_name());

    unsafe 
    {
        // try loading by just name        
        let mut load_res = libloading::Library::new(lib_name.unwrap());
        if !(load_res.is_ok())
        {
            // current directory

            let mut tmp_path_buf : PathBuf = PathBuf::from(".");
            tmp_path_buf.push(lib_name.unwrap());

            load_res = libloading::Library::new(tmp_path_buf.into_os_string() );
        }

        if !(load_res.is_ok()) {
             // Try explicit project folder structure: lib64Connect/windows/rustcbfsconnect24.dll
             let mut tmp_path_buf = PathBuf::from("lib64Connect");
             tmp_path_buf.push("windows");
             tmp_path_buf.push(lib_name.unwrap());
             load_res = libloading::Library::new(tmp_path_buf.into_os_string());
        }
        
        if !(load_res.is_ok())
        {
            // relative path to library directory
            let mut tmp_path_buf : PathBuf = PathBuf::from(get_lib_path());
            tmp_path_buf.push(lib_name.unwrap());
            load_res = libloading::Library::new(tmp_path_buf.into_os_string());
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
                        load_res = libloading::Library::new(tmp_path_buf.into_os_string() );
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

        lib_handle = load_res.ok();
          
        if lib_handle.is_some()
        {
            if (!get_lib_funcs(lib_handle.as_ref().unwrap())) 
            || (!cbcache::get_lib_funcs(lib_handle.as_ref().unwrap()))
            || (!cbfs::get_lib_funcs(lib_handle.as_ref().unwrap()))
            || (!fuse::get_lib_funcs(lib_handle.as_ref().unwrap()))
            || (!nfs::get_lib_funcs(lib_handle.as_ref().unwrap()))
            {
                panic!("Failed to load required functions from '{}'.", lib_name.unwrap().to_str().unwrap());
            }
            lib_loaded = true;
        }
    }
}

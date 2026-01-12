


#![allow(non_snake_case)]

extern crate libloading as lib;

use std::{collections::HashMap, ffi::{c_char, c_long, c_longlong, c_ulong, c_void, CStr, CString}, panic::catch_unwind, sync::{atomic::{AtomicUsize, Ordering::SeqCst}, Mutex} };
use lib::{Library, Symbol, Error};
use std::fmt::Write;
use chrono::Utc;
use once_cell::sync::Lazy;

use crate::{*, cbfsconnectkey};

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_StaticInit)(void *hInst);
type CBFSConnectCBFSStaticInitType = unsafe extern "system" fn(hInst : *mut c_void) -> i32;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_StaticDestroy)();
type CBFSConnectCBFSStaticDestroyType = unsafe extern "system" fn()-> i32;

// typedef void* (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_Create)(PCBFSCONNECT_CALLBACK lpSink, void *lpContext, char *lpOemKey, int opts);
type CBFSConnectCBFSCreateType = unsafe extern "system" fn(lpSink : CBFSConnectSinkDelegateType, lpContext : usize, lpOemKey : *const c_char, opts : i32) -> usize;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_Destroy)(void *lpObj);
type CBFSConnectCBFSDestroyType = unsafe extern "system" fn(lpObj: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_CheckIndex)(void *lpObj, int propid, int arridx);
type CBFSConnectCBFSCheckIndexType = unsafe extern "system" fn(lpObj: usize, propid: c_long, arridx: c_long)-> c_long;

// typedef void* (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_Get)(void *lpObj, int propid, int arridx, int *lpcbVal, int64 *lpllVal);
type CBFSConnectCBFSGetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *mut c_longlong) -> *mut c_void;
type CBFSConnectCBFSGetAsCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *const c_longlong) -> *const c_char;
type CBFSConnectCBFSGetAsIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *const c_void) -> usize;
type CBFSConnectCBFSGetAsInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *mut i64) -> usize;
type CBFSConnectCBFSGetAsBSTRType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut i32, llVal: *const c_void) -> *const u8;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_Set)(void *lpObj, int propid, int arridx, const void *val, int cbVal);
type CBFSConnectCBFSSetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_void, len: c_ulong)-> c_long;
type CBFSConnectCBFSSetCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_char, len: c_ulong)-> c_long;
type CBFSConnectCBFSSetIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: isize, len: c_ulong)-> c_long;
type CBFSConnectCBFSSetInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const i64, len: c_ulong)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_Do)(void *lpObj, int methid, int cparam, void *param[], int cbparam[], int64 *lpllVal);
type CBFSConnectCBFSDoType = unsafe extern "system" fn(p: usize, method_id: c_long, cparam: c_long, params: UIntPtrArrayType, cbparam: IntArrayType, llVal: *mut c_longlong)-> c_long;

// typedef char* (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_GetLastError)(void *lpObj);
type CBFSConnectCBFSGetLastErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_GetLastErrorCode)(void *lpObj);
type CBFSConnectCBFSGetLastErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_SetLastErrorAndCode)(void *lpObj, int code, char *message);
type CBFSConnectCBFSSetLastErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

// typedef char* (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_GetEventError)(void *lpObj);
type CBFSConnectCBFSGetEventErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_GetEventErrorCode)(void *lpObj);
type CBFSConnectCBFSGetEventErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBFS_SetEventErrorAndCode)(void *lpObj, int code, char *message);
type CBFSConnectCBFSSetEventErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

static mut CBFSConnect_CBFS_StaticInit : Option<Symbol<CBFSConnectCBFSStaticInitType>> = None;
static mut CBFSConnect_CBFS_StaticDestroy : Option<Symbol<CBFSConnectCBFSStaticDestroyType>> = None;

static mut CBFSConnect_CBFS_Create: Option<Symbol<CBFSConnectCBFSCreateType>> = None;
static mut CBFSConnect_CBFS_Destroy: Option<Symbol<CBFSConnectCBFSDestroyType>> = None;
static mut CBFSConnect_CBFS_Set: Option<Symbol<CBFSConnectCBFSSetType>> = None;
static mut CBFSConnect_CBFS_SetCStr: Option<Symbol<CBFSConnectCBFSSetCStrType>> = None;
static mut CBFSConnect_CBFS_SetInt: Option<Symbol<CBFSConnectCBFSSetIntType>> = None;
static mut CBFSConnect_CBFS_SetInt64: Option<Symbol<CBFSConnectCBFSSetInt64Type>> = None;
static mut CBFSConnect_CBFS_Get: Option<Symbol<CBFSConnectCBFSGetType>> = None;
static mut CBFSConnect_CBFS_GetAsCStr: Option<Symbol<CBFSConnectCBFSGetAsCStrType>> = None;
static mut CBFSConnect_CBFS_GetAsInt: Option<Symbol<CBFSConnectCBFSGetAsIntType>> = None;
static mut CBFSConnect_CBFS_GetAsInt64: Option<Symbol<CBFSConnectCBFSGetAsInt64Type>> = None;
static mut CBFSConnect_CBFS_GetAsBSTR: Option<Symbol<CBFSConnectCBFSGetAsBSTRType>> = None;
static mut CBFSConnect_CBFS_GetLastError: Option<Symbol<CBFSConnectCBFSGetLastErrorType>> = None;
static mut CBFSConnect_CBFS_GetLastErrorCode: Option<Symbol<CBFSConnectCBFSGetLastErrorCodeType>> = None;
static mut CBFSConnect_CBFS_SetLastErrorAndCode: Option<Symbol<CBFSConnectCBFSSetLastErrorAndCodeType>> = None;
static mut CBFSConnect_CBFS_GetEventError: Option<Symbol<CBFSConnectCBFSGetEventErrorType>> = None;
static mut CBFSConnect_CBFS_GetEventErrorCode: Option<Symbol<CBFSConnectCBFSGetEventErrorCodeType>> = None;
static mut CBFSConnect_CBFS_SetEventErrorAndCode: Option<Symbol<CBFSConnectCBFSSetEventErrorAndCodeType>> = None;
static mut CBFSConnect_CBFS_CheckIndex: Option<Symbol<CBFSConnectCBFSCheckIndexType>> = None;
static mut CBFSConnect_CBFS_Do: Option<Symbol<CBFSConnectCBFSDoType>> = None;

static mut CBFSIDSeed : AtomicUsize = AtomicUsize::new(1);

static mut CBFSDict : Lazy<HashMap<usize, CBFS>> = Lazy::new(|| HashMap::new() );
static CBFSDictMutex : Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0) );

const CBFSCreateOpt : i32 = 0;


pub type CBFSStream = crate::CBFSConnectStream;


pub(crate) fn get_lib_funcs( lib_hand : &'static Library) -> bool
{
    #[cfg(target_os = "linux")]
    return true;
    #[cfg(target_os = "macos")]
    return true;
    #[cfg(target_os = "android")]
    return true;
    #[cfg(target_os = "ios")]
    return true;

    unsafe
    {
        // CBFSConnect_CBFS_StaticInit
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSStaticInitType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_StaticInit");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_CBFS_StaticInit = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_StaticDestroy
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSStaticDestroyType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_StaticDestroy");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_CBFS_StaticDestroy = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_Create
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSCreateType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Create");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_CBFS_Create = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_Destroy
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSDestroyType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Destroy");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_CBFS_Destroy = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_Get
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSGetType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_Get = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBFS_GetAsCStr
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSGetAsCStrType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_GetAsCStr = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBFS_GetAsInt
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSGetAsIntType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_GetAsInt = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBFS_GetAsInt64
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSGetAsInt64Type>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_GetAsInt64 = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBFS_GetAsBSTR
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSGetAsBSTRType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_GetAsBSTR = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_Set
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSSetType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_Set = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBFS_SetCStr
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSSetCStrType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_SetCStr = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBFS_SetInt
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSSetIntType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_SetInt = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBFS_SetInt64
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSSetInt64Type>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_SetInt64 = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_GetLastError
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSGetLastErrorType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_GetLastError");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_GetLastError = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_GetLastErrorCode
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSGetLastErrorCodeType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_GetLastErrorCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_GetLastErrorCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_SetLastErrorAndCode
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSSetLastErrorAndCodeType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_SetLastErrorAndCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_SetLastErrorAndCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_GetEventError
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSGetEventErrorType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_GetEventError");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_GetEventError = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_GetEventErrorCode
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSGetEventErrorCodeType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_GetEventErrorCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_GetEventErrorCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_SetEventErrorAndCode
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSSetEventErrorAndCodeType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_SetEventErrorAndCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_SetEventErrorAndCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_CheckIndex
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSCheckIndexType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_CheckIndex");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_CheckIndex = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBFS_Do
        let func_ptr_res : Result<Symbol<CBFSConnectCBFSDoType>, Error> = lib_hand.get(b"CBFSConnect_CBFS_Do");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBFS_Do = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

    }
    return true;
}

//////////////
// Event Types
//////////////


// CBFSCanFileBeDeletedEventArgs carries the parameters of the CanFileBeDeleted event of CBFS
pub struct CBFSCanFileBeDeletedEventArgs
{
    FileName : String,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    CanBeDeleted : bool,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CanFileBeDeletedEventArgs
impl CBFSCanFileBeDeletedEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSCanFileBeDeletedEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the CanFileBeDeleted event of a CBFS instance").to_owned();
            }
        }

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(3) as usize;
        }

        let lvCanBeDeleted : bool;
        unsafe
        {
            lvCanBeDeleted = (*par.add(4) as i32) != 0;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBFSCanFileBeDeletedEventArgs
        {
            FileName: lvFileName,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            CanBeDeleted: lvCanBeDeleted,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.FileContext as isize;
            *(self._Params.add(3)) = self.HandleContext as isize;
            let intValOfCanBeDeleted : i32;
            if self.CanBeDeleted
            {
                intValOfCanBeDeleted = 1;
            }
            else
            {
                intValOfCanBeDeleted = 0;
            }
            *(self._Params.add(4)) = intValOfCanBeDeleted as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn can_be_deleted(&self) -> bool
    {
        self.CanBeDeleted
    }
    pub fn set_can_be_deleted(&mut self, value: bool)
    {
        self.CanBeDeleted = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSCanFileBeDeletedEvent
{
    fn on_can_file_be_deleted(&self, sender : &CBFS, e : &mut CBFSCanFileBeDeletedEventArgs);
}


// CBFSCleanupFileEventArgs carries the parameters of the CleanupFile event of CBFS
pub struct CBFSCleanupFileEventArgs
{
    FileName : String,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CleanupFileEventArgs
impl CBFSCleanupFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSCleanupFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the CleanupFile event of a CBFS instance").to_owned();
            }
        }

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(3) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(4) as i32;
        }
        
        CBFSCleanupFileEventArgs
        {
            FileName: lvFileName,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.FileContext as isize;
            *(self._Params.add(3)) = self.HandleContext as isize;
            *(self._Params.add(4)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSCleanupFileEvent
{
    fn on_cleanup_file(&self, sender : &CBFS, e : &mut CBFSCleanupFileEventArgs);
}


// CBFSCloseDirectoryEnumerationEventArgs carries the parameters of the CloseDirectoryEnumeration event of CBFS
pub struct CBFSCloseDirectoryEnumerationEventArgs
{
    DirectoryName : String,
    DirectoryContext : usize,
    EnumerationContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CloseDirectoryEnumerationEventArgs
impl CBFSCloseDirectoryEnumerationEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSCloseDirectoryEnumerationEventArgs
    {

        let lvhDirectoryNamePtr : *mut c_char;
        let lvDirectoryName : String;
        unsafe
        {
            lvhDirectoryNamePtr = *par.add(0) as *mut c_char;
            if lvhDirectoryNamePtr == std::ptr::null_mut()
            {
                lvDirectoryName = String::default();
            }
            else
            {
                lvDirectoryName = CStr::from_ptr(lvhDirectoryNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'DirectoryName' in the CloseDirectoryEnumeration event of a CBFS instance").to_owned();
            }
        }

        let lvDirectoryContext : usize;
        unsafe
        {
            lvDirectoryContext = *par.add(1) as usize;
        }

        let lvEnumerationContext : usize;
        unsafe
        {
            lvEnumerationContext = *par.add(2) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(3) as i32;
        }
        
        CBFSCloseDirectoryEnumerationEventArgs
        {
            DirectoryName: lvDirectoryName,
            DirectoryContext: lvDirectoryContext,
            EnumerationContext: lvEnumerationContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(1)) = self.DirectoryContext as isize;
            *(self._Params.add(3)) = self.ResultCode as isize;
        }
    }

    pub fn directory_name(&self) -> &String
    {
        &self.DirectoryName
    }
    pub fn directory_context(&self) -> usize
    {
        self.DirectoryContext
    }
    pub fn set_directory_context(&mut self, value: usize)
    {
        self.DirectoryContext = value;
    }
    pub fn enumeration_context(&self) -> usize
    {
        self.EnumerationContext
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSCloseDirectoryEnumerationEvent
{
    fn on_close_directory_enumeration(&self, sender : &CBFS, e : &mut CBFSCloseDirectoryEnumerationEventArgs);
}


// CBFSCloseFileEventArgs carries the parameters of the CloseFile event of CBFS
pub struct CBFSCloseFileEventArgs
{
    FileName : String,
    PendingDeletion : bool,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CloseFileEventArgs
impl CBFSCloseFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSCloseFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the CloseFile event of a CBFS instance").to_owned();
            }
        }

        let lvPendingDeletion : bool;
        unsafe
        {
            lvPendingDeletion = (*par.add(1) as i32) != 0;
        }

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(3) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(4) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBFSCloseFileEventArgs
        {
            FileName: lvFileName,
            PendingDeletion: lvPendingDeletion,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.FileContext as isize;
            *(self._Params.add(4)) = self.HandleContext as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn pending_deletion(&self) -> bool
    {
        self.PendingDeletion
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSCloseFileEvent
{
    fn on_close_file(&self, sender : &CBFS, e : &mut CBFSCloseFileEventArgs);
}


// CBFSCloseHardLinksEnumerationEventArgs carries the parameters of the CloseHardLinksEnumeration event of CBFS
pub struct CBFSCloseHardLinksEnumerationEventArgs
{
    FileName : String,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    EnumerationContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CloseHardLinksEnumerationEventArgs
impl CBFSCloseHardLinksEnumerationEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSCloseHardLinksEnumerationEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the CloseHardLinksEnumeration event of a CBFS instance").to_owned();
            }
        }

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(3) as usize;
        }

        let lvEnumerationContext : usize;
        unsafe
        {
            lvEnumerationContext = *par.add(4) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBFSCloseHardLinksEnumerationEventArgs
        {
            FileName: lvFileName,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            EnumerationContext: lvEnumerationContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.FileContext as isize;
            *(self._Params.add(3)) = self.HandleContext as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn enumeration_context(&self) -> usize
    {
        self.EnumerationContext
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSCloseHardLinksEnumerationEvent
{
    fn on_close_hard_links_enumeration(&self, sender : &CBFS, e : &mut CBFSCloseHardLinksEnumerationEventArgs);
}


// CBFSCloseNamedStreamsEnumerationEventArgs carries the parameters of the CloseNamedStreamsEnumeration event of CBFS
pub struct CBFSCloseNamedStreamsEnumerationEventArgs
{
    FileName : String,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    EnumerationContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CloseNamedStreamsEnumerationEventArgs
impl CBFSCloseNamedStreamsEnumerationEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSCloseNamedStreamsEnumerationEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the CloseNamedStreamsEnumeration event of a CBFS instance").to_owned();
            }
        }

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(3) as usize;
        }

        let lvEnumerationContext : usize;
        unsafe
        {
            lvEnumerationContext = *par.add(4) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBFSCloseNamedStreamsEnumerationEventArgs
        {
            FileName: lvFileName,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            EnumerationContext: lvEnumerationContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.FileContext as isize;
            *(self._Params.add(3)) = self.HandleContext as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn enumeration_context(&self) -> usize
    {
        self.EnumerationContext
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSCloseNamedStreamsEnumerationEvent
{
    fn on_close_named_streams_enumeration(&self, sender : &CBFS, e : &mut CBFSCloseNamedStreamsEnumerationEventArgs);
}


// CBFSCloseQuotasEnumerationEventArgs carries the parameters of the CloseQuotasEnumeration event of CBFS
pub struct CBFSCloseQuotasEnumerationEventArgs
{
    EnumerationContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CloseQuotasEnumerationEventArgs
impl CBFSCloseQuotasEnumerationEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSCloseQuotasEnumerationEventArgs
    {

        let lvEnumerationContext : usize;
        unsafe
        {
            lvEnumerationContext = *par.add(0) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(1) as i32;
        }
        
        CBFSCloseQuotasEnumerationEventArgs
        {
            EnumerationContext: lvEnumerationContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(1)) = self.ResultCode as isize;
        }
    }

    pub fn enumeration_context(&self) -> usize
    {
        self.EnumerationContext
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSCloseQuotasEnumerationEvent
{
    fn on_close_quotas_enumeration(&self, sender : &CBFS, e : &mut CBFSCloseQuotasEnumerationEventArgs);
}


// CBFSCreateFileEventArgs carries the parameters of the CreateFile event of CBFS
pub struct CBFSCreateFileEventArgs
{
    FileName : String,
    DesiredAccess : i32,
    Attributes : i32,
    ShareMode : i32,
    NTCreateDisposition : i32,
    NTDesiredAccess : i32,
    FileInfo : i64,
    HandleInfo : i64,
    Reserved : bool,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CreateFileEventArgs
impl CBFSCreateFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSCreateFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the CreateFile event of a CBFS instance").to_owned();
            }
        }

        let lvDesiredAccess : i32;
        unsafe
        {
            lvDesiredAccess = *par.add(1) as i32;
        }
        
        let lvAttributes : i32;
        unsafe
        {
            lvAttributes = *par.add(2) as i32;
        }
        
        let lvShareMode : i32;
        unsafe
        {
            lvShareMode = *par.add(3) as i32;
        }
        
        let lvNTCreateDisposition : i32;
        unsafe
        {
            lvNTCreateDisposition = *par.add(4) as i32;
        }
        
        let lvNTDesiredAccess : i32;
        unsafe
        {
            lvNTDesiredAccess = *par.add(5) as i32;
        }
        
        let lvFileInfo : i64;
        unsafe
        {
            let lvFileInfoLPtr : *mut i64 = *par.add(6) as *mut i64;
            lvFileInfo = *lvFileInfoLPtr;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(7) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvReserved : bool;
        unsafe
        {
            lvReserved = (*par.add(8) as i32) != 0;
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(9) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(10) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(11) as i32;
        }
        
        CBFSCreateFileEventArgs
        {
            FileName: lvFileName,
            DesiredAccess: lvDesiredAccess,
            Attributes: lvAttributes,
            ShareMode: lvShareMode,
            NTCreateDisposition: lvNTCreateDisposition,
            NTDesiredAccess: lvNTDesiredAccess,
            FileInfo: lvFileInfo,
            HandleInfo: lvHandleInfo,
            Reserved: lvReserved,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfReserved : i32;
            if self.Reserved
            {
                intValOfReserved = 1;
            }
            else
            {
                intValOfReserved = 0;
            }
            *(self._Params.add(8)) = intValOfReserved as isize;
            *(self._Params.add(9)) = self.FileContext as isize;
            *(self._Params.add(10)) = self.HandleContext as isize;
            *(self._Params.add(11)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn desired_access(&self) -> i32
    {
        self.DesiredAccess
    }
    pub fn attributes(&self) -> i32
    {
        self.Attributes
    }
    pub fn share_mode(&self) -> i32
    {
        self.ShareMode
    }
    pub fn nt_create_disposition(&self) -> i32
    {
        self.NTCreateDisposition
    }
    pub fn nt_desired_access(&self) -> i32
    {
        self.NTDesiredAccess
    }
    pub fn file_info(&self) -> i64
    {
        self.FileInfo
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn reserved(&self) -> bool
    {
        self.Reserved
    }
    pub fn set_reserved(&mut self, value: bool)
    {
        self.Reserved = value;
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSCreateFileEvent
{
    fn on_create_file(&self, sender : &CBFS, e : &mut CBFSCreateFileEventArgs);
}


// CBFSCreateHardLinkEventArgs carries the parameters of the CreateHardLink event of CBFS
pub struct CBFSCreateHardLinkEventArgs
{
    FileName : String,
    LinkName : String,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CreateHardLinkEventArgs
impl CBFSCreateHardLinkEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSCreateHardLinkEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the CreateHardLink event of a CBFS instance").to_owned();
            }
        }

        let lvhLinkNamePtr : *mut c_char;
        let lvLinkName : String;
        unsafe
        {
            lvhLinkNamePtr = *par.add(1) as *mut c_char;
            if lvhLinkNamePtr == std::ptr::null_mut()
            {
                lvLinkName = String::default();
            }
            else
            {
                lvLinkName = CStr::from_ptr(lvhLinkNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'LinkName' in the CreateHardLink event of a CBFS instance").to_owned();
            }
        }

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(3) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(4) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBFSCreateHardLinkEventArgs
        {
            FileName: lvFileName,
            LinkName: lvLinkName,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.FileContext as isize;
            *(self._Params.add(4)) = self.HandleContext as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn link_name(&self) -> &String
    {
        &self.LinkName
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSCreateHardLinkEvent
{
    fn on_create_hard_link(&self, sender : &CBFS, e : &mut CBFSCreateHardLinkEventArgs);
}


// CBFSDeleteFileEventArgs carries the parameters of the DeleteFile event of CBFS
pub struct CBFSDeleteFileEventArgs
{
    FileName : String,
    FileContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of DeleteFileEventArgs
impl CBFSDeleteFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSDeleteFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the DeleteFile event of a CBFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(2) as i32;
        }
        
        CBFSDeleteFileEventArgs
        {
            FileName: lvFileName,
            FileContext: lvFileContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(1)) = self.FileContext as isize;
            *(self._Params.add(2)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSDeleteFileEvent
{
    fn on_delete_file(&self, sender : &CBFS, e : &mut CBFSDeleteFileEventArgs);
}


// CBFSDeleteObjectIdEventArgs carries the parameters of the DeleteObjectId event of CBFS
pub struct CBFSDeleteObjectIdEventArgs
{
    FileName : String,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of DeleteObjectIdEventArgs
impl CBFSDeleteObjectIdEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSDeleteObjectIdEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the DeleteObjectId event of a CBFS instance").to_owned();
            }
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(1) as i32;
        }
        
        CBFSDeleteObjectIdEventArgs
        {
            FileName: lvFileName,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(1)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSDeleteObjectIdEvent
{
    fn on_delete_object_id(&self, sender : &CBFS, e : &mut CBFSDeleteObjectIdEventArgs);
}


// CBFSDeleteReparsePointEventArgs carries the parameters of the DeleteReparsePoint event of CBFS
pub struct CBFSDeleteReparsePointEventArgs
{
    FileName : String,
    hReparseBufferPtr : *mut u8,
    ReparseBufferLength : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of DeleteReparsePointEventArgs
impl CBFSDeleteReparsePointEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSDeleteReparsePointEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the DeleteReparsePoint event of a CBFS instance").to_owned();
            }
        }

        let lvhReparseBufferPtr : *mut u8;
        unsafe
        {
            lvhReparseBufferPtr = *par.add(1) as *mut u8;
        }

        let lvReparseBufferLength : i32;
        unsafe
        {
            lvReparseBufferLength = *par.add(2) as i32;
        }
        // lvhReparseBufferLen = lvReparseBufferLength;

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(4) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(5) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(6) as i32;
        }
        
        CBFSDeleteReparsePointEventArgs
        {
            FileName: lvFileName,
            hReparseBufferPtr: lvhReparseBufferPtr,
            ReparseBufferLength: lvReparseBufferLength,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(4)) = self.FileContext as isize;
            *(self._Params.add(5)) = self.HandleContext as isize;
            *(self._Params.add(6)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn reparse_buffer(&self) -> *mut u8
    {
        self.hReparseBufferPtr
    }
    pub fn reparse_buffer_length(&self) -> i32
    {
        self.ReparseBufferLength
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSDeleteReparsePointEvent
{
    fn on_delete_reparse_point(&self, sender : &CBFS, e : &mut CBFSDeleteReparsePointEventArgs);
}


// CBFSEjectedEventArgs carries the parameters of the Ejected event of CBFS
pub struct CBFSEjectedEventArgs
{
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of EjectedEventArgs
impl CBFSEjectedEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSEjectedEventArgs
    {

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(0) as i32;
        }
        
        CBFSEjectedEventArgs
        {
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(0)) = self.ResultCode as isize;
        }
    }

    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSEjectedEvent
{
    fn on_ejected(&self, sender : &CBFS, e : &mut CBFSEjectedEventArgs);
}


// CBFSEnumerateDirectoryEventArgs carries the parameters of the EnumerateDirectory event of CBFS
pub struct CBFSEnumerateDirectoryEventArgs
{
    DirectoryName : String,
    Mask : String,
    CaseSensitive : bool,
    Restart : bool,
    RequestedInfo : i32,
    FileFound : bool,
    hFileNamePtr : *mut c_char,
    hFileNameLen : i32,
    FileName : String,
    hShortFileNamePtr : *mut c_char,
    hShortFileNameLen : i32,
    ShortFileName : String,
    CreationTime : chrono::DateTime<Utc>,
    LastAccessTime : chrono::DateTime<Utc>,
    LastWriteTime : chrono::DateTime<Utc>,
    ChangeTime : chrono::DateTime<Utc>,
    Size : i64,
    AllocationSize : i64,
    FileId : i64,
    Attributes : i64,
    ReparseTag : i64,
    EaSize : i32,
    HandleInfo : i64,
    DirectoryContext : usize,
    HandleContext : usize,
    EnumerationContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType,
    _Cbparam : IntArrayType
}

// Constructor and marshalRefParams() of EnumerateDirectoryEventArgs
impl CBFSEnumerateDirectoryEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSEnumerateDirectoryEventArgs
    {

        let lvhDirectoryNamePtr : *mut c_char;
        let lvDirectoryName : String;
        unsafe
        {
            lvhDirectoryNamePtr = *par.add(0) as *mut c_char;
            if lvhDirectoryNamePtr == std::ptr::null_mut()
            {
                lvDirectoryName = String::default();
            }
            else
            {
                lvDirectoryName = CStr::from_ptr(lvhDirectoryNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'DirectoryName' in the EnumerateDirectory event of a CBFS instance").to_owned();
            }
        }

        let lvhMaskPtr : *mut c_char;
        let lvMask : String;
        unsafe
        {
            lvhMaskPtr = *par.add(1) as *mut c_char;
            if lvhMaskPtr == std::ptr::null_mut()
            {
                lvMask = String::default();
            }
            else
            {
                lvMask = CStr::from_ptr(lvhMaskPtr).to_str().expect("Valid UTF8 not received for the parameter 'Mask' in the EnumerateDirectory event of a CBFS instance").to_owned();
            }
        }

        let lvCaseSensitive : bool;
        unsafe
        {
            lvCaseSensitive = (*par.add(2) as i32) != 0;
        }

        let lvRestart : bool;
        unsafe
        {
            lvRestart = (*par.add(3) as i32) != 0;
        }

        let lvRequestedInfo : i32;
        unsafe
        {
            lvRequestedInfo = *par.add(4) as i32;
        }
        
        let lvFileFound : bool;
        unsafe
        {
            lvFileFound = (*par.add(5) as i32) != 0;
        }

        let lvhFileNamePtr : *mut c_char;
        let lvhFileNameLen : i32;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(6) as *mut c_char;
            lvhFileNameLen = *_cbpar.add(6);
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the EnumerateDirectory event of a CBFS instance").to_owned();
            }
        }

        let lvhShortFileNamePtr : *mut c_char;
        let lvhShortFileNameLen : i32;
        let lvShortFileName : String;
        unsafe
        {
            lvhShortFileNamePtr = *par.add(7) as *mut c_char;
            lvhShortFileNameLen = *_cbpar.add(7);
            if lvhShortFileNamePtr == std::ptr::null_mut()
            {
                lvShortFileName = String::default();
            }
            else
            {
                lvShortFileName = CStr::from_ptr(lvhShortFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'ShortFileName' in the EnumerateDirectory event of a CBFS instance").to_owned();
            }
        }

        let lvCreationTimeLong : i64;
        let lvCreationTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvCreationTimeLPtr : *mut i64 = *par.add(8) as *mut i64;
            lvCreationTimeLong = *lvCreationTimeLPtr;
            lvCreationTime = file_time_to_chrono_time(lvCreationTimeLong);
        }

        let lvLastAccessTimeLong : i64;
        let lvLastAccessTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvLastAccessTimeLPtr : *mut i64 = *par.add(9) as *mut i64;
            lvLastAccessTimeLong = *lvLastAccessTimeLPtr;
            lvLastAccessTime = file_time_to_chrono_time(lvLastAccessTimeLong);
        }

        let lvLastWriteTimeLong : i64;
        let lvLastWriteTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvLastWriteTimeLPtr : *mut i64 = *par.add(10) as *mut i64;
            lvLastWriteTimeLong = *lvLastWriteTimeLPtr;
            lvLastWriteTime = file_time_to_chrono_time(lvLastWriteTimeLong);
        }

        let lvChangeTimeLong : i64;
        let lvChangeTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvChangeTimeLPtr : *mut i64 = *par.add(11) as *mut i64;
            lvChangeTimeLong = *lvChangeTimeLPtr;
            lvChangeTime = file_time_to_chrono_time(lvChangeTimeLong);
        }

        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(12) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvAllocationSize : i64;
        unsafe
        {
            let lvAllocationSizeLPtr : *mut i64 = *par.add(13) as *mut i64;
            lvAllocationSize = *lvAllocationSizeLPtr;
        }
        
        let lvFileId : i64;
        unsafe
        {
            let lvFileIdLPtr : *mut i64 = *par.add(14) as *mut i64;
            lvFileId = *lvFileIdLPtr;
        }
        
        let lvAttributes : i64;
        unsafe
        {
            let lvAttributesLPtr : *mut i64 = *par.add(15) as *mut i64;
            lvAttributes = *lvAttributesLPtr;
        }
        
        let lvReparseTag : i64;
        unsafe
        {
            let lvReparseTagLPtr : *mut i64 = *par.add(16) as *mut i64;
            lvReparseTag = *lvReparseTagLPtr;
        }
        
        let lvEaSize : i32;
        unsafe
        {
            lvEaSize = *par.add(17) as i32;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(18) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvDirectoryContext : usize;
        unsafe
        {
            lvDirectoryContext = *par.add(19) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(20) as usize;
        }

        let lvEnumerationContext : usize;
        unsafe
        {
            lvEnumerationContext = *par.add(21) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(22) as i32;
        }
        
        CBFSEnumerateDirectoryEventArgs
        {
            DirectoryName: lvDirectoryName,
            Mask: lvMask,
            CaseSensitive: lvCaseSensitive,
            Restart: lvRestart,
            RequestedInfo: lvRequestedInfo,
            FileFound: lvFileFound,
            hFileNamePtr: lvhFileNamePtr,
            hFileNameLen: lvhFileNameLen,
            FileName: lvFileName,
            hShortFileNamePtr: lvhShortFileNamePtr,
            hShortFileNameLen: lvhShortFileNameLen,
            ShortFileName: lvShortFileName,
            CreationTime: lvCreationTime,
            LastAccessTime: lvLastAccessTime,
            LastWriteTime: lvLastWriteTime,
            ChangeTime: lvChangeTime,
            Size: lvSize,
            AllocationSize: lvAllocationSize,
            FileId: lvFileId,
            Attributes: lvAttributes,
            ReparseTag: lvReparseTag,
            EaSize: lvEaSize,
            HandleInfo: lvHandleInfo,
            DirectoryContext: lvDirectoryContext,
            HandleContext: lvHandleContext,
            EnumerationContext: lvEnumerationContext,
            ResultCode: lvResultCode,
            _Params: par,
            _Cbparam: _cbpar
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfFileFound : i32;
            if self.FileFound
            {
                intValOfFileFound = 1;
            }
            else
            {
                intValOfFileFound = 0;
            }
            *(self._Params.add(5)) = intValOfFileFound as isize;
            let bytesFileName = self.FileName.as_bytes();
            let to_copy : usize;
            let bytesFileNameLen = bytesFileName.len();
            if bytesFileNameLen + 1 < self.hFileNameLen as usize
            {
                to_copy = bytesFileNameLen;
            }
            else
            {
                to_copy = self.hFileNameLen as usize - 1;
            }
            if to_copy > 0
            {
                std::ptr::copy_nonoverlapping(bytesFileName.as_ptr(), self.hFileNamePtr as *mut u8, to_copy);
            }
            *(self.hFileNamePtr.add(to_copy)) = 0;
            *(self._Cbparam.add(6)) = to_copy as i32;
            let bytesShortFileName = self.ShortFileName.as_bytes();
            let to_copy : usize;
            let bytesShortFileNameLen = bytesShortFileName.len();
            if bytesShortFileNameLen + 1 < self.hShortFileNameLen as usize
            {
                to_copy = bytesShortFileNameLen;
            }
            else
            {
                to_copy = self.hShortFileNameLen as usize - 1;
            }
            if to_copy > 0
            {
                std::ptr::copy_nonoverlapping(bytesShortFileName.as_ptr(), self.hShortFileNamePtr as *mut u8, to_copy);
            }
            *(self.hShortFileNamePtr.add(to_copy)) = 0;
            *(self._Cbparam.add(7)) = to_copy as i32;
            let intValOfCreationTime : i64 = chrono_time_to_file_time(&self.CreationTime);
            let lvCreationTimeLPtr : *mut i64 = *self._Params.add(8) as *mut i64;
            *lvCreationTimeLPtr = intValOfCreationTime as i64;
            let intValOfLastAccessTime : i64 = chrono_time_to_file_time(&self.LastAccessTime);
            let lvLastAccessTimeLPtr : *mut i64 = *self._Params.add(9) as *mut i64;
            *lvLastAccessTimeLPtr = intValOfLastAccessTime as i64;
            let intValOfLastWriteTime : i64 = chrono_time_to_file_time(&self.LastWriteTime);
            let lvLastWriteTimeLPtr : *mut i64 = *self._Params.add(10) as *mut i64;
            *lvLastWriteTimeLPtr = intValOfLastWriteTime as i64;
            let intValOfChangeTime : i64 = chrono_time_to_file_time(&self.ChangeTime);
            let lvChangeTimeLPtr : *mut i64 = *self._Params.add(11) as *mut i64;
            *lvChangeTimeLPtr = intValOfChangeTime as i64;
            let lvSizeLPtr : *mut i64 = *self._Params.add(12) as *mut i64;
            *lvSizeLPtr = self.Size ;
            let lvAllocationSizeLPtr : *mut i64 = *self._Params.add(13) as *mut i64;
            *lvAllocationSizeLPtr = self.AllocationSize ;
            let lvFileIdLPtr : *mut i64 = *self._Params.add(14) as *mut i64;
            *lvFileIdLPtr = self.FileId ;
            let lvAttributesLPtr : *mut i64 = *self._Params.add(15) as *mut i64;
            *lvAttributesLPtr = self.Attributes ;
            let lvReparseTagLPtr : *mut i64 = *self._Params.add(16) as *mut i64;
            *lvReparseTagLPtr = self.ReparseTag ;
            *(self._Params.add(17)) = self.EaSize as isize;
            *(self._Params.add(19)) = self.DirectoryContext as isize;
            *(self._Params.add(20)) = self.HandleContext as isize;
            *(self._Params.add(21)) = self.EnumerationContext as isize;
            *(self._Params.add(22)) = self.ResultCode as isize;
        }
    }

    pub fn directory_name(&self) -> &String
    {
        &self.DirectoryName
    }
    pub fn mask(&self) -> &String
    {
        &self.Mask
    }
    pub fn case_sensitive(&self) -> bool
    {
        self.CaseSensitive
    }
    pub fn restart(&self) -> bool
    {
        self.Restart
    }
    pub fn requested_info(&self) -> i32
    {
        self.RequestedInfo
    }
    pub fn file_found(&self) -> bool
    {
        self.FileFound
    }
    pub fn set_file_found(&mut self, value: bool)
    {
        self.FileFound = value;
    }
    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn set_file_name_ref(&mut self, value: &String)
    {
        self.FileName = value.clone();
    }
    pub fn set_file_name(&mut self, value: String)
    {
        self.FileName = value;
    }
    pub fn short_file_name(&self) -> &String
    {
        &self.ShortFileName
    }
    pub fn set_short_file_name_ref(&mut self, value: &String)
    {
        self.ShortFileName = value.clone();
    }
    pub fn set_short_file_name(&mut self, value: String)
    {
        self.ShortFileName = value;
    }
    pub fn creation_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.CreationTime
    }
    pub fn set_creation_time_ref(&mut self, value: &chrono::DateTime<Utc>)
    {
        self.CreationTime = value.clone();
    }
    pub fn set_creation_time(&mut self, value: chrono::DateTime<Utc>)
    {
        self.CreationTime = value;
    }
    pub fn last_access_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.LastAccessTime
    }
    pub fn set_last_access_time_ref(&mut self, value: &chrono::DateTime<Utc>)
    {
        self.LastAccessTime = value.clone();
    }
    pub fn set_last_access_time(&mut self, value: chrono::DateTime<Utc>)
    {
        self.LastAccessTime = value;
    }
    pub fn last_write_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.LastWriteTime
    }
    pub fn set_last_write_time_ref(&mut self, value: &chrono::DateTime<Utc>)
    {
        self.LastWriteTime = value.clone();
    }
    pub fn set_last_write_time(&mut self, value: chrono::DateTime<Utc>)
    {
        self.LastWriteTime = value;
    }
    pub fn change_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.ChangeTime
    }
    pub fn set_change_time_ref(&mut self, value: &chrono::DateTime<Utc>)
    {
        self.ChangeTime = value.clone();
    }
    pub fn set_change_time(&mut self, value: chrono::DateTime<Utc>)
    {
        self.ChangeTime = value;
    }
    pub fn size(&self) -> i64
    {
        self.Size
    }
    pub fn set_size(&mut self, value: i64)
    {
        self.Size = value;
    }
    pub fn allocation_size(&self) -> i64
    {
        self.AllocationSize
    }
    pub fn set_allocation_size(&mut self, value: i64)
    {
        self.AllocationSize = value;
    }
    pub fn file_id(&self) -> i64
    {
        self.FileId
    }
    pub fn set_file_id(&mut self, value: i64)
    {
        self.FileId = value;
    }
    pub fn attributes(&self) -> i64
    {
        self.Attributes
    }
    pub fn set_attributes(&mut self, value: i64)
    {
        self.Attributes = value;
    }
    pub fn reparse_tag(&self) -> i64
    {
        self.ReparseTag
    }
    pub fn set_reparse_tag(&mut self, value: i64)
    {
        self.ReparseTag = value;
    }
    pub fn ea_size(&self) -> i32
    {
        self.EaSize
    }
    pub fn set_ea_size(&mut self, value: i32)
    {
        self.EaSize = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn directory_context(&self) -> usize
    {
        self.DirectoryContext
    }
    pub fn set_directory_context(&mut self, value: usize)
    {
        self.DirectoryContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn enumeration_context(&self) -> usize
    {
        self.EnumerationContext
    }
    pub fn set_enumeration_context(&mut self, value: usize)
    {
        self.EnumerationContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSEnumerateDirectoryEvent
{
    fn on_enumerate_directory(&self, sender : &CBFS, e : &mut CBFSEnumerateDirectoryEventArgs);
}


// CBFSEnumerateHardLinksEventArgs carries the parameters of the EnumerateHardLinks event of CBFS
pub struct CBFSEnumerateHardLinksEventArgs
{
    FileName : String,
    LinkFound : bool,
    hLinkNamePtr : *mut c_char,
    hLinkNameLen : i32,
    LinkName : String,
    ParentId : i64,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    EnumerationContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType,
    _Cbparam : IntArrayType
}

// Constructor and marshalRefParams() of EnumerateHardLinksEventArgs
impl CBFSEnumerateHardLinksEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSEnumerateHardLinksEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the EnumerateHardLinks event of a CBFS instance").to_owned();
            }
        }

        let lvLinkFound : bool;
        unsafe
        {
            lvLinkFound = (*par.add(1) as i32) != 0;
        }

        let lvhLinkNamePtr : *mut c_char;
        let lvhLinkNameLen : i32;
        let lvLinkName : String;
        unsafe
        {
            lvhLinkNamePtr = *par.add(2) as *mut c_char;
            lvhLinkNameLen = *_cbpar.add(2);
            if lvhLinkNamePtr == std::ptr::null_mut()
            {
                lvLinkName = String::default();
            }
            else
            {
                lvLinkName = CStr::from_ptr(lvhLinkNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'LinkName' in the EnumerateHardLinks event of a CBFS instance").to_owned();
            }
        }

        let lvParentId : i64;
        unsafe
        {
            let lvParentIdLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvParentId = *lvParentIdLPtr;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(5) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(6) as usize;
        }

        let lvEnumerationContext : usize;
        unsafe
        {
            lvEnumerationContext = *par.add(7) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(8) as i32;
        }
        
        CBFSEnumerateHardLinksEventArgs
        {
            FileName: lvFileName,
            LinkFound: lvLinkFound,
            hLinkNamePtr: lvhLinkNamePtr,
            hLinkNameLen: lvhLinkNameLen,
            LinkName: lvLinkName,
            ParentId: lvParentId,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            EnumerationContext: lvEnumerationContext,
            ResultCode: lvResultCode,
            _Params: par,
            _Cbparam: _cbpar
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfLinkFound : i32;
            if self.LinkFound
            {
                intValOfLinkFound = 1;
            }
            else
            {
                intValOfLinkFound = 0;
            }
            *(self._Params.add(1)) = intValOfLinkFound as isize;
            let bytesLinkName = self.LinkName.as_bytes();
            let to_copy : usize;
            let bytesLinkNameLen = bytesLinkName.len();
            if bytesLinkNameLen + 1 < self.hLinkNameLen as usize
            {
                to_copy = bytesLinkNameLen;
            }
            else
            {
                to_copy = self.hLinkNameLen as usize - 1;
            }
            if to_copy > 0
            {
                std::ptr::copy_nonoverlapping(bytesLinkName.as_ptr(), self.hLinkNamePtr as *mut u8, to_copy);
            }
            *(self.hLinkNamePtr.add(to_copy)) = 0;
            *(self._Cbparam.add(2)) = to_copy as i32;
            let lvParentIdLPtr : *mut i64 = *self._Params.add(3) as *mut i64;
            *lvParentIdLPtr = self.ParentId ;
            *(self._Params.add(5)) = self.FileContext as isize;
            *(self._Params.add(6)) = self.HandleContext as isize;
            *(self._Params.add(7)) = self.EnumerationContext as isize;
            *(self._Params.add(8)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn link_found(&self) -> bool
    {
        self.LinkFound
    }
    pub fn set_link_found(&mut self, value: bool)
    {
        self.LinkFound = value;
    }
    pub fn link_name(&self) -> &String
    {
        &self.LinkName
    }
    pub fn set_link_name_ref(&mut self, value: &String)
    {
        self.LinkName = value.clone();
    }
    pub fn set_link_name(&mut self, value: String)
    {
        self.LinkName = value;
    }
    pub fn parent_id(&self) -> i64
    {
        self.ParentId
    }
    pub fn set_parent_id(&mut self, value: i64)
    {
        self.ParentId = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn enumeration_context(&self) -> usize
    {
        self.EnumerationContext
    }
    pub fn set_enumeration_context(&mut self, value: usize)
    {
        self.EnumerationContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSEnumerateHardLinksEvent
{
    fn on_enumerate_hard_links(&self, sender : &CBFS, e : &mut CBFSEnumerateHardLinksEventArgs);
}


// CBFSEnumerateNamedStreamsEventArgs carries the parameters of the EnumerateNamedStreams event of CBFS
pub struct CBFSEnumerateNamedStreamsEventArgs
{
    FileName : String,
    NamedStreamFound : bool,
    hStreamNamePtr : *mut c_char,
    hStreamNameLen : i32,
    StreamName : String,
    StreamSize : i64,
    StreamAllocationSize : i64,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    EnumerationContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType,
    _Cbparam : IntArrayType
}

// Constructor and marshalRefParams() of EnumerateNamedStreamsEventArgs
impl CBFSEnumerateNamedStreamsEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSEnumerateNamedStreamsEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the EnumerateNamedStreams event of a CBFS instance").to_owned();
            }
        }

        let lvNamedStreamFound : bool;
        unsafe
        {
            lvNamedStreamFound = (*par.add(1) as i32) != 0;
        }

        let lvhStreamNamePtr : *mut c_char;
        let lvhStreamNameLen : i32;
        let lvStreamName : String;
        unsafe
        {
            lvhStreamNamePtr = *par.add(2) as *mut c_char;
            lvhStreamNameLen = *_cbpar.add(2);
            if lvhStreamNamePtr == std::ptr::null_mut()
            {
                lvStreamName = String::default();
            }
            else
            {
                lvStreamName = CStr::from_ptr(lvhStreamNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'StreamName' in the EnumerateNamedStreams event of a CBFS instance").to_owned();
            }
        }

        let lvStreamSize : i64;
        unsafe
        {
            let lvStreamSizeLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvStreamSize = *lvStreamSizeLPtr;
        }
        
        let lvStreamAllocationSize : i64;
        unsafe
        {
            let lvStreamAllocationSizeLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvStreamAllocationSize = *lvStreamAllocationSizeLPtr;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(6) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(7) as usize;
        }

        let lvEnumerationContext : usize;
        unsafe
        {
            lvEnumerationContext = *par.add(8) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(9) as i32;
        }
        
        CBFSEnumerateNamedStreamsEventArgs
        {
            FileName: lvFileName,
            NamedStreamFound: lvNamedStreamFound,
            hStreamNamePtr: lvhStreamNamePtr,
            hStreamNameLen: lvhStreamNameLen,
            StreamName: lvStreamName,
            StreamSize: lvStreamSize,
            StreamAllocationSize: lvStreamAllocationSize,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            EnumerationContext: lvEnumerationContext,
            ResultCode: lvResultCode,
            _Params: par,
            _Cbparam: _cbpar
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfNamedStreamFound : i32;
            if self.NamedStreamFound
            {
                intValOfNamedStreamFound = 1;
            }
            else
            {
                intValOfNamedStreamFound = 0;
            }
            *(self._Params.add(1)) = intValOfNamedStreamFound as isize;
            let bytesStreamName = self.StreamName.as_bytes();
            let to_copy : usize;
            let bytesStreamNameLen = bytesStreamName.len();
            if bytesStreamNameLen + 1 < self.hStreamNameLen as usize
            {
                to_copy = bytesStreamNameLen;
            }
            else
            {
                to_copy = self.hStreamNameLen as usize - 1;
            }
            if to_copy > 0
            {
                std::ptr::copy_nonoverlapping(bytesStreamName.as_ptr(), self.hStreamNamePtr as *mut u8, to_copy);
            }
            *(self.hStreamNamePtr.add(to_copy)) = 0;
            *(self._Cbparam.add(2)) = to_copy as i32;
            let lvStreamSizeLPtr : *mut i64 = *self._Params.add(3) as *mut i64;
            *lvStreamSizeLPtr = self.StreamSize ;
            let lvStreamAllocationSizeLPtr : *mut i64 = *self._Params.add(4) as *mut i64;
            *lvStreamAllocationSizeLPtr = self.StreamAllocationSize ;
            *(self._Params.add(6)) = self.FileContext as isize;
            *(self._Params.add(7)) = self.HandleContext as isize;
            *(self._Params.add(8)) = self.EnumerationContext as isize;
            *(self._Params.add(9)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn named_stream_found(&self) -> bool
    {
        self.NamedStreamFound
    }
    pub fn set_named_stream_found(&mut self, value: bool)
    {
        self.NamedStreamFound = value;
    }
    pub fn stream_name(&self) -> &String
    {
        &self.StreamName
    }
    pub fn set_stream_name_ref(&mut self, value: &String)
    {
        self.StreamName = value.clone();
    }
    pub fn set_stream_name(&mut self, value: String)
    {
        self.StreamName = value;
    }
    pub fn stream_size(&self) -> i64
    {
        self.StreamSize
    }
    pub fn set_stream_size(&mut self, value: i64)
    {
        self.StreamSize = value;
    }
    pub fn stream_allocation_size(&self) -> i64
    {
        self.StreamAllocationSize
    }
    pub fn set_stream_allocation_size(&mut self, value: i64)
    {
        self.StreamAllocationSize = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn enumeration_context(&self) -> usize
    {
        self.EnumerationContext
    }
    pub fn set_enumeration_context(&mut self, value: usize)
    {
        self.EnumerationContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSEnumerateNamedStreamsEvent
{
    fn on_enumerate_named_streams(&self, sender : &CBFS, e : &mut CBFSEnumerateNamedStreamsEventArgs);
}


// CBFSErrorEventArgs carries the parameters of the Error event of CBFS
pub struct CBFSErrorEventArgs
{
    ErrorCode : i32,
    Description : String,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ErrorEventArgs
impl CBFSErrorEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSErrorEventArgs
    {

        let lvErrorCode : i32;
        unsafe
        {
            lvErrorCode = *par.add(0) as i32;
        }
        
        let lvhDescriptionPtr : *mut c_char;
        let lvDescription : String;
        unsafe
        {
            lvhDescriptionPtr = *par.add(1) as *mut c_char;
            if lvhDescriptionPtr == std::ptr::null_mut()
            {
                lvDescription = String::default();
            }
            else
            {
                lvDescription = CStr::from_ptr(lvhDescriptionPtr).to_str().expect("Valid UTF8 not received for the parameter 'Description' in the Error event of a CBFS instance").to_owned();
            }
        }

        CBFSErrorEventArgs
        {
            ErrorCode: lvErrorCode,
            Description: lvDescription,
            _Params: par
        }
    }


    pub fn error_code(&self) -> i32
    {
        self.ErrorCode
    }
    pub fn description(&self) -> &String
    {
        &self.Description
    }
}

pub trait CBFSErrorEvent
{
    fn on_error(&self, sender : &CBFS, e : &mut CBFSErrorEventArgs);
}


// CBFSFlushFileEventArgs carries the parameters of the FlushFile event of CBFS
pub struct CBFSFlushFileEventArgs
{
    FileName : String,
    FileContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of FlushFileEventArgs
impl CBFSFlushFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSFlushFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the FlushFile event of a CBFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(2) as i32;
        }
        
        CBFSFlushFileEventArgs
        {
            FileName: lvFileName,
            FileContext: lvFileContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSFlushFileEvent
{
    fn on_flush_file(&self, sender : &CBFS, e : &mut CBFSFlushFileEventArgs);
}


// CBFSFsctlEventArgs carries the parameters of the Fsctl event of CBFS
pub struct CBFSFsctlEventArgs
{
    FileName : String,
    Code : i32,
    hInputBufferPtr : *mut u8,
    InputBufferLength : i32,
    hOutputBufferPtr : *mut u8,
    OutputBufferLength : i32,
    BytesReturned : i32,
    FileContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of FsctlEventArgs
impl CBFSFsctlEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSFsctlEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the Fsctl event of a CBFS instance").to_owned();
            }
        }

        let lvCode : i32;
        unsafe
        {
            lvCode = *par.add(1) as i32;
        }
        
        let lvhInputBufferPtr : *mut u8;
        unsafe
        {
            lvhInputBufferPtr = *par.add(2) as *mut u8;
        }

        let lvInputBufferLength : i32;
        unsafe
        {
            lvInputBufferLength = *par.add(3) as i32;
        }
        // lvhInputBufferLen = lvInputBufferLength;

        let lvhOutputBufferPtr : *mut u8;
        unsafe
        {
            lvhOutputBufferPtr = *par.add(4) as *mut u8;
        }

        let lvOutputBufferLength : i32;
        unsafe
        {
            lvOutputBufferLength = *par.add(5) as i32;
        }
        // lvhOutputBufferLen = lvOutputBufferLength;

        let lvBytesReturned : i32;
        unsafe
        {
            lvBytesReturned = *par.add(6) as i32;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(7) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(8) as i32;
        }
        
        CBFSFsctlEventArgs
        {
            FileName: lvFileName,
            Code: lvCode,
            hInputBufferPtr: lvhInputBufferPtr,
            InputBufferLength: lvInputBufferLength,
            hOutputBufferPtr: lvhOutputBufferPtr,
            OutputBufferLength: lvOutputBufferLength,
            BytesReturned: lvBytesReturned,
            FileContext: lvFileContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(6)) = self.BytesReturned as isize;
            *(self._Params.add(7)) = self.FileContext as isize;
            *(self._Params.add(8)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn code(&self) -> i32
    {
        self.Code
    }
    pub fn input_buffer(&self) -> *mut u8
    {
        self.hInputBufferPtr
    }
    pub fn input_buffer_length(&self) -> i32
    {
        self.InputBufferLength
    }
    pub fn output_buffer(&self) -> *mut u8
    {
        self.hOutputBufferPtr
    }
    pub fn output_buffer_length(&self) -> i32
    {
        self.OutputBufferLength
    }
    pub fn bytes_returned(&self) -> i32
    {
        self.BytesReturned
    }
    pub fn set_bytes_returned(&mut self, value: i32)
    {
        self.BytesReturned = value;
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSFsctlEvent
{
    fn on_fsctl(&self, sender : &CBFS, e : &mut CBFSFsctlEventArgs);
}


// CBFSGetDefaultQuotaInfoEventArgs carries the parameters of the GetDefaultQuotaInfo event of CBFS
pub struct CBFSGetDefaultQuotaInfoEventArgs
{
    DefaultQuotaThreshold : i64,
    DefaultQuotaLimit : i64,
    FileSystemControlFlags : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of GetDefaultQuotaInfoEventArgs
impl CBFSGetDefaultQuotaInfoEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSGetDefaultQuotaInfoEventArgs
    {

        let lvDefaultQuotaThreshold : i64;
        unsafe
        {
            let lvDefaultQuotaThresholdLPtr : *mut i64 = *par.add(0) as *mut i64;
            lvDefaultQuotaThreshold = *lvDefaultQuotaThresholdLPtr;
        }
        
        let lvDefaultQuotaLimit : i64;
        unsafe
        {
            let lvDefaultQuotaLimitLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvDefaultQuotaLimit = *lvDefaultQuotaLimitLPtr;
        }
        
        let lvFileSystemControlFlags : i64;
        unsafe
        {
            let lvFileSystemControlFlagsLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvFileSystemControlFlags = *lvFileSystemControlFlagsLPtr;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(3) as i32;
        }
        
        CBFSGetDefaultQuotaInfoEventArgs
        {
            DefaultQuotaThreshold: lvDefaultQuotaThreshold,
            DefaultQuotaLimit: lvDefaultQuotaLimit,
            FileSystemControlFlags: lvFileSystemControlFlags,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvDefaultQuotaThresholdLPtr : *mut i64 = *self._Params.add(0) as *mut i64;
            *lvDefaultQuotaThresholdLPtr = self.DefaultQuotaThreshold ;
            let lvDefaultQuotaLimitLPtr : *mut i64 = *self._Params.add(1) as *mut i64;
            *lvDefaultQuotaLimitLPtr = self.DefaultQuotaLimit ;
            let lvFileSystemControlFlagsLPtr : *mut i64 = *self._Params.add(2) as *mut i64;
            *lvFileSystemControlFlagsLPtr = self.FileSystemControlFlags ;
            *(self._Params.add(3)) = self.ResultCode as isize;
        }
    }

    pub fn default_quota_threshold(&self) -> i64
    {
        self.DefaultQuotaThreshold
    }
    pub fn set_default_quota_threshold(&mut self, value: i64)
    {
        self.DefaultQuotaThreshold = value;
    }
    pub fn default_quota_limit(&self) -> i64
    {
        self.DefaultQuotaLimit
    }
    pub fn set_default_quota_limit(&mut self, value: i64)
    {
        self.DefaultQuotaLimit = value;
    }
    pub fn file_system_control_flags(&self) -> i64
    {
        self.FileSystemControlFlags
    }
    pub fn set_file_system_control_flags(&mut self, value: i64)
    {
        self.FileSystemControlFlags = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSGetDefaultQuotaInfoEvent
{
    fn on_get_default_quota_info(&self, sender : &CBFS, e : &mut CBFSGetDefaultQuotaInfoEventArgs);
}


// CBFSGetFileInfoEventArgs carries the parameters of the GetFileInfo event of CBFS
pub struct CBFSGetFileInfoEventArgs
{
    FileName : String,
    RequestedInfo : i32,
    FileExists : bool,
    CreationTime : chrono::DateTime<Utc>,
    LastAccessTime : chrono::DateTime<Utc>,
    LastWriteTime : chrono::DateTime<Utc>,
    ChangeTime : chrono::DateTime<Utc>,
    Size : i64,
    AllocationSize : i64,
    FileId : i64,
    Attributes : i64,
    ReparseTag : i64,
    HardLinkCount : i32,
    hShortFileNamePtr : *mut c_char,
    hShortFileNameLen : i32,
    ShortFileName : String,
    hRealFileNamePtr : *mut c_char,
    hRealFileNameLen : i32,
    RealFileName : String,
    EaSize : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType,
    _Cbparam : IntArrayType
}

// Constructor and marshalRefParams() of GetFileInfoEventArgs
impl CBFSGetFileInfoEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSGetFileInfoEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the GetFileInfo event of a CBFS instance").to_owned();
            }
        }

        let lvRequestedInfo : i32;
        unsafe
        {
            lvRequestedInfo = *par.add(1) as i32;
        }
        
        let lvFileExists : bool;
        unsafe
        {
            lvFileExists = (*par.add(2) as i32) != 0;
        }

        let lvCreationTimeLong : i64;
        let lvCreationTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvCreationTimeLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvCreationTimeLong = *lvCreationTimeLPtr;
            lvCreationTime = file_time_to_chrono_time(lvCreationTimeLong);
        }

        let lvLastAccessTimeLong : i64;
        let lvLastAccessTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvLastAccessTimeLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvLastAccessTimeLong = *lvLastAccessTimeLPtr;
            lvLastAccessTime = file_time_to_chrono_time(lvLastAccessTimeLong);
        }

        let lvLastWriteTimeLong : i64;
        let lvLastWriteTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvLastWriteTimeLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvLastWriteTimeLong = *lvLastWriteTimeLPtr;
            lvLastWriteTime = file_time_to_chrono_time(lvLastWriteTimeLong);
        }

        let lvChangeTimeLong : i64;
        let lvChangeTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvChangeTimeLPtr : *mut i64 = *par.add(6) as *mut i64;
            lvChangeTimeLong = *lvChangeTimeLPtr;
            lvChangeTime = file_time_to_chrono_time(lvChangeTimeLong);
        }

        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(7) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvAllocationSize : i64;
        unsafe
        {
            let lvAllocationSizeLPtr : *mut i64 = *par.add(8) as *mut i64;
            lvAllocationSize = *lvAllocationSizeLPtr;
        }
        
        let lvFileId : i64;
        unsafe
        {
            let lvFileIdLPtr : *mut i64 = *par.add(9) as *mut i64;
            lvFileId = *lvFileIdLPtr;
        }
        
        let lvAttributes : i64;
        unsafe
        {
            let lvAttributesLPtr : *mut i64 = *par.add(10) as *mut i64;
            lvAttributes = *lvAttributesLPtr;
        }
        
        let lvReparseTag : i64;
        unsafe
        {
            let lvReparseTagLPtr : *mut i64 = *par.add(11) as *mut i64;
            lvReparseTag = *lvReparseTagLPtr;
        }
        
        let lvHardLinkCount : i32;
        unsafe
        {
            lvHardLinkCount = *par.add(12) as i32;
        }
        
        let lvhShortFileNamePtr : *mut c_char;
        let lvhShortFileNameLen : i32;
        let lvShortFileName : String;
        unsafe
        {
            lvhShortFileNamePtr = *par.add(13) as *mut c_char;
            lvhShortFileNameLen = *_cbpar.add(13);
            if lvhShortFileNamePtr == std::ptr::null_mut()
            {
                lvShortFileName = String::default();
            }
            else
            {
                lvShortFileName = CStr::from_ptr(lvhShortFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'ShortFileName' in the GetFileInfo event of a CBFS instance").to_owned();
            }
        }

        let lvhRealFileNamePtr : *mut c_char;
        let lvhRealFileNameLen : i32;
        let lvRealFileName : String;
        unsafe
        {
            lvhRealFileNamePtr = *par.add(14) as *mut c_char;
            lvhRealFileNameLen = *_cbpar.add(14);
            if lvhRealFileNamePtr == std::ptr::null_mut()
            {
                lvRealFileName = String::default();
            }
            else
            {
                lvRealFileName = CStr::from_ptr(lvhRealFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'RealFileName' in the GetFileInfo event of a CBFS instance").to_owned();
            }
        }

        let lvEaSize : i32;
        unsafe
        {
            lvEaSize = *par.add(15) as i32;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(16) as i32;
        }
        
        CBFSGetFileInfoEventArgs
        {
            FileName: lvFileName,
            RequestedInfo: lvRequestedInfo,
            FileExists: lvFileExists,
            CreationTime: lvCreationTime,
            LastAccessTime: lvLastAccessTime,
            LastWriteTime: lvLastWriteTime,
            ChangeTime: lvChangeTime,
            Size: lvSize,
            AllocationSize: lvAllocationSize,
            FileId: lvFileId,
            Attributes: lvAttributes,
            ReparseTag: lvReparseTag,
            HardLinkCount: lvHardLinkCount,
            hShortFileNamePtr: lvhShortFileNamePtr,
            hShortFileNameLen: lvhShortFileNameLen,
            ShortFileName: lvShortFileName,
            hRealFileNamePtr: lvhRealFileNamePtr,
            hRealFileNameLen: lvhRealFileNameLen,
            RealFileName: lvRealFileName,
            EaSize: lvEaSize,
            ResultCode: lvResultCode,
            _Params: par,
            _Cbparam: _cbpar
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfFileExists : i32;
            if self.FileExists
            {
                intValOfFileExists = 1;
            }
            else
            {
                intValOfFileExists = 0;
            }
            *(self._Params.add(2)) = intValOfFileExists as isize;
            let intValOfCreationTime : i64 = chrono_time_to_file_time(&self.CreationTime);
            let lvCreationTimeLPtr : *mut i64 = *self._Params.add(3) as *mut i64;
            *lvCreationTimeLPtr = intValOfCreationTime as i64;
            let intValOfLastAccessTime : i64 = chrono_time_to_file_time(&self.LastAccessTime);
            let lvLastAccessTimeLPtr : *mut i64 = *self._Params.add(4) as *mut i64;
            *lvLastAccessTimeLPtr = intValOfLastAccessTime as i64;
            let intValOfLastWriteTime : i64 = chrono_time_to_file_time(&self.LastWriteTime);
            let lvLastWriteTimeLPtr : *mut i64 = *self._Params.add(5) as *mut i64;
            *lvLastWriteTimeLPtr = intValOfLastWriteTime as i64;
            let intValOfChangeTime : i64 = chrono_time_to_file_time(&self.ChangeTime);
            let lvChangeTimeLPtr : *mut i64 = *self._Params.add(6) as *mut i64;
            *lvChangeTimeLPtr = intValOfChangeTime as i64;
            let lvSizeLPtr : *mut i64 = *self._Params.add(7) as *mut i64;
            *lvSizeLPtr = self.Size ;
            let lvAllocationSizeLPtr : *mut i64 = *self._Params.add(8) as *mut i64;
            *lvAllocationSizeLPtr = self.AllocationSize ;
            let lvFileIdLPtr : *mut i64 = *self._Params.add(9) as *mut i64;
            *lvFileIdLPtr = self.FileId ;
            let lvAttributesLPtr : *mut i64 = *self._Params.add(10) as *mut i64;
            *lvAttributesLPtr = self.Attributes ;
            let lvReparseTagLPtr : *mut i64 = *self._Params.add(11) as *mut i64;
            *lvReparseTagLPtr = self.ReparseTag ;
            *(self._Params.add(12)) = self.HardLinkCount as isize;
            let bytesShortFileName = self.ShortFileName.as_bytes();
            let to_copy : usize;
            let bytesShortFileNameLen = bytesShortFileName.len();
            if bytesShortFileNameLen + 1 < self.hShortFileNameLen as usize
            {
                to_copy = bytesShortFileNameLen;
            }
            else
            {
                to_copy = self.hShortFileNameLen as usize - 1;
            }
            if to_copy > 0
            {
                std::ptr::copy_nonoverlapping(bytesShortFileName.as_ptr(), self.hShortFileNamePtr as *mut u8, to_copy);
            }
            *(self.hShortFileNamePtr.add(to_copy)) = 0;
            *(self._Cbparam.add(13)) = to_copy as i32;
            let bytesRealFileName = self.RealFileName.as_bytes();
            let to_copy : usize;
            let bytesRealFileNameLen = bytesRealFileName.len();
            if bytesRealFileNameLen + 1 < self.hRealFileNameLen as usize
            {
                to_copy = bytesRealFileNameLen;
            }
            else
            {
                to_copy = self.hRealFileNameLen as usize - 1;
            }
            if to_copy > 0
            {
                std::ptr::copy_nonoverlapping(bytesRealFileName.as_ptr(), self.hRealFileNamePtr as *mut u8, to_copy);
            }
            *(self.hRealFileNamePtr.add(to_copy)) = 0;
            *(self._Cbparam.add(14)) = to_copy as i32;
            *(self._Params.add(15)) = self.EaSize as isize;
            *(self._Params.add(16)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn requested_info(&self) -> i32
    {
        self.RequestedInfo
    }
    pub fn file_exists(&self) -> bool
    {
        self.FileExists
    }
    pub fn set_file_exists(&mut self, value: bool)
    {
        self.FileExists = value;
    }
    pub fn creation_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.CreationTime
    }
    pub fn set_creation_time_ref(&mut self, value: &chrono::DateTime<Utc>)
    {
        self.CreationTime = value.clone();
    }
    pub fn set_creation_time(&mut self, value: chrono::DateTime<Utc>)
    {
        self.CreationTime = value;
    }
    pub fn last_access_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.LastAccessTime
    }
    pub fn set_last_access_time_ref(&mut self, value: &chrono::DateTime<Utc>)
    {
        self.LastAccessTime = value.clone();
    }
    pub fn set_last_access_time(&mut self, value: chrono::DateTime<Utc>)
    {
        self.LastAccessTime = value;
    }
    pub fn last_write_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.LastWriteTime
    }
    pub fn set_last_write_time_ref(&mut self, value: &chrono::DateTime<Utc>)
    {
        self.LastWriteTime = value.clone();
    }
    pub fn set_last_write_time(&mut self, value: chrono::DateTime<Utc>)
    {
        self.LastWriteTime = value;
    }
    pub fn change_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.ChangeTime
    }
    pub fn set_change_time_ref(&mut self, value: &chrono::DateTime<Utc>)
    {
        self.ChangeTime = value.clone();
    }
    pub fn set_change_time(&mut self, value: chrono::DateTime<Utc>)
    {
        self.ChangeTime = value;
    }
    pub fn size(&self) -> i64
    {
        self.Size
    }
    pub fn set_size(&mut self, value: i64)
    {
        self.Size = value;
    }
    pub fn allocation_size(&self) -> i64
    {
        self.AllocationSize
    }
    pub fn set_allocation_size(&mut self, value: i64)
    {
        self.AllocationSize = value;
    }
    pub fn file_id(&self) -> i64
    {
        self.FileId
    }
    pub fn set_file_id(&mut self, value: i64)
    {
        self.FileId = value;
    }
    pub fn attributes(&self) -> i64
    {
        self.Attributes
    }
    pub fn set_attributes(&mut self, value: i64)
    {
        self.Attributes = value;
    }
    pub fn reparse_tag(&self) -> i64
    {
        self.ReparseTag
    }
    pub fn set_reparse_tag(&mut self, value: i64)
    {
        self.ReparseTag = value;
    }
    pub fn hard_link_count(&self) -> i32
    {
        self.HardLinkCount
    }
    pub fn set_hard_link_count(&mut self, value: i32)
    {
        self.HardLinkCount = value;
    }
    pub fn short_file_name(&self) -> &String
    {
        &self.ShortFileName
    }
    pub fn set_short_file_name_ref(&mut self, value: &String)
    {
        self.ShortFileName = value.clone();
    }
    pub fn set_short_file_name(&mut self, value: String)
    {
        self.ShortFileName = value;
    }
    pub fn real_file_name(&self) -> &String
    {
        &self.RealFileName
    }
    pub fn set_real_file_name_ref(&mut self, value: &String)
    {
        self.RealFileName = value.clone();
    }
    pub fn set_real_file_name(&mut self, value: String)
    {
        self.RealFileName = value;
    }
    pub fn ea_size(&self) -> i32
    {
        self.EaSize
    }
    pub fn set_ea_size(&mut self, value: i32)
    {
        self.EaSize = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSGetFileInfoEvent
{
    fn on_get_file_info(&self, sender : &CBFS, e : &mut CBFSGetFileInfoEventArgs);
}


// CBFSGetFileNameByFileIdEventArgs carries the parameters of the GetFileNameByFileId event of CBFS
pub struct CBFSGetFileNameByFileIdEventArgs
{
    FileId : i64,
    hFilePathPtr : *mut c_char,
    hFilePathLen : i32,
    FilePath : String,
    ResultCode : i32,

    _Params  : IntPtrArrayType,
    _Cbparam : IntArrayType
}

// Constructor and marshalRefParams() of GetFileNameByFileIdEventArgs
impl CBFSGetFileNameByFileIdEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSGetFileNameByFileIdEventArgs
    {

        let lvFileId : i64;
        unsafe
        {
            let lvFileIdLPtr : *mut i64 = *par.add(0) as *mut i64;
            lvFileId = *lvFileIdLPtr;
        }
        
        let lvhFilePathPtr : *mut c_char;
        let lvhFilePathLen : i32;
        let lvFilePath : String;
        unsafe
        {
            lvhFilePathPtr = *par.add(1) as *mut c_char;
            lvhFilePathLen = *_cbpar.add(1);
            if lvhFilePathPtr == std::ptr::null_mut()
            {
                lvFilePath = String::default();
            }
            else
            {
                lvFilePath = CStr::from_ptr(lvhFilePathPtr).to_str().expect("Valid UTF8 not received for the parameter 'FilePath' in the GetFileNameByFileId event of a CBFS instance").to_owned();
            }
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(2) as i32;
        }
        
        CBFSGetFileNameByFileIdEventArgs
        {
            FileId: lvFileId,
            hFilePathPtr: lvhFilePathPtr,
            hFilePathLen: lvhFilePathLen,
            FilePath: lvFilePath,
            ResultCode: lvResultCode,
            _Params: par,
            _Cbparam: _cbpar
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let bytesFilePath = self.FilePath.as_bytes();
            let to_copy : usize;
            let bytesFilePathLen = bytesFilePath.len();
            if bytesFilePathLen + 1 < self.hFilePathLen as usize
            {
                to_copy = bytesFilePathLen;
            }
            else
            {
                to_copy = self.hFilePathLen as usize - 1;
            }
            if to_copy > 0
            {
                std::ptr::copy_nonoverlapping(bytesFilePath.as_ptr(), self.hFilePathPtr as *mut u8, to_copy);
            }
            *(self.hFilePathPtr.add(to_copy)) = 0;
            *(self._Cbparam.add(1)) = to_copy as i32;
            *(self._Params.add(2)) = self.ResultCode as isize;
        }
    }

    pub fn file_id(&self) -> i64
    {
        self.FileId
    }
    pub fn file_path(&self) -> &String
    {
        &self.FilePath
    }
    pub fn set_file_path_ref(&mut self, value: &String)
    {
        self.FilePath = value.clone();
    }
    pub fn set_file_path(&mut self, value: String)
    {
        self.FilePath = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSGetFileNameByFileIdEvent
{
    fn on_get_file_name_by_file_id(&self, sender : &CBFS, e : &mut CBFSGetFileNameByFileIdEventArgs);
}


// CBFSGetFileSecurityEventArgs carries the parameters of the GetFileSecurity event of CBFS
pub struct CBFSGetFileSecurityEventArgs
{
    FileName : String,
    SecurityInformation : i32,
    hSecurityDescriptorPtr : *mut u8,
    BufferLength : i32,
    DescriptorLength : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of GetFileSecurityEventArgs
impl CBFSGetFileSecurityEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSGetFileSecurityEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the GetFileSecurity event of a CBFS instance").to_owned();
            }
        }

        let lvSecurityInformation : i32;
        unsafe
        {
            lvSecurityInformation = *par.add(1) as i32;
        }
        
        let lvhSecurityDescriptorPtr : *mut u8;
        unsafe
        {
            lvhSecurityDescriptorPtr = *par.add(2) as *mut u8;
        }

        let lvBufferLength : i32;
        unsafe
        {
            lvBufferLength = *par.add(3) as i32;
        }
        // lvhSecurityDescriptorLen = lvBufferLength;

        let lvDescriptorLength : i32;
        unsafe
        {
            lvDescriptorLength = *par.add(4) as i32;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(6) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(7) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(8) as i32;
        }
        
        CBFSGetFileSecurityEventArgs
        {
            FileName: lvFileName,
            SecurityInformation: lvSecurityInformation,
            hSecurityDescriptorPtr: lvhSecurityDescriptorPtr,
            BufferLength: lvBufferLength,
            DescriptorLength: lvDescriptorLength,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(4)) = self.DescriptorLength as isize;
            *(self._Params.add(6)) = self.FileContext as isize;
            *(self._Params.add(7)) = self.HandleContext as isize;
            *(self._Params.add(8)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn security_information(&self) -> i32
    {
        self.SecurityInformation
    }
    pub fn security_descriptor(&self) -> *mut u8
    {
        self.hSecurityDescriptorPtr
    }
    pub fn buffer_length(&self) -> i32
    {
        self.BufferLength
    }
    pub fn descriptor_length(&self) -> i32
    {
        self.DescriptorLength
    }
    pub fn set_descriptor_length(&mut self, value: i32)
    {
        self.DescriptorLength = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSGetFileSecurityEvent
{
    fn on_get_file_security(&self, sender : &CBFS, e : &mut CBFSGetFileSecurityEventArgs);
}


// CBFSGetObjectIdEventArgs carries the parameters of the GetObjectId event of CBFS
pub struct CBFSGetObjectIdEventArgs
{
    FileName : String,
    ShouldCreate : bool,
    hObjectIdPtr : *mut u8,
    ObjectIdLength : i32,
    hExtendedInformationPtr : *mut u8,
    ExtendedInformationLength : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of GetObjectIdEventArgs
impl CBFSGetObjectIdEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSGetObjectIdEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the GetObjectId event of a CBFS instance").to_owned();
            }
        }

        let lvShouldCreate : bool;
        unsafe
        {
            lvShouldCreate = (*par.add(1) as i32) != 0;
        }

        let lvhObjectIdPtr : *mut u8;
        unsafe
        {
            lvhObjectIdPtr = *par.add(2) as *mut u8;
        }

        let lvObjectIdLength : i32;
        unsafe
        {
            lvObjectIdLength = *par.add(3) as i32;
        }
        // lvhObjectIdLen = lvObjectIdLength;

        let lvhExtendedInformationPtr : *mut u8;
        unsafe
        {
            lvhExtendedInformationPtr = *par.add(4) as *mut u8;
        }

        let lvExtendedInformationLength : i32;
        unsafe
        {
            lvExtendedInformationLength = *par.add(5) as i32;
        }
        // lvhExtendedInformationLen = lvExtendedInformationLength;

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(6) as i32;
        }
        
        CBFSGetObjectIdEventArgs
        {
            FileName: lvFileName,
            ShouldCreate: lvShouldCreate,
            hObjectIdPtr: lvhObjectIdPtr,
            ObjectIdLength: lvObjectIdLength,
            hExtendedInformationPtr: lvhExtendedInformationPtr,
            ExtendedInformationLength: lvExtendedInformationLength,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(6)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn should_create(&self) -> bool
    {
        self.ShouldCreate
    }
    pub fn object_id(&self) -> *mut u8
    {
        self.hObjectIdPtr
    }
    pub fn object_id_length(&self) -> i32
    {
        self.ObjectIdLength
    }
    pub fn extended_information(&self) -> *mut u8
    {
        self.hExtendedInformationPtr
    }
    pub fn extended_information_length(&self) -> i32
    {
        self.ExtendedInformationLength
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSGetObjectIdEvent
{
    fn on_get_object_id(&self, sender : &CBFS, e : &mut CBFSGetObjectIdEventArgs);
}


// CBFSGetReparsePointEventArgs carries the parameters of the GetReparsePoint event of CBFS
pub struct CBFSGetReparsePointEventArgs
{
    FileName : String,
    hReparseBufferPtr : *mut u8,
    ReparseBufferLength : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of GetReparsePointEventArgs
impl CBFSGetReparsePointEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSGetReparsePointEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the GetReparsePoint event of a CBFS instance").to_owned();
            }
        }

        let lvhReparseBufferPtr : *mut u8;
        unsafe
        {
            lvhReparseBufferPtr = *par.add(1) as *mut u8;
        }

        let lvReparseBufferLength : i32;
        unsafe
        {
            lvReparseBufferLength = *par.add(2) as i32;
        }
        // lvhReparseBufferLen = lvReparseBufferLength;

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(4) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(5) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(6) as i32;
        }
        
        CBFSGetReparsePointEventArgs
        {
            FileName: lvFileName,
            hReparseBufferPtr: lvhReparseBufferPtr,
            ReparseBufferLength: lvReparseBufferLength,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.ReparseBufferLength as isize;
            *(self._Params.add(4)) = self.FileContext as isize;
            *(self._Params.add(5)) = self.HandleContext as isize;
            *(self._Params.add(6)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn reparse_buffer(&self) -> *mut u8
    {
        self.hReparseBufferPtr
    }
    pub fn reparse_buffer_length(&self) -> i32
    {
        self.ReparseBufferLength
    }
    pub fn set_reparse_buffer_length(&mut self, value: i32)
    {
        self.ReparseBufferLength = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSGetReparsePointEvent
{
    fn on_get_reparse_point(&self, sender : &CBFS, e : &mut CBFSGetReparsePointEventArgs);
}


// CBFSGetVolumeIdEventArgs carries the parameters of the GetVolumeId event of CBFS
pub struct CBFSGetVolumeIdEventArgs
{
    VolumeId : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of GetVolumeIdEventArgs
impl CBFSGetVolumeIdEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSGetVolumeIdEventArgs
    {

        let lvVolumeId : i32;
        unsafe
        {
            lvVolumeId = *par.add(0) as i32;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(1) as i32;
        }
        
        CBFSGetVolumeIdEventArgs
        {
            VolumeId: lvVolumeId,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(0)) = self.VolumeId as isize;
            *(self._Params.add(1)) = self.ResultCode as isize;
        }
    }

    pub fn volume_id(&self) -> i32
    {
        self.VolumeId
    }
    pub fn set_volume_id(&mut self, value: i32)
    {
        self.VolumeId = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSGetVolumeIdEvent
{
    fn on_get_volume_id(&self, sender : &CBFS, e : &mut CBFSGetVolumeIdEventArgs);
}


// CBFSGetVolumeLabelEventArgs carries the parameters of the GetVolumeLabel event of CBFS
pub struct CBFSGetVolumeLabelEventArgs
{
    hBufferPtr : *mut c_char,
    hBufferLen : i32,
    Buffer : String,
    ResultCode : i32,

    _Params  : IntPtrArrayType,
    _Cbparam : IntArrayType
}

// Constructor and marshalRefParams() of GetVolumeLabelEventArgs
impl CBFSGetVolumeLabelEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSGetVolumeLabelEventArgs
    {

        let lvhBufferPtr : *mut c_char;
        let lvhBufferLen : i32;
        let lvBuffer : String;
        unsafe
        {
            lvhBufferPtr = *par.add(0) as *mut c_char;
            lvhBufferLen = *_cbpar.add(0);
            if lvhBufferPtr == std::ptr::null_mut()
            {
                lvBuffer = String::default();
            }
            else
            {
                lvBuffer = CStr::from_ptr(lvhBufferPtr).to_str().expect("Valid UTF8 not received for the parameter 'Buffer' in the GetVolumeLabel event of a CBFS instance").to_owned();
            }
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(1) as i32;
        }
        
        CBFSGetVolumeLabelEventArgs
        {
            hBufferPtr: lvhBufferPtr,
            hBufferLen: lvhBufferLen,
            Buffer: lvBuffer,
            ResultCode: lvResultCode,
            _Params: par,
            _Cbparam: _cbpar
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let bytesBuffer = self.Buffer.as_bytes();
            let to_copy : usize;
            let bytesBufferLen = bytesBuffer.len();
            if bytesBufferLen + 1 < self.hBufferLen as usize
            {
                to_copy = bytesBufferLen;
            }
            else
            {
                to_copy = self.hBufferLen as usize - 1;
            }
            if to_copy > 0
            {
                std::ptr::copy_nonoverlapping(bytesBuffer.as_ptr(), self.hBufferPtr as *mut u8, to_copy);
            }
            *(self.hBufferPtr.add(to_copy)) = 0;
            *(self._Cbparam.add(0)) = to_copy as i32;
            *(self._Params.add(1)) = self.ResultCode as isize;
        }
    }

    pub fn buffer(&self) -> &String
    {
        &self.Buffer
    }
    pub fn set_buffer_ref(&mut self, value: &String)
    {
        self.Buffer = value.clone();
    }
    pub fn set_buffer(&mut self, value: String)
    {
        self.Buffer = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSGetVolumeLabelEvent
{
    fn on_get_volume_label(&self, sender : &CBFS, e : &mut CBFSGetVolumeLabelEventArgs);
}


// CBFSGetVolumeSizeEventArgs carries the parameters of the GetVolumeSize event of CBFS
pub struct CBFSGetVolumeSizeEventArgs
{
    TotalSectors : i64,
    AvailableSectors : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of GetVolumeSizeEventArgs
impl CBFSGetVolumeSizeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSGetVolumeSizeEventArgs
    {

        let lvTotalSectors : i64;
        unsafe
        {
            let lvTotalSectorsLPtr : *mut i64 = *par.add(0) as *mut i64;
            lvTotalSectors = *lvTotalSectorsLPtr;
        }
        
        let lvAvailableSectors : i64;
        unsafe
        {
            let lvAvailableSectorsLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvAvailableSectors = *lvAvailableSectorsLPtr;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(2) as i32;
        }
        
        CBFSGetVolumeSizeEventArgs
        {
            TotalSectors: lvTotalSectors,
            AvailableSectors: lvAvailableSectors,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvTotalSectorsLPtr : *mut i64 = *self._Params.add(0) as *mut i64;
            *lvTotalSectorsLPtr = self.TotalSectors ;
            let lvAvailableSectorsLPtr : *mut i64 = *self._Params.add(1) as *mut i64;
            *lvAvailableSectorsLPtr = self.AvailableSectors ;
            *(self._Params.add(2)) = self.ResultCode as isize;
        }
    }

    pub fn total_sectors(&self) -> i64
    {
        self.TotalSectors
    }
    pub fn set_total_sectors(&mut self, value: i64)
    {
        self.TotalSectors = value;
    }
    pub fn available_sectors(&self) -> i64
    {
        self.AvailableSectors
    }
    pub fn set_available_sectors(&mut self, value: i64)
    {
        self.AvailableSectors = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSGetVolumeSizeEvent
{
    fn on_get_volume_size(&self, sender : &CBFS, e : &mut CBFSGetVolumeSizeEventArgs);
}


// CBFSIoctlEventArgs carries the parameters of the Ioctl event of CBFS
pub struct CBFSIoctlEventArgs
{
    FileName : String,
    Code : i32,
    hInputBufferPtr : *mut u8,
    InputBufferLength : i32,
    hOutputBufferPtr : *mut u8,
    OutputBufferLength : i32,
    BytesReturned : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of IoctlEventArgs
impl CBFSIoctlEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSIoctlEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the Ioctl event of a CBFS instance").to_owned();
            }
        }

        let lvCode : i32;
        unsafe
        {
            lvCode = *par.add(1) as i32;
        }
        
        let lvhInputBufferPtr : *mut u8;
        unsafe
        {
            lvhInputBufferPtr = *par.add(2) as *mut u8;
        }

        let lvInputBufferLength : i32;
        unsafe
        {
            lvInputBufferLength = *par.add(3) as i32;
        }
        // lvhInputBufferLen = lvInputBufferLength;

        let lvhOutputBufferPtr : *mut u8;
        unsafe
        {
            lvhOutputBufferPtr = *par.add(4) as *mut u8;
        }

        let lvOutputBufferLength : i32;
        unsafe
        {
            lvOutputBufferLength = *par.add(5) as i32;
        }
        // lvhOutputBufferLen = lvOutputBufferLength;

        let lvBytesReturned : i32;
        unsafe
        {
            lvBytesReturned = *par.add(6) as i32;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(7) as i32;
        }
        
        CBFSIoctlEventArgs
        {
            FileName: lvFileName,
            Code: lvCode,
            hInputBufferPtr: lvhInputBufferPtr,
            InputBufferLength: lvInputBufferLength,
            hOutputBufferPtr: lvhOutputBufferPtr,
            OutputBufferLength: lvOutputBufferLength,
            BytesReturned: lvBytesReturned,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(6)) = self.BytesReturned as isize;
            *(self._Params.add(7)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn code(&self) -> i32
    {
        self.Code
    }
    pub fn input_buffer(&self) -> *mut u8
    {
        self.hInputBufferPtr
    }
    pub fn input_buffer_length(&self) -> i32
    {
        self.InputBufferLength
    }
    pub fn output_buffer(&self) -> *mut u8
    {
        self.hOutputBufferPtr
    }
    pub fn output_buffer_length(&self) -> i32
    {
        self.OutputBufferLength
    }
    pub fn bytes_returned(&self) -> i32
    {
        self.BytesReturned
    }
    pub fn set_bytes_returned(&mut self, value: i32)
    {
        self.BytesReturned = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSIoctlEvent
{
    fn on_ioctl(&self, sender : &CBFS, e : &mut CBFSIoctlEventArgs);
}


// CBFSIsDirectoryEmptyEventArgs carries the parameters of the IsDirectoryEmpty event of CBFS
pub struct CBFSIsDirectoryEmptyEventArgs
{
    DirectoryName : String,
    IsEmpty : bool,
    DirectoryContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of IsDirectoryEmptyEventArgs
impl CBFSIsDirectoryEmptyEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSIsDirectoryEmptyEventArgs
    {

        let lvhDirectoryNamePtr : *mut c_char;
        let lvDirectoryName : String;
        unsafe
        {
            lvhDirectoryNamePtr = *par.add(0) as *mut c_char;
            if lvhDirectoryNamePtr == std::ptr::null_mut()
            {
                lvDirectoryName = String::default();
            }
            else
            {
                lvDirectoryName = CStr::from_ptr(lvhDirectoryNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'DirectoryName' in the IsDirectoryEmpty event of a CBFS instance").to_owned();
            }
        }

        let lvIsEmpty : bool;
        unsafe
        {
            lvIsEmpty = (*par.add(1) as i32) != 0;
        }

        let lvDirectoryContext : usize;
        unsafe
        {
            lvDirectoryContext = *par.add(2) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(3) as i32;
        }
        
        CBFSIsDirectoryEmptyEventArgs
        {
            DirectoryName: lvDirectoryName,
            IsEmpty: lvIsEmpty,
            DirectoryContext: lvDirectoryContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfIsEmpty : i32;
            if self.IsEmpty
            {
                intValOfIsEmpty = 1;
            }
            else
            {
                intValOfIsEmpty = 0;
            }
            *(self._Params.add(1)) = intValOfIsEmpty as isize;
            *(self._Params.add(2)) = self.DirectoryContext as isize;
            *(self._Params.add(3)) = self.ResultCode as isize;
        }
    }

    pub fn directory_name(&self) -> &String
    {
        &self.DirectoryName
    }
    pub fn is_empty(&self) -> bool
    {
        self.IsEmpty
    }
    pub fn set_is_empty(&mut self, value: bool)
    {
        self.IsEmpty = value;
    }
    pub fn directory_context(&self) -> usize
    {
        self.DirectoryContext
    }
    pub fn set_directory_context(&mut self, value: usize)
    {
        self.DirectoryContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSIsDirectoryEmptyEvent
{
    fn on_is_directory_empty(&self, sender : &CBFS, e : &mut CBFSIsDirectoryEmptyEventArgs);
}


// CBFSLockFileEventArgs carries the parameters of the LockFile event of CBFS
pub struct CBFSLockFileEventArgs
{
    FileName : String,
    ByteOffset : i64,
    Length : i64,
    Key : i32,
    ExclusiveLock : bool,
    FailImmediately : bool,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of LockFileEventArgs
impl CBFSLockFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSLockFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the LockFile event of a CBFS instance").to_owned();
            }
        }

        let lvByteOffset : i64;
        unsafe
        {
            let lvByteOffsetLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvByteOffset = *lvByteOffsetLPtr;
        }
        
        let lvLength : i64;
        unsafe
        {
            let lvLengthLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvLength = *lvLengthLPtr;
        }
        
        let lvKey : i32;
        unsafe
        {
            lvKey = *par.add(3) as i32;
        }
        
        let lvExclusiveLock : bool;
        unsafe
        {
            lvExclusiveLock = (*par.add(4) as i32) != 0;
        }

        let lvFailImmediately : bool;
        unsafe
        {
            lvFailImmediately = (*par.add(5) as i32) != 0;
        }

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(6) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(7) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(8) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(9) as i32;
        }
        
        CBFSLockFileEventArgs
        {
            FileName: lvFileName,
            ByteOffset: lvByteOffset,
            Length: lvLength,
            Key: lvKey,
            ExclusiveLock: lvExclusiveLock,
            FailImmediately: lvFailImmediately,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(7)) = self.FileContext as isize;
            *(self._Params.add(8)) = self.HandleContext as isize;
            *(self._Params.add(9)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn byte_offset(&self) -> i64
    {
        self.ByteOffset
    }
    pub fn length(&self) -> i64
    {
        self.Length
    }
    pub fn key(&self) -> i32
    {
        self.Key
    }
    pub fn exclusive_lock(&self) -> bool
    {
        self.ExclusiveLock
    }
    pub fn fail_immediately(&self) -> bool
    {
        self.FailImmediately
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSLockFileEvent
{
    fn on_lock_file(&self, sender : &CBFS, e : &mut CBFSLockFileEventArgs);
}


// CBFSMountEventArgs carries the parameters of the Mount event of CBFS
pub struct CBFSMountEventArgs
{
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of MountEventArgs
impl CBFSMountEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSMountEventArgs
    {

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(0) as i32;
        }
        
        CBFSMountEventArgs
        {
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(0)) = self.ResultCode as isize;
        }
    }

    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSMountEvent
{
    fn on_mount(&self, sender : &CBFS, e : &mut CBFSMountEventArgs);
}


// CBFSOffloadReadFileEventArgs carries the parameters of the OffloadReadFile event of CBFS
pub struct CBFSOffloadReadFileEventArgs
{
    FileName : String,
    TokenTimeToLive : i32,
    Position : i64,
    CopyLength : i64,
    TransferLength : i64,
    TokenType : i32,
    hTokenBufferPtr : *mut u8,
    BufferLength : i64,
    TokenLength : i32,
    ResultFlags : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of OffloadReadFileEventArgs
impl CBFSOffloadReadFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSOffloadReadFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the OffloadReadFile event of a CBFS instance").to_owned();
            }
        }

        let lvTokenTimeToLive : i32;
        unsafe
        {
            lvTokenTimeToLive = *par.add(1) as i32;
        }
        
        let lvPosition : i64;
        unsafe
        {
            let lvPositionLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvPosition = *lvPositionLPtr;
        }
        
        let lvCopyLength : i64;
        unsafe
        {
            let lvCopyLengthLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvCopyLength = *lvCopyLengthLPtr;
        }
        
        let lvTransferLength : i64;
        unsafe
        {
            let lvTransferLengthLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvTransferLength = *lvTransferLengthLPtr;
        }
        
        let lvTokenType : i32;
        unsafe
        {
            lvTokenType = *par.add(5) as i32;
        }
        
        let lvhTokenBufferPtr : *mut u8;
        unsafe
        {
            lvhTokenBufferPtr = *par.add(6) as *mut u8;
        }

        let lvBufferLength : i64;
        unsafe
        {
            let lvBufferLengthLPtr : *mut i64 = *par.add(7) as *mut i64;
            lvBufferLength = *lvBufferLengthLPtr;
        }
        // lvhTokenBufferLen = lvBufferLength;

        let lvTokenLength : i32;
        unsafe
        {
            lvTokenLength = *par.add(8) as i32;
        }
        
        let lvResultFlags : i32;
        unsafe
        {
            lvResultFlags = *par.add(9) as i32;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(10) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(11) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(12) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(13) as i32;
        }
        
        CBFSOffloadReadFileEventArgs
        {
            FileName: lvFileName,
            TokenTimeToLive: lvTokenTimeToLive,
            Position: lvPosition,
            CopyLength: lvCopyLength,
            TransferLength: lvTransferLength,
            TokenType: lvTokenType,
            hTokenBufferPtr: lvhTokenBufferPtr,
            BufferLength: lvBufferLength,
            TokenLength: lvTokenLength,
            ResultFlags: lvResultFlags,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvTransferLengthLPtr : *mut i64 = *self._Params.add(4) as *mut i64;
            *lvTransferLengthLPtr = self.TransferLength ;
            *(self._Params.add(5)) = self.TokenType as isize;
            *(self._Params.add(8)) = self.TokenLength as isize;
            *(self._Params.add(9)) = self.ResultFlags as isize;
            *(self._Params.add(11)) = self.FileContext as isize;
            *(self._Params.add(12)) = self.HandleContext as isize;
            *(self._Params.add(13)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn token_time_to_live(&self) -> i32
    {
        self.TokenTimeToLive
    }
    pub fn position(&self) -> i64
    {
        self.Position
    }
    pub fn copy_length(&self) -> i64
    {
        self.CopyLength
    }
    pub fn transfer_length(&self) -> i64
    {
        self.TransferLength
    }
    pub fn set_transfer_length(&mut self, value: i64)
    {
        self.TransferLength = value;
    }
    pub fn token_type(&self) -> i32
    {
        self.TokenType
    }
    pub fn set_token_type(&mut self, value: i32)
    {
        self.TokenType = value;
    }
    pub fn token_buffer(&self) -> *mut u8
    {
        self.hTokenBufferPtr
    }
    pub fn buffer_length(&self) -> i64
    {
        self.BufferLength
    }
    pub fn token_length(&self) -> i32
    {
        self.TokenLength
    }
    pub fn set_token_length(&mut self, value: i32)
    {
        self.TokenLength = value;
    }
    pub fn result_flags(&self) -> i32
    {
        self.ResultFlags
    }
    pub fn set_result_flags(&mut self, value: i32)
    {
        self.ResultFlags = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSOffloadReadFileEvent
{
    fn on_offload_read_file(&self, sender : &CBFS, e : &mut CBFSOffloadReadFileEventArgs);
}


// CBFSOffloadWriteFileEventArgs carries the parameters of the OffloadWriteFile event of CBFS
pub struct CBFSOffloadWriteFileEventArgs
{
    FileName : String,
    Position : i64,
    CopyLength : i64,
    TransferOffset : i64,
    TransferLength : i64,
    TokenType : i32,
    hTokenPtr : *mut u8,
    TokenLength : i64,
    ResultFlags : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of OffloadWriteFileEventArgs
impl CBFSOffloadWriteFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSOffloadWriteFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the OffloadWriteFile event of a CBFS instance").to_owned();
            }
        }

        let lvPosition : i64;
        unsafe
        {
            let lvPositionLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvPosition = *lvPositionLPtr;
        }
        
        let lvCopyLength : i64;
        unsafe
        {
            let lvCopyLengthLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvCopyLength = *lvCopyLengthLPtr;
        }
        
        let lvTransferOffset : i64;
        unsafe
        {
            let lvTransferOffsetLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvTransferOffset = *lvTransferOffsetLPtr;
        }
        
        let lvTransferLength : i64;
        unsafe
        {
            let lvTransferLengthLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvTransferLength = *lvTransferLengthLPtr;
        }
        
        let lvTokenType : i32;
        unsafe
        {
            lvTokenType = *par.add(5) as i32;
        }
        
        let lvhTokenPtr : *mut u8;
        unsafe
        {
            lvhTokenPtr = *par.add(6) as *mut u8;
        }

        let lvTokenLength : i64;
        unsafe
        {
            let lvTokenLengthLPtr : *mut i64 = *par.add(7) as *mut i64;
            lvTokenLength = *lvTokenLengthLPtr;
        }
        // lvhTokenLen = lvTokenLength;

        let lvResultFlags : i32;
        unsafe
        {
            lvResultFlags = *par.add(8) as i32;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(9) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(10) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(11) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(12) as i32;
        }
        
        CBFSOffloadWriteFileEventArgs
        {
            FileName: lvFileName,
            Position: lvPosition,
            CopyLength: lvCopyLength,
            TransferOffset: lvTransferOffset,
            TransferLength: lvTransferLength,
            TokenType: lvTokenType,
            hTokenPtr: lvhTokenPtr,
            TokenLength: lvTokenLength,
            ResultFlags: lvResultFlags,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvTransferLengthLPtr : *mut i64 = *self._Params.add(4) as *mut i64;
            *lvTransferLengthLPtr = self.TransferLength ;
            *(self._Params.add(8)) = self.ResultFlags as isize;
            *(self._Params.add(10)) = self.FileContext as isize;
            *(self._Params.add(11)) = self.HandleContext as isize;
            *(self._Params.add(12)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn position(&self) -> i64
    {
        self.Position
    }
    pub fn copy_length(&self) -> i64
    {
        self.CopyLength
    }
    pub fn transfer_offset(&self) -> i64
    {
        self.TransferOffset
    }
    pub fn transfer_length(&self) -> i64
    {
        self.TransferLength
    }
    pub fn set_transfer_length(&mut self, value: i64)
    {
        self.TransferLength = value;
    }
    pub fn token_type(&self) -> i32
    {
        self.TokenType
    }
    pub fn token(&self) -> *mut u8
    {
        self.hTokenPtr
    }
    pub fn token_length(&self) -> i64
    {
        self.TokenLength
    }
    pub fn result_flags(&self) -> i32
    {
        self.ResultFlags
    }
    pub fn set_result_flags(&mut self, value: i32)
    {
        self.ResultFlags = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSOffloadWriteFileEvent
{
    fn on_offload_write_file(&self, sender : &CBFS, e : &mut CBFSOffloadWriteFileEventArgs);
}


// CBFSOpenFileEventArgs carries the parameters of the OpenFile event of CBFS
pub struct CBFSOpenFileEventArgs
{
    FileName : String,
    DesiredAccess : i32,
    Attributes : i32,
    ShareMode : i32,
    NTCreateDisposition : i32,
    NTDesiredAccess : i32,
    FileInfo : i64,
    HandleInfo : i64,
    Reserved : bool,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of OpenFileEventArgs
impl CBFSOpenFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSOpenFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the OpenFile event of a CBFS instance").to_owned();
            }
        }

        let lvDesiredAccess : i32;
        unsafe
        {
            lvDesiredAccess = *par.add(1) as i32;
        }
        
        let lvAttributes : i32;
        unsafe
        {
            lvAttributes = *par.add(2) as i32;
        }
        
        let lvShareMode : i32;
        unsafe
        {
            lvShareMode = *par.add(3) as i32;
        }
        
        let lvNTCreateDisposition : i32;
        unsafe
        {
            lvNTCreateDisposition = *par.add(4) as i32;
        }
        
        let lvNTDesiredAccess : i32;
        unsafe
        {
            lvNTDesiredAccess = *par.add(5) as i32;
        }
        
        let lvFileInfo : i64;
        unsafe
        {
            let lvFileInfoLPtr : *mut i64 = *par.add(6) as *mut i64;
            lvFileInfo = *lvFileInfoLPtr;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(7) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvReserved : bool;
        unsafe
        {
            lvReserved = (*par.add(8) as i32) != 0;
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(9) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(10) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(11) as i32;
        }
        
        CBFSOpenFileEventArgs
        {
            FileName: lvFileName,
            DesiredAccess: lvDesiredAccess,
            Attributes: lvAttributes,
            ShareMode: lvShareMode,
            NTCreateDisposition: lvNTCreateDisposition,
            NTDesiredAccess: lvNTDesiredAccess,
            FileInfo: lvFileInfo,
            HandleInfo: lvHandleInfo,
            Reserved: lvReserved,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfReserved : i32;
            if self.Reserved
            {
                intValOfReserved = 1;
            }
            else
            {
                intValOfReserved = 0;
            }
            *(self._Params.add(8)) = intValOfReserved as isize;
            *(self._Params.add(9)) = self.FileContext as isize;
            *(self._Params.add(10)) = self.HandleContext as isize;
            *(self._Params.add(11)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn desired_access(&self) -> i32
    {
        self.DesiredAccess
    }
    pub fn attributes(&self) -> i32
    {
        self.Attributes
    }
    pub fn share_mode(&self) -> i32
    {
        self.ShareMode
    }
    pub fn nt_create_disposition(&self) -> i32
    {
        self.NTCreateDisposition
    }
    pub fn nt_desired_access(&self) -> i32
    {
        self.NTDesiredAccess
    }
    pub fn file_info(&self) -> i64
    {
        self.FileInfo
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn reserved(&self) -> bool
    {
        self.Reserved
    }
    pub fn set_reserved(&mut self, value: bool)
    {
        self.Reserved = value;
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSOpenFileEvent
{
    fn on_open_file(&self, sender : &CBFS, e : &mut CBFSOpenFileEventArgs);
}


// CBFSQueryAllocatedRangesEventArgs carries the parameters of the QueryAllocatedRanges event of CBFS
pub struct CBFSQueryAllocatedRangesEventArgs
{
    FileName : String,
    Position : i64,
    Length : i64,
    hBufferPtr : *mut u8,
    BufferLength : i32,
    LengthReturned : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of QueryAllocatedRangesEventArgs
impl CBFSQueryAllocatedRangesEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSQueryAllocatedRangesEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the QueryAllocatedRanges event of a CBFS instance").to_owned();
            }
        }

        let lvPosition : i64;
        unsafe
        {
            let lvPositionLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvPosition = *lvPositionLPtr;
        }
        
        let lvLength : i64;
        unsafe
        {
            let lvLengthLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvLength = *lvLengthLPtr;
        }
        
        let lvhBufferPtr : *mut u8;
        unsafe
        {
            lvhBufferPtr = *par.add(3) as *mut u8;
        }

        let lvBufferLength : i32;
        unsafe
        {
            lvBufferLength = *par.add(4) as i32;
        }
        // lvhBufferLen = lvBufferLength;

        let lvLengthReturned : i32;
        unsafe
        {
            lvLengthReturned = *par.add(5) as i32;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(6) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(7) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(8) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(9) as i32;
        }
        
        CBFSQueryAllocatedRangesEventArgs
        {
            FileName: lvFileName,
            Position: lvPosition,
            Length: lvLength,
            hBufferPtr: lvhBufferPtr,
            BufferLength: lvBufferLength,
            LengthReturned: lvLengthReturned,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(5)) = self.LengthReturned as isize;
            *(self._Params.add(7)) = self.FileContext as isize;
            *(self._Params.add(8)) = self.HandleContext as isize;
            *(self._Params.add(9)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn position(&self) -> i64
    {
        self.Position
    }
    pub fn length(&self) -> i64
    {
        self.Length
    }
    pub fn buffer(&self) -> *mut u8
    {
        self.hBufferPtr
    }
    pub fn buffer_length(&self) -> i32
    {
        self.BufferLength
    }
    pub fn length_returned(&self) -> i32
    {
        self.LengthReturned
    }
    pub fn set_length_returned(&mut self, value: i32)
    {
        self.LengthReturned = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSQueryAllocatedRangesEvent
{
    fn on_query_allocated_ranges(&self, sender : &CBFS, e : &mut CBFSQueryAllocatedRangesEventArgs);
}


// CBFSQueryCompressionInfoEventArgs carries the parameters of the QueryCompressionInfo event of CBFS
pub struct CBFSQueryCompressionInfoEventArgs
{
    FileName : String,
    CompressedFileSize : i64,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of QueryCompressionInfoEventArgs
impl CBFSQueryCompressionInfoEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSQueryCompressionInfoEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the QueryCompressionInfo event of a CBFS instance").to_owned();
            }
        }

        let lvCompressedFileSize : i64;
        unsafe
        {
            let lvCompressedFileSizeLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvCompressedFileSize = *lvCompressedFileSizeLPtr;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(3) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(4) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBFSQueryCompressionInfoEventArgs
        {
            FileName: lvFileName,
            CompressedFileSize: lvCompressedFileSize,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvCompressedFileSizeLPtr : *mut i64 = *self._Params.add(1) as *mut i64;
            *lvCompressedFileSizeLPtr = self.CompressedFileSize ;
            *(self._Params.add(3)) = self.FileContext as isize;
            *(self._Params.add(4)) = self.HandleContext as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn compressed_file_size(&self) -> i64
    {
        self.CompressedFileSize
    }
    pub fn set_compressed_file_size(&mut self, value: i64)
    {
        self.CompressedFileSize = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSQueryCompressionInfoEvent
{
    fn on_query_compression_info(&self, sender : &CBFS, e : &mut CBFSQueryCompressionInfoEventArgs);
}


// CBFSQueryEaEventArgs carries the parameters of the QueryEa event of CBFS
pub struct CBFSQueryEaEventArgs
{
    FileName : String,
    hBufferPtr : *mut u8,
    BufferLength : i32,
    ReturnSingleEntry : bool,
    hEaListPtr : *mut u8,
    EaListLength : i32,
    EaIndex : i32,
    RestartScan : bool,
    LengthReturned : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of QueryEaEventArgs
impl CBFSQueryEaEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSQueryEaEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the QueryEa event of a CBFS instance").to_owned();
            }
        }

        let lvhBufferPtr : *mut u8;
        unsafe
        {
            lvhBufferPtr = *par.add(1) as *mut u8;
        }

        let lvBufferLength : i32;
        unsafe
        {
            lvBufferLength = *par.add(2) as i32;
        }
        // lvhBufferLen = lvBufferLength;

        let lvReturnSingleEntry : bool;
        unsafe
        {
            lvReturnSingleEntry = (*par.add(3) as i32) != 0;
        }

        let lvhEaListPtr : *mut u8;
        unsafe
        {
            lvhEaListPtr = *par.add(4) as *mut u8;
        }

        let lvEaListLength : i32;
        unsafe
        {
            lvEaListLength = *par.add(5) as i32;
        }
        // lvhEaListLen = lvEaListLength;

        let lvEaIndex : i32;
        unsafe
        {
            lvEaIndex = *par.add(6) as i32;
        }
        
        let lvRestartScan : bool;
        unsafe
        {
            lvRestartScan = (*par.add(7) as i32) != 0;
        }

        let lvLengthReturned : i32;
        unsafe
        {
            lvLengthReturned = *par.add(8) as i32;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(9) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(10) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(11) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(12) as i32;
        }
        
        CBFSQueryEaEventArgs
        {
            FileName: lvFileName,
            hBufferPtr: lvhBufferPtr,
            BufferLength: lvBufferLength,
            ReturnSingleEntry: lvReturnSingleEntry,
            hEaListPtr: lvhEaListPtr,
            EaListLength: lvEaListLength,
            EaIndex: lvEaIndex,
            RestartScan: lvRestartScan,
            LengthReturned: lvLengthReturned,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(8)) = self.LengthReturned as isize;
            *(self._Params.add(10)) = self.FileContext as isize;
            *(self._Params.add(11)) = self.HandleContext as isize;
            *(self._Params.add(12)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn buffer(&self) -> *mut u8
    {
        self.hBufferPtr
    }
    pub fn buffer_length(&self) -> i32
    {
        self.BufferLength
    }
    pub fn return_single_entry(&self) -> bool
    {
        self.ReturnSingleEntry
    }
    pub fn ea_list(&self) -> *mut u8
    {
        self.hEaListPtr
    }
    pub fn ea_list_length(&self) -> i32
    {
        self.EaListLength
    }
    pub fn ea_index(&self) -> i32
    {
        self.EaIndex
    }
    pub fn restart_scan(&self) -> bool
    {
        self.RestartScan
    }
    pub fn length_returned(&self) -> i32
    {
        self.LengthReturned
    }
    pub fn set_length_returned(&mut self, value: i32)
    {
        self.LengthReturned = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSQueryEaEvent
{
    fn on_query_ea(&self, sender : &CBFS, e : &mut CBFSQueryEaEventArgs);
}


// CBFSQueryQuotasEventArgs carries the parameters of the QueryQuotas event of CBFS
pub struct CBFSQueryQuotasEventArgs
{
    hSIDPtr : *mut u8,
    SIDLength : i32,
    Index : i32,
    QuotaFound : bool,
    QuotaUsed : i64,
    QuotaThreshold : i64,
    QuotaLimit : i64,
    hSIDOutPtr : *mut u8,
    SIDOutLength : i32,
    EnumerationContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of QueryQuotasEventArgs
impl CBFSQueryQuotasEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSQueryQuotasEventArgs
    {

        let lvhSIDPtr : *mut u8;
        unsafe
        {
            lvhSIDPtr = *par.add(0) as *mut u8;
        }

        let lvSIDLength : i32;
        unsafe
        {
            lvSIDLength = *par.add(1) as i32;
        }
        // lvhSIDLen = lvSIDLength;

        let lvIndex : i32;
        unsafe
        {
            lvIndex = *par.add(2) as i32;
        }
        
        let lvQuotaFound : bool;
        unsafe
        {
            lvQuotaFound = (*par.add(3) as i32) != 0;
        }

        let lvQuotaUsed : i64;
        unsafe
        {
            let lvQuotaUsedLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvQuotaUsed = *lvQuotaUsedLPtr;
        }
        
        let lvQuotaThreshold : i64;
        unsafe
        {
            let lvQuotaThresholdLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvQuotaThreshold = *lvQuotaThresholdLPtr;
        }
        
        let lvQuotaLimit : i64;
        unsafe
        {
            let lvQuotaLimitLPtr : *mut i64 = *par.add(6) as *mut i64;
            lvQuotaLimit = *lvQuotaLimitLPtr;
        }
        
        let lvhSIDOutPtr : *mut u8;
        unsafe
        {
            lvhSIDOutPtr = *par.add(7) as *mut u8;
        }

        let lvSIDOutLength : i32;
        unsafe
        {
            lvSIDOutLength = *par.add(8) as i32;
        }
        // lvhSIDOutLen = lvSIDOutLength;

        let lvEnumerationContext : usize;
        unsafe
        {
            lvEnumerationContext = *par.add(9) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(10) as i32;
        }
        
        CBFSQueryQuotasEventArgs
        {
            hSIDPtr: lvhSIDPtr,
            SIDLength: lvSIDLength,
            Index: lvIndex,
            QuotaFound: lvQuotaFound,
            QuotaUsed: lvQuotaUsed,
            QuotaThreshold: lvQuotaThreshold,
            QuotaLimit: lvQuotaLimit,
            hSIDOutPtr: lvhSIDOutPtr,
            SIDOutLength: lvSIDOutLength,
            EnumerationContext: lvEnumerationContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfQuotaFound : i32;
            if self.QuotaFound
            {
                intValOfQuotaFound = 1;
            }
            else
            {
                intValOfQuotaFound = 0;
            }
            *(self._Params.add(3)) = intValOfQuotaFound as isize;
            let lvQuotaUsedLPtr : *mut i64 = *self._Params.add(4) as *mut i64;
            *lvQuotaUsedLPtr = self.QuotaUsed ;
            let lvQuotaThresholdLPtr : *mut i64 = *self._Params.add(5) as *mut i64;
            *lvQuotaThresholdLPtr = self.QuotaThreshold ;
            let lvQuotaLimitLPtr : *mut i64 = *self._Params.add(6) as *mut i64;
            *lvQuotaLimitLPtr = self.QuotaLimit ;
            *(self._Params.add(9)) = self.EnumerationContext as isize;
            *(self._Params.add(10)) = self.ResultCode as isize;
        }
    }

    pub fn sid(&self) -> *mut u8
    {
        self.hSIDPtr
    }
    pub fn sid_length(&self) -> i32
    {
        self.SIDLength
    }
    pub fn index(&self) -> i32
    {
        self.Index
    }
    pub fn quota_found(&self) -> bool
    {
        self.QuotaFound
    }
    pub fn set_quota_found(&mut self, value: bool)
    {
        self.QuotaFound = value;
    }
    pub fn quota_used(&self) -> i64
    {
        self.QuotaUsed
    }
    pub fn set_quota_used(&mut self, value: i64)
    {
        self.QuotaUsed = value;
    }
    pub fn quota_threshold(&self) -> i64
    {
        self.QuotaThreshold
    }
    pub fn set_quota_threshold(&mut self, value: i64)
    {
        self.QuotaThreshold = value;
    }
    pub fn quota_limit(&self) -> i64
    {
        self.QuotaLimit
    }
    pub fn set_quota_limit(&mut self, value: i64)
    {
        self.QuotaLimit = value;
    }
    pub fn sid_out(&self) -> *mut u8
    {
        self.hSIDOutPtr
    }
    pub fn sid_out_length(&self) -> i32
    {
        self.SIDOutLength
    }
    pub fn enumeration_context(&self) -> usize
    {
        self.EnumerationContext
    }
    pub fn set_enumeration_context(&mut self, value: usize)
    {
        self.EnumerationContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSQueryQuotasEvent
{
    fn on_query_quotas(&self, sender : &CBFS, e : &mut CBFSQueryQuotasEventArgs);
}


// CBFSReadFileEventArgs carries the parameters of the ReadFile event of CBFS
pub struct CBFSReadFileEventArgs
{
    FileName : String,
    Position : i64,
    hBufferPtr : *mut u8,
    BytesToRead : i64,
    BytesRead : i64,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ReadFileEventArgs
impl CBFSReadFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSReadFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the ReadFile event of a CBFS instance").to_owned();
            }
        }

        let lvPosition : i64;
        unsafe
        {
            let lvPositionLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvPosition = *lvPositionLPtr;
        }
        
        let lvhBufferPtr : *mut u8;
        unsafe
        {
            lvhBufferPtr = *par.add(2) as *mut u8;
        }

        let lvBytesToRead : i64;
        unsafe
        {
            let lvBytesToReadLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvBytesToRead = *lvBytesToReadLPtr;
        }
        // lvhBufferLen = lvBytesToRead;

        let lvBytesRead : i64;
        unsafe
        {
            let lvBytesReadLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvBytesRead = *lvBytesReadLPtr;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(6) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(7) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(8) as i32;
        }
        
        CBFSReadFileEventArgs
        {
            FileName: lvFileName,
            Position: lvPosition,
            hBufferPtr: lvhBufferPtr,
            BytesToRead: lvBytesToRead,
            BytesRead: lvBytesRead,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvBytesReadLPtr : *mut i64 = *self._Params.add(4) as *mut i64;
            *lvBytesReadLPtr = self.BytesRead ;
            *(self._Params.add(6)) = self.FileContext as isize;
            *(self._Params.add(7)) = self.HandleContext as isize;
            *(self._Params.add(8)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn position(&self) -> i64
    {
        self.Position
    }
    pub fn buffer(&self) -> *mut u8
    {
        self.hBufferPtr
    }
    pub fn bytes_to_read(&self) -> i64
    {
        self.BytesToRead
    }
    pub fn bytes_read(&self) -> i64
    {
        self.BytesRead
    }
    pub fn set_bytes_read(&mut self, value: i64)
    {
        self.BytesRead = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSReadFileEvent
{
    fn on_read_file(&self, sender : &CBFS, e : &mut CBFSReadFileEventArgs);
}


// CBFSRenameOrMoveFileEventArgs carries the parameters of the RenameOrMoveFile event of CBFS
pub struct CBFSRenameOrMoveFileEventArgs
{
    FileName : String,
    NewFileName : String,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of RenameOrMoveFileEventArgs
impl CBFSRenameOrMoveFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSRenameOrMoveFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the RenameOrMoveFile event of a CBFS instance").to_owned();
            }
        }

        let lvhNewFileNamePtr : *mut c_char;
        let lvNewFileName : String;
        unsafe
        {
            lvhNewFileNamePtr = *par.add(1) as *mut c_char;
            if lvhNewFileNamePtr == std::ptr::null_mut()
            {
                lvNewFileName = String::default();
            }
            else
            {
                lvNewFileName = CStr::from_ptr(lvhNewFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'NewFileName' in the RenameOrMoveFile event of a CBFS instance").to_owned();
            }
        }

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(3) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(4) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBFSRenameOrMoveFileEventArgs
        {
            FileName: lvFileName,
            NewFileName: lvNewFileName,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.FileContext as isize;
            *(self._Params.add(4)) = self.HandleContext as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn new_file_name(&self) -> &String
    {
        &self.NewFileName
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSRenameOrMoveFileEvent
{
    fn on_rename_or_move_file(&self, sender : &CBFS, e : &mut CBFSRenameOrMoveFileEventArgs);
}


// CBFSSetAllocationSizeEventArgs carries the parameters of the SetAllocationSize event of CBFS
pub struct CBFSSetAllocationSizeEventArgs
{
    FileName : String,
    AllocationSize : i64,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of SetAllocationSizeEventArgs
impl CBFSSetAllocationSizeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSSetAllocationSizeEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the SetAllocationSize event of a CBFS instance").to_owned();
            }
        }

        let lvAllocationSize : i64;
        unsafe
        {
            let lvAllocationSizeLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvAllocationSize = *lvAllocationSizeLPtr;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(3) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(4) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBFSSetAllocationSizeEventArgs
        {
            FileName: lvFileName,
            AllocationSize: lvAllocationSize,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.FileContext as isize;
            *(self._Params.add(4)) = self.HandleContext as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn allocation_size(&self) -> i64
    {
        self.AllocationSize
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSSetAllocationSizeEvent
{
    fn on_set_allocation_size(&self, sender : &CBFS, e : &mut CBFSSetAllocationSizeEventArgs);
}


// CBFSSetDefaultQuotaInfoEventArgs carries the parameters of the SetDefaultQuotaInfo event of CBFS
pub struct CBFSSetDefaultQuotaInfoEventArgs
{
    DefaultQuotaThreshold : i64,
    DefaultQuotaLimit : i64,
    FileSystemControlFlags : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of SetDefaultQuotaInfoEventArgs
impl CBFSSetDefaultQuotaInfoEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSSetDefaultQuotaInfoEventArgs
    {

        let lvDefaultQuotaThreshold : i64;
        unsafe
        {
            let lvDefaultQuotaThresholdLPtr : *mut i64 = *par.add(0) as *mut i64;
            lvDefaultQuotaThreshold = *lvDefaultQuotaThresholdLPtr;
        }
        
        let lvDefaultQuotaLimit : i64;
        unsafe
        {
            let lvDefaultQuotaLimitLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvDefaultQuotaLimit = *lvDefaultQuotaLimitLPtr;
        }
        
        let lvFileSystemControlFlags : i64;
        unsafe
        {
            let lvFileSystemControlFlagsLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvFileSystemControlFlags = *lvFileSystemControlFlagsLPtr;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(3) as i32;
        }
        
        CBFSSetDefaultQuotaInfoEventArgs
        {
            DefaultQuotaThreshold: lvDefaultQuotaThreshold,
            DefaultQuotaLimit: lvDefaultQuotaLimit,
            FileSystemControlFlags: lvFileSystemControlFlags,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.ResultCode as isize;
        }
    }

    pub fn default_quota_threshold(&self) -> i64
    {
        self.DefaultQuotaThreshold
    }
    pub fn default_quota_limit(&self) -> i64
    {
        self.DefaultQuotaLimit
    }
    pub fn file_system_control_flags(&self) -> i64
    {
        self.FileSystemControlFlags
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSSetDefaultQuotaInfoEvent
{
    fn on_set_default_quota_info(&self, sender : &CBFS, e : &mut CBFSSetDefaultQuotaInfoEventArgs);
}


// CBFSSetEaEventArgs carries the parameters of the SetEa event of CBFS
pub struct CBFSSetEaEventArgs
{
    FileName : String,
    hBufferPtr : *mut u8,
    BufferLength : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of SetEaEventArgs
impl CBFSSetEaEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSSetEaEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the SetEa event of a CBFS instance").to_owned();
            }
        }

        let lvhBufferPtr : *mut u8;
        unsafe
        {
            lvhBufferPtr = *par.add(1) as *mut u8;
        }

        let lvBufferLength : i32;
        unsafe
        {
            lvBufferLength = *par.add(2) as i32;
        }
        // lvhBufferLen = lvBufferLength;

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(4) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(5) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(6) as i32;
        }
        
        CBFSSetEaEventArgs
        {
            FileName: lvFileName,
            hBufferPtr: lvhBufferPtr,
            BufferLength: lvBufferLength,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(4)) = self.FileContext as isize;
            *(self._Params.add(5)) = self.HandleContext as isize;
            *(self._Params.add(6)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn buffer(&self) -> *mut u8
    {
        self.hBufferPtr
    }
    pub fn buffer_length(&self) -> i32
    {
        self.BufferLength
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSSetEaEvent
{
    fn on_set_ea(&self, sender : &CBFS, e : &mut CBFSSetEaEventArgs);
}


// CBFSSetFileAttributesEventArgs carries the parameters of the SetFileAttributes event of CBFS
pub struct CBFSSetFileAttributesEventArgs
{
    FileName : String,
    CreationTime : chrono::DateTime<Utc>,
    LastAccessTime : chrono::DateTime<Utc>,
    LastWriteTime : chrono::DateTime<Utc>,
    ChangeTime : chrono::DateTime<Utc>,
    Attributes : i32,
    EventOrigin : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of SetFileAttributesEventArgs
impl CBFSSetFileAttributesEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSSetFileAttributesEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the SetFileAttributes event of a CBFS instance").to_owned();
            }
        }

        let lvCreationTimeLong : i64;
        let lvCreationTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvCreationTimeLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvCreationTimeLong = *lvCreationTimeLPtr;
            lvCreationTime = file_time_to_chrono_time(lvCreationTimeLong);
        }

        let lvLastAccessTimeLong : i64;
        let lvLastAccessTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvLastAccessTimeLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvLastAccessTimeLong = *lvLastAccessTimeLPtr;
            lvLastAccessTime = file_time_to_chrono_time(lvLastAccessTimeLong);
        }

        let lvLastWriteTimeLong : i64;
        let lvLastWriteTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvLastWriteTimeLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvLastWriteTimeLong = *lvLastWriteTimeLPtr;
            lvLastWriteTime = file_time_to_chrono_time(lvLastWriteTimeLong);
        }

        let lvChangeTimeLong : i64;
        let lvChangeTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvChangeTimeLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvChangeTimeLong = *lvChangeTimeLPtr;
            lvChangeTime = file_time_to_chrono_time(lvChangeTimeLong);
        }

        let lvAttributes : i32;
        unsafe
        {
            lvAttributes = *par.add(5) as i32;
        }
        
        let lvEventOrigin : i32;
        unsafe
        {
            lvEventOrigin = *par.add(6) as i32;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(7) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(8) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(9) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(10) as i32;
        }
        
        CBFSSetFileAttributesEventArgs
        {
            FileName: lvFileName,
            CreationTime: lvCreationTime,
            LastAccessTime: lvLastAccessTime,
            LastWriteTime: lvLastWriteTime,
            ChangeTime: lvChangeTime,
            Attributes: lvAttributes,
            EventOrigin: lvEventOrigin,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(8)) = self.FileContext as isize;
            *(self._Params.add(9)) = self.HandleContext as isize;
            *(self._Params.add(10)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn creation_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.CreationTime
    }
    pub fn last_access_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.LastAccessTime
    }
    pub fn last_write_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.LastWriteTime
    }
    pub fn change_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.ChangeTime
    }
    pub fn attributes(&self) -> i32
    {
        self.Attributes
    }
    pub fn event_origin(&self) -> i32
    {
        self.EventOrigin
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSSetFileAttributesEvent
{
    fn on_set_file_attributes(&self, sender : &CBFS, e : &mut CBFSSetFileAttributesEventArgs);
}


// CBFSSetFileSecurityEventArgs carries the parameters of the SetFileSecurity event of CBFS
pub struct CBFSSetFileSecurityEventArgs
{
    FileName : String,
    SecurityInformation : i32,
    hSecurityDescriptorPtr : *mut u8,
    Length : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of SetFileSecurityEventArgs
impl CBFSSetFileSecurityEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSSetFileSecurityEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the SetFileSecurity event of a CBFS instance").to_owned();
            }
        }

        let lvSecurityInformation : i32;
        unsafe
        {
            lvSecurityInformation = *par.add(1) as i32;
        }
        
        let lvhSecurityDescriptorPtr : *mut u8;
        unsafe
        {
            lvhSecurityDescriptorPtr = *par.add(2) as *mut u8;
        }

        let lvLength : i32;
        unsafe
        {
            lvLength = *par.add(3) as i32;
        }
        // lvhSecurityDescriptorLen = lvLength;

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(5) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(6) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(7) as i32;
        }
        
        CBFSSetFileSecurityEventArgs
        {
            FileName: lvFileName,
            SecurityInformation: lvSecurityInformation,
            hSecurityDescriptorPtr: lvhSecurityDescriptorPtr,
            Length: lvLength,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(5)) = self.FileContext as isize;
            *(self._Params.add(6)) = self.HandleContext as isize;
            *(self._Params.add(7)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn security_information(&self) -> i32
    {
        self.SecurityInformation
    }
    pub fn security_descriptor(&self) -> *mut u8
    {
        self.hSecurityDescriptorPtr
    }
    pub fn length(&self) -> i32
    {
        self.Length
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSSetFileSecurityEvent
{
    fn on_set_file_security(&self, sender : &CBFS, e : &mut CBFSSetFileSecurityEventArgs);
}


// CBFSSetFileSizeEventArgs carries the parameters of the SetFileSize event of CBFS
pub struct CBFSSetFileSizeEventArgs
{
    FileName : String,
    Size : i64,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of SetFileSizeEventArgs
impl CBFSSetFileSizeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSSetFileSizeEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the SetFileSize event of a CBFS instance").to_owned();
            }
        }

        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(3) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(4) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBFSSetFileSizeEventArgs
        {
            FileName: lvFileName,
            Size: lvSize,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.FileContext as isize;
            *(self._Params.add(4)) = self.HandleContext as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn size(&self) -> i64
    {
        self.Size
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSSetFileSizeEvent
{
    fn on_set_file_size(&self, sender : &CBFS, e : &mut CBFSSetFileSizeEventArgs);
}


// CBFSSetObjectIdEventArgs carries the parameters of the SetObjectId event of CBFS
pub struct CBFSSetObjectIdEventArgs
{
    FileName : String,
    OnlyExtendedInfo : bool,
    hObjectIdPtr : *mut u8,
    ObjectIdLength : i32,
    hExtendedInformationPtr : *mut u8,
    ExtendedInformationLength : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of SetObjectIdEventArgs
impl CBFSSetObjectIdEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSSetObjectIdEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the SetObjectId event of a CBFS instance").to_owned();
            }
        }

        let lvOnlyExtendedInfo : bool;
        unsafe
        {
            lvOnlyExtendedInfo = (*par.add(1) as i32) != 0;
        }

        let lvhObjectIdPtr : *mut u8;
        unsafe
        {
            lvhObjectIdPtr = *par.add(2) as *mut u8;
        }

        let lvObjectIdLength : i32;
        unsafe
        {
            lvObjectIdLength = *par.add(3) as i32;
        }
        // lvhObjectIdLen = lvObjectIdLength;

        let lvhExtendedInformationPtr : *mut u8;
        unsafe
        {
            lvhExtendedInformationPtr = *par.add(4) as *mut u8;
        }

        let lvExtendedInformationLength : i32;
        unsafe
        {
            lvExtendedInformationLength = *par.add(5) as i32;
        }
        // lvhExtendedInformationLen = lvExtendedInformationLength;

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(6) as i32;
        }
        
        CBFSSetObjectIdEventArgs
        {
            FileName: lvFileName,
            OnlyExtendedInfo: lvOnlyExtendedInfo,
            hObjectIdPtr: lvhObjectIdPtr,
            ObjectIdLength: lvObjectIdLength,
            hExtendedInformationPtr: lvhExtendedInformationPtr,
            ExtendedInformationLength: lvExtendedInformationLength,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(6)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn only_extended_info(&self) -> bool
    {
        self.OnlyExtendedInfo
    }
    pub fn object_id(&self) -> *mut u8
    {
        self.hObjectIdPtr
    }
    pub fn object_id_length(&self) -> i32
    {
        self.ObjectIdLength
    }
    pub fn extended_information(&self) -> *mut u8
    {
        self.hExtendedInformationPtr
    }
    pub fn extended_information_length(&self) -> i32
    {
        self.ExtendedInformationLength
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSSetObjectIdEvent
{
    fn on_set_object_id(&self, sender : &CBFS, e : &mut CBFSSetObjectIdEventArgs);
}


// CBFSSetQuotasEventArgs carries the parameters of the SetQuotas event of CBFS
pub struct CBFSSetQuotasEventArgs
{
    hSIDPtr : *mut u8,
    SIDLength : i32,
    RemoveQuota : bool,
    QuotaFound : bool,
    QuotaUsed : i64,
    QuotaThreshold : i64,
    QuotaLimit : i64,
    EnumerationContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of SetQuotasEventArgs
impl CBFSSetQuotasEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSSetQuotasEventArgs
    {

        let lvhSIDPtr : *mut u8;
        unsafe
        {
            lvhSIDPtr = *par.add(0) as *mut u8;
        }

        let lvSIDLength : i32;
        unsafe
        {
            lvSIDLength = *par.add(1) as i32;
        }
        // lvhSIDLen = lvSIDLength;

        let lvRemoveQuota : bool;
        unsafe
        {
            lvRemoveQuota = (*par.add(2) as i32) != 0;
        }

        let lvQuotaFound : bool;
        unsafe
        {
            lvQuotaFound = (*par.add(3) as i32) != 0;
        }

        let lvQuotaUsed : i64;
        unsafe
        {
            let lvQuotaUsedLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvQuotaUsed = *lvQuotaUsedLPtr;
        }
        
        let lvQuotaThreshold : i64;
        unsafe
        {
            let lvQuotaThresholdLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvQuotaThreshold = *lvQuotaThresholdLPtr;
        }
        
        let lvQuotaLimit : i64;
        unsafe
        {
            let lvQuotaLimitLPtr : *mut i64 = *par.add(6) as *mut i64;
            lvQuotaLimit = *lvQuotaLimitLPtr;
        }
        
        let lvEnumerationContext : usize;
        unsafe
        {
            lvEnumerationContext = *par.add(7) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(8) as i32;
        }
        
        CBFSSetQuotasEventArgs
        {
            hSIDPtr: lvhSIDPtr,
            SIDLength: lvSIDLength,
            RemoveQuota: lvRemoveQuota,
            QuotaFound: lvQuotaFound,
            QuotaUsed: lvQuotaUsed,
            QuotaThreshold: lvQuotaThreshold,
            QuotaLimit: lvQuotaLimit,
            EnumerationContext: lvEnumerationContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfQuotaFound : i32;
            if self.QuotaFound
            {
                intValOfQuotaFound = 1;
            }
            else
            {
                intValOfQuotaFound = 0;
            }
            *(self._Params.add(3)) = intValOfQuotaFound as isize;
            *(self._Params.add(7)) = self.EnumerationContext as isize;
            *(self._Params.add(8)) = self.ResultCode as isize;
        }
    }

    pub fn sid(&self) -> *mut u8
    {
        self.hSIDPtr
    }
    pub fn sid_length(&self) -> i32
    {
        self.SIDLength
    }
    pub fn remove_quota(&self) -> bool
    {
        self.RemoveQuota
    }
    pub fn quota_found(&self) -> bool
    {
        self.QuotaFound
    }
    pub fn set_quota_found(&mut self, value: bool)
    {
        self.QuotaFound = value;
    }
    pub fn quota_used(&self) -> i64
    {
        self.QuotaUsed
    }
    pub fn quota_threshold(&self) -> i64
    {
        self.QuotaThreshold
    }
    pub fn quota_limit(&self) -> i64
    {
        self.QuotaLimit
    }
    pub fn enumeration_context(&self) -> usize
    {
        self.EnumerationContext
    }
    pub fn set_enumeration_context(&mut self, value: usize)
    {
        self.EnumerationContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSSetQuotasEvent
{
    fn on_set_quotas(&self, sender : &CBFS, e : &mut CBFSSetQuotasEventArgs);
}


// CBFSSetReparsePointEventArgs carries the parameters of the SetReparsePoint event of CBFS
pub struct CBFSSetReparsePointEventArgs
{
    FileName : String,
    ReparseTag : i64,
    hReparseBufferPtr : *mut u8,
    ReparseBufferLength : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of SetReparsePointEventArgs
impl CBFSSetReparsePointEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSSetReparsePointEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the SetReparsePoint event of a CBFS instance").to_owned();
            }
        }

        let lvReparseTag : i64;
        unsafe
        {
            let lvReparseTagLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvReparseTag = *lvReparseTagLPtr;
        }
        
        let lvhReparseBufferPtr : *mut u8;
        unsafe
        {
            lvhReparseBufferPtr = *par.add(2) as *mut u8;
        }

        let lvReparseBufferLength : i32;
        unsafe
        {
            lvReparseBufferLength = *par.add(3) as i32;
        }
        // lvhReparseBufferLen = lvReparseBufferLength;

        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(5) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(6) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(7) as i32;
        }
        
        CBFSSetReparsePointEventArgs
        {
            FileName: lvFileName,
            ReparseTag: lvReparseTag,
            hReparseBufferPtr: lvhReparseBufferPtr,
            ReparseBufferLength: lvReparseBufferLength,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(5)) = self.FileContext as isize;
            *(self._Params.add(6)) = self.HandleContext as isize;
            *(self._Params.add(7)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn reparse_tag(&self) -> i64
    {
        self.ReparseTag
    }
    pub fn reparse_buffer(&self) -> *mut u8
    {
        self.hReparseBufferPtr
    }
    pub fn reparse_buffer_length(&self) -> i32
    {
        self.ReparseBufferLength
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSSetReparsePointEvent
{
    fn on_set_reparse_point(&self, sender : &CBFS, e : &mut CBFSSetReparsePointEventArgs);
}


// CBFSSetValidDataLengthEventArgs carries the parameters of the SetValidDataLength event of CBFS
pub struct CBFSSetValidDataLengthEventArgs
{
    FileName : String,
    ValidDataLength : i64,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of SetValidDataLengthEventArgs
impl CBFSSetValidDataLengthEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSSetValidDataLengthEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the SetValidDataLength event of a CBFS instance").to_owned();
            }
        }

        let lvValidDataLength : i64;
        unsafe
        {
            let lvValidDataLengthLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvValidDataLength = *lvValidDataLengthLPtr;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(3) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(4) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBFSSetValidDataLengthEventArgs
        {
            FileName: lvFileName,
            ValidDataLength: lvValidDataLength,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.FileContext as isize;
            *(self._Params.add(4)) = self.HandleContext as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn valid_data_length(&self) -> i64
    {
        self.ValidDataLength
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSSetValidDataLengthEvent
{
    fn on_set_valid_data_length(&self, sender : &CBFS, e : &mut CBFSSetValidDataLengthEventArgs);
}


// CBFSSetVolumeLabelEventArgs carries the parameters of the SetVolumeLabel event of CBFS
pub struct CBFSSetVolumeLabelEventArgs
{
    VolumeLabel : String,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of SetVolumeLabelEventArgs
impl CBFSSetVolumeLabelEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSSetVolumeLabelEventArgs
    {

        let lvhVolumeLabelPtr : *mut c_char;
        let lvVolumeLabel : String;
        unsafe
        {
            lvhVolumeLabelPtr = *par.add(0) as *mut c_char;
            if lvhVolumeLabelPtr == std::ptr::null_mut()
            {
                lvVolumeLabel = String::default();
            }
            else
            {
                lvVolumeLabel = CStr::from_ptr(lvhVolumeLabelPtr).to_str().expect("Valid UTF8 not received for the parameter 'VolumeLabel' in the SetVolumeLabel event of a CBFS instance").to_owned();
            }
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(1) as i32;
        }
        
        CBFSSetVolumeLabelEventArgs
        {
            VolumeLabel: lvVolumeLabel,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(1)) = self.ResultCode as isize;
        }
    }

    pub fn volume_label(&self) -> &String
    {
        &self.VolumeLabel
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSSetVolumeLabelEvent
{
    fn on_set_volume_label(&self, sender : &CBFS, e : &mut CBFSSetVolumeLabelEventArgs);
}


// CBFSUnlockFileEventArgs carries the parameters of the UnlockFile event of CBFS
pub struct CBFSUnlockFileEventArgs
{
    FileName : String,
    ByteOffset : i64,
    Length : i64,
    Key : i32,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of UnlockFileEventArgs
impl CBFSUnlockFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSUnlockFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the UnlockFile event of a CBFS instance").to_owned();
            }
        }

        let lvByteOffset : i64;
        unsafe
        {
            let lvByteOffsetLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvByteOffset = *lvByteOffsetLPtr;
        }
        
        let lvLength : i64;
        unsafe
        {
            let lvLengthLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvLength = *lvLengthLPtr;
        }
        
        let lvKey : i32;
        unsafe
        {
            lvKey = *par.add(3) as i32;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(5) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(6) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(7) as i32;
        }
        
        CBFSUnlockFileEventArgs
        {
            FileName: lvFileName,
            ByteOffset: lvByteOffset,
            Length: lvLength,
            Key: lvKey,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(5)) = self.FileContext as isize;
            *(self._Params.add(6)) = self.HandleContext as isize;
            *(self._Params.add(7)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn byte_offset(&self) -> i64
    {
        self.ByteOffset
    }
    pub fn length(&self) -> i64
    {
        self.Length
    }
    pub fn key(&self) -> i32
    {
        self.Key
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSUnlockFileEvent
{
    fn on_unlock_file(&self, sender : &CBFS, e : &mut CBFSUnlockFileEventArgs);
}


// CBFSUnmountEventArgs carries the parameters of the Unmount event of CBFS
pub struct CBFSUnmountEventArgs
{
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of UnmountEventArgs
impl CBFSUnmountEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSUnmountEventArgs
    {

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(0) as i32;
        }
        
        CBFSUnmountEventArgs
        {
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(0)) = self.ResultCode as isize;
        }
    }

    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSUnmountEvent
{
    fn on_unmount(&self, sender : &CBFS, e : &mut CBFSUnmountEventArgs);
}


// CBFSWorkerThreadCreationEventArgs carries the parameters of the WorkerThreadCreation event of CBFS
pub struct CBFSWorkerThreadCreationEventArgs
{
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of WorkerThreadCreationEventArgs
impl CBFSWorkerThreadCreationEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSWorkerThreadCreationEventArgs
    {

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(0) as i32;
        }
        
        CBFSWorkerThreadCreationEventArgs
        {
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(0)) = self.ResultCode as isize;
        }
    }

    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSWorkerThreadCreationEvent
{
    fn on_worker_thread_creation(&self, sender : &CBFS, e : &mut CBFSWorkerThreadCreationEventArgs);
}


// CBFSWorkerThreadTerminationEventArgs carries the parameters of the WorkerThreadTermination event of CBFS
pub struct CBFSWorkerThreadTerminationEventArgs
{

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of WorkerThreadTerminationEventArgs
impl CBFSWorkerThreadTerminationEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSWorkerThreadTerminationEventArgs
    {

        CBFSWorkerThreadTerminationEventArgs
        {
            _Params: par
        }
    }


}

pub trait CBFSWorkerThreadTerminationEvent
{
    fn on_worker_thread_termination(&self, sender : &CBFS, e : &mut CBFSWorkerThreadTerminationEventArgs);
}


// CBFSWriteFileEventArgs carries the parameters of the WriteFile event of CBFS
pub struct CBFSWriteFileEventArgs
{
    FileName : String,
    Position : i64,
    hBufferPtr : *mut u8,
    BytesToWrite : i64,
    BytesWritten : i64,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of WriteFileEventArgs
impl CBFSWriteFileEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSWriteFileEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the WriteFile event of a CBFS instance").to_owned();
            }
        }

        let lvPosition : i64;
        unsafe
        {
            let lvPositionLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvPosition = *lvPositionLPtr;
        }
        
        let lvhBufferPtr : *mut u8;
        unsafe
        {
            lvhBufferPtr = *par.add(2) as *mut u8;
        }

        let lvBytesToWrite : i64;
        unsafe
        {
            let lvBytesToWriteLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvBytesToWrite = *lvBytesToWriteLPtr;
        }
        // lvhBufferLen = lvBytesToWrite;

        let lvBytesWritten : i64;
        unsafe
        {
            let lvBytesWrittenLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvBytesWritten = *lvBytesWrittenLPtr;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(6) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(7) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(8) as i32;
        }
        
        CBFSWriteFileEventArgs
        {
            FileName: lvFileName,
            Position: lvPosition,
            hBufferPtr: lvhBufferPtr,
            BytesToWrite: lvBytesToWrite,
            BytesWritten: lvBytesWritten,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvBytesWrittenLPtr : *mut i64 = *self._Params.add(4) as *mut i64;
            *lvBytesWrittenLPtr = self.BytesWritten ;
            *(self._Params.add(6)) = self.FileContext as isize;
            *(self._Params.add(7)) = self.HandleContext as isize;
            *(self._Params.add(8)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn position(&self) -> i64
    {
        self.Position
    }
    pub fn buffer(&self) -> *mut u8
    {
        self.hBufferPtr
    }
    pub fn bytes_to_write(&self) -> i64
    {
        self.BytesToWrite
    }
    pub fn bytes_written(&self) -> i64
    {
        self.BytesWritten
    }
    pub fn set_bytes_written(&mut self, value: i64)
    {
        self.BytesWritten = value;
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSWriteFileEvent
{
    fn on_write_file(&self, sender : &CBFS, e : &mut CBFSWriteFileEventArgs);
}


// CBFSZeroizeFileRangeEventArgs carries the parameters of the ZeroizeFileRange event of CBFS
pub struct CBFSZeroizeFileRangeEventArgs
{
    FileName : String,
    Position : i64,
    Length : i64,
    HandleInfo : i64,
    FileContext : usize,
    HandleContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ZeroizeFileRangeEventArgs
impl CBFSZeroizeFileRangeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBFSZeroizeFileRangeEventArgs
    {

        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(0) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the ZeroizeFileRange event of a CBFS instance").to_owned();
            }
        }

        let lvPosition : i64;
        unsafe
        {
            let lvPositionLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvPosition = *lvPositionLPtr;
        }
        
        let lvLength : i64;
        unsafe
        {
            let lvLengthLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvLength = *lvLengthLPtr;
        }
        
        let lvHandleInfo : i64;
        unsafe
        {
            let lvHandleInfoLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvHandleInfo = *lvHandleInfoLPtr;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(4) as usize;
        }

        let lvHandleContext : usize;
        unsafe
        {
            lvHandleContext = *par.add(5) as usize;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(6) as i32;
        }
        
        CBFSZeroizeFileRangeEventArgs
        {
            FileName: lvFileName,
            Position: lvPosition,
            Length: lvLength,
            HandleInfo: lvHandleInfo,
            FileContext: lvFileContext,
            HandleContext: lvHandleContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(4)) = self.FileContext as isize;
            *(self._Params.add(5)) = self.HandleContext as isize;
            *(self._Params.add(6)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn position(&self) -> i64
    {
        self.Position
    }
    pub fn length(&self) -> i64
    {
        self.Length
    }
    pub fn handle_info(&self) -> i64
    {
        self.HandleInfo
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn handle_context(&self) -> usize
    {
        self.HandleContext
    }
    pub fn set_handle_context(&mut self, value: usize)
    {
        self.HandleContext = value;
    }
    pub fn result_code(&self) -> i32
    {
        self.ResultCode
    }
    pub fn set_result_code(&mut self, value: i32)
    {
        self.ResultCode = value;
    }
}

pub trait CBFSZeroizeFileRangeEvent
{
    fn on_zeroize_file_range(&self, sender : &CBFS, e : &mut CBFSZeroizeFileRangeEventArgs);
}


////////////////////////////
// Main Class Implementation
////////////////////////////

/* The CBFS component gives applications the ability to create a virtual filesystem visible to all or selected processes in Windows. */
//pub struct CBFS<'a>
pub struct CBFS
{

    // onCanFileBeDeleted : Option<&'a dyn CBFSCanFileBeDeletedEvent>,
    onCanFileBeDeleted : Option<fn (sender : &CBFS, e : &mut CBFSCanFileBeDeletedEventArgs) >,
    // onCleanupFile : Option<&'a dyn CBFSCleanupFileEvent>,
    onCleanupFile : Option<fn (sender : &CBFS, e : &mut CBFSCleanupFileEventArgs) >,
    // onCloseDirectoryEnumeration : Option<&'a dyn CBFSCloseDirectoryEnumerationEvent>,
    onCloseDirectoryEnumeration : Option<fn (sender : &CBFS, e : &mut CBFSCloseDirectoryEnumerationEventArgs) >,
    // onCloseFile : Option<&'a dyn CBFSCloseFileEvent>,
    onCloseFile : Option<fn (sender : &CBFS, e : &mut CBFSCloseFileEventArgs) >,
    // onCloseHardLinksEnumeration : Option<&'a dyn CBFSCloseHardLinksEnumerationEvent>,
    onCloseHardLinksEnumeration : Option<fn (sender : &CBFS, e : &mut CBFSCloseHardLinksEnumerationEventArgs) >,
    // onCloseNamedStreamsEnumeration : Option<&'a dyn CBFSCloseNamedStreamsEnumerationEvent>,
    onCloseNamedStreamsEnumeration : Option<fn (sender : &CBFS, e : &mut CBFSCloseNamedStreamsEnumerationEventArgs) >,
    // onCloseQuotasEnumeration : Option<&'a dyn CBFSCloseQuotasEnumerationEvent>,
    onCloseQuotasEnumeration : Option<fn (sender : &CBFS, e : &mut CBFSCloseQuotasEnumerationEventArgs) >,
    // onCreateFile : Option<&'a dyn CBFSCreateFileEvent>,
    onCreateFile : Option<fn (sender : &CBFS, e : &mut CBFSCreateFileEventArgs) >,
    // onCreateHardLink : Option<&'a dyn CBFSCreateHardLinkEvent>,
    onCreateHardLink : Option<fn (sender : &CBFS, e : &mut CBFSCreateHardLinkEventArgs) >,
    // onDeleteFile : Option<&'a dyn CBFSDeleteFileEvent>,
    onDeleteFile : Option<fn (sender : &CBFS, e : &mut CBFSDeleteFileEventArgs) >,
    // onDeleteObjectId : Option<&'a dyn CBFSDeleteObjectIdEvent>,
    onDeleteObjectId : Option<fn (sender : &CBFS, e : &mut CBFSDeleteObjectIdEventArgs) >,
    // onDeleteReparsePoint : Option<&'a dyn CBFSDeleteReparsePointEvent>,
    onDeleteReparsePoint : Option<fn (sender : &CBFS, e : &mut CBFSDeleteReparsePointEventArgs) >,
    // onEjected : Option<&'a dyn CBFSEjectedEvent>,
    onEjected : Option<fn (sender : &CBFS, e : &mut CBFSEjectedEventArgs) >,
    // onEnumerateDirectory : Option<&'a dyn CBFSEnumerateDirectoryEvent>,
    onEnumerateDirectory : Option<fn (sender : &CBFS, e : &mut CBFSEnumerateDirectoryEventArgs) >,
    // onEnumerateHardLinks : Option<&'a dyn CBFSEnumerateHardLinksEvent>,
    onEnumerateHardLinks : Option<fn (sender : &CBFS, e : &mut CBFSEnumerateHardLinksEventArgs) >,
    // onEnumerateNamedStreams : Option<&'a dyn CBFSEnumerateNamedStreamsEvent>,
    onEnumerateNamedStreams : Option<fn (sender : &CBFS, e : &mut CBFSEnumerateNamedStreamsEventArgs) >,
    // onError : Option<&'a dyn CBFSErrorEvent>,
    onError : Option<fn (sender : &CBFS, e : &mut CBFSErrorEventArgs) >,
    // onFlushFile : Option<&'a dyn CBFSFlushFileEvent>,
    onFlushFile : Option<fn (sender : &CBFS, e : &mut CBFSFlushFileEventArgs) >,
    // onFsctl : Option<&'a dyn CBFSFsctlEvent>,
    onFsctl : Option<fn (sender : &CBFS, e : &mut CBFSFsctlEventArgs) >,
    // onGetDefaultQuotaInfo : Option<&'a dyn CBFSGetDefaultQuotaInfoEvent>,
    onGetDefaultQuotaInfo : Option<fn (sender : &CBFS, e : &mut CBFSGetDefaultQuotaInfoEventArgs) >,
    // onGetFileInfo : Option<&'a dyn CBFSGetFileInfoEvent>,
    onGetFileInfo : Option<fn (sender : &CBFS, e : &mut CBFSGetFileInfoEventArgs) >,
    // onGetFileNameByFileId : Option<&'a dyn CBFSGetFileNameByFileIdEvent>,
    onGetFileNameByFileId : Option<fn (sender : &CBFS, e : &mut CBFSGetFileNameByFileIdEventArgs) >,
    // onGetFileSecurity : Option<&'a dyn CBFSGetFileSecurityEvent>,
    onGetFileSecurity : Option<fn (sender : &CBFS, e : &mut CBFSGetFileSecurityEventArgs) >,
    // onGetObjectId : Option<&'a dyn CBFSGetObjectIdEvent>,
    onGetObjectId : Option<fn (sender : &CBFS, e : &mut CBFSGetObjectIdEventArgs) >,
    // onGetReparsePoint : Option<&'a dyn CBFSGetReparsePointEvent>,
    onGetReparsePoint : Option<fn (sender : &CBFS, e : &mut CBFSGetReparsePointEventArgs) >,
    // onGetVolumeId : Option<&'a dyn CBFSGetVolumeIdEvent>,
    onGetVolumeId : Option<fn (sender : &CBFS, e : &mut CBFSGetVolumeIdEventArgs) >,
    // onGetVolumeLabel : Option<&'a dyn CBFSGetVolumeLabelEvent>,
    onGetVolumeLabel : Option<fn (sender : &CBFS, e : &mut CBFSGetVolumeLabelEventArgs) >,
    // onGetVolumeSize : Option<&'a dyn CBFSGetVolumeSizeEvent>,
    onGetVolumeSize : Option<fn (sender : &CBFS, e : &mut CBFSGetVolumeSizeEventArgs) >,
    // onIoctl : Option<&'a dyn CBFSIoctlEvent>,
    onIoctl : Option<fn (sender : &CBFS, e : &mut CBFSIoctlEventArgs) >,
    // onIsDirectoryEmpty : Option<&'a dyn CBFSIsDirectoryEmptyEvent>,
    onIsDirectoryEmpty : Option<fn (sender : &CBFS, e : &mut CBFSIsDirectoryEmptyEventArgs) >,
    // onLockFile : Option<&'a dyn CBFSLockFileEvent>,
    onLockFile : Option<fn (sender : &CBFS, e : &mut CBFSLockFileEventArgs) >,
    // onMount : Option<&'a dyn CBFSMountEvent>,
    onMount : Option<fn (sender : &CBFS, e : &mut CBFSMountEventArgs) >,
    // onOffloadReadFile : Option<&'a dyn CBFSOffloadReadFileEvent>,
    onOffloadReadFile : Option<fn (sender : &CBFS, e : &mut CBFSOffloadReadFileEventArgs) >,
    // onOffloadWriteFile : Option<&'a dyn CBFSOffloadWriteFileEvent>,
    onOffloadWriteFile : Option<fn (sender : &CBFS, e : &mut CBFSOffloadWriteFileEventArgs) >,
    // onOpenFile : Option<&'a dyn CBFSOpenFileEvent>,
    onOpenFile : Option<fn (sender : &CBFS, e : &mut CBFSOpenFileEventArgs) >,
    // onQueryAllocatedRanges : Option<&'a dyn CBFSQueryAllocatedRangesEvent>,
    onQueryAllocatedRanges : Option<fn (sender : &CBFS, e : &mut CBFSQueryAllocatedRangesEventArgs) >,
    // onQueryCompressionInfo : Option<&'a dyn CBFSQueryCompressionInfoEvent>,
    onQueryCompressionInfo : Option<fn (sender : &CBFS, e : &mut CBFSQueryCompressionInfoEventArgs) >,
    // onQueryEa : Option<&'a dyn CBFSQueryEaEvent>,
    onQueryEa : Option<fn (sender : &CBFS, e : &mut CBFSQueryEaEventArgs) >,
    // onQueryQuotas : Option<&'a dyn CBFSQueryQuotasEvent>,
    onQueryQuotas : Option<fn (sender : &CBFS, e : &mut CBFSQueryQuotasEventArgs) >,
    // onReadFile : Option<&'a dyn CBFSReadFileEvent>,
    onReadFile : Option<fn (sender : &CBFS, e : &mut CBFSReadFileEventArgs) >,
    // onRenameOrMoveFile : Option<&'a dyn CBFSRenameOrMoveFileEvent>,
    onRenameOrMoveFile : Option<fn (sender : &CBFS, e : &mut CBFSRenameOrMoveFileEventArgs) >,
    // onSetAllocationSize : Option<&'a dyn CBFSSetAllocationSizeEvent>,
    onSetAllocationSize : Option<fn (sender : &CBFS, e : &mut CBFSSetAllocationSizeEventArgs) >,
    // onSetDefaultQuotaInfo : Option<&'a dyn CBFSSetDefaultQuotaInfoEvent>,
    onSetDefaultQuotaInfo : Option<fn (sender : &CBFS, e : &mut CBFSSetDefaultQuotaInfoEventArgs) >,
    // onSetEa : Option<&'a dyn CBFSSetEaEvent>,
    onSetEa : Option<fn (sender : &CBFS, e : &mut CBFSSetEaEventArgs) >,
    // onSetFileAttributes : Option<&'a dyn CBFSSetFileAttributesEvent>,
    onSetFileAttributes : Option<fn (sender : &CBFS, e : &mut CBFSSetFileAttributesEventArgs) >,
    // onSetFileSecurity : Option<&'a dyn CBFSSetFileSecurityEvent>,
    onSetFileSecurity : Option<fn (sender : &CBFS, e : &mut CBFSSetFileSecurityEventArgs) >,
    // onSetFileSize : Option<&'a dyn CBFSSetFileSizeEvent>,
    onSetFileSize : Option<fn (sender : &CBFS, e : &mut CBFSSetFileSizeEventArgs) >,
    // onSetObjectId : Option<&'a dyn CBFSSetObjectIdEvent>,
    onSetObjectId : Option<fn (sender : &CBFS, e : &mut CBFSSetObjectIdEventArgs) >,
    // onSetQuotas : Option<&'a dyn CBFSSetQuotasEvent>,
    onSetQuotas : Option<fn (sender : &CBFS, e : &mut CBFSSetQuotasEventArgs) >,
    // onSetReparsePoint : Option<&'a dyn CBFSSetReparsePointEvent>,
    onSetReparsePoint : Option<fn (sender : &CBFS, e : &mut CBFSSetReparsePointEventArgs) >,
    // onSetValidDataLength : Option<&'a dyn CBFSSetValidDataLengthEvent>,
    onSetValidDataLength : Option<fn (sender : &CBFS, e : &mut CBFSSetValidDataLengthEventArgs) >,
    // onSetVolumeLabel : Option<&'a dyn CBFSSetVolumeLabelEvent>,
    onSetVolumeLabel : Option<fn (sender : &CBFS, e : &mut CBFSSetVolumeLabelEventArgs) >,
    // onUnlockFile : Option<&'a dyn CBFSUnlockFileEvent>,
    onUnlockFile : Option<fn (sender : &CBFS, e : &mut CBFSUnlockFileEventArgs) >,
    // onUnmount : Option<&'a dyn CBFSUnmountEvent>,
    onUnmount : Option<fn (sender : &CBFS, e : &mut CBFSUnmountEventArgs) >,
    // onWorkerThreadCreation : Option<&'a dyn CBFSWorkerThreadCreationEvent>,
    onWorkerThreadCreation : Option<fn (sender : &CBFS, e : &mut CBFSWorkerThreadCreationEventArgs) >,
    // onWorkerThreadTermination : Option<&'a dyn CBFSWorkerThreadTerminationEvent>,
    onWorkerThreadTermination : Option<fn (sender : &CBFS, e : &mut CBFSWorkerThreadTerminationEventArgs) >,
    // onWriteFile : Option<&'a dyn CBFSWriteFileEvent>,
    onWriteFile : Option<fn (sender : &CBFS, e : &mut CBFSWriteFileEventArgs) >,
    // onZeroizeFileRange : Option<&'a dyn CBFSZeroizeFileRangeEvent>,
    onZeroizeFileRange : Option<fn (sender : &CBFS, e : &mut CBFSZeroizeFileRangeEventArgs) >,

    Id : usize,
    Handle : usize 
}

//impl<'a> Drop for CBFS<'a>
impl Drop for CBFS
{
    fn drop(&mut self)
    {
        self.dispose();
    }
}

impl CBFS
{
    pub fn new() -> &'static mut CBFS
    {        
        #[cfg(target_os = "linux")]
        panic!("CBFS is not available on Linux");
        #[cfg(target_os = "macos")]
        panic!("CBFS is not available on macOS");
         #[cfg(target_os = "android")]
         panic!("CBFS is not available on Android");
        #[cfg(target_os = "ios")]
        panic!("CBFS is not available on iOS");
        unsafe
        {
            if !lib_loaded
            {
                init_on_demand();
            }
        }
        
        let ret_code : i32;
        let lId : usize;
        unsafe
        {
            lId = CBFSIDSeed.fetch_add(1, SeqCst) as usize;
        }

        let lHandle : isize;
        unsafe
        {
            let callable = CBFSConnect_CBFS_Create.clone().unwrap();
            lHandle = callable(CBFSEventDispatcher, lId, std::ptr::null(), CBFSCreateOpt) as isize;
        }
        if lHandle < 0
        {
            panic!("Failed to instantiate CBFS. Please verify that it is supported on this platform");
        }

        let result : CBFS = CBFS
        {
            onCanFileBeDeleted: None,
            onCleanupFile: None,
            onCloseDirectoryEnumeration: None,
            onCloseFile: None,
            onCloseHardLinksEnumeration: None,
            onCloseNamedStreamsEnumeration: None,
            onCloseQuotasEnumeration: None,
            onCreateFile: None,
            onCreateHardLink: None,
            onDeleteFile: None,
            onDeleteObjectId: None,
            onDeleteReparsePoint: None,
            onEjected: None,
            onEnumerateDirectory: None,
            onEnumerateHardLinks: None,
            onEnumerateNamedStreams: None,
            onError: None,
            onFlushFile: None,
            onFsctl: None,
            onGetDefaultQuotaInfo: None,
            onGetFileInfo: None,
            onGetFileNameByFileId: None,
            onGetFileSecurity: None,
            onGetObjectId: None,
            onGetReparsePoint: None,
            onGetVolumeId: None,
            onGetVolumeLabel: None,
            onGetVolumeSize: None,
            onIoctl: None,
            onIsDirectoryEmpty: None,
            onLockFile: None,
            onMount: None,
            onOffloadReadFile: None,
            onOffloadWriteFile: None,
            onOpenFile: None,
            onQueryAllocatedRanges: None,
            onQueryCompressionInfo: None,
            onQueryEa: None,
            onQueryQuotas: None,
            onReadFile: None,
            onRenameOrMoveFile: None,
            onSetAllocationSize: None,
            onSetDefaultQuotaInfo: None,
            onSetEa: None,
            onSetFileAttributes: None,
            onSetFileSecurity: None,
            onSetFileSize: None,
            onSetObjectId: None,
            onSetQuotas: None,
            onSetReparsePoint: None,
            onSetValidDataLength: None,
            onSetVolumeLabel: None,
            onUnlockFile: None,
            onUnmount: None,
            onWorkerThreadCreation: None,
            onWorkerThreadTermination: None,
            onWriteFile: None,
            onZeroizeFileRange: None,
            Id: lId,
            Handle: lHandle as usize
        };

        let oem_key = CString::new(cbfsconnectkey::rtkCBFSConnect).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();

        unsafe
        {
            let callable = CBFSConnect_CBFS_SetCStr.clone().unwrap();
            ret_code = callable(lHandle as usize, 8012/*PID_KEYCHECK_RUST*/, 0, oem_key_ptr, 0) as i32;
            let _ = CString::from_raw(oem_key_ptr);
        }
        if ret_code != 0
        {
            panic!("Initialization of CBFS has failed with error {}", ret_code);
        }

        // Lock the Mutex to get access to the HashMap
        unsafe
        {
            let _map = CBFSDictMutex.lock().unwrap(); // It is used as a synchronization primitive - don't touch
            CBFSDict.insert(lId, result);
            let res = CBFSDict.get_mut(&lId).unwrap();
            return res;
        } // The lock is automatically released here
    }

    pub fn dispose(&self)
    {
        let mut _aself : Option<CBFS>;
        unsafe
        {
            let _map = CBFSDictMutex.lock().unwrap(); // It is used as a synchronization primitive - don't touch

            if !CBFSDict.contains_key(&self.Id)
            {
                return;
            }

            // Remove itself from the list
            _aself = CBFSDict.remove(&self.Id);

            // finalize the ctlclass
            let callable = CBFSConnect_CBFS_Destroy.clone().unwrap();
            callable(self.Handle);
        }
    }

/////////
// Events
/////////

    fn fire_can_file_be_deleted(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCanFileBeDeleted
        {
            let mut args : CBFSCanFileBeDeletedEventArgs = CBFSCanFileBeDeletedEventArgs::new(par, cbpar);
            callable/*.on_can_file_be_deleted*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_can_file_be_deleted(&self) -> &'a dyn CBFSCanFileBeDeletedEvent
    pub fn on_can_file_be_deleted(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSCanFileBeDeletedEventArgs)>
    {
        self.onCanFileBeDeleted
    }

    //pub fn set_on_can_file_be_deleted(&mut self, value : &'a dyn CBFSCanFileBeDeletedEvent)
    pub fn set_on_can_file_be_deleted(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSCanFileBeDeletedEventArgs)>)
    {
        self.onCanFileBeDeleted = value;
    }

    fn fire_cleanup_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCleanupFile
        {
            let mut args : CBFSCleanupFileEventArgs = CBFSCleanupFileEventArgs::new(par, cbpar);
            callable/*.on_cleanup_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_cleanup_file(&self) -> &'a dyn CBFSCleanupFileEvent
    pub fn on_cleanup_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSCleanupFileEventArgs)>
    {
        self.onCleanupFile
    }

    //pub fn set_on_cleanup_file(&mut self, value : &'a dyn CBFSCleanupFileEvent)
    pub fn set_on_cleanup_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSCleanupFileEventArgs)>)
    {
        self.onCleanupFile = value;
    }

    fn fire_close_directory_enumeration(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCloseDirectoryEnumeration
        {
            let mut args : CBFSCloseDirectoryEnumerationEventArgs = CBFSCloseDirectoryEnumerationEventArgs::new(par, cbpar);
            callable/*.on_close_directory_enumeration*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_close_directory_enumeration(&self) -> &'a dyn CBFSCloseDirectoryEnumerationEvent
    pub fn on_close_directory_enumeration(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSCloseDirectoryEnumerationEventArgs)>
    {
        self.onCloseDirectoryEnumeration
    }

    //pub fn set_on_close_directory_enumeration(&mut self, value : &'a dyn CBFSCloseDirectoryEnumerationEvent)
    pub fn set_on_close_directory_enumeration(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSCloseDirectoryEnumerationEventArgs)>)
    {
        self.onCloseDirectoryEnumeration = value;
    }

    fn fire_close_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCloseFile
        {
            let mut args : CBFSCloseFileEventArgs = CBFSCloseFileEventArgs::new(par, cbpar);
            callable/*.on_close_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_close_file(&self) -> &'a dyn CBFSCloseFileEvent
    pub fn on_close_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSCloseFileEventArgs)>
    {
        self.onCloseFile
    }

    //pub fn set_on_close_file(&mut self, value : &'a dyn CBFSCloseFileEvent)
    pub fn set_on_close_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSCloseFileEventArgs)>)
    {
        self.onCloseFile = value;
    }

    fn fire_close_hard_links_enumeration(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCloseHardLinksEnumeration
        {
            let mut args : CBFSCloseHardLinksEnumerationEventArgs = CBFSCloseHardLinksEnumerationEventArgs::new(par, cbpar);
            callable/*.on_close_hard_links_enumeration*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_close_hard_links_enumeration(&self) -> &'a dyn CBFSCloseHardLinksEnumerationEvent
    pub fn on_close_hard_links_enumeration(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSCloseHardLinksEnumerationEventArgs)>
    {
        self.onCloseHardLinksEnumeration
    }

    //pub fn set_on_close_hard_links_enumeration(&mut self, value : &'a dyn CBFSCloseHardLinksEnumerationEvent)
    pub fn set_on_close_hard_links_enumeration(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSCloseHardLinksEnumerationEventArgs)>)
    {
        self.onCloseHardLinksEnumeration = value;
    }

    fn fire_close_named_streams_enumeration(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCloseNamedStreamsEnumeration
        {
            let mut args : CBFSCloseNamedStreamsEnumerationEventArgs = CBFSCloseNamedStreamsEnumerationEventArgs::new(par, cbpar);
            callable/*.on_close_named_streams_enumeration*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_close_named_streams_enumeration(&self) -> &'a dyn CBFSCloseNamedStreamsEnumerationEvent
    pub fn on_close_named_streams_enumeration(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSCloseNamedStreamsEnumerationEventArgs)>
    {
        self.onCloseNamedStreamsEnumeration
    }

    //pub fn set_on_close_named_streams_enumeration(&mut self, value : &'a dyn CBFSCloseNamedStreamsEnumerationEvent)
    pub fn set_on_close_named_streams_enumeration(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSCloseNamedStreamsEnumerationEventArgs)>)
    {
        self.onCloseNamedStreamsEnumeration = value;
    }

    fn fire_close_quotas_enumeration(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCloseQuotasEnumeration
        {
            let mut args : CBFSCloseQuotasEnumerationEventArgs = CBFSCloseQuotasEnumerationEventArgs::new(par, cbpar);
            callable/*.on_close_quotas_enumeration*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_close_quotas_enumeration(&self) -> &'a dyn CBFSCloseQuotasEnumerationEvent
    pub fn on_close_quotas_enumeration(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSCloseQuotasEnumerationEventArgs)>
    {
        self.onCloseQuotasEnumeration
    }

    //pub fn set_on_close_quotas_enumeration(&mut self, value : &'a dyn CBFSCloseQuotasEnumerationEvent)
    pub fn set_on_close_quotas_enumeration(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSCloseQuotasEnumerationEventArgs)>)
    {
        self.onCloseQuotasEnumeration = value;
    }

    fn fire_create_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCreateFile
        {
            let mut args : CBFSCreateFileEventArgs = CBFSCreateFileEventArgs::new(par, cbpar);
            callable/*.on_create_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_create_file(&self) -> &'a dyn CBFSCreateFileEvent
    pub fn on_create_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSCreateFileEventArgs)>
    {
        self.onCreateFile
    }

    //pub fn set_on_create_file(&mut self, value : &'a dyn CBFSCreateFileEvent)
    pub fn set_on_create_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSCreateFileEventArgs)>)
    {
        self.onCreateFile = value;
    }

    fn fire_create_hard_link(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCreateHardLink
        {
            let mut args : CBFSCreateHardLinkEventArgs = CBFSCreateHardLinkEventArgs::new(par, cbpar);
            callable/*.on_create_hard_link*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_create_hard_link(&self) -> &'a dyn CBFSCreateHardLinkEvent
    pub fn on_create_hard_link(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSCreateHardLinkEventArgs)>
    {
        self.onCreateHardLink
    }

    //pub fn set_on_create_hard_link(&mut self, value : &'a dyn CBFSCreateHardLinkEvent)
    pub fn set_on_create_hard_link(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSCreateHardLinkEventArgs)>)
    {
        self.onCreateHardLink = value;
    }

    fn fire_delete_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDeleteFile
        {
            let mut args : CBFSDeleteFileEventArgs = CBFSDeleteFileEventArgs::new(par, cbpar);
            callable/*.on_delete_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_delete_file(&self) -> &'a dyn CBFSDeleteFileEvent
    pub fn on_delete_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSDeleteFileEventArgs)>
    {
        self.onDeleteFile
    }

    //pub fn set_on_delete_file(&mut self, value : &'a dyn CBFSDeleteFileEvent)
    pub fn set_on_delete_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSDeleteFileEventArgs)>)
    {
        self.onDeleteFile = value;
    }

    fn fire_delete_object_id(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDeleteObjectId
        {
            let mut args : CBFSDeleteObjectIdEventArgs = CBFSDeleteObjectIdEventArgs::new(par, cbpar);
            callable/*.on_delete_object_id*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_delete_object_id(&self) -> &'a dyn CBFSDeleteObjectIdEvent
    pub fn on_delete_object_id(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSDeleteObjectIdEventArgs)>
    {
        self.onDeleteObjectId
    }

    //pub fn set_on_delete_object_id(&mut self, value : &'a dyn CBFSDeleteObjectIdEvent)
    pub fn set_on_delete_object_id(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSDeleteObjectIdEventArgs)>)
    {
        self.onDeleteObjectId = value;
    }

    fn fire_delete_reparse_point(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDeleteReparsePoint
        {
            let mut args : CBFSDeleteReparsePointEventArgs = CBFSDeleteReparsePointEventArgs::new(par, cbpar);
            callable/*.on_delete_reparse_point*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_delete_reparse_point(&self) -> &'a dyn CBFSDeleteReparsePointEvent
    pub fn on_delete_reparse_point(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSDeleteReparsePointEventArgs)>
    {
        self.onDeleteReparsePoint
    }

    //pub fn set_on_delete_reparse_point(&mut self, value : &'a dyn CBFSDeleteReparsePointEvent)
    pub fn set_on_delete_reparse_point(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSDeleteReparsePointEventArgs)>)
    {
        self.onDeleteReparsePoint = value;
    }

    fn fire_ejected(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onEjected
        {
            let mut args : CBFSEjectedEventArgs = CBFSEjectedEventArgs::new(par, cbpar);
            callable/*.on_ejected*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_ejected(&self) -> &'a dyn CBFSEjectedEvent
    pub fn on_ejected(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSEjectedEventArgs)>
    {
        self.onEjected
    }

    //pub fn set_on_ejected(&mut self, value : &'a dyn CBFSEjectedEvent)
    pub fn set_on_ejected(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSEjectedEventArgs)>)
    {
        self.onEjected = value;
    }

    fn fire_enumerate_directory(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onEnumerateDirectory
        {
            let mut args : CBFSEnumerateDirectoryEventArgs = CBFSEnumerateDirectoryEventArgs::new(par, cbpar);
            callable/*.on_enumerate_directory*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_enumerate_directory(&self) -> &'a dyn CBFSEnumerateDirectoryEvent
    pub fn on_enumerate_directory(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSEnumerateDirectoryEventArgs)>
    {
        self.onEnumerateDirectory
    }

    //pub fn set_on_enumerate_directory(&mut self, value : &'a dyn CBFSEnumerateDirectoryEvent)
    pub fn set_on_enumerate_directory(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSEnumerateDirectoryEventArgs)>)
    {
        self.onEnumerateDirectory = value;
    }

    fn fire_enumerate_hard_links(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onEnumerateHardLinks
        {
            let mut args : CBFSEnumerateHardLinksEventArgs = CBFSEnumerateHardLinksEventArgs::new(par, cbpar);
            callable/*.on_enumerate_hard_links*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_enumerate_hard_links(&self) -> &'a dyn CBFSEnumerateHardLinksEvent
    pub fn on_enumerate_hard_links(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSEnumerateHardLinksEventArgs)>
    {
        self.onEnumerateHardLinks
    }

    //pub fn set_on_enumerate_hard_links(&mut self, value : &'a dyn CBFSEnumerateHardLinksEvent)
    pub fn set_on_enumerate_hard_links(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSEnumerateHardLinksEventArgs)>)
    {
        self.onEnumerateHardLinks = value;
    }

    fn fire_enumerate_named_streams(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onEnumerateNamedStreams
        {
            let mut args : CBFSEnumerateNamedStreamsEventArgs = CBFSEnumerateNamedStreamsEventArgs::new(par, cbpar);
            callable/*.on_enumerate_named_streams*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_enumerate_named_streams(&self) -> &'a dyn CBFSEnumerateNamedStreamsEvent
    pub fn on_enumerate_named_streams(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSEnumerateNamedStreamsEventArgs)>
    {
        self.onEnumerateNamedStreams
    }

    //pub fn set_on_enumerate_named_streams(&mut self, value : &'a dyn CBFSEnumerateNamedStreamsEvent)
    pub fn set_on_enumerate_named_streams(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSEnumerateNamedStreamsEventArgs)>)
    {
        self.onEnumerateNamedStreams = value;
    }

    fn fire_error(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onError
        {
            let mut args : CBFSErrorEventArgs = CBFSErrorEventArgs::new(par, cbpar);
            callable/*.on_error*/(&self, &mut args);
        }
    }

    //pub fn on_error(&self) -> &'a dyn CBFSErrorEvent
    pub fn on_error(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSErrorEventArgs)>
    {
        self.onError
    }

    //pub fn set_on_error(&mut self, value : &'a dyn CBFSErrorEvent)
    pub fn set_on_error(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSErrorEventArgs)>)
    {
        self.onError = value;
    }

    fn fire_flush_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFlushFile
        {
            let mut args : CBFSFlushFileEventArgs = CBFSFlushFileEventArgs::new(par, cbpar);
            callable/*.on_flush_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_flush_file(&self) -> &'a dyn CBFSFlushFileEvent
    pub fn on_flush_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSFlushFileEventArgs)>
    {
        self.onFlushFile
    }

    //pub fn set_on_flush_file(&mut self, value : &'a dyn CBFSFlushFileEvent)
    pub fn set_on_flush_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSFlushFileEventArgs)>)
    {
        self.onFlushFile = value;
    }

    fn fire_fsctl(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFsctl
        {
            let mut args : CBFSFsctlEventArgs = CBFSFsctlEventArgs::new(par, cbpar);
            callable/*.on_fsctl*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_fsctl(&self) -> &'a dyn CBFSFsctlEvent
    pub fn on_fsctl(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSFsctlEventArgs)>
    {
        self.onFsctl
    }

    //pub fn set_on_fsctl(&mut self, value : &'a dyn CBFSFsctlEvent)
    pub fn set_on_fsctl(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSFsctlEventArgs)>)
    {
        self.onFsctl = value;
    }

    fn fire_get_default_quota_info(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onGetDefaultQuotaInfo
        {
            let mut args : CBFSGetDefaultQuotaInfoEventArgs = CBFSGetDefaultQuotaInfoEventArgs::new(par, cbpar);
            callable/*.on_get_default_quota_info*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_get_default_quota_info(&self) -> &'a dyn CBFSGetDefaultQuotaInfoEvent
    pub fn on_get_default_quota_info(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSGetDefaultQuotaInfoEventArgs)>
    {
        self.onGetDefaultQuotaInfo
    }

    //pub fn set_on_get_default_quota_info(&mut self, value : &'a dyn CBFSGetDefaultQuotaInfoEvent)
    pub fn set_on_get_default_quota_info(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSGetDefaultQuotaInfoEventArgs)>)
    {
        self.onGetDefaultQuotaInfo = value;
    }

    fn fire_get_file_info(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onGetFileInfo
        {
            let mut args : CBFSGetFileInfoEventArgs = CBFSGetFileInfoEventArgs::new(par, cbpar);
            callable/*.on_get_file_info*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_get_file_info(&self) -> &'a dyn CBFSGetFileInfoEvent
    pub fn on_get_file_info(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSGetFileInfoEventArgs)>
    {
        self.onGetFileInfo
    }

    //pub fn set_on_get_file_info(&mut self, value : &'a dyn CBFSGetFileInfoEvent)
    pub fn set_on_get_file_info(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSGetFileInfoEventArgs)>)
    {
        self.onGetFileInfo = value;
    }

    fn fire_get_file_name_by_file_id(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onGetFileNameByFileId
        {
            let mut args : CBFSGetFileNameByFileIdEventArgs = CBFSGetFileNameByFileIdEventArgs::new(par, cbpar);
            callable/*.on_get_file_name_by_file_id*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_get_file_name_by_file_id(&self) -> &'a dyn CBFSGetFileNameByFileIdEvent
    pub fn on_get_file_name_by_file_id(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSGetFileNameByFileIdEventArgs)>
    {
        self.onGetFileNameByFileId
    }

    //pub fn set_on_get_file_name_by_file_id(&mut self, value : &'a dyn CBFSGetFileNameByFileIdEvent)
    pub fn set_on_get_file_name_by_file_id(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSGetFileNameByFileIdEventArgs)>)
    {
        self.onGetFileNameByFileId = value;
    }

    fn fire_get_file_security(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onGetFileSecurity
        {
            let mut args : CBFSGetFileSecurityEventArgs = CBFSGetFileSecurityEventArgs::new(par, cbpar);
            callable/*.on_get_file_security*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_get_file_security(&self) -> &'a dyn CBFSGetFileSecurityEvent
    pub fn on_get_file_security(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSGetFileSecurityEventArgs)>
    {
        self.onGetFileSecurity
    }

    //pub fn set_on_get_file_security(&mut self, value : &'a dyn CBFSGetFileSecurityEvent)
    pub fn set_on_get_file_security(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSGetFileSecurityEventArgs)>)
    {
        self.onGetFileSecurity = value;
    }

    fn fire_get_object_id(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onGetObjectId
        {
            let mut args : CBFSGetObjectIdEventArgs = CBFSGetObjectIdEventArgs::new(par, cbpar);
            callable/*.on_get_object_id*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_get_object_id(&self) -> &'a dyn CBFSGetObjectIdEvent
    pub fn on_get_object_id(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSGetObjectIdEventArgs)>
    {
        self.onGetObjectId
    }

    //pub fn set_on_get_object_id(&mut self, value : &'a dyn CBFSGetObjectIdEvent)
    pub fn set_on_get_object_id(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSGetObjectIdEventArgs)>)
    {
        self.onGetObjectId = value;
    }

    fn fire_get_reparse_point(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onGetReparsePoint
        {
            let mut args : CBFSGetReparsePointEventArgs = CBFSGetReparsePointEventArgs::new(par, cbpar);
            callable/*.on_get_reparse_point*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_get_reparse_point(&self) -> &'a dyn CBFSGetReparsePointEvent
    pub fn on_get_reparse_point(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSGetReparsePointEventArgs)>
    {
        self.onGetReparsePoint
    }

    //pub fn set_on_get_reparse_point(&mut self, value : &'a dyn CBFSGetReparsePointEvent)
    pub fn set_on_get_reparse_point(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSGetReparsePointEventArgs)>)
    {
        self.onGetReparsePoint = value;
    }

    fn fire_get_volume_id(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onGetVolumeId
        {
            let mut args : CBFSGetVolumeIdEventArgs = CBFSGetVolumeIdEventArgs::new(par, cbpar);
            callable/*.on_get_volume_id*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_get_volume_id(&self) -> &'a dyn CBFSGetVolumeIdEvent
    pub fn on_get_volume_id(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSGetVolumeIdEventArgs)>
    {
        self.onGetVolumeId
    }

    //pub fn set_on_get_volume_id(&mut self, value : &'a dyn CBFSGetVolumeIdEvent)
    pub fn set_on_get_volume_id(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSGetVolumeIdEventArgs)>)
    {
        self.onGetVolumeId = value;
    }

    fn fire_get_volume_label(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onGetVolumeLabel
        {
            let mut args : CBFSGetVolumeLabelEventArgs = CBFSGetVolumeLabelEventArgs::new(par, cbpar);
            callable/*.on_get_volume_label*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_get_volume_label(&self) -> &'a dyn CBFSGetVolumeLabelEvent
    pub fn on_get_volume_label(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSGetVolumeLabelEventArgs)>
    {
        self.onGetVolumeLabel
    }

    //pub fn set_on_get_volume_label(&mut self, value : &'a dyn CBFSGetVolumeLabelEvent)
    pub fn set_on_get_volume_label(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSGetVolumeLabelEventArgs)>)
    {
        self.onGetVolumeLabel = value;
    }

    fn fire_get_volume_size(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onGetVolumeSize
        {
            let mut args : CBFSGetVolumeSizeEventArgs = CBFSGetVolumeSizeEventArgs::new(par, cbpar);
            callable/*.on_get_volume_size*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_get_volume_size(&self) -> &'a dyn CBFSGetVolumeSizeEvent
    pub fn on_get_volume_size(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSGetVolumeSizeEventArgs)>
    {
        self.onGetVolumeSize
    }

    //pub fn set_on_get_volume_size(&mut self, value : &'a dyn CBFSGetVolumeSizeEvent)
    pub fn set_on_get_volume_size(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSGetVolumeSizeEventArgs)>)
    {
        self.onGetVolumeSize = value;
    }

    fn fire_ioctl(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onIoctl
        {
            let mut args : CBFSIoctlEventArgs = CBFSIoctlEventArgs::new(par, cbpar);
            callable/*.on_ioctl*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_ioctl(&self) -> &'a dyn CBFSIoctlEvent
    pub fn on_ioctl(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSIoctlEventArgs)>
    {
        self.onIoctl
    }

    //pub fn set_on_ioctl(&mut self, value : &'a dyn CBFSIoctlEvent)
    pub fn set_on_ioctl(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSIoctlEventArgs)>)
    {
        self.onIoctl = value;
    }

    fn fire_is_directory_empty(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onIsDirectoryEmpty
        {
            let mut args : CBFSIsDirectoryEmptyEventArgs = CBFSIsDirectoryEmptyEventArgs::new(par, cbpar);
            callable/*.on_is_directory_empty*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_is_directory_empty(&self) -> &'a dyn CBFSIsDirectoryEmptyEvent
    pub fn on_is_directory_empty(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSIsDirectoryEmptyEventArgs)>
    {
        self.onIsDirectoryEmpty
    }

    //pub fn set_on_is_directory_empty(&mut self, value : &'a dyn CBFSIsDirectoryEmptyEvent)
    pub fn set_on_is_directory_empty(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSIsDirectoryEmptyEventArgs)>)
    {
        self.onIsDirectoryEmpty = value;
    }

    fn fire_lock_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onLockFile
        {
            let mut args : CBFSLockFileEventArgs = CBFSLockFileEventArgs::new(par, cbpar);
            callable/*.on_lock_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_lock_file(&self) -> &'a dyn CBFSLockFileEvent
    pub fn on_lock_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSLockFileEventArgs)>
    {
        self.onLockFile
    }

    //pub fn set_on_lock_file(&mut self, value : &'a dyn CBFSLockFileEvent)
    pub fn set_on_lock_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSLockFileEventArgs)>)
    {
        self.onLockFile = value;
    }

    fn fire_mount(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onMount
        {
            let mut args : CBFSMountEventArgs = CBFSMountEventArgs::new(par, cbpar);
            callable/*.on_mount*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_mount(&self) -> &'a dyn CBFSMountEvent
    pub fn on_mount(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSMountEventArgs)>
    {
        self.onMount
    }

    //pub fn set_on_mount(&mut self, value : &'a dyn CBFSMountEvent)
    pub fn set_on_mount(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSMountEventArgs)>)
    {
        self.onMount = value;
    }

    fn fire_offload_read_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onOffloadReadFile
        {
            let mut args : CBFSOffloadReadFileEventArgs = CBFSOffloadReadFileEventArgs::new(par, cbpar);
            callable/*.on_offload_read_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_offload_read_file(&self) -> &'a dyn CBFSOffloadReadFileEvent
    pub fn on_offload_read_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSOffloadReadFileEventArgs)>
    {
        self.onOffloadReadFile
    }

    //pub fn set_on_offload_read_file(&mut self, value : &'a dyn CBFSOffloadReadFileEvent)
    pub fn set_on_offload_read_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSOffloadReadFileEventArgs)>)
    {
        self.onOffloadReadFile = value;
    }

    fn fire_offload_write_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onOffloadWriteFile
        {
            let mut args : CBFSOffloadWriteFileEventArgs = CBFSOffloadWriteFileEventArgs::new(par, cbpar);
            callable/*.on_offload_write_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_offload_write_file(&self) -> &'a dyn CBFSOffloadWriteFileEvent
    pub fn on_offload_write_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSOffloadWriteFileEventArgs)>
    {
        self.onOffloadWriteFile
    }

    //pub fn set_on_offload_write_file(&mut self, value : &'a dyn CBFSOffloadWriteFileEvent)
    pub fn set_on_offload_write_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSOffloadWriteFileEventArgs)>)
    {
        self.onOffloadWriteFile = value;
    }

    fn fire_open_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onOpenFile
        {
            let mut args : CBFSOpenFileEventArgs = CBFSOpenFileEventArgs::new(par, cbpar);
            callable/*.on_open_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_open_file(&self) -> &'a dyn CBFSOpenFileEvent
    pub fn on_open_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSOpenFileEventArgs)>
    {
        self.onOpenFile
    }

    //pub fn set_on_open_file(&mut self, value : &'a dyn CBFSOpenFileEvent)
    pub fn set_on_open_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSOpenFileEventArgs)>)
    {
        self.onOpenFile = value;
    }

    fn fire_query_allocated_ranges(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onQueryAllocatedRanges
        {
            let mut args : CBFSQueryAllocatedRangesEventArgs = CBFSQueryAllocatedRangesEventArgs::new(par, cbpar);
            callable/*.on_query_allocated_ranges*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_query_allocated_ranges(&self) -> &'a dyn CBFSQueryAllocatedRangesEvent
    pub fn on_query_allocated_ranges(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSQueryAllocatedRangesEventArgs)>
    {
        self.onQueryAllocatedRanges
    }

    //pub fn set_on_query_allocated_ranges(&mut self, value : &'a dyn CBFSQueryAllocatedRangesEvent)
    pub fn set_on_query_allocated_ranges(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSQueryAllocatedRangesEventArgs)>)
    {
        self.onQueryAllocatedRanges = value;
    }

    fn fire_query_compression_info(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onQueryCompressionInfo
        {
            let mut args : CBFSQueryCompressionInfoEventArgs = CBFSQueryCompressionInfoEventArgs::new(par, cbpar);
            callable/*.on_query_compression_info*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_query_compression_info(&self) -> &'a dyn CBFSQueryCompressionInfoEvent
    pub fn on_query_compression_info(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSQueryCompressionInfoEventArgs)>
    {
        self.onQueryCompressionInfo
    }

    //pub fn set_on_query_compression_info(&mut self, value : &'a dyn CBFSQueryCompressionInfoEvent)
    pub fn set_on_query_compression_info(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSQueryCompressionInfoEventArgs)>)
    {
        self.onQueryCompressionInfo = value;
    }

    fn fire_query_ea(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onQueryEa
        {
            let mut args : CBFSQueryEaEventArgs = CBFSQueryEaEventArgs::new(par, cbpar);
            callable/*.on_query_ea*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_query_ea(&self) -> &'a dyn CBFSQueryEaEvent
    pub fn on_query_ea(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSQueryEaEventArgs)>
    {
        self.onQueryEa
    }

    //pub fn set_on_query_ea(&mut self, value : &'a dyn CBFSQueryEaEvent)
    pub fn set_on_query_ea(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSQueryEaEventArgs)>)
    {
        self.onQueryEa = value;
    }

    fn fire_query_quotas(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onQueryQuotas
        {
            let mut args : CBFSQueryQuotasEventArgs = CBFSQueryQuotasEventArgs::new(par, cbpar);
            callable/*.on_query_quotas*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_query_quotas(&self) -> &'a dyn CBFSQueryQuotasEvent
    pub fn on_query_quotas(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSQueryQuotasEventArgs)>
    {
        self.onQueryQuotas
    }

    //pub fn set_on_query_quotas(&mut self, value : &'a dyn CBFSQueryQuotasEvent)
    pub fn set_on_query_quotas(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSQueryQuotasEventArgs)>)
    {
        self.onQueryQuotas = value;
    }

    fn fire_read_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onReadFile
        {
            let mut args : CBFSReadFileEventArgs = CBFSReadFileEventArgs::new(par, cbpar);
            callable/*.on_read_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_read_file(&self) -> &'a dyn CBFSReadFileEvent
    pub fn on_read_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSReadFileEventArgs)>
    {
        self.onReadFile
    }

    //pub fn set_on_read_file(&mut self, value : &'a dyn CBFSReadFileEvent)
    pub fn set_on_read_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSReadFileEventArgs)>)
    {
        self.onReadFile = value;
    }

    fn fire_rename_or_move_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onRenameOrMoveFile
        {
            let mut args : CBFSRenameOrMoveFileEventArgs = CBFSRenameOrMoveFileEventArgs::new(par, cbpar);
            callable/*.on_rename_or_move_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_rename_or_move_file(&self) -> &'a dyn CBFSRenameOrMoveFileEvent
    pub fn on_rename_or_move_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSRenameOrMoveFileEventArgs)>
    {
        self.onRenameOrMoveFile
    }

    //pub fn set_on_rename_or_move_file(&mut self, value : &'a dyn CBFSRenameOrMoveFileEvent)
    pub fn set_on_rename_or_move_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSRenameOrMoveFileEventArgs)>)
    {
        self.onRenameOrMoveFile = value;
    }

    fn fire_set_allocation_size(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onSetAllocationSize
        {
            let mut args : CBFSSetAllocationSizeEventArgs = CBFSSetAllocationSizeEventArgs::new(par, cbpar);
            callable/*.on_set_allocation_size*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_set_allocation_size(&self) -> &'a dyn CBFSSetAllocationSizeEvent
    pub fn on_set_allocation_size(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSSetAllocationSizeEventArgs)>
    {
        self.onSetAllocationSize
    }

    //pub fn set_on_set_allocation_size(&mut self, value : &'a dyn CBFSSetAllocationSizeEvent)
    pub fn set_on_set_allocation_size(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSSetAllocationSizeEventArgs)>)
    {
        self.onSetAllocationSize = value;
    }

    fn fire_set_default_quota_info(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onSetDefaultQuotaInfo
        {
            let mut args : CBFSSetDefaultQuotaInfoEventArgs = CBFSSetDefaultQuotaInfoEventArgs::new(par, cbpar);
            callable/*.on_set_default_quota_info*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_set_default_quota_info(&self) -> &'a dyn CBFSSetDefaultQuotaInfoEvent
    pub fn on_set_default_quota_info(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSSetDefaultQuotaInfoEventArgs)>
    {
        self.onSetDefaultQuotaInfo
    }

    //pub fn set_on_set_default_quota_info(&mut self, value : &'a dyn CBFSSetDefaultQuotaInfoEvent)
    pub fn set_on_set_default_quota_info(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSSetDefaultQuotaInfoEventArgs)>)
    {
        self.onSetDefaultQuotaInfo = value;
    }

    fn fire_set_ea(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onSetEa
        {
            let mut args : CBFSSetEaEventArgs = CBFSSetEaEventArgs::new(par, cbpar);
            callable/*.on_set_ea*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_set_ea(&self) -> &'a dyn CBFSSetEaEvent
    pub fn on_set_ea(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSSetEaEventArgs)>
    {
        self.onSetEa
    }

    //pub fn set_on_set_ea(&mut self, value : &'a dyn CBFSSetEaEvent)
    pub fn set_on_set_ea(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSSetEaEventArgs)>)
    {
        self.onSetEa = value;
    }

    fn fire_set_file_attributes(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onSetFileAttributes
        {
            let mut args : CBFSSetFileAttributesEventArgs = CBFSSetFileAttributesEventArgs::new(par, cbpar);
            callable/*.on_set_file_attributes*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_set_file_attributes(&self) -> &'a dyn CBFSSetFileAttributesEvent
    pub fn on_set_file_attributes(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSSetFileAttributesEventArgs)>
    {
        self.onSetFileAttributes
    }

    //pub fn set_on_set_file_attributes(&mut self, value : &'a dyn CBFSSetFileAttributesEvent)
    pub fn set_on_set_file_attributes(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSSetFileAttributesEventArgs)>)
    {
        self.onSetFileAttributes = value;
    }

    fn fire_set_file_security(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onSetFileSecurity
        {
            let mut args : CBFSSetFileSecurityEventArgs = CBFSSetFileSecurityEventArgs::new(par, cbpar);
            callable/*.on_set_file_security*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_set_file_security(&self) -> &'a dyn CBFSSetFileSecurityEvent
    pub fn on_set_file_security(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSSetFileSecurityEventArgs)>
    {
        self.onSetFileSecurity
    }

    //pub fn set_on_set_file_security(&mut self, value : &'a dyn CBFSSetFileSecurityEvent)
    pub fn set_on_set_file_security(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSSetFileSecurityEventArgs)>)
    {
        self.onSetFileSecurity = value;
    }

    fn fire_set_file_size(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onSetFileSize
        {
            let mut args : CBFSSetFileSizeEventArgs = CBFSSetFileSizeEventArgs::new(par, cbpar);
            callable/*.on_set_file_size*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_set_file_size(&self) -> &'a dyn CBFSSetFileSizeEvent
    pub fn on_set_file_size(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSSetFileSizeEventArgs)>
    {
        self.onSetFileSize
    }

    //pub fn set_on_set_file_size(&mut self, value : &'a dyn CBFSSetFileSizeEvent)
    pub fn set_on_set_file_size(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSSetFileSizeEventArgs)>)
    {
        self.onSetFileSize = value;
    }

    fn fire_set_object_id(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onSetObjectId
        {
            let mut args : CBFSSetObjectIdEventArgs = CBFSSetObjectIdEventArgs::new(par, cbpar);
            callable/*.on_set_object_id*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_set_object_id(&self) -> &'a dyn CBFSSetObjectIdEvent
    pub fn on_set_object_id(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSSetObjectIdEventArgs)>
    {
        self.onSetObjectId
    }

    //pub fn set_on_set_object_id(&mut self, value : &'a dyn CBFSSetObjectIdEvent)
    pub fn set_on_set_object_id(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSSetObjectIdEventArgs)>)
    {
        self.onSetObjectId = value;
    }

    fn fire_set_quotas(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onSetQuotas
        {
            let mut args : CBFSSetQuotasEventArgs = CBFSSetQuotasEventArgs::new(par, cbpar);
            callable/*.on_set_quotas*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_set_quotas(&self) -> &'a dyn CBFSSetQuotasEvent
    pub fn on_set_quotas(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSSetQuotasEventArgs)>
    {
        self.onSetQuotas
    }

    //pub fn set_on_set_quotas(&mut self, value : &'a dyn CBFSSetQuotasEvent)
    pub fn set_on_set_quotas(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSSetQuotasEventArgs)>)
    {
        self.onSetQuotas = value;
    }

    fn fire_set_reparse_point(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onSetReparsePoint
        {
            let mut args : CBFSSetReparsePointEventArgs = CBFSSetReparsePointEventArgs::new(par, cbpar);
            callable/*.on_set_reparse_point*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_set_reparse_point(&self) -> &'a dyn CBFSSetReparsePointEvent
    pub fn on_set_reparse_point(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSSetReparsePointEventArgs)>
    {
        self.onSetReparsePoint
    }

    //pub fn set_on_set_reparse_point(&mut self, value : &'a dyn CBFSSetReparsePointEvent)
    pub fn set_on_set_reparse_point(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSSetReparsePointEventArgs)>)
    {
        self.onSetReparsePoint = value;
    }

    fn fire_set_valid_data_length(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onSetValidDataLength
        {
            let mut args : CBFSSetValidDataLengthEventArgs = CBFSSetValidDataLengthEventArgs::new(par, cbpar);
            callable/*.on_set_valid_data_length*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_set_valid_data_length(&self) -> &'a dyn CBFSSetValidDataLengthEvent
    pub fn on_set_valid_data_length(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSSetValidDataLengthEventArgs)>
    {
        self.onSetValidDataLength
    }

    //pub fn set_on_set_valid_data_length(&mut self, value : &'a dyn CBFSSetValidDataLengthEvent)
    pub fn set_on_set_valid_data_length(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSSetValidDataLengthEventArgs)>)
    {
        self.onSetValidDataLength = value;
    }

    fn fire_set_volume_label(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onSetVolumeLabel
        {
            let mut args : CBFSSetVolumeLabelEventArgs = CBFSSetVolumeLabelEventArgs::new(par, cbpar);
            callable/*.on_set_volume_label*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_set_volume_label(&self) -> &'a dyn CBFSSetVolumeLabelEvent
    pub fn on_set_volume_label(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSSetVolumeLabelEventArgs)>
    {
        self.onSetVolumeLabel
    }

    //pub fn set_on_set_volume_label(&mut self, value : &'a dyn CBFSSetVolumeLabelEvent)
    pub fn set_on_set_volume_label(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSSetVolumeLabelEventArgs)>)
    {
        self.onSetVolumeLabel = value;
    }

    fn fire_unlock_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onUnlockFile
        {
            let mut args : CBFSUnlockFileEventArgs = CBFSUnlockFileEventArgs::new(par, cbpar);
            callable/*.on_unlock_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_unlock_file(&self) -> &'a dyn CBFSUnlockFileEvent
    pub fn on_unlock_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSUnlockFileEventArgs)>
    {
        self.onUnlockFile
    }

    //pub fn set_on_unlock_file(&mut self, value : &'a dyn CBFSUnlockFileEvent)
    pub fn set_on_unlock_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSUnlockFileEventArgs)>)
    {
        self.onUnlockFile = value;
    }

    fn fire_unmount(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onUnmount
        {
            let mut args : CBFSUnmountEventArgs = CBFSUnmountEventArgs::new(par, cbpar);
            callable/*.on_unmount*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_unmount(&self) -> &'a dyn CBFSUnmountEvent
    pub fn on_unmount(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSUnmountEventArgs)>
    {
        self.onUnmount
    }

    //pub fn set_on_unmount(&mut self, value : &'a dyn CBFSUnmountEvent)
    pub fn set_on_unmount(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSUnmountEventArgs)>)
    {
        self.onUnmount = value;
    }

    fn fire_worker_thread_creation(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onWorkerThreadCreation
        {
            let mut args : CBFSWorkerThreadCreationEventArgs = CBFSWorkerThreadCreationEventArgs::new(par, cbpar);
            callable/*.on_worker_thread_creation*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_worker_thread_creation(&self) -> &'a dyn CBFSWorkerThreadCreationEvent
    pub fn on_worker_thread_creation(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSWorkerThreadCreationEventArgs)>
    {
        self.onWorkerThreadCreation
    }

    //pub fn set_on_worker_thread_creation(&mut self, value : &'a dyn CBFSWorkerThreadCreationEvent)
    pub fn set_on_worker_thread_creation(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSWorkerThreadCreationEventArgs)>)
    {
        self.onWorkerThreadCreation = value;
    }

    fn fire_worker_thread_termination(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onWorkerThreadTermination
        {
            let mut args : CBFSWorkerThreadTerminationEventArgs = CBFSWorkerThreadTerminationEventArgs::new(par, cbpar);
            callable/*.on_worker_thread_termination*/(&self, &mut args);
        }
    }

    //pub fn on_worker_thread_termination(&self) -> &'a dyn CBFSWorkerThreadTerminationEvent
    pub fn on_worker_thread_termination(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSWorkerThreadTerminationEventArgs)>
    {
        self.onWorkerThreadTermination
    }

    //pub fn set_on_worker_thread_termination(&mut self, value : &'a dyn CBFSWorkerThreadTerminationEvent)
    pub fn set_on_worker_thread_termination(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSWorkerThreadTerminationEventArgs)>)
    {
        self.onWorkerThreadTermination = value;
    }

    fn fire_write_file(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onWriteFile
        {
            let mut args : CBFSWriteFileEventArgs = CBFSWriteFileEventArgs::new(par, cbpar);
            callable/*.on_write_file*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_write_file(&self) -> &'a dyn CBFSWriteFileEvent
    pub fn on_write_file(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSWriteFileEventArgs)>
    {
        self.onWriteFile
    }

    //pub fn set_on_write_file(&mut self, value : &'a dyn CBFSWriteFileEvent)
    pub fn set_on_write_file(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSWriteFileEventArgs)>)
    {
        self.onWriteFile = value;
    }

    fn fire_zeroize_file_range(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onZeroizeFileRange
        {
            let mut args : CBFSZeroizeFileRangeEventArgs = CBFSZeroizeFileRangeEventArgs::new(par, cbpar);
            callable/*.on_zeroize_file_range*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_zeroize_file_range(&self) -> &'a dyn CBFSZeroizeFileRangeEvent
    pub fn on_zeroize_file_range(&self) -> Option<fn (sender : &CBFS, e : &mut CBFSZeroizeFileRangeEventArgs)>
    {
        self.onZeroizeFileRange
    }

    //pub fn set_on_zeroize_file_range(&mut self, value : &'a dyn CBFSZeroizeFileRangeEvent)
    pub fn set_on_zeroize_file_range(&mut self, value : Option<fn (sender : &CBFS, e : &mut CBFSZeroizeFileRangeEventArgs)>)
    {
        self.onZeroizeFileRange = value;
    }


    pub(crate) fn report_error_info(&self, error_info : &str)
    {
        if let Some(callable) = self.onError
        {
            let mut args : CBFSErrorEventArgs = CBFSErrorEventArgs
                {
                    ErrorCode: -1,
                    Description: error_info.to_string(),
                    _Params: std::ptr::null_mut()
                } ;
            callable(&self, &mut args);
        }
    }

/////////////
// Properties
/////////////

    // GetLastError returns the error message reported by the last call, if any.
    pub fn get_last_error(&self) -> String
    {
        let result : String;
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetLastError.clone().unwrap();
            result = charptr_to_string(callable(self.Handle)).unwrap_or_else(|_| String::default());
        }
        result
    }

    // GetLastErrorCode returns the error code reported by the last call, if any.
    pub fn get_last_error_code(&self) -> i32
    {
        let result : i32;
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetLastErrorCode.clone().unwrap();
            result = callable(self.Handle) as i32;
        }
        result
    }

    // GetRuntimeLicense returns the runtime license key set for CBFS.
    pub fn get_runtime_license(&self) -> Result<String, CBFSConnectError>
    {
        let result : String;
        //let length : c_long;
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 8001 /*PID_RUNTIME_LICENSE*/, 0, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            result = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            Result::Ok(result)
        }
    }

    // SetRuntimeLicense sets the runtime license key for CBFS.
    pub fn set_runtime_license(&self, value : String) -> Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let oem_key = CString::new(value).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();
        unsafe
        {
            let callable = CBFSConnect_CBFS_SetCStr.clone().unwrap();
            ret_code = callable(self.Handle, 8001/*PID_RUNTIME_LICENSE*/, 0, oem_key_ptr, 0) as i32;
            let _ = CString::from_raw(oem_key_ptr);
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }
        else
        {
            return Result::Ok(());
        }
    }

    // Gets the value of the AccessDeniedProcessCount property: The number of records in the AccessDeniedProcess arrays.
    pub fn access_denied_process_count(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 1, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessDeniedProcessDesiredAccess property: The kind of access granted or denied.
    pub fn access_denied_process_desired_access(&self, AccessDeniedProcessIndex : i32) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 2, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessDesiredAccess"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 2, AccessDeniedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessDeniedProcessIncludeChildren property: Whether child processes are affected.
    pub fn access_denied_process_include_children(&self, AccessDeniedProcessIndex : i32) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 3, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessIncludeChildren"));
            }
        }
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 3, AccessDeniedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessDeniedProcessId property: The Id of the target process.
    pub fn access_denied_process_id(&self, AccessDeniedProcessIndex : i32) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 4, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessId"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 4, AccessDeniedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessDeniedProcessName property: The filename of the target process's executable.
    pub fn access_denied_process_name(&self, AccessDeniedProcessIndex : i32) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 5, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessName"));
            }
        }
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 5, AccessDeniedProcessIndex as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the AccessGrantedProcessCount property: The number of records in the AccessGrantedProcess arrays.
    pub fn access_granted_process_count(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 6, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessGrantedProcessDesiredAccess property: The kind of access granted or denied.
    pub fn access_granted_process_desired_access(&self, AccessGrantedProcessIndex : i32) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 7, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessDesiredAccess"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 7, AccessGrantedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessGrantedProcessIncludeChildren property: Whether child processes are affected.
    pub fn access_granted_process_include_children(&self, AccessGrantedProcessIndex : i32) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 8, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessIncludeChildren"));
            }
        }
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 8, AccessGrantedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessGrantedProcessId property: The Id of the target process.
    pub fn access_granted_process_id(&self, AccessGrantedProcessIndex : i32) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 9, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessId"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 9, AccessGrantedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessGrantedProcessName property: The filename of the target process's executable.
    pub fn access_granted_process_name(&self, AccessGrantedProcessIndex : i32) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 10, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessName"));
            }
        }
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 10, AccessGrantedProcessIndex as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the Active property: This property specifies whether the struct is active and handling OS requests.
    pub fn active(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 11, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the FileCache property: This property specifies which file data cache implementation to use.
    pub fn file_cache(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 12, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the FileCache property: This property specifies which file data cache implementation to use.
    pub fn set_file_cache(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 12, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the FileSystemName property: The name of the virtual filesystem.
    pub fn file_system_name(&self) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 13, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the FileSystemName property: The name of the virtual filesystem.
    pub fn set_file_system_name(&self, value : &str) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            //let CStrValue : CString = CString::from_vec_unchecked(value./*clone().*/into_bytes());
            //let cstrvalue_ptr: *mut c_char = CStrValue.into_raw();

            let cstrvalue_ptr: *mut c_char;
            match CString::new(value)
            {
                Ok(CStrValue) => { cstrvalue_ptr = CStrValue.into_raw(); }
                Err(_) => { return Result::Err(CBFSConnectError::new(-1, "String conversion error")); }
            }

            let callable = CBFSConnect_CBFS_SetCStr.clone().unwrap();
            ret_code = callable(self.Handle, 13, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the FireAllOpenCloseEvents property: This property specifies whether to fire events for all file open/close operations, or just the first and last.
    pub fn fire_all_open_close_events(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 14, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the FireAllOpenCloseEvents property: This property specifies whether to fire events for all file open/close operations, or just the first and last.
    pub fn set_fire_all_open_close_events(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 14, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the HandleAllFsctls property: This property specifies whether or not to fire the Fsctl event for all FSCTL_* requests.
    pub fn handle_all_fsctls(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 15, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the HandleAllFsctls property: This property specifies whether or not to fire the Fsctl event for all FSCTL_* requests.
    pub fn set_handle_all_fsctls(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 15, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the MaxFileNameLength property: This property includes the maximum file name length supported by the virtual filesystem.
    pub fn max_file_name_length(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 16, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the MaxFileNameLength property: This property includes the maximum file name length supported by the virtual filesystem.
    pub fn set_max_file_name_length(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 16, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the MaxFilePathLength property: This property includes the maximum file path length supported by the virtual filesystem.
    pub fn max_file_path_length(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 17, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the MaxFilePathLength property: This property includes the maximum file path length supported by the virtual filesystem.
    pub fn set_max_file_path_length(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 17, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the MaxFileSize property: This property includes the maximum file size supported by the virtual filesystem.
    pub fn max_file_size(&self) -> Result<i64, CBFSConnectError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetAsInt64.clone().unwrap();
            callable(self.Handle, 18, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the MaxFileSize property: This property includes the maximum file size supported by the virtual filesystem.
    pub fn set_max_file_size(&self, value : i64) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe 
        {
            let ValuePtr = &value;
            let callable = CBFSConnect_CBFS_SetInt64.clone().unwrap();
            ret_code = callable(self.Handle, 18, 0 as c_long, ValuePtr as *const i64, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the MetadataCacheEnabled property: This property specifies whether or not the metadata cache should be used.
    pub fn metadata_cache_enabled(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 19, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the MetadataCacheEnabled property: This property specifies whether or not the metadata cache should be used.
    pub fn set_metadata_cache_enabled(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 19, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the MetadataCacheSize property: This property includes the size of the metadata cache.
    pub fn metadata_cache_size(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 20, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the MetadataCacheSize property: This property includes the size of the metadata cache.
    pub fn set_metadata_cache_size(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 20, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the MountingPointCount property: The number of records in the MountingPoint arrays.
    pub fn mounting_point_count(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 21, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the MountingPointAuthenticationId property: The Authentication ID used when creating the mounting point, if applicable.
    pub fn mounting_point_authentication_id(&self, MountingPointIndex : i32) -> Result<i64, CBFSConnectError>
    {
        let ret_val : i64; // = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 22, MountingPointIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value MountingPointIndex for MountingPointAuthenticationId"));
            }
        }
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetAsInt64.clone().unwrap();
            callable(self.Handle, 22, MountingPointIndex as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the MountingPointFlags property: The flags used to create the mounting point.
    pub fn mounting_point_flags(&self, MountingPointIndex : i32) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 23, MountingPointIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value MountingPointIndex for MountingPointFlags"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 23, MountingPointIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the MountingPointName property: The mounting point name.
    pub fn mounting_point_name(&self, MountingPointIndex : i32) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 24, MountingPointIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value MountingPointIndex for MountingPointName"));
            }
        }
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 24, MountingPointIndex as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the NonExistentFilesCacheEnabled property: This property specifies whether or not the nonexistent files cache should be used.
    pub fn non_existent_files_cache_enabled(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 25, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the NonExistentFilesCacheEnabled property: This property specifies whether or not the nonexistent files cache should be used.
    pub fn set_non_existent_files_cache_enabled(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 25, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the NonExistentFilesCacheSize property: This property includes the size of the nonexistent files cache.
    pub fn non_existent_files_cache_size(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 26, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the NonExistentFilesCacheSize property: This property includes the size of the nonexistent files cache.
    pub fn set_non_existent_files_cache_size(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 26, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the OpenFilesCount property: The number of records in the OpenFile arrays.
    pub fn open_files_count(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 27, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the OpenFileHandleClosed property: This property reflects whether the handle to the file has been closed.
    pub fn open_file_handle_closed(&self, OpenFileIndex : i32) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 28, OpenFileIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value OpenFileIndex for OpenFileHandleClosed"));
            }
        }
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 28, OpenFileIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the OpenFileName property: This property reflects the name of the open file.
    pub fn open_file_name(&self, OpenFileIndex : i32) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 29, OpenFileIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value OpenFileIndex for OpenFileName"));
            }
        }
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 29, OpenFileIndex as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the OpenFileProcessId property: This property reflects the Id of the process that opened the file.
    pub fn open_file_process_id(&self, OpenFileIndex : i32) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 30, OpenFileIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value OpenFileIndex for OpenFileProcessId"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 30, OpenFileIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the OpenFileProcessName property: This property reflects the name of the process that opened the file.
    pub fn open_file_process_name(&self, OpenFileIndex : i32) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_CBFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 31, OpenFileIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value OpenFileIndex for OpenFileProcessName"));
            }
        }
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 31, OpenFileIndex as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the OpenHandlesCount property: This property includes the number of handles to filesystem objects in the virtual drive that are currently open.
    pub fn open_handles_count(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 32, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the OpenObjectsCount property: This property includes the number of filesystem objects in the virtual drive that are currently open.
    pub fn open_objects_count(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 33, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the ProcessRestrictionsEnabled property: Whether process access restrictions are enabled (Windows and Linux).
    pub fn process_restrictions_enabled(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 34, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the ProcessRestrictionsEnabled property: Whether process access restrictions are enabled (Windows and Linux).
    pub fn set_process_restrictions_enabled(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 34, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the SerializeAccess property: This property specifies whether nonintersecting operations against the same file should execute serially or in parallel.
    pub fn serialize_access(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 35, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the SerializeAccess property: This property specifies whether nonintersecting operations against the same file should execute serially or in parallel.
    pub fn set_serialize_access(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 35, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the SerializeEvents property: Whether events should be fired on a single worker thread, or many.
    pub fn serialize_events(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 36, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the SerializeEvents property: Whether events should be fired on a single worker thread, or many.
    pub fn set_serialize_events(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 36, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the StorageCharacteristics property: The characteristic flags to create the virtual drive with (Windows only).
    pub fn storage_characteristics(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 37, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the StorageCharacteristics property: The characteristic flags to create the virtual drive with (Windows only).
    pub fn set_storage_characteristics(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 37, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the StorageGUID property: The GUID to create the virtual drive with.
    pub fn storage_guid(&self) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 38, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the StorageGUID property: The GUID to create the virtual drive with.
    pub fn set_storage_guid(&self, value : &str) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            //let CStrValue : CString = CString::from_vec_unchecked(value./*clone().*/into_bytes());
            //let cstrvalue_ptr: *mut c_char = CStrValue.into_raw();

            let cstrvalue_ptr: *mut c_char;
            match CString::new(value)
            {
                Ok(CStrValue) => { cstrvalue_ptr = CStrValue.into_raw(); }
                Err(_) => { return Result::Err(CBFSConnectError::new(-1, "String conversion error")); }
            }

            let callable = CBFSConnect_CBFS_SetCStr.clone().unwrap();
            ret_code = callable(self.Handle, 38, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the StoragePresent property: This property specifies whether a virtual drive has been created.
    pub fn storage_present(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 39, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the StorageType property: The type of virtual drive to create (Windows only).
    pub fn storage_type(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 40, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the StorageType property: The type of virtual drive to create (Windows only).
    pub fn set_storage_type(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 40, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the SupportChangeTimeAttribute property: This property specifies whether the virtual filesystem supports the ChangeTime file attribute.
    pub fn support_change_time_attribute(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 41, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the SupportChangeTimeAttribute property: This property specifies whether the virtual filesystem supports the ChangeTime file attribute.
    pub fn set_support_change_time_attribute(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 41, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the SupportCompressedAttribute property: This property specifies whether the virtual filesystem supports the Compressed file attribute.
    pub fn support_compressed_attribute(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 42, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the SupportCompressedAttribute property: This property specifies whether the virtual filesystem supports the Compressed file attribute.
    pub fn set_support_compressed_attribute(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 42, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the SupportExtendedAttributes property: This property specifies whether the virtual filesystem supports Extended Attributes of files and operations on them.
    pub fn support_extended_attributes(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 43, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the SupportExtendedAttributes property: This property specifies whether the virtual filesystem supports Extended Attributes of files and operations on them.
    pub fn set_support_extended_attributes(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 43, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the SupportLastAccessTimeAttribute property: This property specifies whether the virtual filesystem supports the LastAccessTime file attribute.
    pub fn support_last_access_time_attribute(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 44, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the SupportLastAccessTimeAttribute property: This property specifies whether the virtual filesystem supports the LastAccessTime file attribute.
    pub fn set_support_last_access_time_attribute(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 44, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the SupportODXReadWrite property: This property specifies whether the virtual filesystem supports ODX (Offloaded Data Transfer) operations.
    pub fn support_odx_read_write(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 45, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the SupportODXReadWrite property: This property specifies whether the virtual filesystem supports ODX (Offloaded Data Transfer) operations.
    pub fn set_support_odx_read_write(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 45, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the SupportSparseFileAttribute property: This property specifies whether the virtual filesystem supports operations with sparse files.
    pub fn support_sparse_file_attribute(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 46, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the SupportSparseFileAttribute property: This property specifies whether the virtual filesystem supports operations with sparse files.
    pub fn set_support_sparse_file_attribute(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 46, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the Tag property: This property stores application-defined data specific to a particular instance of the struct.
    pub fn tag(&self) -> Result<i64, CBFSConnectError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_GetAsInt64.clone().unwrap();
            callable(self.Handle, 47, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the Tag property: This property stores application-defined data specific to a particular instance of the struct.
    pub fn set_tag(&self, value : i64) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe 
        {
            let ValuePtr = &value;
            let callable = CBFSConnect_CBFS_SetInt64.clone().unwrap();
            ret_code = callable(self.Handle, 47, 0 as c_long, ValuePtr as *const i64, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseAlternateDataStreams property: This property specifies whether or not the virtual filesystem supports alternate data streams.
    pub fn use_alternate_data_streams(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 48, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseAlternateDataStreams property: This property specifies whether or not the virtual filesystem supports alternate data streams.
    pub fn set_use_alternate_data_streams(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 48, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseCaseSensitiveFileNames property: This property specifies whether the virtual filesystem is case-sensitive or just case-preserving.
    pub fn use_case_sensitive_file_names(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 49, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseCaseSensitiveFileNames property: This property specifies whether the virtual filesystem is case-sensitive or just case-preserving.
    pub fn set_use_case_sensitive_file_names(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 49, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseDirectoryEmptyCheck property: This property specifies whether the IsDirectoryEmpty event should be used.
    pub fn use_directory_empty_check(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 50, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseDirectoryEmptyCheck property: This property specifies whether the IsDirectoryEmpty event should be used.
    pub fn set_use_directory_empty_check(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 50, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseDiskQuotas property: This property specifies whether the virtual filesystem supports disk quotas.
    pub fn use_disk_quotas(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 51, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseDiskQuotas property: This property specifies whether the virtual filesystem supports disk quotas.
    pub fn set_use_disk_quotas(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 51, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseFileIds property: This property specifies whether the virtual filesystem supports file Ids.
    pub fn use_file_ids(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 52, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseFileIds property: This property specifies whether the virtual filesystem supports file Ids.
    pub fn set_use_file_ids(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 52, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseHardLinks property: This property specifies whether the virtual filesystem supports hard links.
    pub fn use_hard_links(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 53, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseHardLinks property: This property specifies whether the virtual filesystem supports hard links.
    pub fn set_use_hard_links(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 53, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseObjectIds property: This property specifies whether the virtual filesystem supports object Id operations.
    pub fn use_object_ids(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 54, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseObjectIds property: This property specifies whether the virtual filesystem supports object Id operations.
    pub fn set_use_object_ids(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 54, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseReparsePoints property: This property specifies whether the virtual filesystem supports reparse points.
    pub fn use_reparse_points(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 55, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseReparsePoints property: This property specifies whether the virtual filesystem supports reparse points.
    pub fn set_use_reparse_points(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 55, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseShortFileNames property: This property specifies whether the virtual filesystem supports short (8.3) file names.
    pub fn use_short_file_names(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 56, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseShortFileNames property: This property specifies whether the virtual filesystem supports short (8.3) file names.
    pub fn set_use_short_file_names(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 56, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseWindowsSecurity property: This property specifies whether the virtual filesystem supports Windows' ACL-based security mechanisms.
    pub fn use_windows_security(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 57, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseWindowsSecurity property: This property specifies whether the virtual filesystem supports Windows' ACL-based security mechanisms.
    pub fn set_use_windows_security(&self, value : bool) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        let IntValue : i32;
        if value
        {
            IntValue = 1;
        }
        else
        {
            IntValue = 0;
        }
         unsafe
         {
            let callable = CBFSConnect_CBFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 57, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }



//////////
// Methods
//////////

    // AddDeniedProcess: Adds a rule that prevents a process from accessing the virtual drive (Windows and Linux).
    pub fn add_denied_process(&self, process_file_name : &str, process_id : i32, child_processes : bool, desired_access : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 4 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0];

        let CStrProcessFileNamePtr : *mut c_char;
        match CString::new(process_file_name)
        {
            Ok(CStrValueProcessFileName) => { CStrProcessFileNamePtr = CStrValueProcessFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrProcessFileNamePtr as usize;
        CParams[1] = (process_id as isize) as usize;
        let ChildProcessesBool : i32;
        if child_processes
        {
            ChildProcessesBool = 1;
        } else {
            ChildProcessesBool = 0;
        }
        CParams[2] = ChildProcessesBool as usize;

        CParams[3] = (desired_access as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 2, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProcessFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn add_denied_process

    // AddGrantedProcess: Adds a rule that allows a process to access the virtual drive (Windows and Linux).
    pub fn add_granted_process(&self, process_file_name : &str, process_id : i32, child_processes : bool, desired_access : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 4 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0];

        let CStrProcessFileNamePtr : *mut c_char;
        match CString::new(process_file_name)
        {
            Ok(CStrValueProcessFileName) => { CStrProcessFileNamePtr = CStrValueProcessFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrProcessFileNamePtr as usize;
        CParams[1] = (process_id as isize) as usize;
        let ChildProcessesBool : i32;
        if child_processes
        {
            ChildProcessesBool = 1;
        } else {
            ChildProcessesBool = 0;
        }
        CParams[2] = ChildProcessesBool as usize;

        CParams[3] = (desired_access as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 3, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProcessFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn add_granted_process

    // AddMountingPoint: Adds a mounting point for the virtual drive.
    pub fn add_mounting_point(&self, mounting_point : &str, flags : i32, authentication_id : i64) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrMountingPointPtr : *mut c_char;
        match CString::new(mounting_point)
        {
            Ok(CStrValueMountingPoint) => { CStrMountingPointPtr = CStrValueMountingPoint.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrMountingPointPtr as usize;
        CParams[1] = (flags as isize) as usize;
        let AuthenticationIdArr : Vec<i64> = vec![authentication_id];
        CParams[2] = AuthenticationIdArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 4, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrMountingPointPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn add_mounting_point

    // CloseOpenedFilesSnapshot: Closes the previously-created opened files snapshot.
    pub fn close_opened_files_snapshot(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 5, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn close_opened_files_snapshot

    // Config: Sets or retrieves a configuration setting.
    pub fn config(&self, configuration_string : &str) ->  Result<String, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrConfigurationStringPtr : *mut c_char;
        match CString::new(configuration_string)
        {
            Ok(CStrValueConfigurationString) => { CStrConfigurationStringPtr = CStrValueConfigurationString.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrConfigurationStringPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 6, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrConfigurationStringPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn config

    // CreateOpenedFilesSnapshot: Creates a snapshot of information about files that are currently open.
    pub fn create_opened_files_snapshot(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 7, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn create_opened_files_snapshot

    // CreateStorage: This method creates the virtual drive.
    pub fn create_storage(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 8, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn create_storage

    // DeleteStorage: This method deletes the virtual drive.
    pub fn delete_storage(&self, force_unmount : bool) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let ForceUnmountBool : i32;
        if force_unmount
        {
            ForceUnmountBool = 1;
        } else {
            ForceUnmountBool = 0;
        }
        CParams[0] = ForceUnmountBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 9, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn delete_storage

    // DisableRouteCache: This method disables the automatic routing of requests to local files.
    pub fn disable_route_cache(&self, invalidate_routed_files : bool) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let InvalidateRoutedFilesBool : i32;
        if invalidate_routed_files
        {
            InvalidateRoutedFilesBool = 1;
        } else {
            InvalidateRoutedFilesBool = 0;
        }
        CParams[0] = InvalidateRoutedFilesBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 10, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn disable_route_cache

    // DispatchEvents: Reserved, do not use.
    pub fn dispatch_events(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 11, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn dispatch_events

    // EnableRouteCache: This method enables the automatic routing of requests to local files.
    pub fn enable_route_cache(&self, location : &str, flags : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrLocationPtr : *mut c_char;
        match CString::new(location)
        {
            Ok(CStrValueLocation) => { CStrLocationPtr = CStrValueLocation.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrLocationPtr as usize;
        CParams[1] = (flags as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 12, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrLocationPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn enable_route_cache

    // FileMatchesMask: This method checks whether a particular file or directory name matches the specified mask.
    pub fn file_matches_mask(&self, mask : &str, file_name : &str, case_sensitive : bool) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrMaskPtr : *mut c_char;
        match CString::new(mask)
        {
            Ok(CStrValueMask) => { CStrMaskPtr = CStrValueMask.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrMaskPtr as usize;
        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrFileNamePtr as usize;
        let CaseSensitiveBool : i32;
        if case_sensitive
        {
            CaseSensitiveBool = 1;
        } else {
            CaseSensitiveBool = 0;
        }
        CParams[2] = CaseSensitiveBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 13, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrMaskPtr);
            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[3] != 0;
        return Result::Ok(ret_val);
    } // fn file_matches_mask

    // FlushFile: This method flushes cached data of a file or its part.
    pub fn flush_file(&self, file_name : &str, offset : i64, length : i64, wait : bool) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 4 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let OffsetArr : Vec<i64> = vec![offset];
        CParams[1] = OffsetArr.as_ptr() as usize;
        let LengthArr : Vec<i64> = vec![length];
        CParams[2] = LengthArr.as_ptr() as usize;
        let WaitBool : i32;
        if wait
        {
            WaitBool = 1;
        } else {
            WaitBool = 0;
        }
        CParams[3] = WaitBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 14, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn flush_file

    // GetDriverStatus: Retrieves the status of the system driver.
    pub fn get_driver_status(&self, product_guid : &str, module : i32) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrProductGUIDPtr : *mut c_char;
        match CString::new(product_guid)
        {
            Ok(CStrValueProductGUID) => { CStrProductGUIDPtr = CStrValueProductGUID.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrProductGUIDPtr as usize;
        CParams[1] = (module as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 15, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProductGUIDPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] as i32;
        return Result::Ok(ret_val);
    } // fn get_driver_status

    // GetEventFileName: This method retrieves the name of the file or directory, to which the event applies.
    pub fn get_event_file_name(&self, ) ->  Result<String, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 16, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[0] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn get_event_file_name

    // GetHandleCreatorProcessId: This method retrieves the Id of the process (PID) that opened the specified file handle.
    pub fn get_handle_creator_process_id(&self, handle_info : i64) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let HandleInfoArr : Vec<i64> = vec![handle_info];
        CParams[0] = HandleInfoArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 17, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] as i32;
        return Result::Ok(ret_val);
    } // fn get_handle_creator_process_id

    // GetHandleCreatorProcessName: This method retrieves the name of the process that opened the specified file handle.
    pub fn get_handle_creator_process_name(&self, handle_info : i64) ->  Result<String, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let HandleInfoArr : Vec<i64> = vec![handle_info];
        CParams[0] = HandleInfoArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 18, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn get_handle_creator_process_name

    // GetHandleCreatorThreadId: This method retrieves the Id of the thread that opened the specified file handle.
    pub fn get_handle_creator_thread_id(&self, handle_info : i64) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let HandleInfoArr : Vec<i64> = vec![handle_info];
        CParams[0] = HandleInfoArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 19, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] as i32;
        return Result::Ok(ret_val);
    } // fn get_handle_creator_thread_id

    // GetHandleCreatorToken: This method retrieves the security token associated with the process that opened the specified file handle.
    pub fn get_handle_creator_token(&self, handle_info : i64) ->  Result<i64, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let HandleInfoArr : Vec<i64> = vec![handle_info];
        CParams[0] = HandleInfoArr.as_ptr() as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 20, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_handle_creator_token

    // GetModuleVersion: Retrieves the version of a given product module.
    pub fn get_module_version(&self, product_guid : &str, module : i32) ->  Result<i64, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrProductGUIDPtr : *mut c_char;
        match CString::new(product_guid)
        {
            Ok(CStrValueProductGUID) => { CStrProductGUIDPtr = CStrValueProductGUID.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrProductGUIDPtr as usize;
        CParams[1] = (module as isize) as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 21, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrProductGUIDPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_module_version

    // GetOriginatorProcessId: Retrieves the Id of the process (PID) that initiated the operation (Windows and Linux).
    pub fn get_originator_process_id(&self, ) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 22, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[0] as i32;
        return Result::Ok(ret_val);
    } // fn get_originator_process_id

    // GetOriginatorProcessName: Retrieves the name of the process that initiated the operation (Windows and Linux).
    pub fn get_originator_process_name(&self, ) ->  Result<String, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 23, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[0] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn get_originator_process_name

    // GetOriginatorThreadId: Retrieves the Id of the thread that initiated the operation (Windows only).
    pub fn get_originator_thread_id(&self, ) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 24, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[0] as i32;
        return Result::Ok(ret_val);
    } // fn get_originator_thread_id

    // GetOriginatorToken: Retrieves the security token associated with the process that initiated the operation (Windows only).
    pub fn get_originator_token(&self, ) ->  Result<i64, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 25, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_originator_token

    // Initialize: This method initializes the struct.
    pub fn initialize(&self, product_guid : &str) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrProductGUIDPtr : *mut c_char;
        match CString::new(product_guid)
        {
            Ok(CStrValueProductGUID) => { CStrProductGUIDPtr = CStrValueProductGUID.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrProductGUIDPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 26, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProductGUIDPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn initialize

    // Install: Installs (or upgrades) the product's system drivers and/or the helper DLL (Windows only).
    pub fn install(&self, cab_file_name : &str, product_guid : &str, path_to_install : &str, modules_to_install : i32, flags : i32) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 5 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0, 0];

        let CStrCabFileNamePtr : *mut c_char;
        match CString::new(cab_file_name)
        {
            Ok(CStrValueCabFileName) => { CStrCabFileNamePtr = CStrValueCabFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrCabFileNamePtr as usize;
        let CStrProductGUIDPtr : *mut c_char;
        match CString::new(product_guid)
        {
            Ok(CStrValueProductGUID) => { CStrProductGUIDPtr = CStrValueProductGUID.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrProductGUIDPtr as usize;
        let CStrPathToInstallPtr : *mut c_char;
        match CString::new(path_to_install)
        {
            Ok(CStrValuePathToInstall) => { CStrPathToInstallPtr = CStrValuePathToInstall.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrPathToInstallPtr as usize;
        CParams[3] = (modules_to_install as isize) as usize;
        CParams[4] = (flags as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 27, 5, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrCabFileNamePtr);
            let _ = CString::from_raw(CStrProductGUIDPtr);
            let _ = CString::from_raw(CStrPathToInstallPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[5] as i32;
        return Result::Ok(ret_val);
    } // fn install

    // IsCBFSVolume: This method checks whether the specified volume is powered by CBFS.
    pub fn is_cbfs_volume(&self, volume_path : &str) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrVolumePathPtr : *mut c_char;
        match CString::new(volume_path)
        {
            Ok(CStrValueVolumePath) => { CStrVolumePathPtr = CStrValueVolumePath.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrVolumePathPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 28, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrVolumePathPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn is_cbfs_volume

    // IsIconRegistered: Checks whether the specified icon is registered (Windows only).
    pub fn is_icon_registered(&self, icon_id : &str) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrIconIdPtr : *mut c_char;
        match CString::new(icon_id)
        {
            Ok(CStrValueIconId) => { CStrIconIdPtr = CStrValueIconId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrIconIdPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 29, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrIconIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn is_icon_registered

    // MountMedia: This method mounts media in the virtual drive, making it accessible for reading and writing.
    pub fn mount_media(&self, timeout : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        CParams[0] = (timeout as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 30, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn mount_media

    // NotifyDirectoryChange: This method notifies the OS that a file or directory has changed.
    pub fn notify_directory_change(&self, file_name : &str, action : i32, new_file_name : &str, wait : bool) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 4 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        CParams[1] = (action as isize) as usize;
        let CStrNewFileNamePtr : *mut c_char;
        match CString::new(new_file_name)
        {
            Ok(CStrValueNewFileName) => { CStrNewFileNamePtr = CStrValueNewFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrNewFileNamePtr as usize;
        let WaitBool : i32;
        if wait
        {
            WaitBool = 1;
        } else {
            WaitBool = 0;
        }
        CParams[3] = WaitBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 31, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrNewFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[4] != 0;
        return Result::Ok(ret_val);
    } // fn notify_directory_change

    // RegisterIcon: Registers an icon that can be displayed as an overlay on the virtual drive in Windows File Explorer (Windows only).
    pub fn register_icon(&self, icon_path : &str, product_guid : &str, icon_id : &str) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrIconPathPtr : *mut c_char;
        match CString::new(icon_path)
        {
            Ok(CStrValueIconPath) => { CStrIconPathPtr = CStrValueIconPath.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrIconPathPtr as usize;
        let CStrProductGUIDPtr : *mut c_char;
        match CString::new(product_guid)
        {
            Ok(CStrValueProductGUID) => { CStrProductGUIDPtr = CStrValueProductGUID.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrProductGUIDPtr as usize;
        let CStrIconIdPtr : *mut c_char;
        match CString::new(icon_id)
        {
            Ok(CStrValueIconId) => { CStrIconIdPtr = CStrValueIconId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrIconIdPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 32, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrIconPathPtr);
            let _ = CString::from_raw(CStrProductGUIDPtr);
            let _ = CString::from_raw(CStrIconIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[3] != 0;
        return Result::Ok(ret_val);
    } // fn register_icon

    // ReleaseUnusedFiles: This method instructs the OS to release any files kept open by the cache manager.
    pub fn release_unused_files(&self, ) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 33, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[0] != 0;
        return Result::Ok(ret_val);
    } // fn release_unused_files

    // RemoveDeniedProcess: Removes a rule that prevents a process from accessing the virtual drive (Windows and Linux).
    pub fn remove_denied_process(&self, process_file_name : &str, process_id : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrProcessFileNamePtr : *mut c_char;
        match CString::new(process_file_name)
        {
            Ok(CStrValueProcessFileName) => { CStrProcessFileNamePtr = CStrValueProcessFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrProcessFileNamePtr as usize;
        CParams[1] = (process_id as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 34, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProcessFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn remove_denied_process

    // RemoveGrantedProcess: Removes a rule that allows a process to access the virtual drive (Windows and Linux).
    pub fn remove_granted_process(&self, process_file_name : &str, process_id : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrProcessFileNamePtr : *mut c_char;
        match CString::new(process_file_name)
        {
            Ok(CStrValueProcessFileName) => { CStrProcessFileNamePtr = CStrValueProcessFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrProcessFileNamePtr as usize;
        CParams[1] = (process_id as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 35, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProcessFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn remove_granted_process

    // RemoveMountingPoint: Removes a mounting point for the virtual drive.
    pub fn remove_mounting_point(&self, index : i32, mounting_point : &str, flags : i32, authentication_id : i64) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 4 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0];

        CParams[0] = (index as isize) as usize;
        let CStrMountingPointPtr : *mut c_char;
        match CString::new(mounting_point)
        {
            Ok(CStrValueMountingPoint) => { CStrMountingPointPtr = CStrValueMountingPoint.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrMountingPointPtr as usize;
        CParams[2] = (flags as isize) as usize;
        let AuthenticationIdArr : Vec<i64> = vec![authentication_id];
        CParams[3] = AuthenticationIdArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 36, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrMountingPointPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn remove_mounting_point

    // ResetIcon: Resets the virtual drive's icon back to default by deselecting the active overlay icon (Windows only).
    pub fn reset_icon(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 37, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn reset_icon

    // ResetTimeout: This method resets the timeout duration for the current event handler.
    pub fn reset_timeout(&self, timeout : i32) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        CParams[0] = (timeout as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 38, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn reset_timeout

    // RouteToFile: This method instructs the struct to route future requests directly to a given file.
    pub fn route_to_file(&self, file_info : i64, file_name : &str, flags : i32) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let FileInfoArr : Vec<i64> = vec![file_info];
        CParams[0] = FileInfoArr.as_ptr() as usize;
        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrFileNamePtr as usize;
        CParams[2] = (flags as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 39, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[3] != 0;
        return Result::Ok(ret_val);
    } // fn route_to_file

    // SetIcon: Selects a registered overlay icon for display on the virtual drive in Windows File Explorer (Windows only).
    pub fn set_icon(&self, icon_id : &str) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrIconIdPtr : *mut c_char;
        match CString::new(icon_id)
        {
            Ok(CStrValueIconId) => { CStrIconIdPtr = CStrValueIconId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrIconIdPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 40, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrIconIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_icon

    // ShutdownSystem: Shuts down or reboots the operating system.
    pub fn shutdown_system(&self, shutdown_prompt : &str, timeout : i32, force_close_apps : bool, reboot : bool) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 4 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0];

        let CStrShutdownPromptPtr : *mut c_char;
        match CString::new(shutdown_prompt)
        {
            Ok(CStrValueShutdownPrompt) => { CStrShutdownPromptPtr = CStrValueShutdownPrompt.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrShutdownPromptPtr as usize;
        CParams[1] = (timeout as isize) as usize;
        let ForceCloseAppsBool : i32;
        if force_close_apps
        {
            ForceCloseAppsBool = 1;
        } else {
            ForceCloseAppsBool = 0;
        }
        CParams[2] = ForceCloseAppsBool as usize;

        let RebootBool : i32;
        if reboot
        {
            RebootBool = 1;
        } else {
            RebootBool = 0;
        }
        CParams[3] = RebootBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 41, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrShutdownPromptPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[4] != 0;
        return Result::Ok(ret_val);
    } // fn shutdown_system

    // Uninstall: Uninstalls the product's system drivers and/or helper DLL (Windows only).
    pub fn uninstall(&self, cab_file_name : &str, product_guid : &str, installed_path : &str, flags : i32) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 4 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0];

        let CStrCabFileNamePtr : *mut c_char;
        match CString::new(cab_file_name)
        {
            Ok(CStrValueCabFileName) => { CStrCabFileNamePtr = CStrValueCabFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrCabFileNamePtr as usize;
        let CStrProductGUIDPtr : *mut c_char;
        match CString::new(product_guid)
        {
            Ok(CStrValueProductGUID) => { CStrProductGUIDPtr = CStrValueProductGUID.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrProductGUIDPtr as usize;
        let CStrInstalledPathPtr : *mut c_char;
        match CString::new(installed_path)
        {
            Ok(CStrValueInstalledPath) => { CStrInstalledPathPtr = CStrValueInstalledPath.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrInstalledPathPtr as usize;
        CParams[3] = (flags as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 42, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrCabFileNamePtr);
            let _ = CString::from_raw(CStrProductGUIDPtr);
            let _ = CString::from_raw(CStrInstalledPathPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[4] as i32;
        return Result::Ok(ret_val);
    } // fn uninstall

    // UnmountMedia: This method unmounts media from the virtual drive.
    pub fn unmount_media(&self, force_unmount : bool) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let ForceUnmountBool : i32;
        if force_unmount
        {
            ForceUnmountBool = 1;
        } else {
            ForceUnmountBool = 0;
        }
        CParams[0] = ForceUnmountBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 43, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn unmount_media

    // UnregisterIcon: Unregisters an existing overlay icon (Windows only).
    pub fn unregister_icon(&self, product_guid : &str, icon_id : &str) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrProductGUIDPtr : *mut c_char;
        match CString::new(product_guid)
        {
            Ok(CStrValueProductGUID) => { CStrProductGUIDPtr = CStrValueProductGUID.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrProductGUIDPtr as usize;
        let CStrIconIdPtr : *mut c_char;
        match CString::new(icon_id)
        {
            Ok(CStrValueIconId) => { CStrIconIdPtr = CStrValueIconId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrIconIdPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 44, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProductGUIDPtr);
            let _ = CString::from_raw(CStrIconIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] != 0;
        return Result::Ok(ret_val);
    } // fn unregister_icon

} // CBFS

extern "system" fn CBFSEventDispatcher(pObj : usize, event_id : c_long, _cparam : c_long, param : IntPtrArrayType, cbparam : IntArrayType) -> c_long
{
    let obj: &'static CBFS;
    // Lock the Mutex to get access to the HashMap
    unsafe
    {
        let _map = CBFSDictMutex.lock().unwrap();
        let objOpt = CBFSDict.get(&pObj);
        if objOpt.is_none()
        {
            return -1;
        }
        obj = &(objOpt.unwrap());
    } // The lock is automatically released here

    let event_result: Result<(), Box<dyn std::any::Any + Send>> = catch_unwind (||
    {
        match event_id
        {
            1 /* CanFileBeDeleted */=> obj.fire_can_file_be_deleted(/*cparam as i32, */param, cbparam),

            2 /* CleanupFile */=> obj.fire_cleanup_file(/*cparam as i32, */param, cbparam),

            3 /* CloseDirectoryEnumeration */=> obj.fire_close_directory_enumeration(/*cparam as i32, */param, cbparam),

            4 /* CloseFile */=> obj.fire_close_file(/*cparam as i32, */param, cbparam),

            5 /* CloseHardLinksEnumeration */=> obj.fire_close_hard_links_enumeration(/*cparam as i32, */param, cbparam),

            6 /* CloseNamedStreamsEnumeration */=> obj.fire_close_named_streams_enumeration(/*cparam as i32, */param, cbparam),

            7 /* CloseQuotasEnumeration */=> obj.fire_close_quotas_enumeration(/*cparam as i32, */param, cbparam),

            8 /* CreateFile */=> obj.fire_create_file(/*cparam as i32, */param, cbparam),

            9 /* CreateHardLink */=> obj.fire_create_hard_link(/*cparam as i32, */param, cbparam),

            10 /* DeleteFile */=> obj.fire_delete_file(/*cparam as i32, */param, cbparam),

            11 /* DeleteObjectId */=> obj.fire_delete_object_id(/*cparam as i32, */param, cbparam),

            12 /* DeleteReparsePoint */=> obj.fire_delete_reparse_point(/*cparam as i32, */param, cbparam),

            13 /* Ejected */=> obj.fire_ejected(/*cparam as i32, */param, cbparam),

            14 /* EnumerateDirectory */=> obj.fire_enumerate_directory(/*cparam as i32, */param, cbparam),

            15 /* EnumerateHardLinks */=> obj.fire_enumerate_hard_links(/*cparam as i32, */param, cbparam),

            16 /* EnumerateNamedStreams */=> obj.fire_enumerate_named_streams(/*cparam as i32, */param, cbparam),

            17 /* Error */=> obj.fire_error(/*cparam as i32, */param, cbparam),

            18 /* FlushFile */=> obj.fire_flush_file(/*cparam as i32, */param, cbparam),

            19 /* Fsctl */=> obj.fire_fsctl(/*cparam as i32, */param, cbparam),

            20 /* GetDefaultQuotaInfo */=> obj.fire_get_default_quota_info(/*cparam as i32, */param, cbparam),

            21 /* GetFileInfo */=> obj.fire_get_file_info(/*cparam as i32, */param, cbparam),

            22 /* GetFileNameByFileId */=> obj.fire_get_file_name_by_file_id(/*cparam as i32, */param, cbparam),

            23 /* GetFileSecurity */=> obj.fire_get_file_security(/*cparam as i32, */param, cbparam),

            24 /* GetObjectId */=> obj.fire_get_object_id(/*cparam as i32, */param, cbparam),

            25 /* GetReparsePoint */=> obj.fire_get_reparse_point(/*cparam as i32, */param, cbparam),

            26 /* GetVolumeId */=> obj.fire_get_volume_id(/*cparam as i32, */param, cbparam),

            27 /* GetVolumeLabel */=> obj.fire_get_volume_label(/*cparam as i32, */param, cbparam),

            28 /* GetVolumeSize */=> obj.fire_get_volume_size(/*cparam as i32, */param, cbparam),

            29 /* Ioctl */=> obj.fire_ioctl(/*cparam as i32, */param, cbparam),

            30 /* IsDirectoryEmpty */=> obj.fire_is_directory_empty(/*cparam as i32, */param, cbparam),

            31 /* LockFile */=> obj.fire_lock_file(/*cparam as i32, */param, cbparam),

            32 /* Mount */=> obj.fire_mount(/*cparam as i32, */param, cbparam),

            33 /* OffloadReadFile */=> obj.fire_offload_read_file(/*cparam as i32, */param, cbparam),

            34 /* OffloadWriteFile */=> obj.fire_offload_write_file(/*cparam as i32, */param, cbparam),

            35 /* OpenFile */=> obj.fire_open_file(/*cparam as i32, */param, cbparam),

            36 /* QueryAllocatedRanges */=> obj.fire_query_allocated_ranges(/*cparam as i32, */param, cbparam),

            37 /* QueryCompressionInfo */=> obj.fire_query_compression_info(/*cparam as i32, */param, cbparam),

            38 /* QueryEa */=> obj.fire_query_ea(/*cparam as i32, */param, cbparam),

            39 /* QueryQuotas */=> obj.fire_query_quotas(/*cparam as i32, */param, cbparam),

            40 /* ReadFile */=> obj.fire_read_file(/*cparam as i32, */param, cbparam),

            41 /* RenameOrMoveFile */=> obj.fire_rename_or_move_file(/*cparam as i32, */param, cbparam),

            42 /* SetAllocationSize */=> obj.fire_set_allocation_size(/*cparam as i32, */param, cbparam),

            43 /* SetDefaultQuotaInfo */=> obj.fire_set_default_quota_info(/*cparam as i32, */param, cbparam),

            44 /* SetEa */=> obj.fire_set_ea(/*cparam as i32, */param, cbparam),

            45 /* SetFileAttributes */=> obj.fire_set_file_attributes(/*cparam as i32, */param, cbparam),

            46 /* SetFileSecurity */=> obj.fire_set_file_security(/*cparam as i32, */param, cbparam),

            47 /* SetFileSize */=> obj.fire_set_file_size(/*cparam as i32, */param, cbparam),

            48 /* SetObjectId */=> obj.fire_set_object_id(/*cparam as i32, */param, cbparam),

            49 /* SetQuotas */=> obj.fire_set_quotas(/*cparam as i32, */param, cbparam),

            50 /* SetReparsePoint */=> obj.fire_set_reparse_point(/*cparam as i32, */param, cbparam),

            51 /* SetValidDataLength */=> obj.fire_set_valid_data_length(/*cparam as i32, */param, cbparam),

            52 /* SetVolumeLabel */=> obj.fire_set_volume_label(/*cparam as i32, */param, cbparam),

            53 /* UnlockFile */=> obj.fire_unlock_file(/*cparam as i32, */param, cbparam),

            54 /* Unmount */=> obj.fire_unmount(/*cparam as i32, */param, cbparam),

            55 /* WorkerThreadCreation */=> obj.fire_worker_thread_creation(/*cparam as i32, */param, cbparam),

            56 /* WorkerThreadTermination */=> obj.fire_worker_thread_termination(/*cparam as i32, */param, cbparam),

            57 /* WriteFile */=> obj.fire_write_file(/*cparam as i32, */param, cbparam),

            58 /* ZeroizeFileRange */=> obj.fire_zeroize_file_range(/*cparam as i32, */param, cbparam),

            _ => {}
        }
    } );

    return match event_result
    {
        Ok(_) => 0,
        Err(panic_info) =>
        {
            let mut info_str = String::new();
            if let Some(payload) = panic_info.downcast_ref::<&str>()
            {
                write!(info_str, "Panic message: {}", payload).unwrap();
            }
            else
            {
                info_str = "Unknown panic occurred.".to_string();
            }
            obj.report_error_info(&info_str);
             -1
        }
    };
}


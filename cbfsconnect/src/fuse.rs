


#![allow(non_snake_case)]

extern crate libloading as lib;

use std::{collections::HashMap, ffi::{c_char, c_long, c_longlong, c_ulong, c_void, CStr, CString}, panic::catch_unwind, sync::{atomic::{AtomicUsize, Ordering::SeqCst}, Mutex} };
use lib::{Library, Symbol, Error};
use std::fmt::Write;
use chrono::Utc;
use once_cell::sync::Lazy;

use crate::{*, cbfsconnectkey};

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_StaticInit)(void *hInst);
type CBFSConnectFUSEStaticInitType = unsafe extern "system" fn(hInst : *mut c_void) -> i32;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_StaticDestroy)();
type CBFSConnectFUSEStaticDestroyType = unsafe extern "system" fn()-> i32;

// typedef void* (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_Create)(PCBFSCONNECT_CALLBACK lpSink, void *lpContext, char *lpOemKey, int opts);
type CBFSConnectFUSECreateType = unsafe extern "system" fn(lpSink : CBFSConnectSinkDelegateType, lpContext : usize, lpOemKey : *const c_char, opts : i32) -> usize;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_Destroy)(void *lpObj);
type CBFSConnectFUSEDestroyType = unsafe extern "system" fn(lpObj: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_CheckIndex)(void *lpObj, int propid, int arridx);
type CBFSConnectFUSECheckIndexType = unsafe extern "system" fn(lpObj: usize, propid: c_long, arridx: c_long)-> c_long;

// typedef void* (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_Get)(void *lpObj, int propid, int arridx, int *lpcbVal, int64 *lpllVal);
type CBFSConnectFUSEGetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *mut c_longlong) -> *mut c_void;
type CBFSConnectFUSEGetAsCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *const c_longlong) -> *const c_char;
type CBFSConnectFUSEGetAsIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *const c_void) -> usize;
type CBFSConnectFUSEGetAsInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *mut i64) -> usize;
type CBFSConnectFUSEGetAsBSTRType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut i32, llVal: *const c_void) -> *const u8;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_Set)(void *lpObj, int propid, int arridx, const void *val, int cbVal);
type CBFSConnectFUSESetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_void, len: c_ulong)-> c_long;
type CBFSConnectFUSESetCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_char, len: c_ulong)-> c_long;
type CBFSConnectFUSESetIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: isize, len: c_ulong)-> c_long;
type CBFSConnectFUSESetInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const i64, len: c_ulong)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_Do)(void *lpObj, int methid, int cparam, void *param[], int cbparam[], int64 *lpllVal);
type CBFSConnectFUSEDoType = unsafe extern "system" fn(p: usize, method_id: c_long, cparam: c_long, params: UIntPtrArrayType, cbparam: IntArrayType, llVal: *mut c_longlong)-> c_long;

// typedef char* (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_GetLastError)(void *lpObj);
type CBFSConnectFUSEGetLastErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_GetLastErrorCode)(void *lpObj);
type CBFSConnectFUSEGetLastErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_SetLastErrorAndCode)(void *lpObj, int code, char *message);
type CBFSConnectFUSESetLastErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

// typedef char* (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_GetEventError)(void *lpObj);
type CBFSConnectFUSEGetEventErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_GetEventErrorCode)(void *lpObj);
type CBFSConnectFUSEGetEventErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_FUSE_SetEventErrorAndCode)(void *lpObj, int code, char *message);
type CBFSConnectFUSESetEventErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

static mut CBFSConnect_FUSE_StaticInit : Option<Symbol<CBFSConnectFUSEStaticInitType>> = None;
static mut CBFSConnect_FUSE_StaticDestroy : Option<Symbol<CBFSConnectFUSEStaticDestroyType>> = None;

static mut CBFSConnect_FUSE_Create: Option<Symbol<CBFSConnectFUSECreateType>> = None;
static mut CBFSConnect_FUSE_Destroy: Option<Symbol<CBFSConnectFUSEDestroyType>> = None;
static mut CBFSConnect_FUSE_Set: Option<Symbol<CBFSConnectFUSESetType>> = None;
static mut CBFSConnect_FUSE_SetCStr: Option<Symbol<CBFSConnectFUSESetCStrType>> = None;
static mut CBFSConnect_FUSE_SetInt: Option<Symbol<CBFSConnectFUSESetIntType>> = None;
static mut CBFSConnect_FUSE_SetInt64: Option<Symbol<CBFSConnectFUSESetInt64Type>> = None;
static mut CBFSConnect_FUSE_Get: Option<Symbol<CBFSConnectFUSEGetType>> = None;
static mut CBFSConnect_FUSE_GetAsCStr: Option<Symbol<CBFSConnectFUSEGetAsCStrType>> = None;
static mut CBFSConnect_FUSE_GetAsInt: Option<Symbol<CBFSConnectFUSEGetAsIntType>> = None;
static mut CBFSConnect_FUSE_GetAsInt64: Option<Symbol<CBFSConnectFUSEGetAsInt64Type>> = None;
static mut CBFSConnect_FUSE_GetAsBSTR: Option<Symbol<CBFSConnectFUSEGetAsBSTRType>> = None;
static mut CBFSConnect_FUSE_GetLastError: Option<Symbol<CBFSConnectFUSEGetLastErrorType>> = None;
static mut CBFSConnect_FUSE_GetLastErrorCode: Option<Symbol<CBFSConnectFUSEGetLastErrorCodeType>> = None;
static mut CBFSConnect_FUSE_SetLastErrorAndCode: Option<Symbol<CBFSConnectFUSESetLastErrorAndCodeType>> = None;
static mut CBFSConnect_FUSE_GetEventError: Option<Symbol<CBFSConnectFUSEGetEventErrorType>> = None;
static mut CBFSConnect_FUSE_GetEventErrorCode: Option<Symbol<CBFSConnectFUSEGetEventErrorCodeType>> = None;
static mut CBFSConnect_FUSE_SetEventErrorAndCode: Option<Symbol<CBFSConnectFUSESetEventErrorAndCodeType>> = None;
static mut CBFSConnect_FUSE_CheckIndex: Option<Symbol<CBFSConnectFUSECheckIndexType>> = None;
static mut CBFSConnect_FUSE_Do: Option<Symbol<CBFSConnectFUSEDoType>> = None;

static mut FUSEIDSeed : AtomicUsize = AtomicUsize::new(1);

static mut FUSEDict : Lazy<HashMap<usize, FUSE>> = Lazy::new(|| HashMap::new() );
static FUSEDictMutex : Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0) );

const FUSECreateOpt : i32 = 0;


pub type FUSEStream = crate::CBFSConnectStream;


pub(crate) fn get_lib_funcs( lib_hand : &'static Library) -> bool
{
    #[cfg(target_os = "android")]
    return true;
    #[cfg(target_os = "ios")]
    return true;

    unsafe
    {
        // CBFSConnect_FUSE_StaticInit
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEStaticInitType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_StaticInit");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_FUSE_StaticInit = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_StaticDestroy
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEStaticDestroyType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_StaticDestroy");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_FUSE_StaticDestroy = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_Create
        let func_ptr_res : Result<Symbol<CBFSConnectFUSECreateType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Create");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_FUSE_Create = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_Destroy
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEDestroyType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Destroy");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_FUSE_Destroy = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_Get
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEGetType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_Get = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_FUSE_GetAsCStr
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEGetAsCStrType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_GetAsCStr = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_FUSE_GetAsInt
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEGetAsIntType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_GetAsInt = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_FUSE_GetAsInt64
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEGetAsInt64Type>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_GetAsInt64 = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_FUSE_GetAsBSTR
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEGetAsBSTRType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_GetAsBSTR = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_Set
        let func_ptr_res : Result<Symbol<CBFSConnectFUSESetType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_Set = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_FUSE_SetCStr
        let func_ptr_res : Result<Symbol<CBFSConnectFUSESetCStrType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_SetCStr = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_FUSE_SetInt
        let func_ptr_res : Result<Symbol<CBFSConnectFUSESetIntType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_SetInt = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_FUSE_SetInt64
        let func_ptr_res : Result<Symbol<CBFSConnectFUSESetInt64Type>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_SetInt64 = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_GetLastError
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEGetLastErrorType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_GetLastError");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_GetLastError = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_GetLastErrorCode
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEGetLastErrorCodeType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_GetLastErrorCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_GetLastErrorCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_SetLastErrorAndCode
        let func_ptr_res : Result<Symbol<CBFSConnectFUSESetLastErrorAndCodeType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_SetLastErrorAndCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_SetLastErrorAndCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_GetEventError
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEGetEventErrorType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_GetEventError");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_GetEventError = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_GetEventErrorCode
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEGetEventErrorCodeType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_GetEventErrorCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_GetEventErrorCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_SetEventErrorAndCode
        let func_ptr_res : Result<Symbol<CBFSConnectFUSESetEventErrorAndCodeType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_SetEventErrorAndCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_SetEventErrorAndCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_CheckIndex
        let func_ptr_res : Result<Symbol<CBFSConnectFUSECheckIndexType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_CheckIndex");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_CheckIndex = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_FUSE_Do
        let func_ptr_res : Result<Symbol<CBFSConnectFUSEDoType>, Error> = lib_hand.get(b"CBFSConnect_FUSE_Do");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_FUSE_Do = func_ptr_res.ok();
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


// FUSEAccessEventArgs carries the parameters of the Access event of FUSE
pub struct FUSEAccessEventArgs
{
    Path : String,
    Mask : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of AccessEventArgs
impl FUSEAccessEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEAccessEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Access event of a FUSE instance").to_owned();
            }
        }

        let lvMask : i32;
        unsafe
        {
            lvMask = *par.add(1) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(2) as i32;
        }
        
        FUSEAccessEventArgs
        {
            Path: lvPath,
            Mask: lvMask,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn mask(&self) -> i32
    {
        self.Mask
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEAccessEvent
{
    fn on_access(&self, sender : &FUSE, e : &mut FUSEAccessEventArgs);
}


// FUSEChmodEventArgs carries the parameters of the Chmod event of FUSE
pub struct FUSEChmodEventArgs
{
    Path : String,
    FileContext : usize,
    Mode : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ChmodEventArgs
impl FUSEChmodEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEChmodEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Chmod event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvMode : i32;
        unsafe
        {
            lvMode = *par.add(2) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(3) as i32;
        }
        
        FUSEChmodEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            Mode: lvMode,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn mode(&self) -> i32
    {
        self.Mode
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEChmodEvent
{
    fn on_chmod(&self, sender : &FUSE, e : &mut FUSEChmodEventArgs);
}


// FUSEChownEventArgs carries the parameters of the Chown event of FUSE
pub struct FUSEChownEventArgs
{
    Path : String,
    FileContext : usize,
    Uid : i32,
    Gid : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ChownEventArgs
impl FUSEChownEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEChownEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Chown event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvUid : i32;
        unsafe
        {
            lvUid = *par.add(2) as i32;
        }
        
        let lvGid : i32;
        unsafe
        {
            lvGid = *par.add(3) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(4) as i32;
        }
        
        FUSEChownEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            Uid: lvUid,
            Gid: lvGid,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(4)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn uid(&self) -> i32
    {
        self.Uid
    }
    pub fn gid(&self) -> i32
    {
        self.Gid
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEChownEvent
{
    fn on_chown(&self, sender : &FUSE, e : &mut FUSEChownEventArgs);
}


// FUSECopyFileRangeEventArgs carries the parameters of the CopyFileRange event of FUSE
pub struct FUSECopyFileRangeEventArgs
{
    PathIn : String,
    FileContextIn : usize,
    OffsetIn : i64,
    PathOut : String,
    FileContextOut : usize,
    OffsetOut : i64,
    Size : i64,
    Flags : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CopyFileRangeEventArgs
impl FUSECopyFileRangeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSECopyFileRangeEventArgs
    {

        let lvhPathInPtr : *mut c_char;
        let lvPathIn : String;
        unsafe
        {
            lvhPathInPtr = *par.add(0) as *mut c_char;
            if lvhPathInPtr == std::ptr::null_mut()
            {
                lvPathIn = String::default();
            }
            else
            {
                lvPathIn = CStr::from_ptr(lvhPathInPtr).to_str().expect("Valid UTF8 not received for the parameter 'PathIn' in the CopyFileRange event of a FUSE instance").to_owned();
            }
        }

        let lvFileContextIn : usize;
        unsafe
        {
            lvFileContextIn = *par.add(1) as usize;
        }

        let lvOffsetIn : i64;
        unsafe
        {
            let lvOffsetInLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvOffsetIn = *lvOffsetInLPtr;
        }
        
        let lvhPathOutPtr : *mut c_char;
        let lvPathOut : String;
        unsafe
        {
            lvhPathOutPtr = *par.add(3) as *mut c_char;
            if lvhPathOutPtr == std::ptr::null_mut()
            {
                lvPathOut = String::default();
            }
            else
            {
                lvPathOut = CStr::from_ptr(lvhPathOutPtr).to_str().expect("Valid UTF8 not received for the parameter 'PathOut' in the CopyFileRange event of a FUSE instance").to_owned();
            }
        }

        let lvFileContextOut : usize;
        unsafe
        {
            lvFileContextOut = *par.add(4) as usize;
        }

        let lvOffsetOut : i64;
        unsafe
        {
            let lvOffsetOutLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvOffsetOut = *lvOffsetOutLPtr;
        }
        
        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(6) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvFlags : i32;
        unsafe
        {
            lvFlags = *par.add(7) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(8) as i32;
        }
        
        FUSECopyFileRangeEventArgs
        {
            PathIn: lvPathIn,
            FileContextIn: lvFileContextIn,
            OffsetIn: lvOffsetIn,
            PathOut: lvPathOut,
            FileContextOut: lvFileContextOut,
            OffsetOut: lvOffsetOut,
            Size: lvSize,
            Flags: lvFlags,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(8)) = self.Result as isize;
        }
    }

    pub fn path_in(&self) -> &String
    {
        &self.PathIn
    }
    pub fn file_context_in(&self) -> usize
    {
        self.FileContextIn
    }
    pub fn offset_in(&self) -> i64
    {
        self.OffsetIn
    }
    pub fn path_out(&self) -> &String
    {
        &self.PathOut
    }
    pub fn file_context_out(&self) -> usize
    {
        self.FileContextOut
    }
    pub fn offset_out(&self) -> i64
    {
        self.OffsetOut
    }
    pub fn size(&self) -> i64
    {
        self.Size
    }
    pub fn flags(&self) -> i32
    {
        self.Flags
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSECopyFileRangeEvent
{
    fn on_copy_file_range(&self, sender : &FUSE, e : &mut FUSECopyFileRangeEventArgs);
}


// FUSECreateEventArgs carries the parameters of the Create event of FUSE
pub struct FUSECreateEventArgs
{
    Path : String,
    Mode : i32,
    FileContext : usize,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CreateEventArgs
impl FUSECreateEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSECreateEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Create event of a FUSE instance").to_owned();
            }
        }

        let lvMode : i32;
        unsafe
        {
            lvMode = *par.add(1) as i32;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(3) as i32;
        }
        
        FUSECreateEventArgs
        {
            Path: lvPath,
            Mode: lvMode,
            FileContext: lvFileContext,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.FileContext as isize;
            *(self._Params.add(3)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn mode(&self) -> i32
    {
        self.Mode
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSECreateEvent
{
    fn on_create(&self, sender : &FUSE, e : &mut FUSECreateEventArgs);
}


// FUSEDestroyEventArgs carries the parameters of the Destroy event of FUSE
pub struct FUSEDestroyEventArgs
{

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of DestroyEventArgs
impl FUSEDestroyEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEDestroyEventArgs
    {

        FUSEDestroyEventArgs
        {
            _Params: par
        }
    }


}

pub trait FUSEDestroyEvent
{
    fn on_destroy(&self, sender : &FUSE, e : &mut FUSEDestroyEventArgs);
}


// FUSEErrorEventArgs carries the parameters of the Error event of FUSE
pub struct FUSEErrorEventArgs
{
    ErrorCode : i32,
    Description : String,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ErrorEventArgs
impl FUSEErrorEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEErrorEventArgs
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
                lvDescription = CStr::from_ptr(lvhDescriptionPtr).to_str().expect("Valid UTF8 not received for the parameter 'Description' in the Error event of a FUSE instance").to_owned();
            }
        }

        FUSEErrorEventArgs
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

pub trait FUSEErrorEvent
{
    fn on_error(&self, sender : &FUSE, e : &mut FUSEErrorEventArgs);
}


// FUSEFAllocateEventArgs carries the parameters of the FAllocate event of FUSE
pub struct FUSEFAllocateEventArgs
{
    Path : String,
    FileContext : usize,
    Mode : i32,
    Offset : i64,
    Length : i64,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of FAllocateEventArgs
impl FUSEFAllocateEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEFAllocateEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the FAllocate event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvMode : i32;
        unsafe
        {
            lvMode = *par.add(2) as i32;
        }
        
        let lvOffset : i64;
        unsafe
        {
            let lvOffsetLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvOffset = *lvOffsetLPtr;
        }
        
        let lvLength : i64;
        unsafe
        {
            let lvLengthLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvLength = *lvLengthLPtr;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(5) as i32;
        }
        
        FUSEFAllocateEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            Mode: lvMode,
            Offset: lvOffset,
            Length: lvLength,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(5)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn mode(&self) -> i32
    {
        self.Mode
    }
    pub fn offset(&self) -> i64
    {
        self.Offset
    }
    pub fn length(&self) -> i64
    {
        self.Length
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEFAllocateEvent
{
    fn on_f_allocate(&self, sender : &FUSE, e : &mut FUSEFAllocateEventArgs);
}


// FUSEFlushEventArgs carries the parameters of the Flush event of FUSE
pub struct FUSEFlushEventArgs
{
    Path : String,
    FileContext : usize,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of FlushEventArgs
impl FUSEFlushEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEFlushEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Flush event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(2) as i32;
        }
        
        FUSEFlushEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEFlushEvent
{
    fn on_flush(&self, sender : &FUSE, e : &mut FUSEFlushEventArgs);
}


// FUSEFSyncEventArgs carries the parameters of the FSync event of FUSE
pub struct FUSEFSyncEventArgs
{
    Path : String,
    FileContext : usize,
    DataSync : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of FSyncEventArgs
impl FUSEFSyncEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEFSyncEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the FSync event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvDataSync : i32;
        unsafe
        {
            lvDataSync = *par.add(2) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(3) as i32;
        }
        
        FUSEFSyncEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            DataSync: lvDataSync,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn data_sync(&self) -> i32
    {
        self.DataSync
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEFSyncEvent
{
    fn on_f_sync(&self, sender : &FUSE, e : &mut FUSEFSyncEventArgs);
}


// FUSEGetAttrEventArgs carries the parameters of the GetAttr event of FUSE
pub struct FUSEGetAttrEventArgs
{
    Path : String,
    FileContext : usize,
    Ino : i64,
    Mode : i32,
    Uid : i32,
    Gid : i32,
    LinkCount : i32,
    Size : i64,
    ATime : chrono::DateTime<Utc>,
    MTime : chrono::DateTime<Utc>,
    CTime : chrono::DateTime<Utc>,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of GetAttrEventArgs
impl FUSEGetAttrEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEGetAttrEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the GetAttr event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvIno : i64;
        unsafe
        {
            let lvInoLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvIno = *lvInoLPtr;
        }
        
        let lvMode : i32;
        unsafe
        {
            lvMode = *par.add(3) as i32;
        }
        
        let lvUid : i32;
        unsafe
        {
            lvUid = *par.add(4) as i32;
        }
        
        let lvGid : i32;
        unsafe
        {
            lvGid = *par.add(5) as i32;
        }
        
        let lvLinkCount : i32;
        unsafe
        {
            lvLinkCount = *par.add(6) as i32;
        }
        
        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(7) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvATimeLong : i64;
        let lvATime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvATimeLPtr : *mut i64 = *par.add(8) as *mut i64;
            lvATimeLong = *lvATimeLPtr;
            lvATime = file_time_to_chrono_time(lvATimeLong);
        }

        let lvMTimeLong : i64;
        let lvMTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvMTimeLPtr : *mut i64 = *par.add(9) as *mut i64;
            lvMTimeLong = *lvMTimeLPtr;
            lvMTime = file_time_to_chrono_time(lvMTimeLong);
        }

        let lvCTimeLong : i64;
        let lvCTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvCTimeLPtr : *mut i64 = *par.add(10) as *mut i64;
            lvCTimeLong = *lvCTimeLPtr;
            lvCTime = file_time_to_chrono_time(lvCTimeLong);
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(11) as i32;
        }
        
        FUSEGetAttrEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            Ino: lvIno,
            Mode: lvMode,
            Uid: lvUid,
            Gid: lvGid,
            LinkCount: lvLinkCount,
            Size: lvSize,
            ATime: lvATime,
            MTime: lvMTime,
            CTime: lvCTime,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvInoLPtr : *mut i64 = *self._Params.add(2) as *mut i64;
            *lvInoLPtr = self.Ino ;
            *(self._Params.add(3)) = self.Mode as isize;
            *(self._Params.add(4)) = self.Uid as isize;
            *(self._Params.add(5)) = self.Gid as isize;
            *(self._Params.add(6)) = self.LinkCount as isize;
            let lvSizeLPtr : *mut i64 = *self._Params.add(7) as *mut i64;
            *lvSizeLPtr = self.Size ;
            let intValOfATime : i64 = chrono_time_to_file_time(&self.ATime);
            let lvATimeLPtr : *mut i64 = *self._Params.add(8) as *mut i64;
            *lvATimeLPtr = intValOfATime as i64;
            let intValOfMTime : i64 = chrono_time_to_file_time(&self.MTime);
            let lvMTimeLPtr : *mut i64 = *self._Params.add(9) as *mut i64;
            *lvMTimeLPtr = intValOfMTime as i64;
            let intValOfCTime : i64 = chrono_time_to_file_time(&self.CTime);
            let lvCTimeLPtr : *mut i64 = *self._Params.add(10) as *mut i64;
            *lvCTimeLPtr = intValOfCTime as i64;
            *(self._Params.add(11)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn ino(&self) -> i64
    {
        self.Ino
    }
    pub fn set_ino(&mut self, value: i64)
    {
        self.Ino = value;
    }
    pub fn mode(&self) -> i32
    {
        self.Mode
    }
    pub fn set_mode(&mut self, value: i32)
    {
        self.Mode = value;
    }
    pub fn uid(&self) -> i32
    {
        self.Uid
    }
    pub fn set_uid(&mut self, value: i32)
    {
        self.Uid = value;
    }
    pub fn gid(&self) -> i32
    {
        self.Gid
    }
    pub fn set_gid(&mut self, value: i32)
    {
        self.Gid = value;
    }
    pub fn link_count(&self) -> i32
    {
        self.LinkCount
    }
    pub fn set_link_count(&mut self, value: i32)
    {
        self.LinkCount = value;
    }
    pub fn size(&self) -> i64
    {
        self.Size
    }
    pub fn set_size(&mut self, value: i64)
    {
        self.Size = value;
    }
    pub fn a_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.ATime
    }
    pub fn set_a_time_ref(&mut self, value: &chrono::DateTime<Utc>)
    {
        self.ATime = value.clone();
    }
    pub fn set_a_time(&mut self, value: chrono::DateTime<Utc>)
    {
        self.ATime = value;
    }
    pub fn m_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.MTime
    }
    pub fn set_m_time_ref(&mut self, value: &chrono::DateTime<Utc>)
    {
        self.MTime = value.clone();
    }
    pub fn set_m_time(&mut self, value: chrono::DateTime<Utc>)
    {
        self.MTime = value;
    }
    pub fn c_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.CTime
    }
    pub fn set_c_time_ref(&mut self, value: &chrono::DateTime<Utc>)
    {
        self.CTime = value.clone();
    }
    pub fn set_c_time(&mut self, value: chrono::DateTime<Utc>)
    {
        self.CTime = value;
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEGetAttrEvent
{
    fn on_get_attr(&self, sender : &FUSE, e : &mut FUSEGetAttrEventArgs);
}


// FUSEInitEventArgs carries the parameters of the Init event of FUSE
pub struct FUSEInitEventArgs
{
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of InitEventArgs
impl FUSEInitEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEInitEventArgs
    {

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(0) as i32;
        }
        
        FUSEInitEventArgs
        {
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(0)) = self.Result as isize;
        }
    }

    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEInitEvent
{
    fn on_init(&self, sender : &FUSE, e : &mut FUSEInitEventArgs);
}


// FUSELockEventArgs carries the parameters of the Lock event of FUSE
pub struct FUSELockEventArgs
{
    Path : String,
    FileContext : usize,
    LockType : i32,
    LockStart : i64,
    LockLen : i64,
    Cmd : i32,
    LockOwner : i64,
    LockPid : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of LockEventArgs
impl FUSELockEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSELockEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Lock event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvLockType : i32;
        unsafe
        {
            lvLockType = *par.add(2) as i32;
        }
        
        let lvLockStart : i64;
        unsafe
        {
            let lvLockStartLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvLockStart = *lvLockStartLPtr;
        }
        
        let lvLockLen : i64;
        unsafe
        {
            let lvLockLenLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvLockLen = *lvLockLenLPtr;
        }
        
        let lvCmd : i32;
        unsafe
        {
            lvCmd = *par.add(5) as i32;
        }
        
        let lvLockOwner : i64;
        unsafe
        {
            let lvLockOwnerLPtr : *mut i64 = *par.add(6) as *mut i64;
            lvLockOwner = *lvLockOwnerLPtr;
        }
        
        let lvLockPid : i32;
        unsafe
        {
            lvLockPid = *par.add(7) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(8) as i32;
        }
        
        FUSELockEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            LockType: lvLockType,
            LockStart: lvLockStart,
            LockLen: lvLockLen,
            Cmd: lvCmd,
            LockOwner: lvLockOwner,
            LockPid: lvLockPid,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(7)) = self.LockPid as isize;
            *(self._Params.add(8)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn lock_type(&self) -> i32
    {
        self.LockType
    }
    pub fn lock_start(&self) -> i64
    {
        self.LockStart
    }
    pub fn lock_len(&self) -> i64
    {
        self.LockLen
    }
    pub fn cmd(&self) -> i32
    {
        self.Cmd
    }
    pub fn lock_owner(&self) -> i64
    {
        self.LockOwner
    }
    pub fn lock_pid(&self) -> i32
    {
        self.LockPid
    }
    pub fn set_lock_pid(&mut self, value: i32)
    {
        self.LockPid = value;
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSELockEvent
{
    fn on_lock(&self, sender : &FUSE, e : &mut FUSELockEventArgs);
}


// FUSEMkDirEventArgs carries the parameters of the MkDir event of FUSE
pub struct FUSEMkDirEventArgs
{
    Path : String,
    Mode : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of MkDirEventArgs
impl FUSEMkDirEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEMkDirEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the MkDir event of a FUSE instance").to_owned();
            }
        }

        let lvMode : i32;
        unsafe
        {
            lvMode = *par.add(1) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(2) as i32;
        }
        
        FUSEMkDirEventArgs
        {
            Path: lvPath,
            Mode: lvMode,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn mode(&self) -> i32
    {
        self.Mode
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEMkDirEvent
{
    fn on_mk_dir(&self, sender : &FUSE, e : &mut FUSEMkDirEventArgs);
}


// FUSEOpenEventArgs carries the parameters of the Open event of FUSE
pub struct FUSEOpenEventArgs
{
    Path : String,
    Flags : i32,
    DirectIO : bool,
    KeepCache : bool,
    NonSeekable : bool,
    FileContext : usize,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of OpenEventArgs
impl FUSEOpenEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEOpenEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Open event of a FUSE instance").to_owned();
            }
        }

        let lvFlags : i32;
        unsafe
        {
            lvFlags = *par.add(1) as i32;
        }
        
        let lvDirectIO : bool;
        unsafe
        {
            lvDirectIO = (*par.add(2) as i32) != 0;
        }

        let lvKeepCache : bool;
        unsafe
        {
            lvKeepCache = (*par.add(3) as i32) != 0;
        }

        let lvNonSeekable : bool;
        unsafe
        {
            lvNonSeekable = (*par.add(4) as i32) != 0;
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(5) as usize;
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(6) as i32;
        }
        
        FUSEOpenEventArgs
        {
            Path: lvPath,
            Flags: lvFlags,
            DirectIO: lvDirectIO,
            KeepCache: lvKeepCache,
            NonSeekable: lvNonSeekable,
            FileContext: lvFileContext,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfDirectIO : i32;
            if self.DirectIO
            {
                intValOfDirectIO = 1;
            }
            else
            {
                intValOfDirectIO = 0;
            }
            *(self._Params.add(2)) = intValOfDirectIO as isize;
            let intValOfKeepCache : i32;
            if self.KeepCache
            {
                intValOfKeepCache = 1;
            }
            else
            {
                intValOfKeepCache = 0;
            }
            *(self._Params.add(3)) = intValOfKeepCache as isize;
            let intValOfNonSeekable : i32;
            if self.NonSeekable
            {
                intValOfNonSeekable = 1;
            }
            else
            {
                intValOfNonSeekable = 0;
            }
            *(self._Params.add(4)) = intValOfNonSeekable as isize;
            *(self._Params.add(5)) = self.FileContext as isize;
            *(self._Params.add(6)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn flags(&self) -> i32
    {
        self.Flags
    }
    pub fn direct_io(&self) -> bool
    {
        self.DirectIO
    }
    pub fn set_direct_io(&mut self, value: bool)
    {
        self.DirectIO = value;
    }
    pub fn keep_cache(&self) -> bool
    {
        self.KeepCache
    }
    pub fn set_keep_cache(&mut self, value: bool)
    {
        self.KeepCache = value;
    }
    pub fn non_seekable(&self) -> bool
    {
        self.NonSeekable
    }
    pub fn set_non_seekable(&mut self, value: bool)
    {
        self.NonSeekable = value;
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEOpenEvent
{
    fn on_open(&self, sender : &FUSE, e : &mut FUSEOpenEventArgs);
}


// FUSEReadEventArgs carries the parameters of the Read event of FUSE
pub struct FUSEReadEventArgs
{
    Path : String,
    FileContext : usize,
    hBufferPtr : *mut u8,
    Size : i64,
    Offset : i64,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ReadEventArgs
impl FUSEReadEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEReadEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Read event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvhBufferPtr : *mut u8;
        unsafe
        {
            lvhBufferPtr = *par.add(2) as *mut u8;
        }

        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        // lvhBufferLen = lvSize;

        let lvOffset : i64;
        unsafe
        {
            let lvOffsetLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvOffset = *lvOffsetLPtr;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(5) as i32;
        }
        
        FUSEReadEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            hBufferPtr: lvhBufferPtr,
            Size: lvSize,
            Offset: lvOffset,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(5)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn buffer(&self) -> *mut u8
    {
        self.hBufferPtr
    }
    pub fn size(&self) -> i64
    {
        self.Size
    }
    pub fn offset(&self) -> i64
    {
        self.Offset
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEReadEvent
{
    fn on_read(&self, sender : &FUSE, e : &mut FUSEReadEventArgs);
}


// FUSEReadDirEventArgs carries the parameters of the ReadDir event of FUSE
pub struct FUSEReadDirEventArgs
{
    Path : String,
    FillerContext : i64,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ReadDirEventArgs
impl FUSEReadDirEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEReadDirEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the ReadDir event of a FUSE instance").to_owned();
            }
        }

        let lvFillerContext : i64;
        unsafe
        {
            let lvFillerContextLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvFillerContext = *lvFillerContextLPtr;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(2) as i32;
        }
        
        FUSEReadDirEventArgs
        {
            Path: lvPath,
            FillerContext: lvFillerContext,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn filler_context(&self) -> i64
    {
        self.FillerContext
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEReadDirEvent
{
    fn on_read_dir(&self, sender : &FUSE, e : &mut FUSEReadDirEventArgs);
}


// FUSEReleaseEventArgs carries the parameters of the Release event of FUSE
pub struct FUSEReleaseEventArgs
{
    Path : String,
    FileContext : usize,
    Flags : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ReleaseEventArgs
impl FUSEReleaseEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEReleaseEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Release event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvFlags : i32;
        unsafe
        {
            lvFlags = *par.add(2) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(3) as i32;
        }
        
        FUSEReleaseEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            Flags: lvFlags,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn flags(&self) -> i32
    {
        self.Flags
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEReleaseEvent
{
    fn on_release(&self, sender : &FUSE, e : &mut FUSEReleaseEventArgs);
}


// FUSERenameEventArgs carries the parameters of the Rename event of FUSE
pub struct FUSERenameEventArgs
{
    OldPath : String,
    NewPath : String,
    Flags : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of RenameEventArgs
impl FUSERenameEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSERenameEventArgs
    {

        let lvhOldPathPtr : *mut c_char;
        let lvOldPath : String;
        unsafe
        {
            lvhOldPathPtr = *par.add(0) as *mut c_char;
            if lvhOldPathPtr == std::ptr::null_mut()
            {
                lvOldPath = String::default();
            }
            else
            {
                lvOldPath = CStr::from_ptr(lvhOldPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'OldPath' in the Rename event of a FUSE instance").to_owned();
            }
        }

        let lvhNewPathPtr : *mut c_char;
        let lvNewPath : String;
        unsafe
        {
            lvhNewPathPtr = *par.add(1) as *mut c_char;
            if lvhNewPathPtr == std::ptr::null_mut()
            {
                lvNewPath = String::default();
            }
            else
            {
                lvNewPath = CStr::from_ptr(lvhNewPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'NewPath' in the Rename event of a FUSE instance").to_owned();
            }
        }

        let lvFlags : i32;
        unsafe
        {
            lvFlags = *par.add(2) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(3) as i32;
        }
        
        FUSERenameEventArgs
        {
            OldPath: lvOldPath,
            NewPath: lvNewPath,
            Flags: lvFlags,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.Result as isize;
        }
    }

    pub fn old_path(&self) -> &String
    {
        &self.OldPath
    }
    pub fn new_path(&self) -> &String
    {
        &self.NewPath
    }
    pub fn flags(&self) -> i32
    {
        self.Flags
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSERenameEvent
{
    fn on_rename(&self, sender : &FUSE, e : &mut FUSERenameEventArgs);
}


// FUSERmDirEventArgs carries the parameters of the RmDir event of FUSE
pub struct FUSERmDirEventArgs
{
    Path : String,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of RmDirEventArgs
impl FUSERmDirEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSERmDirEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the RmDir event of a FUSE instance").to_owned();
            }
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(1) as i32;
        }
        
        FUSERmDirEventArgs
        {
            Path: lvPath,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(1)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSERmDirEvent
{
    fn on_rm_dir(&self, sender : &FUSE, e : &mut FUSERmDirEventArgs);
}


// FUSEStatFSEventArgs carries the parameters of the StatFS event of FUSE
pub struct FUSEStatFSEventArgs
{
    Path : String,
    BlockSize : i64,
    TotalBlocks : i64,
    FreeBlocks : i64,
    FreeBlocksAvail : i64,
    TotalFiles : i64,
    FreeFiles : i64,
    MaxFilenameLength : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of StatFSEventArgs
impl FUSEStatFSEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEStatFSEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the StatFS event of a FUSE instance").to_owned();
            }
        }

        let lvBlockSize : i64;
        unsafe
        {
            let lvBlockSizeLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvBlockSize = *lvBlockSizeLPtr;
        }
        
        let lvTotalBlocks : i64;
        unsafe
        {
            let lvTotalBlocksLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvTotalBlocks = *lvTotalBlocksLPtr;
        }
        
        let lvFreeBlocks : i64;
        unsafe
        {
            let lvFreeBlocksLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvFreeBlocks = *lvFreeBlocksLPtr;
        }
        
        let lvFreeBlocksAvail : i64;
        unsafe
        {
            let lvFreeBlocksAvailLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvFreeBlocksAvail = *lvFreeBlocksAvailLPtr;
        }
        
        let lvTotalFiles : i64;
        unsafe
        {
            let lvTotalFilesLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvTotalFiles = *lvTotalFilesLPtr;
        }
        
        let lvFreeFiles : i64;
        unsafe
        {
            let lvFreeFilesLPtr : *mut i64 = *par.add(6) as *mut i64;
            lvFreeFiles = *lvFreeFilesLPtr;
        }
        
        let lvMaxFilenameLength : i32;
        unsafe
        {
            lvMaxFilenameLength = *par.add(7) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(8) as i32;
        }
        
        FUSEStatFSEventArgs
        {
            Path: lvPath,
            BlockSize: lvBlockSize,
            TotalBlocks: lvTotalBlocks,
            FreeBlocks: lvFreeBlocks,
            FreeBlocksAvail: lvFreeBlocksAvail,
            TotalFiles: lvTotalFiles,
            FreeFiles: lvFreeFiles,
            MaxFilenameLength: lvMaxFilenameLength,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvBlockSizeLPtr : *mut i64 = *self._Params.add(1) as *mut i64;
            *lvBlockSizeLPtr = self.BlockSize ;
            let lvTotalBlocksLPtr : *mut i64 = *self._Params.add(2) as *mut i64;
            *lvTotalBlocksLPtr = self.TotalBlocks ;
            let lvFreeBlocksLPtr : *mut i64 = *self._Params.add(3) as *mut i64;
            *lvFreeBlocksLPtr = self.FreeBlocks ;
            let lvFreeBlocksAvailLPtr : *mut i64 = *self._Params.add(4) as *mut i64;
            *lvFreeBlocksAvailLPtr = self.FreeBlocksAvail ;
            let lvTotalFilesLPtr : *mut i64 = *self._Params.add(5) as *mut i64;
            *lvTotalFilesLPtr = self.TotalFiles ;
            let lvFreeFilesLPtr : *mut i64 = *self._Params.add(6) as *mut i64;
            *lvFreeFilesLPtr = self.FreeFiles ;
            *(self._Params.add(7)) = self.MaxFilenameLength as isize;
            *(self._Params.add(8)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn block_size(&self) -> i64
    {
        self.BlockSize
    }
    pub fn set_block_size(&mut self, value: i64)
    {
        self.BlockSize = value;
    }
    pub fn total_blocks(&self) -> i64
    {
        self.TotalBlocks
    }
    pub fn set_total_blocks(&mut self, value: i64)
    {
        self.TotalBlocks = value;
    }
    pub fn free_blocks(&self) -> i64
    {
        self.FreeBlocks
    }
    pub fn set_free_blocks(&mut self, value: i64)
    {
        self.FreeBlocks = value;
    }
    pub fn free_blocks_avail(&self) -> i64
    {
        self.FreeBlocksAvail
    }
    pub fn set_free_blocks_avail(&mut self, value: i64)
    {
        self.FreeBlocksAvail = value;
    }
    pub fn total_files(&self) -> i64
    {
        self.TotalFiles
    }
    pub fn set_total_files(&mut self, value: i64)
    {
        self.TotalFiles = value;
    }
    pub fn free_files(&self) -> i64
    {
        self.FreeFiles
    }
    pub fn set_free_files(&mut self, value: i64)
    {
        self.FreeFiles = value;
    }
    pub fn max_filename_length(&self) -> i32
    {
        self.MaxFilenameLength
    }
    pub fn set_max_filename_length(&mut self, value: i32)
    {
        self.MaxFilenameLength = value;
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEStatFSEvent
{
    fn on_stat_fs(&self, sender : &FUSE, e : &mut FUSEStatFSEventArgs);
}


// FUSETruncateEventArgs carries the parameters of the Truncate event of FUSE
pub struct FUSETruncateEventArgs
{
    Path : String,
    FileContext : usize,
    Size : i64,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of TruncateEventArgs
impl FUSETruncateEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSETruncateEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Truncate event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(3) as i32;
        }
        
        FUSETruncateEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            Size: lvSize,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn size(&self) -> i64
    {
        self.Size
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSETruncateEvent
{
    fn on_truncate(&self, sender : &FUSE, e : &mut FUSETruncateEventArgs);
}


// FUSEUnlinkEventArgs carries the parameters of the Unlink event of FUSE
pub struct FUSEUnlinkEventArgs
{
    Path : String,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of UnlinkEventArgs
impl FUSEUnlinkEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEUnlinkEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Unlink event of a FUSE instance").to_owned();
            }
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(1) as i32;
        }
        
        FUSEUnlinkEventArgs
        {
            Path: lvPath,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(1)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEUnlinkEvent
{
    fn on_unlink(&self, sender : &FUSE, e : &mut FUSEUnlinkEventArgs);
}


// FUSEUTimeEventArgs carries the parameters of the UTime event of FUSE
pub struct FUSEUTimeEventArgs
{
    Path : String,
    FileContext : usize,
    ATime : chrono::DateTime<Utc>,
    MTime : chrono::DateTime<Utc>,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of UTimeEventArgs
impl FUSEUTimeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEUTimeEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the UTime event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvATimeLong : i64;
        let lvATime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvATimeLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvATimeLong = *lvATimeLPtr;
            lvATime = file_time_to_chrono_time(lvATimeLong);
        }

        let lvMTimeLong : i64;
        let lvMTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvMTimeLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvMTimeLong = *lvMTimeLPtr;
            lvMTime = file_time_to_chrono_time(lvMTimeLong);
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(4) as i32;
        }
        
        FUSEUTimeEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            ATime: lvATime,
            MTime: lvMTime,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(4)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn a_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.ATime
    }
    pub fn m_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.MTime
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEUTimeEvent
{
    fn on_u_time(&self, sender : &FUSE, e : &mut FUSEUTimeEventArgs);
}


// FUSEWriteEventArgs carries the parameters of the Write event of FUSE
pub struct FUSEWriteEventArgs
{
    Path : String,
    FileContext : usize,
    WritePage : bool,
    Offset : i64,
    hBufferPtr : *mut u8,
    Size : i64,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of WriteEventArgs
impl FUSEWriteEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> FUSEWriteEventArgs
    {

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(0) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Write event of a FUSE instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvWritePage : bool;
        unsafe
        {
            lvWritePage = (*par.add(2) as i32) != 0;
        }

        let lvOffset : i64;
        unsafe
        {
            let lvOffsetLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvOffset = *lvOffsetLPtr;
        }
        
        let lvhBufferPtr : *mut u8;
        unsafe
        {
            lvhBufferPtr = *par.add(4) as *mut u8;
        }

        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        // lvhBufferLen = lvSize;

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(6) as i32;
        }
        
        FUSEWriteEventArgs
        {
            Path: lvPath,
            FileContext: lvFileContext,
            WritePage: lvWritePage,
            Offset: lvOffset,
            hBufferPtr: lvhBufferPtr,
            Size: lvSize,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(6)) = self.Result as isize;
        }
    }

    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn write_page(&self) -> bool
    {
        self.WritePage
    }
    pub fn offset(&self) -> i64
    {
        self.Offset
    }
    pub fn buffer(&self) -> *mut u8
    {
        self.hBufferPtr
    }
    pub fn size(&self) -> i64
    {
        self.Size
    }
    pub fn result(&self) -> i32
    {
        self.Result
    }
    pub fn set_result(&mut self, value: i32)
    {
        self.Result = value;
    }
}

pub trait FUSEWriteEvent
{
    fn on_write(&self, sender : &FUSE, e : &mut FUSEWriteEventArgs);
}


////////////////////////////
// Main Class Implementation
////////////////////////////

/* The FUSE component gives applications the ability to create a virtual filesystem using a FUSE-like API in Windows, Linux, macOS. */
//pub struct FUSE<'a>
pub struct FUSE
{

    // onAccess : Option<&'a dyn FUSEAccessEvent>,
    onAccess : Option<fn (sender : &FUSE, e : &mut FUSEAccessEventArgs) >,
    // onChmod : Option<&'a dyn FUSEChmodEvent>,
    onChmod : Option<fn (sender : &FUSE, e : &mut FUSEChmodEventArgs) >,
    // onChown : Option<&'a dyn FUSEChownEvent>,
    onChown : Option<fn (sender : &FUSE, e : &mut FUSEChownEventArgs) >,
    // onCopyFileRange : Option<&'a dyn FUSECopyFileRangeEvent>,
    onCopyFileRange : Option<fn (sender : &FUSE, e : &mut FUSECopyFileRangeEventArgs) >,
    // onCreate : Option<&'a dyn FUSECreateEvent>,
    onCreate : Option<fn (sender : &FUSE, e : &mut FUSECreateEventArgs) >,
    // onDestroy : Option<&'a dyn FUSEDestroyEvent>,
    onDestroy : Option<fn (sender : &FUSE, e : &mut FUSEDestroyEventArgs) >,
    // onError : Option<&'a dyn FUSEErrorEvent>,
    onError : Option<fn (sender : &FUSE, e : &mut FUSEErrorEventArgs) >,
    // onFAllocate : Option<&'a dyn FUSEFAllocateEvent>,
    onFAllocate : Option<fn (sender : &FUSE, e : &mut FUSEFAllocateEventArgs) >,
    // onFlush : Option<&'a dyn FUSEFlushEvent>,
    onFlush : Option<fn (sender : &FUSE, e : &mut FUSEFlushEventArgs) >,
    // onFSync : Option<&'a dyn FUSEFSyncEvent>,
    onFSync : Option<fn (sender : &FUSE, e : &mut FUSEFSyncEventArgs) >,
    // onGetAttr : Option<&'a dyn FUSEGetAttrEvent>,
    onGetAttr : Option<fn (sender : &FUSE, e : &mut FUSEGetAttrEventArgs) >,
    // onInit : Option<&'a dyn FUSEInitEvent>,
    onInit : Option<fn (sender : &FUSE, e : &mut FUSEInitEventArgs) >,
    // onLock : Option<&'a dyn FUSELockEvent>,
    onLock : Option<fn (sender : &FUSE, e : &mut FUSELockEventArgs) >,
    // onMkDir : Option<&'a dyn FUSEMkDirEvent>,
    onMkDir : Option<fn (sender : &FUSE, e : &mut FUSEMkDirEventArgs) >,
    // onOpen : Option<&'a dyn FUSEOpenEvent>,
    onOpen : Option<fn (sender : &FUSE, e : &mut FUSEOpenEventArgs) >,
    // onRead : Option<&'a dyn FUSEReadEvent>,
    onRead : Option<fn (sender : &FUSE, e : &mut FUSEReadEventArgs) >,
    // onReadDir : Option<&'a dyn FUSEReadDirEvent>,
    onReadDir : Option<fn (sender : &FUSE, e : &mut FUSEReadDirEventArgs) >,
    // onRelease : Option<&'a dyn FUSEReleaseEvent>,
    onRelease : Option<fn (sender : &FUSE, e : &mut FUSEReleaseEventArgs) >,
    // onRename : Option<&'a dyn FUSERenameEvent>,
    onRename : Option<fn (sender : &FUSE, e : &mut FUSERenameEventArgs) >,
    // onRmDir : Option<&'a dyn FUSERmDirEvent>,
    onRmDir : Option<fn (sender : &FUSE, e : &mut FUSERmDirEventArgs) >,
    // onStatFS : Option<&'a dyn FUSEStatFSEvent>,
    onStatFS : Option<fn (sender : &FUSE, e : &mut FUSEStatFSEventArgs) >,
    // onTruncate : Option<&'a dyn FUSETruncateEvent>,
    onTruncate : Option<fn (sender : &FUSE, e : &mut FUSETruncateEventArgs) >,
    // onUnlink : Option<&'a dyn FUSEUnlinkEvent>,
    onUnlink : Option<fn (sender : &FUSE, e : &mut FUSEUnlinkEventArgs) >,
    // onUTime : Option<&'a dyn FUSEUTimeEvent>,
    onUTime : Option<fn (sender : &FUSE, e : &mut FUSEUTimeEventArgs) >,
    // onWrite : Option<&'a dyn FUSEWriteEvent>,
    onWrite : Option<fn (sender : &FUSE, e : &mut FUSEWriteEventArgs) >,

    Id : usize,
    Handle : usize 
}

//impl<'a> Drop for FUSE<'a>
impl Drop for FUSE
{
    fn drop(&mut self)
    {
        self.dispose();
    }
}

impl FUSE
{
    pub fn new() -> &'static mut FUSE
    {        
         #[cfg(target_os = "android")]
         panic!("FUSE is not available on Android");
        #[cfg(target_os = "ios")]
        panic!("FUSE is not available on iOS");
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
            lId = FUSEIDSeed.fetch_add(1, SeqCst) as usize;
        }

        let lHandle : isize;
        unsafe
        {
            let callable = CBFSConnect_FUSE_Create.clone().unwrap();
            lHandle = callable(FUSEEventDispatcher, lId, std::ptr::null(), FUSECreateOpt) as isize;
        }
        if lHandle < 0
        {
            panic!("Failed to instantiate FUSE. Please verify that it is supported on this platform");
        }

        let result : FUSE = FUSE
        {
            onAccess: None,
            onChmod: None,
            onChown: None,
            onCopyFileRange: None,
            onCreate: None,
            onDestroy: None,
            onError: None,
            onFAllocate: None,
            onFlush: None,
            onFSync: None,
            onGetAttr: None,
            onInit: None,
            onLock: None,
            onMkDir: None,
            onOpen: None,
            onRead: None,
            onReadDir: None,
            onRelease: None,
            onRename: None,
            onRmDir: None,
            onStatFS: None,
            onTruncate: None,
            onUnlink: None,
            onUTime: None,
            onWrite: None,
            Id: lId,
            Handle: lHandle as usize
        };

        let oem_key = CString::new(cbfsconnectkey::rtkCBFSConnect).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();

        unsafe
        {
            let callable = CBFSConnect_FUSE_SetCStr.clone().unwrap();
            ret_code = callable(lHandle as usize, 8012/*PID_KEYCHECK_RUST*/, 0, oem_key_ptr, 0) as i32;
            let _ = CString::from_raw(oem_key_ptr);
        }
        if ret_code != 0
        {
            panic!("Initialization of FUSE has failed with error {}", ret_code);
        }

        // Lock the Mutex to get access to the HashMap
        unsafe
        {
            let _map = FUSEDictMutex.lock().unwrap(); // It is used as a synchronization primitive - don't touch
            FUSEDict.insert(lId, result);
            let res = FUSEDict.get_mut(&lId).unwrap();
            return res;
        } // The lock is automatically released here
    }

    pub fn dispose(&self)
    {
        let mut _aself : Option<FUSE>;
        unsafe
        {
            let _map = FUSEDictMutex.lock().unwrap(); // It is used as a synchronization primitive - don't touch

            if !FUSEDict.contains_key(&self.Id)
            {
                return;
            }

            // Remove itself from the list
            _aself = FUSEDict.remove(&self.Id);

            // finalize the ctlclass
            let callable = CBFSConnect_FUSE_Destroy.clone().unwrap();
            callable(self.Handle);
        }
    }

/////////
// Events
/////////

    fn fire_access(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onAccess
        {
            let mut args : FUSEAccessEventArgs = FUSEAccessEventArgs::new(par, cbpar);
            callable/*.on_access*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_access(&self) -> &'a dyn FUSEAccessEvent
    pub fn on_access(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEAccessEventArgs)>
    {
        self.onAccess
    }

    //pub fn set_on_access(&mut self, value : &'a dyn FUSEAccessEvent)
    pub fn set_on_access(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEAccessEventArgs)>)
    {
        self.onAccess = value;
    }

    fn fire_chmod(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onChmod
        {
            let mut args : FUSEChmodEventArgs = FUSEChmodEventArgs::new(par, cbpar);
            callable/*.on_chmod*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_chmod(&self) -> &'a dyn FUSEChmodEvent
    pub fn on_chmod(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEChmodEventArgs)>
    {
        self.onChmod
    }

    //pub fn set_on_chmod(&mut self, value : &'a dyn FUSEChmodEvent)
    pub fn set_on_chmod(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEChmodEventArgs)>)
    {
        self.onChmod = value;
    }

    fn fire_chown(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onChown
        {
            let mut args : FUSEChownEventArgs = FUSEChownEventArgs::new(par, cbpar);
            callable/*.on_chown*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_chown(&self) -> &'a dyn FUSEChownEvent
    pub fn on_chown(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEChownEventArgs)>
    {
        self.onChown
    }

    //pub fn set_on_chown(&mut self, value : &'a dyn FUSEChownEvent)
    pub fn set_on_chown(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEChownEventArgs)>)
    {
        self.onChown = value;
    }

    fn fire_copy_file_range(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCopyFileRange
        {
            let mut args : FUSECopyFileRangeEventArgs = FUSECopyFileRangeEventArgs::new(par, cbpar);
            callable/*.on_copy_file_range*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_copy_file_range(&self) -> &'a dyn FUSECopyFileRangeEvent
    pub fn on_copy_file_range(&self) -> Option<fn (sender : &FUSE, e : &mut FUSECopyFileRangeEventArgs)>
    {
        self.onCopyFileRange
    }

    //pub fn set_on_copy_file_range(&mut self, value : &'a dyn FUSECopyFileRangeEvent)
    pub fn set_on_copy_file_range(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSECopyFileRangeEventArgs)>)
    {
        self.onCopyFileRange = value;
    }

    fn fire_create(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCreate
        {
            let mut args : FUSECreateEventArgs = FUSECreateEventArgs::new(par, cbpar);
            callable/*.on_create*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_create(&self) -> &'a dyn FUSECreateEvent
    pub fn on_create(&self) -> Option<fn (sender : &FUSE, e : &mut FUSECreateEventArgs)>
    {
        self.onCreate
    }

    //pub fn set_on_create(&mut self, value : &'a dyn FUSECreateEvent)
    pub fn set_on_create(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSECreateEventArgs)>)
    {
        self.onCreate = value;
    }

    fn fire_destroy(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDestroy
        {
            let mut args : FUSEDestroyEventArgs = FUSEDestroyEventArgs::new(par, cbpar);
            callable/*.on_destroy*/(&self, &mut args);
        }
    }

    //pub fn on_destroy(&self) -> &'a dyn FUSEDestroyEvent
    pub fn on_destroy(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEDestroyEventArgs)>
    {
        self.onDestroy
    }

    //pub fn set_on_destroy(&mut self, value : &'a dyn FUSEDestroyEvent)
    pub fn set_on_destroy(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEDestroyEventArgs)>)
    {
        self.onDestroy = value;
    }

    fn fire_error(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onError
        {
            let mut args : FUSEErrorEventArgs = FUSEErrorEventArgs::new(par, cbpar);
            callable/*.on_error*/(&self, &mut args);
        }
    }

    //pub fn on_error(&self) -> &'a dyn FUSEErrorEvent
    pub fn on_error(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEErrorEventArgs)>
    {
        self.onError
    }

    //pub fn set_on_error(&mut self, value : &'a dyn FUSEErrorEvent)
    pub fn set_on_error(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEErrorEventArgs)>)
    {
        self.onError = value;
    }

    fn fire_f_allocate(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFAllocate
        {
            let mut args : FUSEFAllocateEventArgs = FUSEFAllocateEventArgs::new(par, cbpar);
            callable/*.on_f_allocate*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_f_allocate(&self) -> &'a dyn FUSEFAllocateEvent
    pub fn on_f_allocate(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEFAllocateEventArgs)>
    {
        self.onFAllocate
    }

    //pub fn set_on_f_allocate(&mut self, value : &'a dyn FUSEFAllocateEvent)
    pub fn set_on_f_allocate(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEFAllocateEventArgs)>)
    {
        self.onFAllocate = value;
    }

    fn fire_flush(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFlush
        {
            let mut args : FUSEFlushEventArgs = FUSEFlushEventArgs::new(par, cbpar);
            callable/*.on_flush*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_flush(&self) -> &'a dyn FUSEFlushEvent
    pub fn on_flush(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEFlushEventArgs)>
    {
        self.onFlush
    }

    //pub fn set_on_flush(&mut self, value : &'a dyn FUSEFlushEvent)
    pub fn set_on_flush(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEFlushEventArgs)>)
    {
        self.onFlush = value;
    }

    fn fire_f_sync(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFSync
        {
            let mut args : FUSEFSyncEventArgs = FUSEFSyncEventArgs::new(par, cbpar);
            callable/*.on_f_sync*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_f_sync(&self) -> &'a dyn FUSEFSyncEvent
    pub fn on_f_sync(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEFSyncEventArgs)>
    {
        self.onFSync
    }

    //pub fn set_on_f_sync(&mut self, value : &'a dyn FUSEFSyncEvent)
    pub fn set_on_f_sync(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEFSyncEventArgs)>)
    {
        self.onFSync = value;
    }

    fn fire_get_attr(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onGetAttr
        {
            let mut args : FUSEGetAttrEventArgs = FUSEGetAttrEventArgs::new(par, cbpar);
            callable/*.on_get_attr*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_get_attr(&self) -> &'a dyn FUSEGetAttrEvent
    pub fn on_get_attr(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEGetAttrEventArgs)>
    {
        self.onGetAttr
    }

    //pub fn set_on_get_attr(&mut self, value : &'a dyn FUSEGetAttrEvent)
    pub fn set_on_get_attr(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEGetAttrEventArgs)>)
    {
        self.onGetAttr = value;
    }

    fn fire_init(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onInit
        {
            let mut args : FUSEInitEventArgs = FUSEInitEventArgs::new(par, cbpar);
            callable/*.on_init*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_init(&self) -> &'a dyn FUSEInitEvent
    pub fn on_init(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEInitEventArgs)>
    {
        self.onInit
    }

    //pub fn set_on_init(&mut self, value : &'a dyn FUSEInitEvent)
    pub fn set_on_init(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEInitEventArgs)>)
    {
        self.onInit = value;
    }

    fn fire_lock(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onLock
        {
            let mut args : FUSELockEventArgs = FUSELockEventArgs::new(par, cbpar);
            callable/*.on_lock*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_lock(&self) -> &'a dyn FUSELockEvent
    pub fn on_lock(&self) -> Option<fn (sender : &FUSE, e : &mut FUSELockEventArgs)>
    {
        self.onLock
    }

    //pub fn set_on_lock(&mut self, value : &'a dyn FUSELockEvent)
    pub fn set_on_lock(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSELockEventArgs)>)
    {
        self.onLock = value;
    }

    fn fire_mk_dir(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onMkDir
        {
            let mut args : FUSEMkDirEventArgs = FUSEMkDirEventArgs::new(par, cbpar);
            callable/*.on_mk_dir*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_mk_dir(&self) -> &'a dyn FUSEMkDirEvent
    pub fn on_mk_dir(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEMkDirEventArgs)>
    {
        self.onMkDir
    }

    //pub fn set_on_mk_dir(&mut self, value : &'a dyn FUSEMkDirEvent)
    pub fn set_on_mk_dir(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEMkDirEventArgs)>)
    {
        self.onMkDir = value;
    }

    fn fire_open(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onOpen
        {
            let mut args : FUSEOpenEventArgs = FUSEOpenEventArgs::new(par, cbpar);
            callable/*.on_open*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_open(&self) -> &'a dyn FUSEOpenEvent
    pub fn on_open(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEOpenEventArgs)>
    {
        self.onOpen
    }

    //pub fn set_on_open(&mut self, value : &'a dyn FUSEOpenEvent)
    pub fn set_on_open(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEOpenEventArgs)>)
    {
        self.onOpen = value;
    }

    fn fire_read(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onRead
        {
            let mut args : FUSEReadEventArgs = FUSEReadEventArgs::new(par, cbpar);
            callable/*.on_read*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_read(&self) -> &'a dyn FUSEReadEvent
    pub fn on_read(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEReadEventArgs)>
    {
        self.onRead
    }

    //pub fn set_on_read(&mut self, value : &'a dyn FUSEReadEvent)
    pub fn set_on_read(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEReadEventArgs)>)
    {
        self.onRead = value;
    }

    fn fire_read_dir(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onReadDir
        {
            let mut args : FUSEReadDirEventArgs = FUSEReadDirEventArgs::new(par, cbpar);
            callable/*.on_read_dir*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_read_dir(&self) -> &'a dyn FUSEReadDirEvent
    pub fn on_read_dir(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEReadDirEventArgs)>
    {
        self.onReadDir
    }

    //pub fn set_on_read_dir(&mut self, value : &'a dyn FUSEReadDirEvent)
    pub fn set_on_read_dir(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEReadDirEventArgs)>)
    {
        self.onReadDir = value;
    }

    fn fire_release(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onRelease
        {
            let mut args : FUSEReleaseEventArgs = FUSEReleaseEventArgs::new(par, cbpar);
            callable/*.on_release*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_release(&self) -> &'a dyn FUSEReleaseEvent
    pub fn on_release(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEReleaseEventArgs)>
    {
        self.onRelease
    }

    //pub fn set_on_release(&mut self, value : &'a dyn FUSEReleaseEvent)
    pub fn set_on_release(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEReleaseEventArgs)>)
    {
        self.onRelease = value;
    }

    fn fire_rename(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onRename
        {
            let mut args : FUSERenameEventArgs = FUSERenameEventArgs::new(par, cbpar);
            callable/*.on_rename*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_rename(&self) -> &'a dyn FUSERenameEvent
    pub fn on_rename(&self) -> Option<fn (sender : &FUSE, e : &mut FUSERenameEventArgs)>
    {
        self.onRename
    }

    //pub fn set_on_rename(&mut self, value : &'a dyn FUSERenameEvent)
    pub fn set_on_rename(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSERenameEventArgs)>)
    {
        self.onRename = value;
    }

    fn fire_rm_dir(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onRmDir
        {
            let mut args : FUSERmDirEventArgs = FUSERmDirEventArgs::new(par, cbpar);
            callable/*.on_rm_dir*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_rm_dir(&self) -> &'a dyn FUSERmDirEvent
    pub fn on_rm_dir(&self) -> Option<fn (sender : &FUSE, e : &mut FUSERmDirEventArgs)>
    {
        self.onRmDir
    }

    //pub fn set_on_rm_dir(&mut self, value : &'a dyn FUSERmDirEvent)
    pub fn set_on_rm_dir(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSERmDirEventArgs)>)
    {
        self.onRmDir = value;
    }

    fn fire_stat_fs(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onStatFS
        {
            let mut args : FUSEStatFSEventArgs = FUSEStatFSEventArgs::new(par, cbpar);
            callable/*.on_stat_fs*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_stat_fs(&self) -> &'a dyn FUSEStatFSEvent
    pub fn on_stat_fs(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEStatFSEventArgs)>
    {
        self.onStatFS
    }

    //pub fn set_on_stat_fs(&mut self, value : &'a dyn FUSEStatFSEvent)
    pub fn set_on_stat_fs(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEStatFSEventArgs)>)
    {
        self.onStatFS = value;
    }

    fn fire_truncate(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onTruncate
        {
            let mut args : FUSETruncateEventArgs = FUSETruncateEventArgs::new(par, cbpar);
            callable/*.on_truncate*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_truncate(&self) -> &'a dyn FUSETruncateEvent
    pub fn on_truncate(&self) -> Option<fn (sender : &FUSE, e : &mut FUSETruncateEventArgs)>
    {
        self.onTruncate
    }

    //pub fn set_on_truncate(&mut self, value : &'a dyn FUSETruncateEvent)
    pub fn set_on_truncate(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSETruncateEventArgs)>)
    {
        self.onTruncate = value;
    }

    fn fire_unlink(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onUnlink
        {
            let mut args : FUSEUnlinkEventArgs = FUSEUnlinkEventArgs::new(par, cbpar);
            callable/*.on_unlink*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_unlink(&self) -> &'a dyn FUSEUnlinkEvent
    pub fn on_unlink(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEUnlinkEventArgs)>
    {
        self.onUnlink
    }

    //pub fn set_on_unlink(&mut self, value : &'a dyn FUSEUnlinkEvent)
    pub fn set_on_unlink(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEUnlinkEventArgs)>)
    {
        self.onUnlink = value;
    }

    fn fire_u_time(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onUTime
        {
            let mut args : FUSEUTimeEventArgs = FUSEUTimeEventArgs::new(par, cbpar);
            callable/*.on_u_time*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_u_time(&self) -> &'a dyn FUSEUTimeEvent
    pub fn on_u_time(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEUTimeEventArgs)>
    {
        self.onUTime
    }

    //pub fn set_on_u_time(&mut self, value : &'a dyn FUSEUTimeEvent)
    pub fn set_on_u_time(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEUTimeEventArgs)>)
    {
        self.onUTime = value;
    }

    fn fire_write(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onWrite
        {
            let mut args : FUSEWriteEventArgs = FUSEWriteEventArgs::new(par, cbpar);
            callable/*.on_write*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_write(&self) -> &'a dyn FUSEWriteEvent
    pub fn on_write(&self) -> Option<fn (sender : &FUSE, e : &mut FUSEWriteEventArgs)>
    {
        self.onWrite
    }

    //pub fn set_on_write(&mut self, value : &'a dyn FUSEWriteEvent)
    pub fn set_on_write(&mut self, value : Option<fn (sender : &FUSE, e : &mut FUSEWriteEventArgs)>)
    {
        self.onWrite = value;
    }


    pub(crate) fn report_error_info(&self, error_info : &str)
    {
        if let Some(callable) = self.onError
        {
            let mut args : FUSEErrorEventArgs = FUSEErrorEventArgs
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
            let callable = CBFSConnect_FUSE_GetLastError.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_GetLastErrorCode.clone().unwrap();
            result = callable(self.Handle) as i32;
        }
        result
    }

    // GetRuntimeLicense returns the runtime license key set for FUSE.
    pub fn get_runtime_license(&self) -> Result<String, CBFSConnectError>
    {
        let result : String;
        //let length : c_long;
        unsafe
        {
            let callable = CBFSConnect_FUSE_GetAsCStr.clone().unwrap();

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

    // SetRuntimeLicense sets the runtime license key for FUSE.
    pub fn set_runtime_license(&self, value : String) -> Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let oem_key = CString::new(value).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();
        unsafe
        {
            let callable = CBFSConnect_FUSE_SetCStr.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_GetAsInt.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 2, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessDesiredAccess"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_FUSE_GetAsInt.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 3, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessIncludeChildren"));
            }
        }
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_FUSE_GetAsInt.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 4, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessId"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_FUSE_GetAsInt.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 5, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessName"));
            }
        }
        unsafe
        {
            let callable = CBFSConnect_FUSE_GetAsCStr.clone().unwrap();

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
            let callable = CBFSConnect_FUSE_GetAsInt.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 7, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessDesiredAccess"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_FUSE_GetAsInt.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 8, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessIncludeChildren"));
            }
        }
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_FUSE_GetAsInt.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 9, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessId"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_FUSE_GetAsInt.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 10, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessName"));
            }
        }
        unsafe
        {
            let callable = CBFSConnect_FUSE_GetAsCStr.clone().unwrap();

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


    // Gets the value of the ProcessRestrictionsEnabled property: Whether process access restrictions are enabled (Windows and Linux).
    pub fn process_restrictions_enabled(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_FUSE_GetAsInt.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 11, 0 as c_long, IntValue as isize, 0) as i32;
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
            let callable = CBFSConnect_FUSE_GetAsInt.clone().unwrap();
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


    // Sets the value of the SerializeEvents property: Whether events should be fired on a single worker thread, or many.
    pub fn set_serialize_events(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_FUSE_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 12, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the StorageCharacteristics property: This property includes the characteristic flags with which to create the virtual drive (Windows only).
    pub fn storage_characteristics(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_FUSE_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 13, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the StorageCharacteristics property: This property includes the characteristic flags with which to create the virtual drive (Windows only).
    pub fn set_storage_characteristics(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_FUSE_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 13, 0 as c_long, value as isize, 0) as i32;
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
            let callable = CBFSConnect_FUSE_GetAsInt64.clone().unwrap();
            callable(self.Handle, 14, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
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
            let callable = CBFSConnect_FUSE_SetInt64.clone().unwrap();
            ret_code = callable(self.Handle, 14, 0 as c_long, ValuePtr as *const i64, 0) as i32;
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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 3, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProcessFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn add_granted_process

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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 4, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrConfigurationStringPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn config

    // FileTimeToNanoseconds: This method returns the subsecond part of the time expressed in nanoseconds.
    pub fn file_time_to_nanoseconds(&self, file_time : &chrono::DateTime<Utc>) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let FileTimeUnixDate : i64 = chrono_time_to_file_time(file_time);
        let FileTimeArr : Vec<i64> = vec![FileTimeUnixDate];
        CParams[0] = FileTimeArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 5, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] as i32;
        return Result::Ok(ret_val);
    } // fn file_time_to_nanoseconds

    // FileTimeToUnixTime: This method converts FileTime to the Unix time format.
    pub fn file_time_to_unix_time(&self, file_time : &chrono::DateTime<Utc>) ->  Result<i64, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let FileTimeUnixDate : i64 = chrono_time_to_file_time(file_time);
        let FileTimeArr : Vec<i64> = vec![FileTimeUnixDate];
        CParams[0] = FileTimeArr.as_ptr() as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 6, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn file_time_to_unix_time

    // FillDir: This method fills the buffer with information about a directory entry.
    pub fn fill_dir(&self, filler_context : i64, name : &str, ino : i64, mode : i32, uid : i32, gid : i32, link_count : i32, size : i64, a_time : &chrono::DateTime<Utc>, m_time : &chrono::DateTime<Utc>, c_time : &chrono::DateTime<Utc>) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 11 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let FillerContextArr : Vec<i64> = vec![filler_context];
        CParams[0] = FillerContextArr.as_ptr() as usize;
        let CStrNamePtr : *mut c_char;
        match CString::new(name)
        {
            Ok(CStrValueName) => { CStrNamePtr = CStrValueName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrNamePtr as usize;
        let InoArr : Vec<i64> = vec![ino];
        CParams[2] = InoArr.as_ptr() as usize;
        CParams[3] = (mode as isize) as usize;
        CParams[4] = (uid as isize) as usize;
        CParams[5] = (gid as isize) as usize;
        CParams[6] = (link_count as isize) as usize;
        let SizeArr : Vec<i64> = vec![size];
        CParams[7] = SizeArr.as_ptr() as usize;
        let ATimeUnixDate : i64 = chrono_time_to_file_time(a_time);
        let ATimeArr : Vec<i64> = vec![ATimeUnixDate];
        CParams[8] = ATimeArr.as_ptr() as usize;
        let MTimeUnixDate : i64 = chrono_time_to_file_time(m_time);
        let MTimeArr : Vec<i64> = vec![MTimeUnixDate];
        CParams[9] = MTimeArr.as_ptr() as usize;
        let CTimeUnixDate : i64 = chrono_time_to_file_time(c_time);
        let CTimeArr : Vec<i64> = vec![CTimeUnixDate];
        CParams[10] = CTimeArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 7, 11, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[11] as i32;
        return Result::Ok(ret_val);
    } // fn fill_dir

    // GetDriverStatus: This method retrieves the status of the system driver.
    pub fn get_driver_status(&self, product_guid : &str) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 8, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProductGUIDPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] as i32;
        return Result::Ok(ret_val);
    } // fn get_driver_status

    // GetDriverVersion: This method retrieves the version of the system driver.
    pub fn get_driver_version(&self, product_guid : &str) ->  Result<i64, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
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

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 9, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrProductGUIDPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_driver_version

    // GetGid: This method returns the group Id of the caller process.
    pub fn get_gid(&self, ) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 10, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[0] as i32;
        return Result::Ok(ret_val);
    } // fn get_gid

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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 11, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 12, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 13, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 14, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_originator_token

    // GetUid: The method returns the user Id of the caller process.
    pub fn get_uid(&self, ) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 15, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[0] as i32;
        return Result::Ok(ret_val);
    } // fn get_uid

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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 16, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProductGUIDPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn initialize

    // Install: This method installs or upgrades the product's system drivers (Windows only).
    pub fn install(&self, cab_file_name : &str, product_guid : &str, path_to_install : &str, flags : i32) ->  Result<i32, CBFSConnectError>
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
        let CStrPathToInstallPtr : *mut c_char;
        match CString::new(path_to_install)
        {
            Ok(CStrValuePathToInstall) => { CStrPathToInstallPtr = CStrValuePathToInstall.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrPathToInstallPtr as usize;
        CParams[3] = (flags as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 17, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrCabFileNamePtr);
            let _ = CString::from_raw(CStrProductGUIDPtr);
            let _ = CString::from_raw(CStrPathToInstallPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[4] as i32;
        return Result::Ok(ret_val);
    } // fn install

    // Mount: This method creates a virtual drive or directory and mounts a filesystem.
    pub fn mount(&self, mount_point : &str) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrMountPointPtr : *mut c_char;
        match CString::new(mount_point)
        {
            Ok(CStrValueMountPoint) => { CStrMountPointPtr = CStrValueMountPoint.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrMountPointPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 18, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrMountPointPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn mount

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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 19, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 20, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProcessFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn remove_granted_process

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
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 21, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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

    // UnixTimeToFileTime: This event converts the date/time in Unix format to Windows FileTime format.
    pub fn unix_time_to_file_time(&self, unix_time : i64, nanoseconds : i32) ->  Result<chrono::DateTime<Utc>, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : chrono::DateTime<Utc> ; // = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let UnixTimeArr : Vec<i64> = vec![unix_time];
        CParams[0] = UnixTimeArr.as_ptr() as usize;
        CParams[1] = (nanoseconds as isize) as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 22, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = file_time_to_chrono_time(ll_buf);
        return Result::Ok(ret_val);
    } // fn unix_time_to_file_time

    // Unmount: This method unmounts a filesystem.
    pub fn unmount(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_FUSE_Do.clone().unwrap();
            ret_code = callable(self.Handle, 23, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn unmount

} // FUSE

extern "system" fn FUSEEventDispatcher(pObj : usize, event_id : c_long, _cparam : c_long, param : IntPtrArrayType, cbparam : IntArrayType) -> c_long
{
    let obj: &'static FUSE;
    // Lock the Mutex to get access to the HashMap
    unsafe
    {
        let _map = FUSEDictMutex.lock().unwrap();
        let objOpt = FUSEDict.get(&pObj);
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
            1 /* Access */=> obj.fire_access(/*cparam as i32, */param, cbparam),

            2 /* Chmod */=> obj.fire_chmod(/*cparam as i32, */param, cbparam),

            3 /* Chown */=> obj.fire_chown(/*cparam as i32, */param, cbparam),

            4 /* CopyFileRange */=> obj.fire_copy_file_range(/*cparam as i32, */param, cbparam),

            5 /* Create */=> obj.fire_create(/*cparam as i32, */param, cbparam),

            6 /* Destroy */=> obj.fire_destroy(/*cparam as i32, */param, cbparam),

            7 /* Error */=> obj.fire_error(/*cparam as i32, */param, cbparam),

            8 /* FAllocate */=> obj.fire_f_allocate(/*cparam as i32, */param, cbparam),

            9 /* Flush */=> obj.fire_flush(/*cparam as i32, */param, cbparam),

            10 /* FSync */=> obj.fire_f_sync(/*cparam as i32, */param, cbparam),

            11 /* GetAttr */=> obj.fire_get_attr(/*cparam as i32, */param, cbparam),

            12 /* Init */=> obj.fire_init(/*cparam as i32, */param, cbparam),

            13 /* Lock */=> obj.fire_lock(/*cparam as i32, */param, cbparam),

            14 /* MkDir */=> obj.fire_mk_dir(/*cparam as i32, */param, cbparam),

            15 /* Open */=> obj.fire_open(/*cparam as i32, */param, cbparam),

            16 /* Read */=> obj.fire_read(/*cparam as i32, */param, cbparam),

            17 /* ReadDir */=> obj.fire_read_dir(/*cparam as i32, */param, cbparam),

            18 /* Release */=> obj.fire_release(/*cparam as i32, */param, cbparam),

            19 /* Rename */=> obj.fire_rename(/*cparam as i32, */param, cbparam),

            20 /* RmDir */=> obj.fire_rm_dir(/*cparam as i32, */param, cbparam),

            21 /* StatFS */=> obj.fire_stat_fs(/*cparam as i32, */param, cbparam),

            22 /* Truncate */=> obj.fire_truncate(/*cparam as i32, */param, cbparam),

            23 /* Unlink */=> obj.fire_unlink(/*cparam as i32, */param, cbparam),

            24 /* UTime */=> obj.fire_u_time(/*cparam as i32, */param, cbparam),

            25 /* Write */=> obj.fire_write(/*cparam as i32, */param, cbparam),

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


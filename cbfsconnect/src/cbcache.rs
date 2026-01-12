


#![allow(non_snake_case)]

extern crate libloading as lib;

use std::{collections::HashMap, ffi::{c_char, c_long, c_longlong, c_ulong, c_void, CStr, CString}, panic::catch_unwind, sync::{atomic::{AtomicUsize, Ordering::SeqCst}, Mutex} };
use lib::{Library, Symbol, Error};
use std::fmt::Write;
use chrono::Utc;
use once_cell::sync::Lazy;

use crate::{*, cbfsconnectkey};

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_StaticInit)(void *hInst);
type CBFSConnectCBCacheStaticInitType = unsafe extern "system" fn(hInst : *mut c_void) -> i32;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_StaticDestroy)();
type CBFSConnectCBCacheStaticDestroyType = unsafe extern "system" fn()-> i32;

// typedef void* (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_Create)(PCBFSCONNECT_CALLBACK lpSink, void *lpContext, char *lpOemKey, int opts);
type CBFSConnectCBCacheCreateType = unsafe extern "system" fn(lpSink : CBFSConnectSinkDelegateType, lpContext : usize, lpOemKey : *const c_char, opts : i32) -> usize;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_Destroy)(void *lpObj);
type CBFSConnectCBCacheDestroyType = unsafe extern "system" fn(lpObj: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_CheckIndex)(void *lpObj, int propid, int arridx);
type CBFSConnectCBCacheCheckIndexType = unsafe extern "system" fn(lpObj: usize, propid: c_long, arridx: c_long)-> c_long;

// typedef void* (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_Get)(void *lpObj, int propid, int arridx, int *lpcbVal, int64 *lpllVal);
type CBFSConnectCBCacheGetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *mut c_longlong) -> *mut c_void;
type CBFSConnectCBCacheGetAsCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *const c_longlong) -> *const c_char;
type CBFSConnectCBCacheGetAsIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *const c_void) -> usize;
type CBFSConnectCBCacheGetAsInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *mut i64) -> usize;
type CBFSConnectCBCacheGetAsBSTRType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut i32, llVal: *const c_void) -> *const u8;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_Set)(void *lpObj, int propid, int arridx, const void *val, int cbVal);
type CBFSConnectCBCacheSetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_void, len: c_ulong)-> c_long;
type CBFSConnectCBCacheSetCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_char, len: c_ulong)-> c_long;
type CBFSConnectCBCacheSetIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: isize, len: c_ulong)-> c_long;
type CBFSConnectCBCacheSetInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const i64, len: c_ulong)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_Do)(void *lpObj, int methid, int cparam, void *param[], int cbparam[], int64 *lpllVal);
type CBFSConnectCBCacheDoType = unsafe extern "system" fn(p: usize, method_id: c_long, cparam: c_long, params: UIntPtrArrayType, cbparam: IntArrayType, llVal: *mut c_longlong)-> c_long;

// typedef char* (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_GetLastError)(void *lpObj);
type CBFSConnectCBCacheGetLastErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_GetLastErrorCode)(void *lpObj);
type CBFSConnectCBCacheGetLastErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_SetLastErrorAndCode)(void *lpObj, int code, char *message);
type CBFSConnectCBCacheSetLastErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

// typedef char* (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_GetEventError)(void *lpObj);
type CBFSConnectCBCacheGetEventErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_GetEventErrorCode)(void *lpObj);
type CBFSConnectCBCacheGetEventErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_CBCache_SetEventErrorAndCode)(void *lpObj, int code, char *message);
type CBFSConnectCBCacheSetEventErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

static mut CBFSConnect_CBCache_StaticInit : Option<Symbol<CBFSConnectCBCacheStaticInitType>> = None;
static mut CBFSConnect_CBCache_StaticDestroy : Option<Symbol<CBFSConnectCBCacheStaticDestroyType>> = None;

static mut CBFSConnect_CBCache_Create: Option<Symbol<CBFSConnectCBCacheCreateType>> = None;
static mut CBFSConnect_CBCache_Destroy: Option<Symbol<CBFSConnectCBCacheDestroyType>> = None;
static mut CBFSConnect_CBCache_Set: Option<Symbol<CBFSConnectCBCacheSetType>> = None;
static mut CBFSConnect_CBCache_SetCStr: Option<Symbol<CBFSConnectCBCacheSetCStrType>> = None;
static mut CBFSConnect_CBCache_SetInt: Option<Symbol<CBFSConnectCBCacheSetIntType>> = None;
static mut CBFSConnect_CBCache_SetInt64: Option<Symbol<CBFSConnectCBCacheSetInt64Type>> = None;
static mut CBFSConnect_CBCache_Get: Option<Symbol<CBFSConnectCBCacheGetType>> = None;
static mut CBFSConnect_CBCache_GetAsCStr: Option<Symbol<CBFSConnectCBCacheGetAsCStrType>> = None;
static mut CBFSConnect_CBCache_GetAsInt: Option<Symbol<CBFSConnectCBCacheGetAsIntType>> = None;
static mut CBFSConnect_CBCache_GetAsInt64: Option<Symbol<CBFSConnectCBCacheGetAsInt64Type>> = None;
static mut CBFSConnect_CBCache_GetAsBSTR: Option<Symbol<CBFSConnectCBCacheGetAsBSTRType>> = None;
static mut CBFSConnect_CBCache_GetLastError: Option<Symbol<CBFSConnectCBCacheGetLastErrorType>> = None;
static mut CBFSConnect_CBCache_GetLastErrorCode: Option<Symbol<CBFSConnectCBCacheGetLastErrorCodeType>> = None;
static mut CBFSConnect_CBCache_SetLastErrorAndCode: Option<Symbol<CBFSConnectCBCacheSetLastErrorAndCodeType>> = None;
static mut CBFSConnect_CBCache_GetEventError: Option<Symbol<CBFSConnectCBCacheGetEventErrorType>> = None;
static mut CBFSConnect_CBCache_GetEventErrorCode: Option<Symbol<CBFSConnectCBCacheGetEventErrorCodeType>> = None;
static mut CBFSConnect_CBCache_SetEventErrorAndCode: Option<Symbol<CBFSConnectCBCacheSetEventErrorAndCodeType>> = None;
static mut CBFSConnect_CBCache_CheckIndex: Option<Symbol<CBFSConnectCBCacheCheckIndexType>> = None;
static mut CBFSConnect_CBCache_Do: Option<Symbol<CBFSConnectCBCacheDoType>> = None;

static mut CBCacheIDSeed : AtomicUsize = AtomicUsize::new(1);

static mut CBCacheDict : Lazy<HashMap<usize, CBCache>> = Lazy::new(|| HashMap::new() );
static CBCacheDictMutex : Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0) );

const CBCacheCreateOpt : i32 = 0;


pub type CBCacheStream = crate::CBFSConnectStream;


pub(crate) fn get_lib_funcs( lib_hand : &'static Library) -> bool
{

    unsafe
    {
        // CBFSConnect_CBCache_StaticInit
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheStaticInitType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_StaticInit");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_CBCache_StaticInit = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_StaticDestroy
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheStaticDestroyType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_StaticDestroy");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_CBCache_StaticDestroy = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_Create
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheCreateType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Create");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_CBCache_Create = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_Destroy
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheDestroyType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Destroy");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_CBCache_Destroy = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_Get
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheGetType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_Get = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBCache_GetAsCStr
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheGetAsCStrType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_GetAsCStr = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBCache_GetAsInt
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheGetAsIntType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_GetAsInt = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBCache_GetAsInt64
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheGetAsInt64Type>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_GetAsInt64 = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBCache_GetAsBSTR
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheGetAsBSTRType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_GetAsBSTR = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_Set
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheSetType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_Set = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBCache_SetCStr
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheSetCStrType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_SetCStr = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBCache_SetInt
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheSetIntType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_SetInt = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_CBCache_SetInt64
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheSetInt64Type>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_SetInt64 = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_GetLastError
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheGetLastErrorType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_GetLastError");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_GetLastError = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_GetLastErrorCode
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheGetLastErrorCodeType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_GetLastErrorCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_GetLastErrorCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_SetLastErrorAndCode
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheSetLastErrorAndCodeType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_SetLastErrorAndCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_SetLastErrorAndCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_GetEventError
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheGetEventErrorType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_GetEventError");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_GetEventError = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_GetEventErrorCode
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheGetEventErrorCodeType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_GetEventErrorCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_GetEventErrorCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_SetEventErrorAndCode
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheSetEventErrorAndCodeType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_SetEventErrorAndCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_SetEventErrorAndCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_CheckIndex
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheCheckIndexType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_CheckIndex");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_CheckIndex = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_CBCache_Do
        let func_ptr_res : Result<Symbol<CBFSConnectCBCacheDoType>, Error> = lib_hand.get(b"CBFSConnect_CBCache_Do");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_CBCache_Do = func_ptr_res.ok();
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


// CBCacheBeforeFlushEventArgs carries the parameters of the BeforeFlush event of CBCache
pub struct CBCacheBeforeFlushEventArgs
{
    FileId : String,
    FileContext : usize,
    LastUseTime : chrono::DateTime<Utc>,
    PostponeTime : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of BeforeFlushEventArgs
impl CBCacheBeforeFlushEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBCacheBeforeFlushEventArgs
    {

        let lvhFileIdPtr : *mut c_char;
        let lvFileId : String;
        unsafe
        {
            lvhFileIdPtr = *par.add(0) as *mut c_char;
            if lvhFileIdPtr == std::ptr::null_mut()
            {
                lvFileId = String::default();
            }
            else
            {
                lvFileId = CStr::from_ptr(lvhFileIdPtr).to_str().expect("Valid UTF8 not received for the parameter 'FileId' in the BeforeFlush event of a CBCache instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvLastUseTimeLong : i64;
        let lvLastUseTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvLastUseTimeLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvLastUseTimeLong = *lvLastUseTimeLPtr;
            lvLastUseTime = file_time_to_chrono_time(lvLastUseTimeLong);
        }

        let lvPostponeTime : i32;
        unsafe
        {
            lvPostponeTime = *par.add(3) as i32;
        }
        
        CBCacheBeforeFlushEventArgs
        {
            FileId: lvFileId,
            FileContext: lvFileContext,
            LastUseTime: lvLastUseTime,
            PostponeTime: lvPostponeTime,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.PostponeTime as isize;
        }
    }

    pub fn file_id(&self) -> &String
    {
        &self.FileId
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn last_use_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.LastUseTime
    }
    pub fn postpone_time(&self) -> i32
    {
        self.PostponeTime
    }
    pub fn set_postpone_time(&mut self, value: i32)
    {
        self.PostponeTime = value;
    }
}

pub trait CBCacheBeforeFlushEvent
{
    fn on_before_flush(&self, sender : &CBCache, e : &mut CBCacheBeforeFlushEventArgs);
}


// CBCacheBeforePurgeEventArgs carries the parameters of the BeforePurge event of CBCache
pub struct CBCacheBeforePurgeEventArgs
{
    FileId : String,
    FileContext : usize,
    LastUseTime : chrono::DateTime<Utc>,
    PostponeTime : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of BeforePurgeEventArgs
impl CBCacheBeforePurgeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBCacheBeforePurgeEventArgs
    {

        let lvhFileIdPtr : *mut c_char;
        let lvFileId : String;
        unsafe
        {
            lvhFileIdPtr = *par.add(0) as *mut c_char;
            if lvhFileIdPtr == std::ptr::null_mut()
            {
                lvFileId = String::default();
            }
            else
            {
                lvFileId = CStr::from_ptr(lvhFileIdPtr).to_str().expect("Valid UTF8 not received for the parameter 'FileId' in the BeforePurge event of a CBCache instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvLastUseTimeLong : i64;
        let lvLastUseTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvLastUseTimeLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvLastUseTimeLong = *lvLastUseTimeLPtr;
            lvLastUseTime = file_time_to_chrono_time(lvLastUseTimeLong);
        }

        let lvPostponeTime : i32;
        unsafe
        {
            lvPostponeTime = *par.add(3) as i32;
        }
        
        CBCacheBeforePurgeEventArgs
        {
            FileId: lvFileId,
            FileContext: lvFileContext,
            LastUseTime: lvLastUseTime,
            PostponeTime: lvPostponeTime,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.PostponeTime as isize;
        }
    }

    pub fn file_id(&self) -> &String
    {
        &self.FileId
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn last_use_time(&self) -> &chrono::DateTime<Utc>
    {
        &self.LastUseTime
    }
    pub fn postpone_time(&self) -> i32
    {
        self.PostponeTime
    }
    pub fn set_postpone_time(&mut self, value: i32)
    {
        self.PostponeTime = value;
    }
}

pub trait CBCacheBeforePurgeEvent
{
    fn on_before_purge(&self, sender : &CBCache, e : &mut CBCacheBeforePurgeEventArgs);
}


// CBCacheEndCleanupEventArgs carries the parameters of the EndCleanup event of CBCache
pub struct CBCacheEndCleanupEventArgs
{

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of EndCleanupEventArgs
impl CBCacheEndCleanupEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBCacheEndCleanupEventArgs
    {

        CBCacheEndCleanupEventArgs
        {
            _Params: par
        }
    }


}

pub trait CBCacheEndCleanupEvent
{
    fn on_end_cleanup(&self, sender : &CBCache, e : &mut CBCacheEndCleanupEventArgs);
}


// CBCacheErrorEventArgs carries the parameters of the Error event of CBCache
pub struct CBCacheErrorEventArgs
{
    ErrorCode : i32,
    Description : String,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ErrorEventArgs
impl CBCacheErrorEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBCacheErrorEventArgs
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
                lvDescription = CStr::from_ptr(lvhDescriptionPtr).to_str().expect("Valid UTF8 not received for the parameter 'Description' in the Error event of a CBCache instance").to_owned();
            }
        }

        CBCacheErrorEventArgs
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

pub trait CBCacheErrorEvent
{
    fn on_error(&self, sender : &CBCache, e : &mut CBCacheErrorEventArgs);
}


// CBCacheLogEventArgs carries the parameters of the Log event of CBCache
pub struct CBCacheLogEventArgs
{
    Level : i32,
    Message : String,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of LogEventArgs
impl CBCacheLogEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBCacheLogEventArgs
    {

        let lvLevel : i32;
        unsafe
        {
            lvLevel = *par.add(0) as i32;
        }
        
        let lvhMessagePtr : *mut c_char;
        let lvMessage : String;
        unsafe
        {
            lvhMessagePtr = *par.add(1) as *mut c_char;
            if lvhMessagePtr == std::ptr::null_mut()
            {
                lvMessage = String::default();
            }
            else
            {
                lvMessage = CStr::from_ptr(lvhMessagePtr).to_str().expect("Valid UTF8 not received for the parameter 'Message' in the Log event of a CBCache instance").to_owned();
            }
        }

        CBCacheLogEventArgs
        {
            Level: lvLevel,
            Message: lvMessage,
            _Params: par
        }
    }


    pub fn level(&self) -> i32
    {
        self.Level
    }
    pub fn message(&self) -> &String
    {
        &self.Message
    }
}

pub trait CBCacheLogEvent
{
    fn on_log(&self, sender : &CBCache, e : &mut CBCacheLogEventArgs);
}


// CBCacheProgressEventArgs carries the parameters of the Progress event of CBCache
pub struct CBCacheProgressEventArgs
{
    Operation : i32,
    Current : i64,
    Total : i64,
    Interrupt : bool,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ProgressEventArgs
impl CBCacheProgressEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBCacheProgressEventArgs
    {

        let lvOperation : i32;
        unsafe
        {
            lvOperation = *par.add(0) as i32;
        }
        
        let lvCurrent : i64;
        unsafe
        {
            let lvCurrentLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvCurrent = *lvCurrentLPtr;
        }
        
        let lvTotal : i64;
        unsafe
        {
            let lvTotalLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvTotal = *lvTotalLPtr;
        }
        
        let lvInterrupt : bool;
        unsafe
        {
            lvInterrupt = (*par.add(3) as i32) != 0;
        }

        CBCacheProgressEventArgs
        {
            Operation: lvOperation,
            Current: lvCurrent,
            Total: lvTotal,
            Interrupt: lvInterrupt,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfInterrupt : i32;
            if self.Interrupt
            {
                intValOfInterrupt = 1;
            }
            else
            {
                intValOfInterrupt = 0;
            }
            *(self._Params.add(3)) = intValOfInterrupt as isize;
        }
    }

    pub fn operation(&self) -> i32
    {
        self.Operation
    }
    pub fn current(&self) -> i64
    {
        self.Current
    }
    pub fn total(&self) -> i64
    {
        self.Total
    }
    pub fn interrupt(&self) -> bool
    {
        self.Interrupt
    }
    pub fn set_interrupt(&mut self, value: bool)
    {
        self.Interrupt = value;
    }
}

pub trait CBCacheProgressEvent
{
    fn on_progress(&self, sender : &CBCache, e : &mut CBCacheProgressEventArgs);
}


// CBCacheReadDataEventArgs carries the parameters of the ReadData event of CBCache
pub struct CBCacheReadDataEventArgs
{
    FileId : String,
    Size : i64,
    Position : i64,
    Flags : i32,
    hBufferPtr : *mut u8,
    BytesToRead : i32,
    BytesRead : i32,
    FileContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ReadDataEventArgs
impl CBCacheReadDataEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBCacheReadDataEventArgs
    {

        let lvhFileIdPtr : *mut c_char;
        let lvFileId : String;
        unsafe
        {
            lvhFileIdPtr = *par.add(0) as *mut c_char;
            if lvhFileIdPtr == std::ptr::null_mut()
            {
                lvFileId = String::default();
            }
            else
            {
                lvFileId = CStr::from_ptr(lvhFileIdPtr).to_str().expect("Valid UTF8 not received for the parameter 'FileId' in the ReadData event of a CBCache instance").to_owned();
            }
        }

        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvPosition : i64;
        unsafe
        {
            let lvPositionLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvPosition = *lvPositionLPtr;
        }
        
        let lvFlags : i32;
        unsafe
        {
            lvFlags = *par.add(3) as i32;
        }
        
        let lvhBufferPtr : *mut u8;
        unsafe
        {
            lvhBufferPtr = *par.add(4) as *mut u8;
        }

        let lvBytesToRead : i32;
        unsafe
        {
            lvBytesToRead = *par.add(5) as i32;
        }
        // lvhBufferLen = lvBytesToRead;

        let lvBytesRead : i32;
        unsafe
        {
            lvBytesRead = *par.add(6) as i32;
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
        
        CBCacheReadDataEventArgs
        {
            FileId: lvFileId,
            Size: lvSize,
            Position: lvPosition,
            Flags: lvFlags,
            hBufferPtr: lvhBufferPtr,
            BytesToRead: lvBytesToRead,
            BytesRead: lvBytesRead,
            FileContext: lvFileContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(6)) = self.BytesRead as isize;
            *(self._Params.add(8)) = self.ResultCode as isize;
        }
    }

    pub fn file_id(&self) -> &String
    {
        &self.FileId
    }
    pub fn size(&self) -> i64
    {
        self.Size
    }
    pub fn position(&self) -> i64
    {
        self.Position
    }
    pub fn flags(&self) -> i32
    {
        self.Flags
    }
    pub fn buffer(&self) -> *mut u8
    {
        self.hBufferPtr
    }
    pub fn bytes_to_read(&self) -> i32
    {
        self.BytesToRead
    }
    pub fn bytes_read(&self) -> i32
    {
        self.BytesRead
    }
    pub fn set_bytes_read(&mut self, value: i32)
    {
        self.BytesRead = value;
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

pub trait CBCacheReadDataEvent
{
    fn on_read_data(&self, sender : &CBCache, e : &mut CBCacheReadDataEventArgs);
}


// CBCacheStartCleanupEventArgs carries the parameters of the StartCleanup event of CBCache
pub struct CBCacheStartCleanupEventArgs
{

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of StartCleanupEventArgs
impl CBCacheStartCleanupEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBCacheStartCleanupEventArgs
    {

        CBCacheStartCleanupEventArgs
        {
            _Params: par
        }
    }


}

pub trait CBCacheStartCleanupEvent
{
    fn on_start_cleanup(&self, sender : &CBCache, e : &mut CBCacheStartCleanupEventArgs);
}


// CBCacheStatusEventArgs carries the parameters of the Status event of CBCache
pub struct CBCacheStatusEventArgs
{
    CacheSize : i64,
    TotalData : i64,
    UnflushedData : i64,
    UnflushedFiles : i32,
    CurrentOperation : i32,
    CurrentFileId : String,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of StatusEventArgs
impl CBCacheStatusEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBCacheStatusEventArgs
    {

        let lvCacheSize : i64;
        unsafe
        {
            let lvCacheSizeLPtr : *mut i64 = *par.add(0) as *mut i64;
            lvCacheSize = *lvCacheSizeLPtr;
        }
        
        let lvTotalData : i64;
        unsafe
        {
            let lvTotalDataLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvTotalData = *lvTotalDataLPtr;
        }
        
        let lvUnflushedData : i64;
        unsafe
        {
            let lvUnflushedDataLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvUnflushedData = *lvUnflushedDataLPtr;
        }
        
        let lvUnflushedFiles : i32;
        unsafe
        {
            lvUnflushedFiles = *par.add(3) as i32;
        }
        
        let lvCurrentOperation : i32;
        unsafe
        {
            lvCurrentOperation = *par.add(4) as i32;
        }
        
        let lvhCurrentFileIdPtr : *mut c_char;
        let lvCurrentFileId : String;
        unsafe
        {
            lvhCurrentFileIdPtr = *par.add(5) as *mut c_char;
            if lvhCurrentFileIdPtr == std::ptr::null_mut()
            {
                lvCurrentFileId = String::default();
            }
            else
            {
                lvCurrentFileId = CStr::from_ptr(lvhCurrentFileIdPtr).to_str().expect("Valid UTF8 not received for the parameter 'CurrentFileId' in the Status event of a CBCache instance").to_owned();
            }
        }

        CBCacheStatusEventArgs
        {
            CacheSize: lvCacheSize,
            TotalData: lvTotalData,
            UnflushedData: lvUnflushedData,
            UnflushedFiles: lvUnflushedFiles,
            CurrentOperation: lvCurrentOperation,
            CurrentFileId: lvCurrentFileId,
            _Params: par
        }
    }


    pub fn cache_size(&self) -> i64
    {
        self.CacheSize
    }
    pub fn total_data(&self) -> i64
    {
        self.TotalData
    }
    pub fn unflushed_data(&self) -> i64
    {
        self.UnflushedData
    }
    pub fn unflushed_files(&self) -> i32
    {
        self.UnflushedFiles
    }
    pub fn current_operation(&self) -> i32
    {
        self.CurrentOperation
    }
    pub fn current_file_id(&self) -> &String
    {
        &self.CurrentFileId
    }
}

pub trait CBCacheStatusEvent
{
    fn on_status(&self, sender : &CBCache, e : &mut CBCacheStatusEventArgs);
}


// CBCacheWriteDataEventArgs carries the parameters of the WriteData event of CBCache
pub struct CBCacheWriteDataEventArgs
{
    FileId : String,
    Size : i64,
    Position : i64,
    Flags : i32,
    hBufferPtr : *mut u8,
    BytesToWrite : i32,
    BytesWritten : i32,
    FileContext : usize,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of WriteDataEventArgs
impl CBCacheWriteDataEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBCacheWriteDataEventArgs
    {

        let lvhFileIdPtr : *mut c_char;
        let lvFileId : String;
        unsafe
        {
            lvhFileIdPtr = *par.add(0) as *mut c_char;
            if lvhFileIdPtr == std::ptr::null_mut()
            {
                lvFileId = String::default();
            }
            else
            {
                lvFileId = CStr::from_ptr(lvhFileIdPtr).to_str().expect("Valid UTF8 not received for the parameter 'FileId' in the WriteData event of a CBCache instance").to_owned();
            }
        }

        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvPosition : i64;
        unsafe
        {
            let lvPositionLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvPosition = *lvPositionLPtr;
        }
        
        let lvFlags : i32;
        unsafe
        {
            lvFlags = *par.add(3) as i32;
        }
        
        let lvhBufferPtr : *mut u8;
        unsafe
        {
            lvhBufferPtr = *par.add(4) as *mut u8;
        }

        let lvBytesToWrite : i32;
        unsafe
        {
            lvBytesToWrite = *par.add(5) as i32;
        }
        // lvhBufferLen = lvBytesToWrite;

        let lvBytesWritten : i32;
        unsafe
        {
            lvBytesWritten = *par.add(6) as i32;
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
        
        CBCacheWriteDataEventArgs
        {
            FileId: lvFileId,
            Size: lvSize,
            Position: lvPosition,
            Flags: lvFlags,
            hBufferPtr: lvhBufferPtr,
            BytesToWrite: lvBytesToWrite,
            BytesWritten: lvBytesWritten,
            FileContext: lvFileContext,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(6)) = self.BytesWritten as isize;
            *(self._Params.add(8)) = self.ResultCode as isize;
        }
    }

    pub fn file_id(&self) -> &String
    {
        &self.FileId
    }
    pub fn size(&self) -> i64
    {
        self.Size
    }
    pub fn position(&self) -> i64
    {
        self.Position
    }
    pub fn flags(&self) -> i32
    {
        self.Flags
    }
    pub fn buffer(&self) -> *mut u8
    {
        self.hBufferPtr
    }
    pub fn bytes_to_write(&self) -> i32
    {
        self.BytesToWrite
    }
    pub fn bytes_written(&self) -> i32
    {
        self.BytesWritten
    }
    pub fn set_bytes_written(&mut self, value: i32)
    {
        self.BytesWritten = value;
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

pub trait CBCacheWriteDataEvent
{
    fn on_write_data(&self, sender : &CBCache, e : &mut CBCacheWriteDataEventArgs);
}


////////////////////////////
// Main Class Implementation
////////////////////////////

/* The CBCache component allows applications to easily cache remote file data locally. */
//pub struct CBCache<'a>
pub struct CBCache
{

    // onBeforeFlush : Option<&'a dyn CBCacheBeforeFlushEvent>,
    onBeforeFlush : Option<fn (sender : &CBCache, e : &mut CBCacheBeforeFlushEventArgs) >,
    // onBeforePurge : Option<&'a dyn CBCacheBeforePurgeEvent>,
    onBeforePurge : Option<fn (sender : &CBCache, e : &mut CBCacheBeforePurgeEventArgs) >,
    // onEndCleanup : Option<&'a dyn CBCacheEndCleanupEvent>,
    onEndCleanup : Option<fn (sender : &CBCache, e : &mut CBCacheEndCleanupEventArgs) >,
    // onError : Option<&'a dyn CBCacheErrorEvent>,
    onError : Option<fn (sender : &CBCache, e : &mut CBCacheErrorEventArgs) >,
    // onLog : Option<&'a dyn CBCacheLogEvent>,
    onLog : Option<fn (sender : &CBCache, e : &mut CBCacheLogEventArgs) >,
    // onProgress : Option<&'a dyn CBCacheProgressEvent>,
    onProgress : Option<fn (sender : &CBCache, e : &mut CBCacheProgressEventArgs) >,
    // onReadData : Option<&'a dyn CBCacheReadDataEvent>,
    onReadData : Option<fn (sender : &CBCache, e : &mut CBCacheReadDataEventArgs) >,
    // onStartCleanup : Option<&'a dyn CBCacheStartCleanupEvent>,
    onStartCleanup : Option<fn (sender : &CBCache, e : &mut CBCacheStartCleanupEventArgs) >,
    // onStatus : Option<&'a dyn CBCacheStatusEvent>,
    onStatus : Option<fn (sender : &CBCache, e : &mut CBCacheStatusEventArgs) >,
    // onWriteData : Option<&'a dyn CBCacheWriteDataEvent>,
    onWriteData : Option<fn (sender : &CBCache, e : &mut CBCacheWriteDataEventArgs) >,

    Id : usize,
    Handle : usize 
}

//impl<'a> Drop for CBCache<'a>
impl Drop for CBCache
{
    fn drop(&mut self)
    {
        self.dispose();
    }
}

impl CBCache
{
    pub fn new() -> &'static mut CBCache
    {        
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
            lId = CBCacheIDSeed.fetch_add(1, SeqCst) as usize;
        }

        let lHandle : isize;
        unsafe
        {
            let callable = CBFSConnect_CBCache_Create.clone().unwrap();
            lHandle = callable(CBCacheEventDispatcher, lId, std::ptr::null(), CBCacheCreateOpt) as isize;
        }
        if lHandle < 0
        {
            panic!("Failed to instantiate CBCache. Please verify that it is supported on this platform");
        }

        let result : CBCache = CBCache
        {
            onBeforeFlush: None,
            onBeforePurge: None,
            onEndCleanup: None,
            onError: None,
            onLog: None,
            onProgress: None,
            onReadData: None,
            onStartCleanup: None,
            onStatus: None,
            onWriteData: None,
            Id: lId,
            Handle: lHandle as usize
        };

        let oem_key = CString::new(cbfsconnectkey::rtkCBFSConnect).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();

        unsafe
        {
            let callable = CBFSConnect_CBCache_SetCStr.clone().unwrap();
            ret_code = callable(lHandle as usize, 8012/*PID_KEYCHECK_RUST*/, 0, oem_key_ptr, 0) as i32;
            let _ = CString::from_raw(oem_key_ptr);
        }
        if ret_code != 0
        {
            panic!("Initialization of CBCache has failed with error {}", ret_code);
        }

        // Lock the Mutex to get access to the HashMap
        unsafe
        {
            let _map = CBCacheDictMutex.lock().unwrap(); // It is used as a synchronization primitive - don't touch
            CBCacheDict.insert(lId, result);
            let res = CBCacheDict.get_mut(&lId).unwrap();
            return res;
        } // The lock is automatically released here
    }

    pub fn dispose(&self)
    {
        let mut _aself : Option<CBCache>;
        unsafe
        {
            let _map = CBCacheDictMutex.lock().unwrap(); // It is used as a synchronization primitive - don't touch

            if !CBCacheDict.contains_key(&self.Id)
            {
                return;
            }

            // Remove itself from the list
            _aself = CBCacheDict.remove(&self.Id);

            // finalize the ctlclass
            let callable = CBFSConnect_CBCache_Destroy.clone().unwrap();
            callable(self.Handle);
        }
    }

/////////
// Events
/////////

    fn fire_before_flush(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onBeforeFlush
        {
            let mut args : CBCacheBeforeFlushEventArgs = CBCacheBeforeFlushEventArgs::new(par, cbpar);
            callable/*.on_before_flush*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_before_flush(&self) -> &'a dyn CBCacheBeforeFlushEvent
    pub fn on_before_flush(&self) -> Option<fn (sender : &CBCache, e : &mut CBCacheBeforeFlushEventArgs)>
    {
        self.onBeforeFlush
    }

    //pub fn set_on_before_flush(&mut self, value : &'a dyn CBCacheBeforeFlushEvent)
    pub fn set_on_before_flush(&mut self, value : Option<fn (sender : &CBCache, e : &mut CBCacheBeforeFlushEventArgs)>)
    {
        self.onBeforeFlush = value;
    }

    fn fire_before_purge(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onBeforePurge
        {
            let mut args : CBCacheBeforePurgeEventArgs = CBCacheBeforePurgeEventArgs::new(par, cbpar);
            callable/*.on_before_purge*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_before_purge(&self) -> &'a dyn CBCacheBeforePurgeEvent
    pub fn on_before_purge(&self) -> Option<fn (sender : &CBCache, e : &mut CBCacheBeforePurgeEventArgs)>
    {
        self.onBeforePurge
    }

    //pub fn set_on_before_purge(&mut self, value : &'a dyn CBCacheBeforePurgeEvent)
    pub fn set_on_before_purge(&mut self, value : Option<fn (sender : &CBCache, e : &mut CBCacheBeforePurgeEventArgs)>)
    {
        self.onBeforePurge = value;
    }

    fn fire_end_cleanup(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onEndCleanup
        {
            let mut args : CBCacheEndCleanupEventArgs = CBCacheEndCleanupEventArgs::new(par, cbpar);
            callable/*.on_end_cleanup*/(&self, &mut args);
        }
    }

    //pub fn on_end_cleanup(&self) -> &'a dyn CBCacheEndCleanupEvent
    pub fn on_end_cleanup(&self) -> Option<fn (sender : &CBCache, e : &mut CBCacheEndCleanupEventArgs)>
    {
        self.onEndCleanup
    }

    //pub fn set_on_end_cleanup(&mut self, value : &'a dyn CBCacheEndCleanupEvent)
    pub fn set_on_end_cleanup(&mut self, value : Option<fn (sender : &CBCache, e : &mut CBCacheEndCleanupEventArgs)>)
    {
        self.onEndCleanup = value;
    }

    fn fire_error(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onError
        {
            let mut args : CBCacheErrorEventArgs = CBCacheErrorEventArgs::new(par, cbpar);
            callable/*.on_error*/(&self, &mut args);
        }
    }

    //pub fn on_error(&self) -> &'a dyn CBCacheErrorEvent
    pub fn on_error(&self) -> Option<fn (sender : &CBCache, e : &mut CBCacheErrorEventArgs)>
    {
        self.onError
    }

    //pub fn set_on_error(&mut self, value : &'a dyn CBCacheErrorEvent)
    pub fn set_on_error(&mut self, value : Option<fn (sender : &CBCache, e : &mut CBCacheErrorEventArgs)>)
    {
        self.onError = value;
    }

    fn fire_log(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onLog
        {
            let mut args : CBCacheLogEventArgs = CBCacheLogEventArgs::new(par, cbpar);
            callable/*.on_log*/(&self, &mut args);
        }
    }

    //pub fn on_log(&self) -> &'a dyn CBCacheLogEvent
    pub fn on_log(&self) -> Option<fn (sender : &CBCache, e : &mut CBCacheLogEventArgs)>
    {
        self.onLog
    }

    //pub fn set_on_log(&mut self, value : &'a dyn CBCacheLogEvent)
    pub fn set_on_log(&mut self, value : Option<fn (sender : &CBCache, e : &mut CBCacheLogEventArgs)>)
    {
        self.onLog = value;
    }

    fn fire_progress(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onProgress
        {
            let mut args : CBCacheProgressEventArgs = CBCacheProgressEventArgs::new(par, cbpar);
            callable/*.on_progress*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_progress(&self) -> &'a dyn CBCacheProgressEvent
    pub fn on_progress(&self) -> Option<fn (sender : &CBCache, e : &mut CBCacheProgressEventArgs)>
    {
        self.onProgress
    }

    //pub fn set_on_progress(&mut self, value : &'a dyn CBCacheProgressEvent)
    pub fn set_on_progress(&mut self, value : Option<fn (sender : &CBCache, e : &mut CBCacheProgressEventArgs)>)
    {
        self.onProgress = value;
    }

    fn fire_read_data(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onReadData
        {
            let mut args : CBCacheReadDataEventArgs = CBCacheReadDataEventArgs::new(par, cbpar);
            callable/*.on_read_data*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_read_data(&self) -> &'a dyn CBCacheReadDataEvent
    pub fn on_read_data(&self) -> Option<fn (sender : &CBCache, e : &mut CBCacheReadDataEventArgs)>
    {
        self.onReadData
    }

    //pub fn set_on_read_data(&mut self, value : &'a dyn CBCacheReadDataEvent)
    pub fn set_on_read_data(&mut self, value : Option<fn (sender : &CBCache, e : &mut CBCacheReadDataEventArgs)>)
    {
        self.onReadData = value;
    }

    fn fire_start_cleanup(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onStartCleanup
        {
            let mut args : CBCacheStartCleanupEventArgs = CBCacheStartCleanupEventArgs::new(par, cbpar);
            callable/*.on_start_cleanup*/(&self, &mut args);
        }
    }

    //pub fn on_start_cleanup(&self) -> &'a dyn CBCacheStartCleanupEvent
    pub fn on_start_cleanup(&self) -> Option<fn (sender : &CBCache, e : &mut CBCacheStartCleanupEventArgs)>
    {
        self.onStartCleanup
    }

    //pub fn set_on_start_cleanup(&mut self, value : &'a dyn CBCacheStartCleanupEvent)
    pub fn set_on_start_cleanup(&mut self, value : Option<fn (sender : &CBCache, e : &mut CBCacheStartCleanupEventArgs)>)
    {
        self.onStartCleanup = value;
    }

    fn fire_status(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onStatus
        {
            let mut args : CBCacheStatusEventArgs = CBCacheStatusEventArgs::new(par, cbpar);
            callable/*.on_status*/(&self, &mut args);
        }
    }

    //pub fn on_status(&self) -> &'a dyn CBCacheStatusEvent
    pub fn on_status(&self) -> Option<fn (sender : &CBCache, e : &mut CBCacheStatusEventArgs)>
    {
        self.onStatus
    }

    //pub fn set_on_status(&mut self, value : &'a dyn CBCacheStatusEvent)
    pub fn set_on_status(&mut self, value : Option<fn (sender : &CBCache, e : &mut CBCacheStatusEventArgs)>)
    {
        self.onStatus = value;
    }

    fn fire_write_data(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onWriteData
        {
            let mut args : CBCacheWriteDataEventArgs = CBCacheWriteDataEventArgs::new(par, cbpar);
            callable/*.on_write_data*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_write_data(&self) -> &'a dyn CBCacheWriteDataEvent
    pub fn on_write_data(&self) -> Option<fn (sender : &CBCache, e : &mut CBCacheWriteDataEventArgs)>
    {
        self.onWriteData
    }

    //pub fn set_on_write_data(&mut self, value : &'a dyn CBCacheWriteDataEvent)
    pub fn set_on_write_data(&mut self, value : Option<fn (sender : &CBCache, e : &mut CBCacheWriteDataEventArgs)>)
    {
        self.onWriteData = value;
    }


    pub(crate) fn report_error_info(&self, error_info : &str)
    {
        if let Some(callable) = self.onError
        {
            let mut args : CBCacheErrorEventArgs = CBCacheErrorEventArgs
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
            let callable = CBFSConnect_CBCache_GetLastError.clone().unwrap();
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
            let callable = CBFSConnect_CBCache_GetLastErrorCode.clone().unwrap();
            result = callable(self.Handle) as i32;
        }
        result
    }

    // GetRuntimeLicense returns the runtime license key set for CBCache.
    pub fn get_runtime_license(&self) -> Result<String, CBFSConnectError>
    {
        let result : String;
        //let length : c_long;
        unsafe
        {
            let callable = CBFSConnect_CBCache_GetAsCStr.clone().unwrap();

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

    // SetRuntimeLicense sets the runtime license key for CBCache.
    pub fn set_runtime_license(&self, value : String) -> Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let oem_key = CString::new(value).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetCStr.clone().unwrap();
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

    // Gets the value of the Active property: This property describes whether the file cache is open.
    pub fn active(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 1, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AutoFlush property: This property specifies whether automatic flushing is enabled.
    pub fn auto_flush(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 2, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the AutoFlush property: This property specifies whether automatic flushing is enabled.
    pub fn set_auto_flush(&self, value : bool) -> Result<(), CBFSConnectError> {
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
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 2, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the CacheDirectory property: This property includes the directory in which the cache's storage file is located.
    pub fn cache_directory(&self) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_CBCache_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 3, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the CacheDirectory property: This property includes the directory in which the cache's storage file is located.
    pub fn set_cache_directory(&self, value : &str) -> Result<(), CBFSConnectError> {
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

            let callable = CBFSConnect_CBCache_SetCStr.clone().unwrap();
            ret_code = callable(self.Handle, 3, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the CacheFile property: This property includes the name of the file cache.
    pub fn cache_file(&self) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_CBCache_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 4, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the CacheFile property: This property includes the name of the file cache.
    pub fn set_cache_file(&self, value : &str) -> Result<(), CBFSConnectError> {
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

            let callable = CBFSConnect_CBCache_SetCStr.clone().unwrap();
            ret_code = callable(self.Handle, 4, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the CacheSizeLimit property: This property includes the maximum size of the cache's storage file.
    pub fn cache_size_limit(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 5, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the CacheSizeLimit property: This property includes the maximum size of the cache's storage file.
    pub fn set_cache_size_limit(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 5, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the Compressed property: This property specifies whether or not the cache's storage file should be compressed.
    pub fn compressed(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 6, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the Compressed property: This property specifies whether or not the cache's storage file should be compressed.
    pub fn set_compressed(&self, value : bool) -> Result<(), CBFSConnectError> {
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
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 6, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the FlushAfterCloseDelay property: This property includes the number of seconds to delay flushing after a file is closed.
    pub fn flush_after_close_delay(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 7, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the FlushAfterCloseDelay property: This property includes the number of seconds to delay flushing after a file is closed.
    pub fn set_flush_after_close_delay(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 7, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the FlushAfterSize property: This property includes the amount of data that must be changed for flushing to begin.
    pub fn flush_after_size(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 8, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the FlushAfterSize property: This property includes the amount of data that must be changed for flushing to begin.
    pub fn set_flush_after_size(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 8, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the FlushAfterTime property: This property includes the inactivity timeout that must elapse for flushing to begin.
    pub fn flush_after_time(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 9, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the FlushAfterTime property: This property includes the inactivity timeout that must elapse for flushing to begin.
    pub fn set_flush_after_time(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 9, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the FlushPolicy property: This property includes the file flushing strategy that the cache should use.
    pub fn flush_policy(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 10, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the FlushPolicy property: This property includes the file flushing strategy that the cache should use.
    pub fn set_flush_policy(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 10, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the HasOrphans property: This property notes whether or not the cache contains any orphan files.
    pub fn has_orphans(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
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


    // Gets the value of the MaxCachedFileSize property: This property includes the maximum amount of data that should be cached for any given file.
    pub fn max_cached_file_size(&self) -> Result<i64, CBFSConnectError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBCache_GetAsInt64.clone().unwrap();
            callable(self.Handle, 12, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the MaxCachedFileSize property: This property includes the maximum amount of data that should be cached for any given file.
    pub fn set_max_cached_file_size(&self, value : i64) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe 
        {
            let ValuePtr = &value;
            let callable = CBFSConnect_CBCache_SetInt64.clone().unwrap();
            ret_code = callable(self.Handle, 12, 0 as c_long, ValuePtr as *const i64, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the Mode property: This property includes the mode in which the cache operates in regard to access to the external storage.
    pub fn mode(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
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


    // Sets the value of the Mode property: This property includes the mode in which the cache operates in regard to access to the external storage.
    pub fn set_mode(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 13, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the Offline property: This property specifies whether or not the cache works in Offline mode.
    pub fn offline(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
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


    // Sets the value of the Offline property: This property specifies whether or not the cache works in Offline mode.
    pub fn set_offline(&self, value : bool) -> Result<(), CBFSConnectError> {
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
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 14, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the PurgeAfterCloseDelay property: This property includes the number of seconds to delay purging for after a file is closed.
    pub fn purge_after_close_delay(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 15, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the PurgeAfterCloseDelay property: This property includes the number of seconds to delay purging for after a file is closed.
    pub fn set_purge_after_close_delay(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 15, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the PurgeAfterTime property: This property includes the inactivity timeout that must elapse for purging to begin.
    pub fn purge_after_time(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
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


    // Sets the value of the PurgeAfterTime property: This property includes the inactivity timeout that must elapse for purging to begin.
    pub fn set_purge_after_time(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 16, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the PurgePolicy property: This property includes the file block purging strategy that the cache should use.
    pub fn purge_policy(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
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


    // Sets the value of the PurgePolicy property: This property includes the file block purging strategy that the cache should use.
    pub fn set_purge_policy(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 17, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the ReadBlockSize property: This property includes the block size to use when reading file data from external storage.
    pub fn read_block_size(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 18, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the ReadBlockSize property: This property includes the block size to use when reading file data from external storage.
    pub fn set_read_block_size(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 18, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the ReadingCapabilities property: This property includes the reading capabilities supported by the external storage.
    pub fn reading_capabilities(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 19, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the ReadingCapabilities property: This property includes the reading capabilities supported by the external storage.
    pub fn set_reading_capabilities(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 19, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the ResizingCapabilities property: This property includes the file resizing capabilities supported by the external storage.
    pub fn resizing_capabilities(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
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


    // Sets the value of the ResizingCapabilities property: This property includes the file resizing capabilities supported by the external storage.
    pub fn set_resizing_capabilities(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 20, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the Serialization property: This property specifies whether nonintersecting operations against the same file or cache should execute serially or in parallel.
    pub fn serialization(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
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


    // Sets the value of the Serialization property: This property specifies whether nonintersecting operations against the same file or cache should execute serially or in parallel.
    pub fn set_serialization(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 21, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the StatsCacheSize property: This property contains the current size of the cache.
    pub fn stats_cache_size(&self) -> Result<i64, CBFSConnectError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBCache_GetAsInt64.clone().unwrap();
            callable(self.Handle, 22, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the StatsCurrentFileId property: This property contains the file currently being read or written by the cache.
    pub fn stats_current_file_id(&self) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_CBCache_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 23, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the StatsCurrentOperation property: This property contains the operation currently being performed by the cache.
    pub fn stats_current_operation(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 24, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the StatsTotalData property: This property contains the total amount of data present in the cache.
    pub fn stats_total_data(&self) -> Result<i64, CBFSConnectError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBCache_GetAsInt64.clone().unwrap();
            callable(self.Handle, 25, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the StatsUnflushedData property: This property contains the amount of unflushed data present in the cache.
    pub fn stats_unflushed_data(&self) -> Result<i64, CBFSConnectError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBCache_GetAsInt64.clone().unwrap();
            callable(self.Handle, 26, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the StatsUnflushedFiles property: This property contains the number of unflushed files present in the cache.
    pub fn stats_unflushed_files(&self) -> Result<i64, CBFSConnectError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBCache_GetAsInt64.clone().unwrap();
            callable(self.Handle, 27, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the Tag property: This property stores application-defined data specific to a particular instance of the struct.
    pub fn tag(&self) -> Result<i64, CBFSConnectError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBCache_GetAsInt64.clone().unwrap();
            callable(self.Handle, 28, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
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
            let callable = CBFSConnect_CBCache_SetInt64.clone().unwrap();
            ret_code = callable(self.Handle, 28, 0 as c_long, ValuePtr as *const i64, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the WriteBlockSize property: This property includes the block size to use when writing file data to external storage.
    pub fn write_block_size(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 29, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the WriteBlockSize property: This property includes the block size to use when writing file data to external storage.
    pub fn set_write_block_size(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 29, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the WritingCapabilities property: This property includes the writing capabilities supported by the external storage.
    pub fn writing_capabilities(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_CBCache_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 30, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the WritingCapabilities property: This property includes the writing capabilities supported by the external storage.
    pub fn set_writing_capabilities(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_CBCache_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 30, 0 as c_long, value as isize, 0) as i32;
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

    // CancelCleanup: This method cancels a background cleanup operation, if necessary.
    pub fn cancel_cleanup(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 2, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn cancel_cleanup

    // CheckAndRepair: This method checks a cache vault's consistency and repairs it as necessary.
    pub fn check_and_repair(&self, encryption_password : &str, flags : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrEncryptionPasswordPtr : *mut c_char;
        match CString::new(encryption_password)
        {
            Ok(CStrValueEncryptionPassword) => { CStrEncryptionPasswordPtr = CStrValueEncryptionPassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrEncryptionPasswordPtr as usize;
        CParams[1] = (flags as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 3, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrEncryptionPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn check_and_repair

    // Clear: This method removes files from the cache.
    pub fn clear(&self, remove_orphans : bool, remove_locals : bool) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let RemoveOrphansBool : i32;
        if remove_orphans
        {
            RemoveOrphansBool = 1;
        } else {
            RemoveOrphansBool = 0;
        }
        CParams[0] = RemoveOrphansBool as usize;

        let RemoveLocalsBool : i32;
        if remove_locals
        {
            RemoveLocalsBool = 1;
        } else {
            RemoveLocalsBool = 0;
        }
        CParams[1] = RemoveLocalsBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 4, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn clear

    // CloseCache: This method closes the cache.
    pub fn close_cache(&self, flush_action : i32, purge_action : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        CParams[0] = (flush_action as isize) as usize;
        CParams[1] = (purge_action as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 5, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn close_cache

    // CloseEnumeration: This method closes the given file enumeration.
    pub fn close_enumeration(&self, enumeration_handle : i64) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let EnumerationHandleArr : Vec<i64> = vec![enumeration_handle];
        CParams[0] = EnumerationHandleArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 6, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn close_enumeration

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
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 7, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrConfigurationStringPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn config

    // DeleteCache: This method deletes the cache completely.
    pub fn delete_cache(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 8, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn delete_cache

    // EnumerateCachedFiles: This method enumerates the files in the cache.
    pub fn enumerate_cached_files(&self, enumeration_mode : i32) ->  Result<i64, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        CParams[0] = (enumeration_mode as isize) as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 9, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn enumerate_cached_files

    // FileChangeId: This method changes the Id of a cached file.
    pub fn file_change_id(&self, file_id : &str, new_file_id : &str) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        let CStrNewFileIdPtr : *mut c_char;
        match CString::new(new_file_id)
        {
            Ok(CStrValueNewFileId) => { CStrNewFileIdPtr = CStrValueNewFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrNewFileIdPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 10, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
            let _ = CString::from_raw(CStrNewFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_change_id

    // FileClose: This method closes a cached file.
    pub fn file_close(&self, file_id : &str) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 11, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_close

    // FileCloseEx: This method closes a cached file, specifying flushing and purging behaviors explicitly.
    pub fn file_close_ex(&self, file_id : &str, flush_action : i32, purge_action : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        CParams[1] = (flush_action as isize) as usize;
        CParams[2] = (purge_action as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 12, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_close_ex

    // FileDelete: This method deletes a file from the cache.
    pub fn file_delete(&self, file_id : &str) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 13, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_delete

    // FileExists: This method checks if a file with the given Id is present in the cache.
    pub fn file_exists(&self, file_id : &str) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 14, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn file_exists

    // FileFlush: This method flushes the specified file's modified blocks out to external storage.
    pub fn file_flush(&self, file_id : &str, flush_mode : i32, delay : i32, file_context : usize) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 4 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        CParams[1] = (flush_mode as isize) as usize;
        CParams[2] = (delay as isize) as usize;
        CParams[3] = (file_context as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 15, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[4] as i32;
        return Result::Ok(ret_val);
    } // fn file_flush

    // FileGetPinned: This method returns the pinned state of a cached file.
    pub fn file_get_pinned(&self, file_id : &str) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 16, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn file_get_pinned

    // FileGetSize: This method gets the "real size" of a cached file.
    pub fn file_get_size(&self, file_id : &str) ->  Result<i64, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 17, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn file_get_size

    // FileGetStatus: This method gets the status of a cached file.
    pub fn file_get_status(&self, file_id : &str, status_kind : i32) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        CParams[1] = (status_kind as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 18, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] != 0;
        return Result::Ok(ret_val);
    } // fn file_get_status

    // FileOpen: This method opens the specified cached file, creating it if necessary.
    pub fn file_open(&self, file_id : &str, real_file_size : i64, prefetch_size : i64, file_context : usize) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 4 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        let RealFileSizeArr : Vec<i64> = vec![real_file_size];
        CParams[1] = RealFileSizeArr.as_ptr() as usize;
        let PrefetchSizeArr : Vec<i64> = vec![prefetch_size];
        CParams[2] = PrefetchSizeArr.as_ptr() as usize;
        CParams[3] = (file_context as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 19, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_open

    // FileOpenEx: This method opens the specified cached file, creating it if necessary.
    pub fn file_open_ex(&self, file_id : &str, real_file_size : i64, is_local : bool, read_block_size : i32, write_block_size : i32, prefetch_size : i64, file_context : usize) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 7 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0, 0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        let RealFileSizeArr : Vec<i64> = vec![real_file_size];
        CParams[1] = RealFileSizeArr.as_ptr() as usize;
        let IsLocalBool : i32;
        if is_local
        {
            IsLocalBool = 1;
        } else {
            IsLocalBool = 0;
        }
        CParams[2] = IsLocalBool as usize;

        CParams[3] = (read_block_size as isize) as usize;
        CParams[4] = (write_block_size as isize) as usize;
        let PrefetchSizeArr : Vec<i64> = vec![prefetch_size];
        CParams[5] = PrefetchSizeArr.as_ptr() as usize;
        CParams[6] = (file_context as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 20, 7, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_open_ex

    // FilePurge: This method purges unmodified data from the cache.
    pub fn file_purge(&self, file_id : &str) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 21, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_purge

    // FileRead: This method reads the specified part of a cached file.
    pub fn file_read(&self, file_id : &str, position : i64, buffer : &mut [u8], index : i32, count : i32) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 5 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, buffer.len() as i32, 0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        let PositionArr : Vec<i64> = vec![position];
        CParams[1] = PositionArr.as_ptr() as usize;
        CParams[2] = buffer.as_mut_ptr() as usize;
        CParams[3] = (index as isize) as usize;
        CParams[4] = (count as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 22, 5, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[5] != 0;
        return Result::Ok(ret_val);
    } // fn file_read

    // FileSetPinned: This method sets the pinned state of a cached file.
    pub fn file_set_pinned(&self, file_id : &str, pinned : bool) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        let PinnedBool : i32;
        if pinned
        {
            PinnedBool = 1;
        } else {
            PinnedBool = 0;
        }
        CParams[1] = PinnedBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 23, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_set_pinned

    // FileSetSize: This method sets the "real size" of a cached file.
    pub fn file_set_size(&self, file_id : &str, new_size : i64, fetch_missing_data : bool) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        let NewSizeArr : Vec<i64> = vec![new_size];
        CParams[1] = NewSizeArr.as_ptr() as usize;
        let FetchMissingDataBool : i32;
        if fetch_missing_data
        {
            FetchMissingDataBool = 1;
        } else {
            FetchMissingDataBool = 0;
        }
        CParams[2] = FetchMissingDataBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 24, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_set_size

    // FileSetSizeEx: This method sets the "real size" of a cached file.
    pub fn file_set_size_ex(&self, file_id : &str, new_size : i64, read_block_size : i32, write_block_size : i32, fetch_missing_data : bool) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 5 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        let NewSizeArr : Vec<i64> = vec![new_size];
        CParams[1] = NewSizeArr.as_ptr() as usize;
        CParams[2] = (read_block_size as isize) as usize;
        CParams[3] = (write_block_size as isize) as usize;
        let FetchMissingDataBool : i32;
        if fetch_missing_data
        {
            FetchMissingDataBool = 1;
        } else {
            FetchMissingDataBool = 0;
        }
        CParams[4] = FetchMissingDataBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 25, 5, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_set_size_ex

    // FileTagDelete: This method deletes a file tag.
    pub fn file_tag_delete(&self, file_id : &str, tag_id : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        CParams[1] = (tag_id as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 26, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_tag_delete

    // FileTagExists: This method checks whether or not a file has a specific tag associated with it.
    pub fn file_tag_exists(&self, file_id : &str, tag_id : i32) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        CParams[1] = (tag_id as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 27, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] != 0;
        return Result::Ok(ret_val);
    } // fn file_tag_exists

    // FileTagGet: This method retrieves the binary data held by a file tag attached to the specified cached file.
    pub fn file_tag_get(&self, file_id : &str, tag_id : i32) ->  Result<Vec<u8>, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : Vec<u8> ; // = Vec::new();
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        CParams[1] = (tag_id as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 28, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        unsafe
        {
            if CBParams[2] == 0
            {
                ret_val = vec![0 as u8; 0];
            }
            else
            {
                ret_val = std::slice::from_raw_parts(CParams[2] as *mut u8, CBParams[2] as usize).to_vec();
            }
        }
        return Result::Ok(ret_val);
    } // fn file_tag_get

    // FileTagGetSize: This method retrieves the size of a file tag attached to the specified cached file.
    pub fn file_tag_get_size(&self, file_id : &str, tag_id : i32) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        CParams[1] = (tag_id as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 29, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] as i32;
        return Result::Ok(ret_val);
    } // fn file_tag_get_size

    // FileTagSet: This method attaches a file tag with binary data to the specified cached file.
    pub fn file_tag_set(&self, file_id : &str, tag_id : i32, data : &[u8]) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, data.len() as i32, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        CParams[1] = (tag_id as isize) as usize;
        CParams[2] = data.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 30, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_tag_set

    // FileTouch: This method touches a range of data in a cached file.
    pub fn file_touch(&self, file_id : &str, position : i64, count : i32, flush : bool) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 4 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        let PositionArr : Vec<i64> = vec![position];
        CParams[1] = PositionArr.as_ptr() as usize;
        CParams[2] = (count as isize) as usize;
        let FlushBool : i32;
        if flush
        {
            FlushBool = 1;
        } else {
            FlushBool = 0;
        }
        CParams[3] = FlushBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 31, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn file_touch

    // FileWrite: This method writes the specified part of a cached file.
    pub fn file_write(&self, file_id : &str, position : i64, buffer : &[u8], index : i32, count : i32) ->  Result<bool, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 5 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, buffer.len() as i32, 0, 0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;
        let PositionArr : Vec<i64> = vec![position];
        CParams[1] = PositionArr.as_ptr() as usize;
        CParams[2] = buffer.as_ptr() as usize;
        CParams[3] = (index as isize) as usize;
        CParams[4] = (count as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 32, 5, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[5] != 0;
        return Result::Ok(ret_val);
    } // fn file_write

    // GetNextEnumeratedFile: This method returns the next file in the list of enumeration results.
    pub fn get_next_enumerated_file(&self, enumeration_handle : i64) ->  Result<String, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let EnumerationHandleArr : Vec<i64> = vec![enumeration_handle];
        CParams[0] = EnumerationHandleArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 33, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn get_next_enumerated_file

    // OpenCache: This method opens the cache.
    pub fn open_cache(&self, encryption_password : &str, case_sensitive : bool) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrEncryptionPasswordPtr : *mut c_char;
        match CString::new(encryption_password)
        {
            Ok(CStrValueEncryptionPassword) => { CStrEncryptionPasswordPtr = CStrValueEncryptionPassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrEncryptionPasswordPtr as usize;
        let CaseSensitiveBool : i32;
        if case_sensitive
        {
            CaseSensitiveBool = 1;
        } else {
            CaseSensitiveBool = 0;
        }
        CParams[1] = CaseSensitiveBool as usize;


        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 34, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrEncryptionPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn open_cache

    // RefreshStatistics: This method forces a refresh of the cache's statistics.
    pub fn refresh_statistics(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 35, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn refresh_statistics

    // ResetErrorState: This method resets any outstanding errors for a file or external storage.
    pub fn reset_error_state(&self, file_id : &str) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileIdPtr : *mut c_char;
        match CString::new(file_id)
        {
            Ok(CStrValueFileId) => { CStrFileIdPtr = CStrValueFileId.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileIdPtr as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 36, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn reset_error_state

    // StartCleanup: This method starts a background cleanup operation to remove unused files from the cache.
    pub fn start_cleanup(&self, max_age : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        CParams[0] = (max_age as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_CBCache_Do.clone().unwrap();
            ret_code = callable(self.Handle, 37, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn start_cleanup

} // CBCache

extern "system" fn CBCacheEventDispatcher(pObj : usize, event_id : c_long, _cparam : c_long, param : IntPtrArrayType, cbparam : IntArrayType) -> c_long
{
    let obj: &'static CBCache;
    // Lock the Mutex to get access to the HashMap
    unsafe
    {
        let _map = CBCacheDictMutex.lock().unwrap();
        let objOpt = CBCacheDict.get(&pObj);
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
            1 /* BeforeFlush */=> obj.fire_before_flush(/*cparam as i32, */param, cbparam),

            2 /* BeforePurge */=> obj.fire_before_purge(/*cparam as i32, */param, cbparam),

            3 /* EndCleanup */=> obj.fire_end_cleanup(/*cparam as i32, */param, cbparam),

            4 /* Error */=> obj.fire_error(/*cparam as i32, */param, cbparam),

            5 /* Log */=> obj.fire_log(/*cparam as i32, */param, cbparam),

            6 /* Progress */=> obj.fire_progress(/*cparam as i32, */param, cbparam),

            7 /* ReadData */=> obj.fire_read_data(/*cparam as i32, */param, cbparam),

            8 /* StartCleanup */=> obj.fire_start_cleanup(/*cparam as i32, */param, cbparam),

            9 /* Status */=> obj.fire_status(/*cparam as i32, */param, cbparam),

            10 /* WriteData */=> obj.fire_write_data(/*cparam as i32, */param, cbparam),

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


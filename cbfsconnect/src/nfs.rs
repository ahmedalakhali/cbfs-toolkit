


#![allow(non_snake_case)]

extern crate libloading as lib;

use std::{collections::HashMap, ffi::{c_char, c_long, c_longlong, c_ulong, c_void, CStr, CString}, panic::catch_unwind, sync::{atomic::{AtomicUsize, Ordering::SeqCst}, Mutex} };
use lib::{Library, Symbol, Error};
use std::fmt::Write;
use chrono::Utc;
use once_cell::sync::Lazy;

use crate::{*, cbfsconnectkey};

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_NFS_StaticInit)(void *hInst);
type CBFSConnectNFSStaticInitType = unsafe extern "system" fn(hInst : *mut c_void) -> i32;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_NFS_StaticDestroy)();
type CBFSConnectNFSStaticDestroyType = unsafe extern "system" fn()-> i32;

// typedef void* (CBFSCONNECT_CALL *lpCBFSConnect_NFS_Create)(PCBFSCONNECT_CALLBACK lpSink, void *lpContext, char *lpOemKey, int opts);
type CBFSConnectNFSCreateType = unsafe extern "system" fn(lpSink : CBFSConnectSinkDelegateType, lpContext : usize, lpOemKey : *const c_char, opts : i32) -> usize;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_NFS_Destroy)(void *lpObj);
type CBFSConnectNFSDestroyType = unsafe extern "system" fn(lpObj: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_NFS_CheckIndex)(void *lpObj, int propid, int arridx);
type CBFSConnectNFSCheckIndexType = unsafe extern "system" fn(lpObj: usize, propid: c_long, arridx: c_long)-> c_long;

// typedef void* (CBFSCONNECT_CALL *lpCBFSConnect_NFS_Get)(void *lpObj, int propid, int arridx, int *lpcbVal, int64 *lpllVal);
type CBFSConnectNFSGetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *mut c_longlong) -> *mut c_void;
type CBFSConnectNFSGetAsCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *const c_longlong) -> *const c_char;
type CBFSConnectNFSGetAsIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *const c_void) -> usize;
type CBFSConnectNFSGetAsInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *mut i64) -> usize;
type CBFSConnectNFSGetAsBSTRType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut i32, llVal: *const c_void) -> *const u8;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_NFS_Set)(void *lpObj, int propid, int arridx, const void *val, int cbVal);
type CBFSConnectNFSSetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_void, len: c_ulong)-> c_long;
type CBFSConnectNFSSetCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_char, len: c_ulong)-> c_long;
type CBFSConnectNFSSetIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: isize, len: c_ulong)-> c_long;
type CBFSConnectNFSSetInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const i64, len: c_ulong)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_NFS_Do)(void *lpObj, int methid, int cparam, void *param[], int cbparam[], int64 *lpllVal);
type CBFSConnectNFSDoType = unsafe extern "system" fn(p: usize, method_id: c_long, cparam: c_long, params: UIntPtrArrayType, cbparam: IntArrayType, llVal: *mut c_longlong)-> c_long;

// typedef char* (CBFSCONNECT_CALL *lpCBFSConnect_NFS_GetLastError)(void *lpObj);
type CBFSConnectNFSGetLastErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_NFS_GetLastErrorCode)(void *lpObj);
type CBFSConnectNFSGetLastErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_NFS_SetLastErrorAndCode)(void *lpObj, int code, char *message);
type CBFSConnectNFSSetLastErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

// typedef char* (CBFSCONNECT_CALL *lpCBFSConnect_NFS_GetEventError)(void *lpObj);
type CBFSConnectNFSGetEventErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_NFS_GetEventErrorCode)(void *lpObj);
type CBFSConnectNFSGetEventErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSCONNECT_CALL *lpCBFSConnect_NFS_SetEventErrorAndCode)(void *lpObj, int code, char *message);
type CBFSConnectNFSSetEventErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

static mut CBFSConnect_NFS_StaticInit : Option<Symbol<CBFSConnectNFSStaticInitType>> = None;
static mut CBFSConnect_NFS_StaticDestroy : Option<Symbol<CBFSConnectNFSStaticDestroyType>> = None;

static mut CBFSConnect_NFS_Create: Option<Symbol<CBFSConnectNFSCreateType>> = None;
static mut CBFSConnect_NFS_Destroy: Option<Symbol<CBFSConnectNFSDestroyType>> = None;
static mut CBFSConnect_NFS_Set: Option<Symbol<CBFSConnectNFSSetType>> = None;
static mut CBFSConnect_NFS_SetCStr: Option<Symbol<CBFSConnectNFSSetCStrType>> = None;
static mut CBFSConnect_NFS_SetInt: Option<Symbol<CBFSConnectNFSSetIntType>> = None;
static mut CBFSConnect_NFS_SetInt64: Option<Symbol<CBFSConnectNFSSetInt64Type>> = None;
static mut CBFSConnect_NFS_Get: Option<Symbol<CBFSConnectNFSGetType>> = None;
static mut CBFSConnect_NFS_GetAsCStr: Option<Symbol<CBFSConnectNFSGetAsCStrType>> = None;
static mut CBFSConnect_NFS_GetAsInt: Option<Symbol<CBFSConnectNFSGetAsIntType>> = None;
static mut CBFSConnect_NFS_GetAsInt64: Option<Symbol<CBFSConnectNFSGetAsInt64Type>> = None;
static mut CBFSConnect_NFS_GetAsBSTR: Option<Symbol<CBFSConnectNFSGetAsBSTRType>> = None;
static mut CBFSConnect_NFS_GetLastError: Option<Symbol<CBFSConnectNFSGetLastErrorType>> = None;
static mut CBFSConnect_NFS_GetLastErrorCode: Option<Symbol<CBFSConnectNFSGetLastErrorCodeType>> = None;
static mut CBFSConnect_NFS_SetLastErrorAndCode: Option<Symbol<CBFSConnectNFSSetLastErrorAndCodeType>> = None;
static mut CBFSConnect_NFS_GetEventError: Option<Symbol<CBFSConnectNFSGetEventErrorType>> = None;
static mut CBFSConnect_NFS_GetEventErrorCode: Option<Symbol<CBFSConnectNFSGetEventErrorCodeType>> = None;
static mut CBFSConnect_NFS_SetEventErrorAndCode: Option<Symbol<CBFSConnectNFSSetEventErrorAndCodeType>> = None;
static mut CBFSConnect_NFS_CheckIndex: Option<Symbol<CBFSConnectNFSCheckIndexType>> = None;
static mut CBFSConnect_NFS_Do: Option<Symbol<CBFSConnectNFSDoType>> = None;

static mut NFSIDSeed : AtomicUsize = AtomicUsize::new(1);

static mut NFSDict : Lazy<HashMap<usize, NFS>> = Lazy::new(|| HashMap::new() );
static NFSDictMutex : Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0) );

const NFSCreateOpt : i32 = 0;


pub type NFSStream = crate::CBFSConnectStream;


pub(crate) fn get_lib_funcs( lib_hand : &'static Library) -> bool
{
    #[cfg(target_os = "android")]
    return true;
    #[cfg(target_os = "ios")]
    return true;

    unsafe
    {
        // CBFSConnect_NFS_StaticInit
        let func_ptr_res : Result<Symbol<CBFSConnectNFSStaticInitType>, Error> = lib_hand.get(b"CBFSConnect_NFS_StaticInit");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_NFS_StaticInit = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_StaticDestroy
        let func_ptr_res : Result<Symbol<CBFSConnectNFSStaticDestroyType>, Error> = lib_hand.get(b"CBFSConnect_NFS_StaticDestroy");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_NFS_StaticDestroy = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_Create
        let func_ptr_res : Result<Symbol<CBFSConnectNFSCreateType>, Error> = lib_hand.get(b"CBFSConnect_NFS_Create");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_NFS_Create = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_Destroy
        let func_ptr_res : Result<Symbol<CBFSConnectNFSDestroyType>, Error> = lib_hand.get(b"CBFSConnect_NFS_Destroy");
        if func_ptr_res.is_ok()
        {
            CBFSConnect_NFS_Destroy = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_Get
        let func_ptr_res : Result<Symbol<CBFSConnectNFSGetType>, Error> = lib_hand.get(b"CBFSConnect_NFS_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_Get = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_NFS_GetAsCStr
        let func_ptr_res : Result<Symbol<CBFSConnectNFSGetAsCStrType>, Error> = lib_hand.get(b"CBFSConnect_NFS_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_GetAsCStr = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_NFS_GetAsInt
        let func_ptr_res : Result<Symbol<CBFSConnectNFSGetAsIntType>, Error> = lib_hand.get(b"CBFSConnect_NFS_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_GetAsInt = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_NFS_GetAsInt64
        let func_ptr_res : Result<Symbol<CBFSConnectNFSGetAsInt64Type>, Error> = lib_hand.get(b"CBFSConnect_NFS_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_GetAsInt64 = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_NFS_GetAsBSTR
        let func_ptr_res : Result<Symbol<CBFSConnectNFSGetAsBSTRType>, Error> = lib_hand.get(b"CBFSConnect_NFS_Get");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_GetAsBSTR = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_Set
        let func_ptr_res : Result<Symbol<CBFSConnectNFSSetType>, Error> = lib_hand.get(b"CBFSConnect_NFS_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_Set = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_NFS_SetCStr
        let func_ptr_res : Result<Symbol<CBFSConnectNFSSetCStrType>, Error> = lib_hand.get(b"CBFSConnect_NFS_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_SetCStr = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_NFS_SetInt
        let func_ptr_res : Result<Symbol<CBFSConnectNFSSetIntType>, Error> = lib_hand.get(b"CBFSConnect_NFS_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_SetInt = func_ptr_res.ok();
        }
        else
        {
            return false;
        }
        // CBFSConnect_NFS_SetInt64
        let func_ptr_res : Result<Symbol<CBFSConnectNFSSetInt64Type>, Error> = lib_hand.get(b"CBFSConnect_NFS_Set");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_SetInt64 = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_GetLastError
        let func_ptr_res : Result<Symbol<CBFSConnectNFSGetLastErrorType>, Error> = lib_hand.get(b"CBFSConnect_NFS_GetLastError");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_GetLastError = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_GetLastErrorCode
        let func_ptr_res : Result<Symbol<CBFSConnectNFSGetLastErrorCodeType>, Error> = lib_hand.get(b"CBFSConnect_NFS_GetLastErrorCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_GetLastErrorCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_SetLastErrorAndCode
        let func_ptr_res : Result<Symbol<CBFSConnectNFSSetLastErrorAndCodeType>, Error> = lib_hand.get(b"CBFSConnect_NFS_SetLastErrorAndCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_SetLastErrorAndCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_GetEventError
        let func_ptr_res : Result<Symbol<CBFSConnectNFSGetEventErrorType>, Error> = lib_hand.get(b"CBFSConnect_NFS_GetEventError");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_GetEventError = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_GetEventErrorCode
        let func_ptr_res : Result<Symbol<CBFSConnectNFSGetEventErrorCodeType>, Error> = lib_hand.get(b"CBFSConnect_NFS_GetEventErrorCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_GetEventErrorCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_SetEventErrorAndCode
        let func_ptr_res : Result<Symbol<CBFSConnectNFSSetEventErrorAndCodeType>, Error> = lib_hand.get(b"CBFSConnect_NFS_SetEventErrorAndCode");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_SetEventErrorAndCode = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_CheckIndex
        let func_ptr_res : Result<Symbol<CBFSConnectNFSCheckIndexType>, Error> = lib_hand.get(b"CBFSConnect_NFS_CheckIndex");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_CheckIndex = func_ptr_res.ok();
        }
        else
        {
            return false;
        }

        // CBFSConnect_NFS_Do
        let func_ptr_res : Result<Symbol<CBFSConnectNFSDoType>, Error> = lib_hand.get(b"CBFSConnect_NFS_Do");
        if func_ptr_res.is_ok()
        {
           CBFSConnect_NFS_Do = func_ptr_res.ok();
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


// NFSAccessEventArgs carries the parameters of the Access event of NFS
pub struct NFSAccessEventArgs
{
    ConnectionId : i32,
    Path : String,
    Access : i32,
    Supported : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of AccessEventArgs
impl NFSAccessEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSAccessEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Access event of a NFS instance").to_owned();
            }
        }

        let lvAccess : i32;
        unsafe
        {
            lvAccess = *par.add(2) as i32;
        }
        
        let lvSupported : i32;
        unsafe
        {
            lvSupported = *par.add(3) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(4) as i32;
        }
        
        NFSAccessEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            Access: lvAccess,
            Supported: lvSupported,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.Access as isize;
            *(self._Params.add(3)) = self.Supported as isize;
            *(self._Params.add(4)) = self.Result as isize;
        }
    }

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn access(&self) -> i32
    {
        self.Access
    }
    pub fn set_access(&mut self, value: i32)
    {
        self.Access = value;
    }
    pub fn supported(&self) -> i32
    {
        self.Supported
    }
    pub fn set_supported(&mut self, value: i32)
    {
        self.Supported = value;
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

pub trait NFSAccessEvent
{
    fn on_access(&self, sender : &NFS, e : &mut NFSAccessEventArgs);
}


// NFSChmodEventArgs carries the parameters of the Chmod event of NFS
pub struct NFSChmodEventArgs
{
    ConnectionId : i32,
    Path : String,
    FileContext : usize,
    Mode : i32,
    OwnerId : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ChmodEventArgs
impl NFSChmodEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSChmodEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Chmod event of a NFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvMode : i32;
        unsafe
        {
            lvMode = *par.add(3) as i32;
        }
        
        let lvOwnerId : i32;
        unsafe
        {
            lvOwnerId = *par.add(4) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(5) as i32;
        }
        
        NFSChmodEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            FileContext: lvFileContext,
            Mode: lvMode,
            OwnerId: lvOwnerId,
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

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
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
    pub fn owner_id(&self) -> i32
    {
        self.OwnerId
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

pub trait NFSChmodEvent
{
    fn on_chmod(&self, sender : &NFS, e : &mut NFSChmodEventArgs);
}


// NFSChownEventArgs carries the parameters of the Chown event of NFS
pub struct NFSChownEventArgs
{
    ConnectionId : i32,
    Path : String,
    FileContext : usize,
    User : String,
    Group : String,
    OwnerId : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ChownEventArgs
impl NFSChownEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSChownEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Chown event of a NFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvhUserPtr : *mut c_char;
        let lvUser : String;
        unsafe
        {
            lvhUserPtr = *par.add(3) as *mut c_char;
            if lvhUserPtr == std::ptr::null_mut()
            {
                lvUser = String::default();
            }
            else
            {
                lvUser = CStr::from_ptr(lvhUserPtr).to_str().expect("Valid UTF8 not received for the parameter 'User' in the Chown event of a NFS instance").to_owned();
            }
        }

        let lvhGroupPtr : *mut c_char;
        let lvGroup : String;
        unsafe
        {
            lvhGroupPtr = *par.add(4) as *mut c_char;
            if lvhGroupPtr == std::ptr::null_mut()
            {
                lvGroup = String::default();
            }
            else
            {
                lvGroup = CStr::from_ptr(lvhGroupPtr).to_str().expect("Valid UTF8 not received for the parameter 'Group' in the Chown event of a NFS instance").to_owned();
            }
        }

        let lvOwnerId : i32;
        unsafe
        {
            lvOwnerId = *par.add(5) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(6) as i32;
        }
        
        NFSChownEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            FileContext: lvFileContext,
            User: lvUser,
            Group: lvGroup,
            OwnerId: lvOwnerId,
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

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn user(&self) -> &String
    {
        &self.User
    }
    pub fn group(&self) -> &String
    {
        &self.Group
    }
    pub fn owner_id(&self) -> i32
    {
        self.OwnerId
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

pub trait NFSChownEvent
{
    fn on_chown(&self, sender : &NFS, e : &mut NFSChownEventArgs);
}


// NFSCloseEventArgs carries the parameters of the Close event of NFS
pub struct NFSCloseEventArgs
{
    ConnectionId : i32,
    Path : String,
    OwnerId : i32,
    FileContext : usize,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CloseEventArgs
impl NFSCloseEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSCloseEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Close event of a NFS instance").to_owned();
            }
        }

        let lvOwnerId : i32;
        unsafe
        {
            lvOwnerId = *par.add(2) as i32;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(3) as usize;
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(4) as i32;
        }
        
        NFSCloseEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            OwnerId: lvOwnerId,
            FileContext: lvFileContext,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.FileContext as isize;
            *(self._Params.add(4)) = self.Result as isize;
        }
    }

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn owner_id(&self) -> i32
    {
        self.OwnerId
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

pub trait NFSCloseEvent
{
    fn on_close(&self, sender : &NFS, e : &mut NFSCloseEventArgs);
}


// NFSCommitEventArgs carries the parameters of the Commit event of NFS
pub struct NFSCommitEventArgs
{
    ConnectionId : i32,
    Path : String,
    FileContext : usize,
    Offset : i64,
    Count : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of CommitEventArgs
impl NFSCommitEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSCommitEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Commit event of a NFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvOffset : i64;
        unsafe
        {
            let lvOffsetLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvOffset = *lvOffsetLPtr;
        }
        
        let lvCount : i32;
        unsafe
        {
            lvCount = *par.add(4) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(5) as i32;
        }
        
        NFSCommitEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            FileContext: lvFileContext,
            Offset: lvOffset,
            Count: lvCount,
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

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn offset(&self) -> i64
    {
        self.Offset
    }
    pub fn count(&self) -> i32
    {
        self.Count
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

pub trait NFSCommitEvent
{
    fn on_commit(&self, sender : &NFS, e : &mut NFSCommitEventArgs);
}


// NFSConnectedEventArgs carries the parameters of the Connected event of NFS
pub struct NFSConnectedEventArgs
{
    ConnectionId : i32,
    StatusCode : i32,
    Description : String,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ConnectedEventArgs
impl NFSConnectedEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSConnectedEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvStatusCode : i32;
        unsafe
        {
            lvStatusCode = *par.add(1) as i32;
        }
        
        let lvhDescriptionPtr : *mut c_char;
        let lvDescription : String;
        unsafe
        {
            lvhDescriptionPtr = *par.add(2) as *mut c_char;
            if lvhDescriptionPtr == std::ptr::null_mut()
            {
                lvDescription = String::default();
            }
            else
            {
                lvDescription = CStr::from_ptr(lvhDescriptionPtr).to_str().expect("Valid UTF8 not received for the parameter 'Description' in the Connected event of a NFS instance").to_owned();
            }
        }

        NFSConnectedEventArgs
        {
            ConnectionId: lvConnectionId,
            StatusCode: lvStatusCode,
            Description: lvDescription,
            _Params: par
        }
    }


    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn status_code(&self) -> i32
    {
        self.StatusCode
    }
    pub fn description(&self) -> &String
    {
        &self.Description
    }
}

pub trait NFSConnectedEvent
{
    fn on_connected(&self, sender : &NFS, e : &mut NFSConnectedEventArgs);
}


// NFSConnectionRequestEventArgs carries the parameters of the ConnectionRequest event of NFS
pub struct NFSConnectionRequestEventArgs
{
    Address : String,
    Port : i32,
    Accept : bool,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ConnectionRequestEventArgs
impl NFSConnectionRequestEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSConnectionRequestEventArgs
    {

        let lvhAddressPtr : *mut c_char;
        let lvAddress : String;
        unsafe
        {
            lvhAddressPtr = *par.add(0) as *mut c_char;
            if lvhAddressPtr == std::ptr::null_mut()
            {
                lvAddress = String::default();
            }
            else
            {
                lvAddress = CStr::from_ptr(lvhAddressPtr).to_str().expect("Valid UTF8 not received for the parameter 'Address' in the ConnectionRequest event of a NFS instance").to_owned();
            }
        }

        let lvPort : i32;
        unsafe
        {
            lvPort = *par.add(1) as i32;
        }
        
        let lvAccept : bool;
        unsafe
        {
            lvAccept = (*par.add(2) as i32) != 0;
        }

        NFSConnectionRequestEventArgs
        {
            Address: lvAddress,
            Port: lvPort,
            Accept: lvAccept,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfAccept : i32;
            if self.Accept
            {
                intValOfAccept = 1;
            }
            else
            {
                intValOfAccept = 0;
            }
            *(self._Params.add(2)) = intValOfAccept as isize;
        }
    }

    pub fn address(&self) -> &String
    {
        &self.Address
    }
    pub fn port(&self) -> i32
    {
        self.Port
    }
    pub fn accept(&self) -> bool
    {
        self.Accept
    }
    pub fn set_accept(&mut self, value: bool)
    {
        self.Accept = value;
    }
}

pub trait NFSConnectionRequestEvent
{
    fn on_connection_request(&self, sender : &NFS, e : &mut NFSConnectionRequestEventArgs);
}


// NFSCreateLinkEventArgs carries the parameters of the CreateLink event of NFS
pub struct NFSCreateLinkEventArgs
{
    ConnectionId : i32,
    Path : String,
    Name : String,
    LinkTarget : Vec<u8>,
    LinkType : i32,
    FileContext : usize,
    Result : i32,

    _Params  : IntPtrArrayType,
    _Cbparam : IntArrayType
}

// Constructor and marshalRefParams() of CreateLinkEventArgs
impl NFSCreateLinkEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSCreateLinkEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the CreateLink event of a NFS instance").to_owned();
            }
        }

        let lvhNamePtr : *mut c_char;
        let lvName : String;
        unsafe
        {
            lvhNamePtr = *par.add(2) as *mut c_char;
            if lvhNamePtr == std::ptr::null_mut()
            {
                lvName = String::default();
            }
            else
            {
                lvName = CStr::from_ptr(lvhNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'Name' in the CreateLink event of a NFS instance").to_owned();
            }
        }

        let lvhLinkTargetPtr : *mut u8;
        let lvhLinkTargetLen : i32;
        let lvLinkTarget : Vec<u8>;
        unsafe
        {
            lvhLinkTargetPtr = *par.add(3) as *mut u8;
            lvhLinkTargetLen = *_cbpar.add(3);
            if lvhLinkTargetLen == 0
            {
                lvLinkTarget = vec![0 as u8; 0];
            }
            else
            {
                lvLinkTarget = std::slice::from_raw_parts(lvhLinkTargetPtr, lvhLinkTargetLen as usize).to_vec();
            }
        }

        let lvLinkType : i32;
        unsafe
        {
            lvLinkType = *par.add(4) as i32;
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
        
        NFSCreateLinkEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            Name: lvName,
            LinkTarget: lvLinkTarget,
            LinkType: lvLinkType,
            FileContext: lvFileContext,
            Result: lvResult,
            _Params: par,
            _Cbparam: _cbpar
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(5)) = self.FileContext as isize;
            *(self._Params.add(6)) = self.Result as isize;
        }
    }

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn name(&self) -> &String
    {
        &self.Name
    }
    pub fn link_target(&self) -> &[u8]
    {
        &self.LinkTarget
    }
    pub fn link_type(&self) -> i32
    {
        self.LinkType
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

pub trait NFSCreateLinkEvent
{
    fn on_create_link(&self, sender : &NFS, e : &mut NFSCreateLinkEventArgs);
}


// NFSDisconnectedEventArgs carries the parameters of the Disconnected event of NFS
pub struct NFSDisconnectedEventArgs
{
    ConnectionId : i32,
    StatusCode : i32,
    Description : String,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of DisconnectedEventArgs
impl NFSDisconnectedEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSDisconnectedEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvStatusCode : i32;
        unsafe
        {
            lvStatusCode = *par.add(1) as i32;
        }
        
        let lvhDescriptionPtr : *mut c_char;
        let lvDescription : String;
        unsafe
        {
            lvhDescriptionPtr = *par.add(2) as *mut c_char;
            if lvhDescriptionPtr == std::ptr::null_mut()
            {
                lvDescription = String::default();
            }
            else
            {
                lvDescription = CStr::from_ptr(lvhDescriptionPtr).to_str().expect("Valid UTF8 not received for the parameter 'Description' in the Disconnected event of a NFS instance").to_owned();
            }
        }

        NFSDisconnectedEventArgs
        {
            ConnectionId: lvConnectionId,
            StatusCode: lvStatusCode,
            Description: lvDescription,
            _Params: par
        }
    }


    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn status_code(&self) -> i32
    {
        self.StatusCode
    }
    pub fn description(&self) -> &String
    {
        &self.Description
    }
}

pub trait NFSDisconnectedEvent
{
    fn on_disconnected(&self, sender : &NFS, e : &mut NFSDisconnectedEventArgs);
}


// NFSErrorEventArgs carries the parameters of the Error event of NFS
pub struct NFSErrorEventArgs
{
    ConnectionId : i32,
    ErrorCode : i32,
    Description : String,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ErrorEventArgs
impl NFSErrorEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSErrorEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvErrorCode : i32;
        unsafe
        {
            lvErrorCode = *par.add(1) as i32;
        }
        
        let lvhDescriptionPtr : *mut c_char;
        let lvDescription : String;
        unsafe
        {
            lvhDescriptionPtr = *par.add(2) as *mut c_char;
            if lvhDescriptionPtr == std::ptr::null_mut()
            {
                lvDescription = String::default();
            }
            else
            {
                lvDescription = CStr::from_ptr(lvhDescriptionPtr).to_str().expect("Valid UTF8 not received for the parameter 'Description' in the Error event of a NFS instance").to_owned();
            }
        }

        NFSErrorEventArgs
        {
            ConnectionId: lvConnectionId,
            ErrorCode: lvErrorCode,
            Description: lvDescription,
            _Params: par
        }
    }


    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
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

pub trait NFSErrorEvent
{
    fn on_error(&self, sender : &NFS, e : &mut NFSErrorEventArgs);
}


// NFSGetAttrEventArgs carries the parameters of the GetAttr event of NFS
pub struct NFSGetAttrEventArgs
{
    ConnectionId : i32,
    Path : String,
    FileContext : usize,
    FileId : i64,
    Mode : i32,
    hUserPtr : *mut c_char,
    User : String,
    hGroupPtr : *mut c_char,
    Group : String,
    LinkCount : i32,
    Size : i64,
    ATime : chrono::DateTime<Utc>,
    MTime : chrono::DateTime<Utc>,
    CTime : chrono::DateTime<Utc>,
    hNFSHandlePtr : *mut c_char,
    NFSHandle : String,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of GetAttrEventArgs
impl NFSGetAttrEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSGetAttrEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the GetAttr event of a NFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvFileId : i64;
        unsafe
        {
            let lvFileIdLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvFileId = *lvFileIdLPtr;
        }
        
        let lvMode : i32;
        unsafe
        {
            lvMode = *par.add(4) as i32;
        }
        
        let lvhUserPtr : *mut c_char;
        let lvUser : String;
        unsafe
        {
            lvhUserPtr = *par.add(5) as *mut c_char;
            let callable = CBFSConnect_EvtStr.clone().unwrap();
            let lvUserEvtStr : *mut c_void = callable(*par.add(5) as usize, 0, std::ptr::null_mut(), 1);
            if lvUserEvtStr == std::ptr::null_mut()
            {
                lvUser = String::default();
            }
            else
            {
                lvUser = CStr::from_ptr(lvUserEvtStr as *const c_char).to_str().expect("Valid UTF8 not received for the parameter 'User' in the GetAttr event of a NFS instance").to_owned();
            }
        }

        let lvhGroupPtr : *mut c_char;
        let lvGroup : String;
        unsafe
        {
            lvhGroupPtr = *par.add(6) as *mut c_char;
            let callable = CBFSConnect_EvtStr.clone().unwrap();
            let lvGroupEvtStr : *mut c_void = callable(*par.add(6) as usize, 0, std::ptr::null_mut(), 1);
            if lvGroupEvtStr == std::ptr::null_mut()
            {
                lvGroup = String::default();
            }
            else
            {
                lvGroup = CStr::from_ptr(lvGroupEvtStr as *const c_char).to_str().expect("Valid UTF8 not received for the parameter 'Group' in the GetAttr event of a NFS instance").to_owned();
            }
        }

        let lvLinkCount : i32;
        unsafe
        {
            lvLinkCount = *par.add(7) as i32;
        }
        
        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(8) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvATimeLong : i64;
        let lvATime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvATimeLPtr : *mut i64 = *par.add(9) as *mut i64;
            lvATimeLong = *lvATimeLPtr;
            lvATime = file_time_to_chrono_time(lvATimeLong);
        }

        let lvMTimeLong : i64;
        let lvMTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvMTimeLPtr : *mut i64 = *par.add(10) as *mut i64;
            lvMTimeLong = *lvMTimeLPtr;
            lvMTime = file_time_to_chrono_time(lvMTimeLong);
        }

        let lvCTimeLong : i64;
        let lvCTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvCTimeLPtr : *mut i64 = *par.add(11) as *mut i64;
            lvCTimeLong = *lvCTimeLPtr;
            lvCTime = file_time_to_chrono_time(lvCTimeLong);
        }

        let lvhNFSHandlePtr : *mut c_char;
        let lvNFSHandle : String;
        unsafe
        {
            lvhNFSHandlePtr = *par.add(12) as *mut c_char;
            let callable = CBFSConnect_EvtStr.clone().unwrap();
            let lvNFSHandleEvtStr : *mut c_void = callable(*par.add(12) as usize, 0, std::ptr::null_mut(), 1);
            if lvNFSHandleEvtStr == std::ptr::null_mut()
            {
                lvNFSHandle = String::default();
            }
            else
            {
                lvNFSHandle = CStr::from_ptr(lvNFSHandleEvtStr as *const c_char).to_str().expect("Valid UTF8 not received for the parameter 'NFSHandle' in the GetAttr event of a NFS instance").to_owned();
            }
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(13) as i32;
        }
        
        NFSGetAttrEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            FileContext: lvFileContext,
            FileId: lvFileId,
            Mode: lvMode,
            hUserPtr: lvhUserPtr,
            User: lvUser,
            hGroupPtr: lvhGroupPtr,
            Group: lvGroup,
            LinkCount: lvLinkCount,
            Size: lvSize,
            ATime: lvATime,
            MTime: lvMTime,
            CTime: lvCTime,
            hNFSHandlePtr: lvhNFSHandlePtr,
            NFSHandle: lvNFSHandle,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvFileIdLPtr : *mut i64 = *self._Params.add(3) as *mut i64;
            *lvFileIdLPtr = self.FileId ;
            *(self._Params.add(4)) = self.Mode as isize;
            let CStrOfUser : CString = CString::from_vec_unchecked(self.User.clone().into_bytes());
            let callable = CBFSConnect_EvtStrSet.clone().unwrap();
            callable(self.hUserPtr, 2, CStrOfUser.as_ptr(), 1);
            let CStrOfGroup : CString = CString::from_vec_unchecked(self.Group.clone().into_bytes());
            let callable = CBFSConnect_EvtStrSet.clone().unwrap();
            callable(self.hGroupPtr, 2, CStrOfGroup.as_ptr(), 1);
            *(self._Params.add(7)) = self.LinkCount as isize;
            let lvSizeLPtr : *mut i64 = *self._Params.add(8) as *mut i64;
            *lvSizeLPtr = self.Size ;
            let intValOfATime : i64 = chrono_time_to_file_time(&self.ATime);
            let lvATimeLPtr : *mut i64 = *self._Params.add(9) as *mut i64;
            *lvATimeLPtr = intValOfATime as i64;
            let intValOfMTime : i64 = chrono_time_to_file_time(&self.MTime);
            let lvMTimeLPtr : *mut i64 = *self._Params.add(10) as *mut i64;
            *lvMTimeLPtr = intValOfMTime as i64;
            let intValOfCTime : i64 = chrono_time_to_file_time(&self.CTime);
            let lvCTimeLPtr : *mut i64 = *self._Params.add(11) as *mut i64;
            *lvCTimeLPtr = intValOfCTime as i64;
            let CStrOfNFSHandle : CString = CString::from_vec_unchecked(self.NFSHandle.clone().into_bytes());
            let callable = CBFSConnect_EvtStrSet.clone().unwrap();
            callable(self.hNFSHandlePtr, 2, CStrOfNFSHandle.as_ptr(), 1);
            *(self._Params.add(13)) = self.Result as isize;
        }
    }

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn file_id(&self) -> i64
    {
        self.FileId
    }
    pub fn set_file_id(&mut self, value: i64)
    {
        self.FileId = value;
    }
    pub fn mode(&self) -> i32
    {
        self.Mode
    }
    pub fn set_mode(&mut self, value: i32)
    {
        self.Mode = value;
    }
    pub fn user(&self) -> &String
    {
        &self.User
    }
    pub fn set_user_ref(&mut self, value: &String)
    {
        self.User = value.clone();
    }
    pub fn set_user(&mut self, value: &str)
    {
        self.User = String::from(value);
    }
    pub fn group(&self) -> &String
    {
        &self.Group
    }
    pub fn set_group_ref(&mut self, value: &String)
    {
        self.Group = value.clone();
    }
    pub fn set_group(&mut self, value: &str)
    {
        self.Group = String::from(value);
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
    pub fn nfs_handle(&self) -> &String
    {
        &self.NFSHandle
    }
    pub fn set_nfs_handle_ref(&mut self, value: &String)
    {
        self.NFSHandle = value.clone();
    }
    pub fn set_nfs_handle(&mut self, value: &str)
    {
        self.NFSHandle = String::from(value);
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

pub trait NFSGetAttrEvent
{
    fn on_get_attr(&self, sender : &NFS, e : &mut NFSGetAttrEventArgs);
}


// NFSLockEventArgs carries the parameters of the Lock event of NFS
pub struct NFSLockEventArgs
{
    ConnectionId : i32,
    Path : String,
    FileContext : usize,
    LockType : i32,
    LockOffset : i64,
    LockLen : i64,
    Test : bool,
    OwnerId : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of LockEventArgs
impl NFSLockEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSLockEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Lock event of a NFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvLockType : i32;
        unsafe
        {
            lvLockType = *par.add(3) as i32;
        }
        
        let lvLockOffset : i64;
        unsafe
        {
            let lvLockOffsetLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvLockOffset = *lvLockOffsetLPtr;
        }
        
        let lvLockLen : i64;
        unsafe
        {
            let lvLockLenLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvLockLen = *lvLockLenLPtr;
        }
        
        let lvTest : bool;
        unsafe
        {
            lvTest = (*par.add(6) as i32) != 0;
        }

        let lvOwnerId : i32;
        unsafe
        {
            lvOwnerId = *par.add(7) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(8) as i32;
        }
        
        NFSLockEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            FileContext: lvFileContext,
            LockType: lvLockType,
            LockOffset: lvLockOffset,
            LockLen: lvLockLen,
            Test: lvTest,
            OwnerId: lvOwnerId,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvLockOffsetLPtr : *mut i64 = *self._Params.add(4) as *mut i64;
            *lvLockOffsetLPtr = self.LockOffset ;
            let lvLockLenLPtr : *mut i64 = *self._Params.add(5) as *mut i64;
            *lvLockLenLPtr = self.LockLen ;
            *(self._Params.add(8)) = self.Result as isize;
        }
    }

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
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
    pub fn lock_offset(&self) -> i64
    {
        self.LockOffset
    }
    pub fn set_lock_offset(&mut self, value: i64)
    {
        self.LockOffset = value;
    }
    pub fn lock_len(&self) -> i64
    {
        self.LockLen
    }
    pub fn set_lock_len(&mut self, value: i64)
    {
        self.LockLen = value;
    }
    pub fn test(&self) -> bool
    {
        self.Test
    }
    pub fn owner_id(&self) -> i32
    {
        self.OwnerId
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

pub trait NFSLockEvent
{
    fn on_lock(&self, sender : &NFS, e : &mut NFSLockEventArgs);
}


// NFSLogEventArgs carries the parameters of the Log event of NFS
pub struct NFSLogEventArgs
{
    ConnectionId : i32,
    LogLevel : i32,
    Message : String,
    LogType : String,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of LogEventArgs
impl NFSLogEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSLogEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvLogLevel : i32;
        unsafe
        {
            lvLogLevel = *par.add(1) as i32;
        }
        
        let lvhMessagePtr : *mut c_char;
        let lvMessage : String;
        unsafe
        {
            lvhMessagePtr = *par.add(2) as *mut c_char;
            if lvhMessagePtr == std::ptr::null_mut()
            {
                lvMessage = String::default();
            }
            else
            {
                lvMessage = CStr::from_ptr(lvhMessagePtr).to_str().expect("Valid UTF8 not received for the parameter 'Message' in the Log event of a NFS instance").to_owned();
            }
        }

        let lvhLogTypePtr : *mut c_char;
        let lvLogType : String;
        unsafe
        {
            lvhLogTypePtr = *par.add(3) as *mut c_char;
            if lvhLogTypePtr == std::ptr::null_mut()
            {
                lvLogType = String::default();
            }
            else
            {
                lvLogType = CStr::from_ptr(lvhLogTypePtr).to_str().expect("Valid UTF8 not received for the parameter 'LogType' in the Log event of a NFS instance").to_owned();
            }
        }

        NFSLogEventArgs
        {
            ConnectionId: lvConnectionId,
            LogLevel: lvLogLevel,
            Message: lvMessage,
            LogType: lvLogType,
            _Params: par
        }
    }


    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn log_level(&self) -> i32
    {
        self.LogLevel
    }
    pub fn message(&self) -> &String
    {
        &self.Message
    }
    pub fn log_type(&self) -> &String
    {
        &self.LogType
    }
}

pub trait NFSLogEvent
{
    fn on_log(&self, sender : &NFS, e : &mut NFSLogEventArgs);
}


// NFSLookupEventArgs carries the parameters of the Lookup event of NFS
pub struct NFSLookupEventArgs
{
    ConnectionId : i32,
    Name : String,
    Path : String,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of LookupEventArgs
impl NFSLookupEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSLookupEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhNamePtr : *mut c_char;
        let lvName : String;
        unsafe
        {
            lvhNamePtr = *par.add(1) as *mut c_char;
            if lvhNamePtr == std::ptr::null_mut()
            {
                lvName = String::default();
            }
            else
            {
                lvName = CStr::from_ptr(lvhNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'Name' in the Lookup event of a NFS instance").to_owned();
            }
        }

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(2) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Lookup event of a NFS instance").to_owned();
            }
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(3) as i32;
        }
        
        NFSLookupEventArgs
        {
            ConnectionId: lvConnectionId,
            Name: lvName,
            Path: lvPath,
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

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn name(&self) -> &String
    {
        &self.Name
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

pub trait NFSLookupEvent
{
    fn on_lookup(&self, sender : &NFS, e : &mut NFSLookupEventArgs);
}


// NFSMkDirEventArgs carries the parameters of the MkDir event of NFS
pub struct NFSMkDirEventArgs
{
    ConnectionId : i32,
    Path : String,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of MkDirEventArgs
impl NFSMkDirEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSMkDirEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the MkDir event of a NFS instance").to_owned();
            }
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(2) as i32;
        }
        
        NFSMkDirEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
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

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
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

pub trait NFSMkDirEvent
{
    fn on_mk_dir(&self, sender : &NFS, e : &mut NFSMkDirEventArgs);
}


// NFSOpenEventArgs carries the parameters of the Open event of NFS
pub struct NFSOpenEventArgs
{
    ConnectionId : i32,
    Path : String,
    ShareAccess : i32,
    ShareDeny : i32,
    CreateMode : i32,
    OpenType : i32,
    Downgrade : bool,
    OwnerId : i32,
    Size : i64,
    ATime : chrono::DateTime<Utc>,
    MTime : chrono::DateTime<Utc>,
    hUserPtr : *mut c_char,
    User : String,
    hGroupPtr : *mut c_char,
    Group : String,
    Mode : i32,
    FileContext : usize,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of OpenEventArgs
impl NFSOpenEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSOpenEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Open event of a NFS instance").to_owned();
            }
        }

        let lvShareAccess : i32;
        unsafe
        {
            lvShareAccess = *par.add(2) as i32;
        }
        
        let lvShareDeny : i32;
        unsafe
        {
            lvShareDeny = *par.add(3) as i32;
        }
        
        let lvCreateMode : i32;
        unsafe
        {
            lvCreateMode = *par.add(4) as i32;
        }
        
        let lvOpenType : i32;
        unsafe
        {
            lvOpenType = *par.add(5) as i32;
        }
        
        let lvDowngrade : bool;
        unsafe
        {
            lvDowngrade = (*par.add(6) as i32) != 0;
        }

        let lvOwnerId : i32;
        unsafe
        {
            lvOwnerId = *par.add(7) as i32;
        }
        
        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(8) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvATimeLong : i64;
        let lvATime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvATimeLPtr : *mut i64 = *par.add(9) as *mut i64;
            lvATimeLong = *lvATimeLPtr;
            lvATime = file_time_to_chrono_time(lvATimeLong);
        }

        let lvMTimeLong : i64;
        let lvMTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvMTimeLPtr : *mut i64 = *par.add(10) as *mut i64;
            lvMTimeLong = *lvMTimeLPtr;
            lvMTime = file_time_to_chrono_time(lvMTimeLong);
        }

        let lvhUserPtr : *mut c_char;
        let lvUser : String;
        unsafe
        {
            lvhUserPtr = *par.add(11) as *mut c_char;
            let callable = CBFSConnect_EvtStr.clone().unwrap();
            let lvUserEvtStr : *mut c_void = callable(*par.add(11) as usize, 0, std::ptr::null_mut(), 1);
            if lvUserEvtStr == std::ptr::null_mut()
            {
                lvUser = String::default();
            }
            else
            {
                lvUser = CStr::from_ptr(lvUserEvtStr as *const c_char).to_str().expect("Valid UTF8 not received for the parameter 'User' in the Open event of a NFS instance").to_owned();
            }
        }

        let lvhGroupPtr : *mut c_char;
        let lvGroup : String;
        unsafe
        {
            lvhGroupPtr = *par.add(12) as *mut c_char;
            let callable = CBFSConnect_EvtStr.clone().unwrap();
            let lvGroupEvtStr : *mut c_void = callable(*par.add(12) as usize, 0, std::ptr::null_mut(), 1);
            if lvGroupEvtStr == std::ptr::null_mut()
            {
                lvGroup = String::default();
            }
            else
            {
                lvGroup = CStr::from_ptr(lvGroupEvtStr as *const c_char).to_str().expect("Valid UTF8 not received for the parameter 'Group' in the Open event of a NFS instance").to_owned();
            }
        }

        let lvMode : i32;
        unsafe
        {
            lvMode = *par.add(13) as i32;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(14) as usize;
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(15) as i32;
        }
        
        NFSOpenEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            ShareAccess: lvShareAccess,
            ShareDeny: lvShareDeny,
            CreateMode: lvCreateMode,
            OpenType: lvOpenType,
            Downgrade: lvDowngrade,
            OwnerId: lvOwnerId,
            Size: lvSize,
            ATime: lvATime,
            MTime: lvMTime,
            hUserPtr: lvhUserPtr,
            User: lvUser,
            hGroupPtr: lvhGroupPtr,
            Group: lvGroup,
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
            let lvSizeLPtr : *mut i64 = *self._Params.add(8) as *mut i64;
            *lvSizeLPtr = self.Size ;
            let intValOfATime : i64 = chrono_time_to_file_time(&self.ATime);
            let lvATimeLPtr : *mut i64 = *self._Params.add(9) as *mut i64;
            *lvATimeLPtr = intValOfATime as i64;
            let intValOfMTime : i64 = chrono_time_to_file_time(&self.MTime);
            let lvMTimeLPtr : *mut i64 = *self._Params.add(10) as *mut i64;
            *lvMTimeLPtr = intValOfMTime as i64;
            let CStrOfUser : CString = CString::from_vec_unchecked(self.User.clone().into_bytes());
            let callable = CBFSConnect_EvtStrSet.clone().unwrap();
            callable(self.hUserPtr, 2, CStrOfUser.as_ptr(), 1);
            let CStrOfGroup : CString = CString::from_vec_unchecked(self.Group.clone().into_bytes());
            let callable = CBFSConnect_EvtStrSet.clone().unwrap();
            callable(self.hGroupPtr, 2, CStrOfGroup.as_ptr(), 1);
            *(self._Params.add(13)) = self.Mode as isize;
            *(self._Params.add(14)) = self.FileContext as isize;
            *(self._Params.add(15)) = self.Result as isize;
        }
    }

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn share_access(&self) -> i32
    {
        self.ShareAccess
    }
    pub fn share_deny(&self) -> i32
    {
        self.ShareDeny
    }
    pub fn create_mode(&self) -> i32
    {
        self.CreateMode
    }
    pub fn open_type(&self) -> i32
    {
        self.OpenType
    }
    pub fn downgrade(&self) -> bool
    {
        self.Downgrade
    }
    pub fn owner_id(&self) -> i32
    {
        self.OwnerId
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
    pub fn user(&self) -> &String
    {
        &self.User
    }
    pub fn set_user_ref(&mut self, value: &String)
    {
        self.User = value.clone();
    }
    pub fn set_user(&mut self, value: &str)
    {
        self.User = String::from(value);
    }
    pub fn group(&self) -> &String
    {
        &self.Group
    }
    pub fn set_group_ref(&mut self, value: &String)
    {
        self.Group = value.clone();
    }
    pub fn set_group(&mut self, value: &str)
    {
        self.Group = String::from(value);
    }
    pub fn mode(&self) -> i32
    {
        self.Mode
    }
    pub fn set_mode(&mut self, value: i32)
    {
        self.Mode = value;
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

pub trait NFSOpenEvent
{
    fn on_open(&self, sender : &NFS, e : &mut NFSOpenEventArgs);
}


// NFSReadEventArgs carries the parameters of the Read event of NFS
pub struct NFSReadEventArgs
{
    ConnectionId : i32,
    Path : String,
    FileContext : usize,
    hBufferPtr : *mut u8,
    hBufferLen : i32,
    Buffer : Vec<u8>,
    Count : i32,
    Offset : i64,
    Eof : bool,
    OwnerId : i32,
    Result : i32,

    _Params  : IntPtrArrayType,
    _Cbparam : IntArrayType
}

// Constructor and marshalRefParams() of ReadEventArgs
impl NFSReadEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSReadEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Read event of a NFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvhBufferPtr : *mut u8;
        let lvhBufferLen : i32;
        let lvBuffer : Vec<u8>;
        unsafe
        {
            lvhBufferPtr = *par.add(3) as *mut u8;
            lvhBufferLen = *_cbpar.add(3);
            if lvhBufferLen == 0
            {
                lvBuffer = vec![0 as u8; 0];
            }
            else
            {
                lvBuffer = std::slice::from_raw_parts(lvhBufferPtr, lvhBufferLen as usize).to_vec();
            }
        }

        let lvCount : i32;
        unsafe
        {
            lvCount = *par.add(4) as i32;
        }
        
        let lvOffset : i64;
        unsafe
        {
            let lvOffsetLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvOffset = *lvOffsetLPtr;
        }
        
        let lvEof : bool;
        unsafe
        {
            lvEof = (*par.add(6) as i32) != 0;
        }

        let lvOwnerId : i32;
        unsafe
        {
            lvOwnerId = *par.add(7) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(8) as i32;
        }
        
        NFSReadEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            FileContext: lvFileContext,
            hBufferPtr: lvhBufferPtr,
            hBufferLen: lvhBufferLen,
            Buffer: lvBuffer,
            Count: lvCount,
            Offset: lvOffset,
            Eof: lvEof,
            OwnerId: lvOwnerId,
            Result: lvResult,
            _Params: par,
            _Cbparam: _cbpar
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let bytesBuffer = &self.Buffer;
            let to_copy : usize;
            let bytesBufferLen = bytesBuffer.len();
            if bytesBufferLen < self.hBufferLen as usize
            {
                to_copy = bytesBufferLen;
            }
            else
            {
                to_copy = self.hBufferLen as usize;
            }
            if to_copy > 0
            {
                std::ptr::copy_nonoverlapping(bytesBuffer.as_ptr(), self.hBufferPtr as *mut u8, to_copy);
            }
            //*self._Cbparam.add(3) = to_copy as i32;
            *(self._Params.add(4)) = self.Count as isize;
            let intValOfEof : i32;
            if self.Eof
            {
                intValOfEof = 1;
            }
            else
            {
                intValOfEof = 0;
            }
            *(self._Params.add(6)) = intValOfEof as isize;
            *(self._Params.add(8)) = self.Result as isize;
        }
    }

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn buffer(&mut self) -> *mut u8
    {
        self.Buffer.as_mut_ptr()
    }
    pub fn count(&self) -> i32
    {
        self.Count
    }
    pub fn set_count(&mut self, value: i32)
    {
        self.Count = value;
    }
    pub fn offset(&self) -> i64
    {
        self.Offset
    }
    pub fn eof(&self) -> bool
    {
        self.Eof
    }
    pub fn set_eof(&mut self, value: bool)
    {
        self.Eof = value;
    }
    pub fn owner_id(&self) -> i32
    {
        self.OwnerId
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

pub trait NFSReadEvent
{
    fn on_read(&self, sender : &NFS, e : &mut NFSReadEventArgs);
}


// NFSReadDirEventArgs carries the parameters of the ReadDir event of NFS
pub struct NFSReadDirEventArgs
{
    ConnectionId : i32,
    FileContext : usize,
    Path : String,
    Cookie : i64,
    CookieVerf : i64,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ReadDirEventArgs
impl NFSReadDirEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSReadDirEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(1) as usize;
        }

        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(2) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the ReadDir event of a NFS instance").to_owned();
            }
        }

        let lvCookie : i64;
        unsafe
        {
            let lvCookieLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvCookie = *lvCookieLPtr;
        }
        
        let lvCookieVerf : i64;
        unsafe
        {
            let lvCookieVerfLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvCookieVerf = *lvCookieVerfLPtr;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(5) as i32;
        }
        
        NFSReadDirEventArgs
        {
            ConnectionId: lvConnectionId,
            FileContext: lvFileContext,
            Path: lvPath,
            Cookie: lvCookie,
            CookieVerf: lvCookieVerf,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(1)) = self.FileContext as isize;
            let lvCookieVerfLPtr : *mut i64 = *self._Params.add(4) as *mut i64;
            *lvCookieVerfLPtr = self.CookieVerf ;
            *(self._Params.add(5)) = self.Result as isize;
        }
    }

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn set_file_context(&mut self, value: usize)
    {
        self.FileContext = value;
    }
    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn cookie(&self) -> i64
    {
        self.Cookie
    }
    pub fn cookie_verf(&self) -> i64
    {
        self.CookieVerf
    }
    pub fn set_cookie_verf(&mut self, value: i64)
    {
        self.CookieVerf = value;
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

pub trait NFSReadDirEvent
{
    fn on_read_dir(&self, sender : &NFS, e : &mut NFSReadDirEventArgs);
}


// NFSReadLinkEventArgs carries the parameters of the ReadLink event of NFS
pub struct NFSReadLinkEventArgs
{
    ConnectionId : i32,
    Path : String,
    FileContext : usize,
    hBufferPtr : *mut u8,
    hBufferLen : i32,
    Buffer : Vec<u8>,
    Count : i32,
    Offset : i64,
    Eof : bool,
    Result : i32,

    _Params  : IntPtrArrayType,
    _Cbparam : IntArrayType
}

// Constructor and marshalRefParams() of ReadLinkEventArgs
impl NFSReadLinkEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSReadLinkEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the ReadLink event of a NFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvhBufferPtr : *mut u8;
        let lvhBufferLen : i32;
        let lvBuffer : Vec<u8>;
        unsafe
        {
            lvhBufferPtr = *par.add(3) as *mut u8;
            lvhBufferLen = *_cbpar.add(3);
            if lvhBufferLen == 0
            {
                lvBuffer = vec![0 as u8; 0];
            }
            else
            {
                lvBuffer = std::slice::from_raw_parts(lvhBufferPtr, lvhBufferLen as usize).to_vec();
            }
        }

        let lvCount : i32;
        unsafe
        {
            lvCount = *par.add(4) as i32;
        }
        
        let lvOffset : i64;
        unsafe
        {
            let lvOffsetLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvOffset = *lvOffsetLPtr;
        }
        
        let lvEof : bool;
        unsafe
        {
            lvEof = (*par.add(6) as i32) != 0;
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(7) as i32;
        }
        
        NFSReadLinkEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            FileContext: lvFileContext,
            hBufferPtr: lvhBufferPtr,
            hBufferLen: lvhBufferLen,
            Buffer: lvBuffer,
            Count: lvCount,
            Offset: lvOffset,
            Eof: lvEof,
            Result: lvResult,
            _Params: par,
            _Cbparam: _cbpar
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let bytesBuffer = &self.Buffer;
            let to_copy : usize;
            let bytesBufferLen = bytesBuffer.len();
            if bytesBufferLen < self.hBufferLen as usize
            {
                to_copy = bytesBufferLen;
            }
            else
            {
                to_copy = self.hBufferLen as usize;
            }
            if to_copy > 0
            {
                std::ptr::copy_nonoverlapping(bytesBuffer.as_ptr(), self.hBufferPtr as *mut u8, to_copy);
            }
            //*self._Cbparam.add(3) = to_copy as i32;
            *(self._Params.add(4)) = self.Count as isize;
            let intValOfEof : i32;
            if self.Eof
            {
                intValOfEof = 1;
            }
            else
            {
                intValOfEof = 0;
            }
            *(self._Params.add(6)) = intValOfEof as isize;
            *(self._Params.add(7)) = self.Result as isize;
        }
    }

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn buffer(&mut self) -> *mut u8
    {
        self.Buffer.as_mut_ptr()
    }
    pub fn count(&self) -> i32
    {
        self.Count
    }
    pub fn set_count(&mut self, value: i32)
    {
        self.Count = value;
    }
    pub fn offset(&self) -> i64
    {
        self.Offset
    }
    pub fn eof(&self) -> bool
    {
        self.Eof
    }
    pub fn set_eof(&mut self, value: bool)
    {
        self.Eof = value;
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

pub trait NFSReadLinkEvent
{
    fn on_read_link(&self, sender : &NFS, e : &mut NFSReadLinkEventArgs);
}


// NFSRenameEventArgs carries the parameters of the Rename event of NFS
pub struct NFSRenameEventArgs
{
    ConnectionId : i32,
    OldPath : String,
    NewPath : String,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of RenameEventArgs
impl NFSRenameEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSRenameEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhOldPathPtr : *mut c_char;
        let lvOldPath : String;
        unsafe
        {
            lvhOldPathPtr = *par.add(1) as *mut c_char;
            if lvhOldPathPtr == std::ptr::null_mut()
            {
                lvOldPath = String::default();
            }
            else
            {
                lvOldPath = CStr::from_ptr(lvhOldPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'OldPath' in the Rename event of a NFS instance").to_owned();
            }
        }

        let lvhNewPathPtr : *mut c_char;
        let lvNewPath : String;
        unsafe
        {
            lvhNewPathPtr = *par.add(2) as *mut c_char;
            if lvhNewPathPtr == std::ptr::null_mut()
            {
                lvNewPath = String::default();
            }
            else
            {
                lvNewPath = CStr::from_ptr(lvhNewPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'NewPath' in the Rename event of a NFS instance").to_owned();
            }
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(3) as i32;
        }
        
        NFSRenameEventArgs
        {
            ConnectionId: lvConnectionId,
            OldPath: lvOldPath,
            NewPath: lvNewPath,
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

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn old_path(&self) -> &String
    {
        &self.OldPath
    }
    pub fn new_path(&self) -> &String
    {
        &self.NewPath
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

pub trait NFSRenameEvent
{
    fn on_rename(&self, sender : &NFS, e : &mut NFSRenameEventArgs);
}


// NFSRmDirEventArgs carries the parameters of the RmDir event of NFS
pub struct NFSRmDirEventArgs
{
    ConnectionId : i32,
    Path : String,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of RmDirEventArgs
impl NFSRmDirEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSRmDirEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the RmDir event of a NFS instance").to_owned();
            }
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(2) as i32;
        }
        
        NFSRmDirEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
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

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
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

pub trait NFSRmDirEvent
{
    fn on_rm_dir(&self, sender : &NFS, e : &mut NFSRmDirEventArgs);
}


// NFSTruncateEventArgs carries the parameters of the Truncate event of NFS
pub struct NFSTruncateEventArgs
{
    ConnectionId : i32,
    Path : String,
    FileContext : usize,
    Size : i64,
    OwnerId : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of TruncateEventArgs
impl NFSTruncateEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSTruncateEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Truncate event of a NFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvOwnerId : i32;
        unsafe
        {
            lvOwnerId = *par.add(4) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(5) as i32;
        }
        
        NFSTruncateEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            FileContext: lvFileContext,
            Size: lvSize,
            OwnerId: lvOwnerId,
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

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
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
    pub fn owner_id(&self) -> i32
    {
        self.OwnerId
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

pub trait NFSTruncateEvent
{
    fn on_truncate(&self, sender : &NFS, e : &mut NFSTruncateEventArgs);
}


// NFSUnlinkEventArgs carries the parameters of the Unlink event of NFS
pub struct NFSUnlinkEventArgs
{
    ConnectionId : i32,
    Path : String,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of UnlinkEventArgs
impl NFSUnlinkEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSUnlinkEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Unlink event of a NFS instance").to_owned();
            }
        }

        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(2) as i32;
        }
        
        NFSUnlinkEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
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

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
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

pub trait NFSUnlinkEvent
{
    fn on_unlink(&self, sender : &NFS, e : &mut NFSUnlinkEventArgs);
}


// NFSUnlockEventArgs carries the parameters of the Unlock event of NFS
pub struct NFSUnlockEventArgs
{
    ConnectionId : i32,
    Path : String,
    FileContext : usize,
    LockType : i32,
    LockOffset : i64,
    LockLen : i64,
    OwnerId : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of UnlockEventArgs
impl NFSUnlockEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSUnlockEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Unlock event of a NFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvLockType : i32;
        unsafe
        {
            lvLockType = *par.add(3) as i32;
        }
        
        let lvLockOffset : i64;
        unsafe
        {
            let lvLockOffsetLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvLockOffset = *lvLockOffsetLPtr;
        }
        
        let lvLockLen : i64;
        unsafe
        {
            let lvLockLenLPtr : *mut i64 = *par.add(5) as *mut i64;
            lvLockLen = *lvLockLenLPtr;
        }
        
        let lvOwnerId : i32;
        unsafe
        {
            lvOwnerId = *par.add(6) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(7) as i32;
        }
        
        NFSUnlockEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            FileContext: lvFileContext,
            LockType: lvLockType,
            LockOffset: lvLockOffset,
            LockLen: lvLockLen,
            OwnerId: lvOwnerId,
            Result: lvResult,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(7)) = self.Result as isize;
        }
    }

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
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
    pub fn lock_offset(&self) -> i64
    {
        self.LockOffset
    }
    pub fn lock_len(&self) -> i64
    {
        self.LockLen
    }
    pub fn owner_id(&self) -> i32
    {
        self.OwnerId
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

pub trait NFSUnlockEvent
{
    fn on_unlock(&self, sender : &NFS, e : &mut NFSUnlockEventArgs);
}


// NFSUTimeEventArgs carries the parameters of the UTime event of NFS
pub struct NFSUTimeEventArgs
{
    ConnectionId : i32,
    Path : String,
    FileContext : usize,
    ATime : chrono::DateTime<Utc>,
    MTime : chrono::DateTime<Utc>,
    OwnerId : i32,
    Result : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of UTimeEventArgs
impl NFSUTimeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSUTimeEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the UTime event of a NFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvATimeLong : i64;
        let lvATime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvATimeLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvATimeLong = *lvATimeLPtr;
            lvATime = file_time_to_chrono_time(lvATimeLong);
        }

        let lvMTimeLong : i64;
        let lvMTime : chrono::DateTime<Utc>;
        unsafe
        {
            let lvMTimeLPtr : *mut i64 = *par.add(4) as *mut i64;
            lvMTimeLong = *lvMTimeLPtr;
            lvMTime = file_time_to_chrono_time(lvMTimeLong);
        }

        let lvOwnerId : i32;
        unsafe
        {
            lvOwnerId = *par.add(5) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(6) as i32;
        }
        
        NFSUTimeEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            FileContext: lvFileContext,
            ATime: lvATime,
            MTime: lvMTime,
            OwnerId: lvOwnerId,
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

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
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
    pub fn owner_id(&self) -> i32
    {
        self.OwnerId
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

pub trait NFSUTimeEvent
{
    fn on_u_time(&self, sender : &NFS, e : &mut NFSUTimeEventArgs);
}


// NFSWriteEventArgs carries the parameters of the Write event of NFS
pub struct NFSWriteEventArgs
{
    ConnectionId : i32,
    Path : String,
    FileContext : usize,
    Offset : i64,
    Buffer : Vec<u8>,
    Count : i32,
    Stable : i32,
    OwnerId : i32,
    Result : i32,

    _Params  : IntPtrArrayType,
    _Cbparam : IntArrayType
}

// Constructor and marshalRefParams() of WriteEventArgs
impl NFSWriteEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> NFSWriteEventArgs
    {

        let lvConnectionId : i32;
        unsafe
        {
            lvConnectionId = *par.add(0) as i32;
        }
        
        let lvhPathPtr : *mut c_char;
        let lvPath : String;
        unsafe
        {
            lvhPathPtr = *par.add(1) as *mut c_char;
            if lvhPathPtr == std::ptr::null_mut()
            {
                lvPath = String::default();
            }
            else
            {
                lvPath = CStr::from_ptr(lvhPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'Path' in the Write event of a NFS instance").to_owned();
            }
        }

        let lvFileContext : usize;
        unsafe
        {
            lvFileContext = *par.add(2) as usize;
        }

        let lvOffset : i64;
        unsafe
        {
            let lvOffsetLPtr : *mut i64 = *par.add(3) as *mut i64;
            lvOffset = *lvOffsetLPtr;
        }
        
        let lvhBufferPtr : *mut u8;
        let lvhBufferLen : i32;
        let lvBuffer : Vec<u8>;
        unsafe
        {
            lvhBufferPtr = *par.add(4) as *mut u8;
            lvhBufferLen = *_cbpar.add(4);
            if lvhBufferLen == 0
            {
                lvBuffer = vec![0 as u8; 0];
            }
            else
            {
                lvBuffer = std::slice::from_raw_parts(lvhBufferPtr, lvhBufferLen as usize).to_vec();
            }
        }

        let lvCount : i32;
        unsafe
        {
            lvCount = *par.add(5) as i32;
        }
        
        let lvStable : i32;
        unsafe
        {
            lvStable = *par.add(6) as i32;
        }
        
        let lvOwnerId : i32;
        unsafe
        {
            lvOwnerId = *par.add(7) as i32;
        }
        
        let lvResult : i32;
        unsafe
        {
            lvResult = *par.add(8) as i32;
        }
        
        NFSWriteEventArgs
        {
            ConnectionId: lvConnectionId,
            Path: lvPath,
            FileContext: lvFileContext,
            Offset: lvOffset,
            Buffer: lvBuffer,
            Count: lvCount,
            Stable: lvStable,
            OwnerId: lvOwnerId,
            Result: lvResult,
            _Params: par,
            _Cbparam: _cbpar
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(5)) = self.Count as isize;
            *(self._Params.add(6)) = self.Stable as isize;
            *(self._Params.add(8)) = self.Result as isize;
        }
    }

    pub fn connection_id(&self) -> i32
    {
        self.ConnectionId
    }
    pub fn path(&self) -> &String
    {
        &self.Path
    }
    pub fn file_context(&self) -> usize
    {
        self.FileContext
    }
    pub fn offset(&self) -> i64
    {
        self.Offset
    }
    pub fn buffer(&self) -> &[u8]
    {
        &self.Buffer
    }
    pub fn count(&self) -> i32
    {
        self.Count
    }
    pub fn set_count(&mut self, value: i32)
    {
        self.Count = value;
    }
    pub fn stable(&self) -> i32
    {
        self.Stable
    }
    pub fn set_stable(&mut self, value: i32)
    {
        self.Stable = value;
    }
    pub fn owner_id(&self) -> i32
    {
        self.OwnerId
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

pub trait NFSWriteEvent
{
    fn on_write(&self, sender : &NFS, e : &mut NFSWriteEventArgs);
}


////////////////////////////
// Main Class Implementation
////////////////////////////

/* This component is used to create a Network File System (NFS) 4.0 server and mount it to a directory in Linux and macOS. */
//pub struct NFS<'a>
pub struct NFS
{

    // onAccess : Option<&'a dyn NFSAccessEvent>,
    onAccess : Option<fn (sender : &NFS, e : &mut NFSAccessEventArgs) >,
    // onChmod : Option<&'a dyn NFSChmodEvent>,
    onChmod : Option<fn (sender : &NFS, e : &mut NFSChmodEventArgs) >,
    // onChown : Option<&'a dyn NFSChownEvent>,
    onChown : Option<fn (sender : &NFS, e : &mut NFSChownEventArgs) >,
    // onClose : Option<&'a dyn NFSCloseEvent>,
    onClose : Option<fn (sender : &NFS, e : &mut NFSCloseEventArgs) >,
    // onCommit : Option<&'a dyn NFSCommitEvent>,
    onCommit : Option<fn (sender : &NFS, e : &mut NFSCommitEventArgs) >,
    // onConnected : Option<&'a dyn NFSConnectedEvent>,
    onConnected : Option<fn (sender : &NFS, e : &mut NFSConnectedEventArgs) >,
    // onConnectionRequest : Option<&'a dyn NFSConnectionRequestEvent>,
    onConnectionRequest : Option<fn (sender : &NFS, e : &mut NFSConnectionRequestEventArgs) >,
    // onCreateLink : Option<&'a dyn NFSCreateLinkEvent>,
    onCreateLink : Option<fn (sender : &NFS, e : &mut NFSCreateLinkEventArgs) >,
    // onDisconnected : Option<&'a dyn NFSDisconnectedEvent>,
    onDisconnected : Option<fn (sender : &NFS, e : &mut NFSDisconnectedEventArgs) >,
    // onError : Option<&'a dyn NFSErrorEvent>,
    onError : Option<fn (sender : &NFS, e : &mut NFSErrorEventArgs) >,
    // onGetAttr : Option<&'a dyn NFSGetAttrEvent>,
    onGetAttr : Option<fn (sender : &NFS, e : &mut NFSGetAttrEventArgs) >,
    // onLock : Option<&'a dyn NFSLockEvent>,
    onLock : Option<fn (sender : &NFS, e : &mut NFSLockEventArgs) >,
    // onLog : Option<&'a dyn NFSLogEvent>,
    onLog : Option<fn (sender : &NFS, e : &mut NFSLogEventArgs) >,
    // onLookup : Option<&'a dyn NFSLookupEvent>,
    onLookup : Option<fn (sender : &NFS, e : &mut NFSLookupEventArgs) >,
    // onMkDir : Option<&'a dyn NFSMkDirEvent>,
    onMkDir : Option<fn (sender : &NFS, e : &mut NFSMkDirEventArgs) >,
    // onOpen : Option<&'a dyn NFSOpenEvent>,
    onOpen : Option<fn (sender : &NFS, e : &mut NFSOpenEventArgs) >,
    // onRead : Option<&'a dyn NFSReadEvent>,
    onRead : Option<fn (sender : &NFS, e : &mut NFSReadEventArgs) >,
    // onReadDir : Option<&'a dyn NFSReadDirEvent>,
    onReadDir : Option<fn (sender : &NFS, e : &mut NFSReadDirEventArgs) >,
    // onReadLink : Option<&'a dyn NFSReadLinkEvent>,
    onReadLink : Option<fn (sender : &NFS, e : &mut NFSReadLinkEventArgs) >,
    // onRename : Option<&'a dyn NFSRenameEvent>,
    onRename : Option<fn (sender : &NFS, e : &mut NFSRenameEventArgs) >,
    // onRmDir : Option<&'a dyn NFSRmDirEvent>,
    onRmDir : Option<fn (sender : &NFS, e : &mut NFSRmDirEventArgs) >,
    // onTruncate : Option<&'a dyn NFSTruncateEvent>,
    onTruncate : Option<fn (sender : &NFS, e : &mut NFSTruncateEventArgs) >,
    // onUnlink : Option<&'a dyn NFSUnlinkEvent>,
    onUnlink : Option<fn (sender : &NFS, e : &mut NFSUnlinkEventArgs) >,
    // onUnlock : Option<&'a dyn NFSUnlockEvent>,
    onUnlock : Option<fn (sender : &NFS, e : &mut NFSUnlockEventArgs) >,
    // onUTime : Option<&'a dyn NFSUTimeEvent>,
    onUTime : Option<fn (sender : &NFS, e : &mut NFSUTimeEventArgs) >,
    // onWrite : Option<&'a dyn NFSWriteEvent>,
    onWrite : Option<fn (sender : &NFS, e : &mut NFSWriteEventArgs) >,

    Id : usize,
    Handle : usize 
}

//impl<'a> Drop for NFS<'a>
impl Drop for NFS
{
    fn drop(&mut self)
    {
        self.dispose();
    }
}

impl NFS
{
    pub fn new() -> &'static mut NFS
    {        
         #[cfg(target_os = "android")]
         panic!("NFS is not available on Android");
        #[cfg(target_os = "ios")]
        panic!("NFS is not available on iOS");
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
            lId = NFSIDSeed.fetch_add(1, SeqCst) as usize;
        }

        let lHandle : isize;
        unsafe
        {
            let callable = CBFSConnect_NFS_Create.clone().unwrap();
            lHandle = callable(NFSEventDispatcher, lId, std::ptr::null(), NFSCreateOpt) as isize;
        }
        if lHandle < 0
        {
            panic!("Failed to instantiate NFS. Please verify that it is supported on this platform");
        }

        let result : NFS = NFS
        {
            onAccess: None,
            onChmod: None,
            onChown: None,
            onClose: None,
            onCommit: None,
            onConnected: None,
            onConnectionRequest: None,
            onCreateLink: None,
            onDisconnected: None,
            onError: None,
            onGetAttr: None,
            onLock: None,
            onLog: None,
            onLookup: None,
            onMkDir: None,
            onOpen: None,
            onRead: None,
            onReadDir: None,
            onReadLink: None,
            onRename: None,
            onRmDir: None,
            onTruncate: None,
            onUnlink: None,
            onUnlock: None,
            onUTime: None,
            onWrite: None,
            Id: lId,
            Handle: lHandle as usize
        };

        let oem_key = CString::new(cbfsconnectkey::rtkCBFSConnect).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();

        unsafe
        {
            let callable = CBFSConnect_NFS_SetCStr.clone().unwrap();
            ret_code = callable(lHandle as usize, 8012/*PID_KEYCHECK_RUST*/, 0, oem_key_ptr, 0) as i32;
            let _ = CString::from_raw(oem_key_ptr);
        }
        if ret_code != 0
        {
            panic!("Initialization of NFS has failed with error {}", ret_code);
        }

        // Lock the Mutex to get access to the HashMap
        unsafe
        {
            let _map = NFSDictMutex.lock().unwrap(); // It is used as a synchronization primitive - don't touch
            NFSDict.insert(lId, result);
            let res = NFSDict.get_mut(&lId).unwrap();
            return res;
        } // The lock is automatically released here
    }

    pub fn dispose(&self)
    {
        let mut _aself : Option<NFS>;
        unsafe
        {
            let _map = NFSDictMutex.lock().unwrap(); // It is used as a synchronization primitive - don't touch

            if !NFSDict.contains_key(&self.Id)
            {
                return;
            }

            // Remove itself from the list
            _aself = NFSDict.remove(&self.Id);

            // finalize the ctlclass
            let callable = CBFSConnect_NFS_Destroy.clone().unwrap();
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
            let mut args : NFSAccessEventArgs = NFSAccessEventArgs::new(par, cbpar);
            callable/*.on_access*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_access(&self) -> &'a dyn NFSAccessEvent
    pub fn on_access(&self) -> Option<fn (sender : &NFS, e : &mut NFSAccessEventArgs)>
    {
        self.onAccess
    }

    //pub fn set_on_access(&mut self, value : &'a dyn NFSAccessEvent)
    pub fn set_on_access(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSAccessEventArgs)>)
    {
        self.onAccess = value;
    }

    fn fire_chmod(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onChmod
        {
            let mut args : NFSChmodEventArgs = NFSChmodEventArgs::new(par, cbpar);
            callable/*.on_chmod*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_chmod(&self) -> &'a dyn NFSChmodEvent
    pub fn on_chmod(&self) -> Option<fn (sender : &NFS, e : &mut NFSChmodEventArgs)>
    {
        self.onChmod
    }

    //pub fn set_on_chmod(&mut self, value : &'a dyn NFSChmodEvent)
    pub fn set_on_chmod(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSChmodEventArgs)>)
    {
        self.onChmod = value;
    }

    fn fire_chown(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onChown
        {
            let mut args : NFSChownEventArgs = NFSChownEventArgs::new(par, cbpar);
            callable/*.on_chown*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_chown(&self) -> &'a dyn NFSChownEvent
    pub fn on_chown(&self) -> Option<fn (sender : &NFS, e : &mut NFSChownEventArgs)>
    {
        self.onChown
    }

    //pub fn set_on_chown(&mut self, value : &'a dyn NFSChownEvent)
    pub fn set_on_chown(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSChownEventArgs)>)
    {
        self.onChown = value;
    }

    fn fire_close(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onClose
        {
            let mut args : NFSCloseEventArgs = NFSCloseEventArgs::new(par, cbpar);
            callable/*.on_close*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_close(&self) -> &'a dyn NFSCloseEvent
    pub fn on_close(&self) -> Option<fn (sender : &NFS, e : &mut NFSCloseEventArgs)>
    {
        self.onClose
    }

    //pub fn set_on_close(&mut self, value : &'a dyn NFSCloseEvent)
    pub fn set_on_close(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSCloseEventArgs)>)
    {
        self.onClose = value;
    }

    fn fire_commit(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCommit
        {
            let mut args : NFSCommitEventArgs = NFSCommitEventArgs::new(par, cbpar);
            callable/*.on_commit*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_commit(&self) -> &'a dyn NFSCommitEvent
    pub fn on_commit(&self) -> Option<fn (sender : &NFS, e : &mut NFSCommitEventArgs)>
    {
        self.onCommit
    }

    //pub fn set_on_commit(&mut self, value : &'a dyn NFSCommitEvent)
    pub fn set_on_commit(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSCommitEventArgs)>)
    {
        self.onCommit = value;
    }

    fn fire_connected(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onConnected
        {
            let mut args : NFSConnectedEventArgs = NFSConnectedEventArgs::new(par, cbpar);
            callable/*.on_connected*/(&self, &mut args);
        }
    }

    //pub fn on_connected(&self) -> &'a dyn NFSConnectedEvent
    pub fn on_connected(&self) -> Option<fn (sender : &NFS, e : &mut NFSConnectedEventArgs)>
    {
        self.onConnected
    }

    //pub fn set_on_connected(&mut self, value : &'a dyn NFSConnectedEvent)
    pub fn set_on_connected(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSConnectedEventArgs)>)
    {
        self.onConnected = value;
    }

    fn fire_connection_request(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onConnectionRequest
        {
            let mut args : NFSConnectionRequestEventArgs = NFSConnectionRequestEventArgs::new(par, cbpar);
            callable/*.on_connection_request*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_connection_request(&self) -> &'a dyn NFSConnectionRequestEvent
    pub fn on_connection_request(&self) -> Option<fn (sender : &NFS, e : &mut NFSConnectionRequestEventArgs)>
    {
        self.onConnectionRequest
    }

    //pub fn set_on_connection_request(&mut self, value : &'a dyn NFSConnectionRequestEvent)
    pub fn set_on_connection_request(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSConnectionRequestEventArgs)>)
    {
        self.onConnectionRequest = value;
    }

    fn fire_create_link(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onCreateLink
        {
            let mut args : NFSCreateLinkEventArgs = NFSCreateLinkEventArgs::new(par, cbpar);
            callable/*.on_create_link*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_create_link(&self) -> &'a dyn NFSCreateLinkEvent
    pub fn on_create_link(&self) -> Option<fn (sender : &NFS, e : &mut NFSCreateLinkEventArgs)>
    {
        self.onCreateLink
    }

    //pub fn set_on_create_link(&mut self, value : &'a dyn NFSCreateLinkEvent)
    pub fn set_on_create_link(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSCreateLinkEventArgs)>)
    {
        self.onCreateLink = value;
    }

    fn fire_disconnected(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDisconnected
        {
            let mut args : NFSDisconnectedEventArgs = NFSDisconnectedEventArgs::new(par, cbpar);
            callable/*.on_disconnected*/(&self, &mut args);
        }
    }

    //pub fn on_disconnected(&self) -> &'a dyn NFSDisconnectedEvent
    pub fn on_disconnected(&self) -> Option<fn (sender : &NFS, e : &mut NFSDisconnectedEventArgs)>
    {
        self.onDisconnected
    }

    //pub fn set_on_disconnected(&mut self, value : &'a dyn NFSDisconnectedEvent)
    pub fn set_on_disconnected(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSDisconnectedEventArgs)>)
    {
        self.onDisconnected = value;
    }

    fn fire_error(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onError
        {
            let mut args : NFSErrorEventArgs = NFSErrorEventArgs::new(par, cbpar);
            callable/*.on_error*/(&self, &mut args);
        }
    }

    //pub fn on_error(&self) -> &'a dyn NFSErrorEvent
    pub fn on_error(&self) -> Option<fn (sender : &NFS, e : &mut NFSErrorEventArgs)>
    {
        self.onError
    }

    //pub fn set_on_error(&mut self, value : &'a dyn NFSErrorEvent)
    pub fn set_on_error(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSErrorEventArgs)>)
    {
        self.onError = value;
    }

    fn fire_get_attr(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onGetAttr
        {
            let mut args : NFSGetAttrEventArgs = NFSGetAttrEventArgs::new(par, cbpar);
            callable/*.on_get_attr*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_get_attr(&self) -> &'a dyn NFSGetAttrEvent
    pub fn on_get_attr(&self) -> Option<fn (sender : &NFS, e : &mut NFSGetAttrEventArgs)>
    {
        self.onGetAttr
    }

    //pub fn set_on_get_attr(&mut self, value : &'a dyn NFSGetAttrEvent)
    pub fn set_on_get_attr(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSGetAttrEventArgs)>)
    {
        self.onGetAttr = value;
    }

    fn fire_lock(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onLock
        {
            let mut args : NFSLockEventArgs = NFSLockEventArgs::new(par, cbpar);
            callable/*.on_lock*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_lock(&self) -> &'a dyn NFSLockEvent
    pub fn on_lock(&self) -> Option<fn (sender : &NFS, e : &mut NFSLockEventArgs)>
    {
        self.onLock
    }

    //pub fn set_on_lock(&mut self, value : &'a dyn NFSLockEvent)
    pub fn set_on_lock(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSLockEventArgs)>)
    {
        self.onLock = value;
    }

    fn fire_log(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onLog
        {
            let mut args : NFSLogEventArgs = NFSLogEventArgs::new(par, cbpar);
            callable/*.on_log*/(&self, &mut args);
        }
    }

    //pub fn on_log(&self) -> &'a dyn NFSLogEvent
    pub fn on_log(&self) -> Option<fn (sender : &NFS, e : &mut NFSLogEventArgs)>
    {
        self.onLog
    }

    //pub fn set_on_log(&mut self, value : &'a dyn NFSLogEvent)
    pub fn set_on_log(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSLogEventArgs)>)
    {
        self.onLog = value;
    }

    fn fire_lookup(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onLookup
        {
            let mut args : NFSLookupEventArgs = NFSLookupEventArgs::new(par, cbpar);
            callable/*.on_lookup*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_lookup(&self) -> &'a dyn NFSLookupEvent
    pub fn on_lookup(&self) -> Option<fn (sender : &NFS, e : &mut NFSLookupEventArgs)>
    {
        self.onLookup
    }

    //pub fn set_on_lookup(&mut self, value : &'a dyn NFSLookupEvent)
    pub fn set_on_lookup(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSLookupEventArgs)>)
    {
        self.onLookup = value;
    }

    fn fire_mk_dir(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onMkDir
        {
            let mut args : NFSMkDirEventArgs = NFSMkDirEventArgs::new(par, cbpar);
            callable/*.on_mk_dir*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_mk_dir(&self) -> &'a dyn NFSMkDirEvent
    pub fn on_mk_dir(&self) -> Option<fn (sender : &NFS, e : &mut NFSMkDirEventArgs)>
    {
        self.onMkDir
    }

    //pub fn set_on_mk_dir(&mut self, value : &'a dyn NFSMkDirEvent)
    pub fn set_on_mk_dir(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSMkDirEventArgs)>)
    {
        self.onMkDir = value;
    }

    fn fire_open(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onOpen
        {
            let mut args : NFSOpenEventArgs = NFSOpenEventArgs::new(par, cbpar);
            callable/*.on_open*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_open(&self) -> &'a dyn NFSOpenEvent
    pub fn on_open(&self) -> Option<fn (sender : &NFS, e : &mut NFSOpenEventArgs)>
    {
        self.onOpen
    }

    //pub fn set_on_open(&mut self, value : &'a dyn NFSOpenEvent)
    pub fn set_on_open(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSOpenEventArgs)>)
    {
        self.onOpen = value;
    }

    fn fire_read(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onRead
        {
            let mut args : NFSReadEventArgs = NFSReadEventArgs::new(par, cbpar);
            callable/*.on_read*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_read(&self) -> &'a dyn NFSReadEvent
    pub fn on_read(&self) -> Option<fn (sender : &NFS, e : &mut NFSReadEventArgs)>
    {
        self.onRead
    }

    //pub fn set_on_read(&mut self, value : &'a dyn NFSReadEvent)
    pub fn set_on_read(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSReadEventArgs)>)
    {
        self.onRead = value;
    }

    fn fire_read_dir(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onReadDir
        {
            let mut args : NFSReadDirEventArgs = NFSReadDirEventArgs::new(par, cbpar);
            callable/*.on_read_dir*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_read_dir(&self) -> &'a dyn NFSReadDirEvent
    pub fn on_read_dir(&self) -> Option<fn (sender : &NFS, e : &mut NFSReadDirEventArgs)>
    {
        self.onReadDir
    }

    //pub fn set_on_read_dir(&mut self, value : &'a dyn NFSReadDirEvent)
    pub fn set_on_read_dir(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSReadDirEventArgs)>)
    {
        self.onReadDir = value;
    }

    fn fire_read_link(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onReadLink
        {
            let mut args : NFSReadLinkEventArgs = NFSReadLinkEventArgs::new(par, cbpar);
            callable/*.on_read_link*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_read_link(&self) -> &'a dyn NFSReadLinkEvent
    pub fn on_read_link(&self) -> Option<fn (sender : &NFS, e : &mut NFSReadLinkEventArgs)>
    {
        self.onReadLink
    }

    //pub fn set_on_read_link(&mut self, value : &'a dyn NFSReadLinkEvent)
    pub fn set_on_read_link(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSReadLinkEventArgs)>)
    {
        self.onReadLink = value;
    }

    fn fire_rename(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onRename
        {
            let mut args : NFSRenameEventArgs = NFSRenameEventArgs::new(par, cbpar);
            callable/*.on_rename*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_rename(&self) -> &'a dyn NFSRenameEvent
    pub fn on_rename(&self) -> Option<fn (sender : &NFS, e : &mut NFSRenameEventArgs)>
    {
        self.onRename
    }

    //pub fn set_on_rename(&mut self, value : &'a dyn NFSRenameEvent)
    pub fn set_on_rename(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSRenameEventArgs)>)
    {
        self.onRename = value;
    }

    fn fire_rm_dir(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onRmDir
        {
            let mut args : NFSRmDirEventArgs = NFSRmDirEventArgs::new(par, cbpar);
            callable/*.on_rm_dir*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_rm_dir(&self) -> &'a dyn NFSRmDirEvent
    pub fn on_rm_dir(&self) -> Option<fn (sender : &NFS, e : &mut NFSRmDirEventArgs)>
    {
        self.onRmDir
    }

    //pub fn set_on_rm_dir(&mut self, value : &'a dyn NFSRmDirEvent)
    pub fn set_on_rm_dir(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSRmDirEventArgs)>)
    {
        self.onRmDir = value;
    }

    fn fire_truncate(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onTruncate
        {
            let mut args : NFSTruncateEventArgs = NFSTruncateEventArgs::new(par, cbpar);
            callable/*.on_truncate*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_truncate(&self) -> &'a dyn NFSTruncateEvent
    pub fn on_truncate(&self) -> Option<fn (sender : &NFS, e : &mut NFSTruncateEventArgs)>
    {
        self.onTruncate
    }

    //pub fn set_on_truncate(&mut self, value : &'a dyn NFSTruncateEvent)
    pub fn set_on_truncate(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSTruncateEventArgs)>)
    {
        self.onTruncate = value;
    }

    fn fire_unlink(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onUnlink
        {
            let mut args : NFSUnlinkEventArgs = NFSUnlinkEventArgs::new(par, cbpar);
            callable/*.on_unlink*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_unlink(&self) -> &'a dyn NFSUnlinkEvent
    pub fn on_unlink(&self) -> Option<fn (sender : &NFS, e : &mut NFSUnlinkEventArgs)>
    {
        self.onUnlink
    }

    //pub fn set_on_unlink(&mut self, value : &'a dyn NFSUnlinkEvent)
    pub fn set_on_unlink(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSUnlinkEventArgs)>)
    {
        self.onUnlink = value;
    }

    fn fire_unlock(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onUnlock
        {
            let mut args : NFSUnlockEventArgs = NFSUnlockEventArgs::new(par, cbpar);
            callable/*.on_unlock*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_unlock(&self) -> &'a dyn NFSUnlockEvent
    pub fn on_unlock(&self) -> Option<fn (sender : &NFS, e : &mut NFSUnlockEventArgs)>
    {
        self.onUnlock
    }

    //pub fn set_on_unlock(&mut self, value : &'a dyn NFSUnlockEvent)
    pub fn set_on_unlock(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSUnlockEventArgs)>)
    {
        self.onUnlock = value;
    }

    fn fire_u_time(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onUTime
        {
            let mut args : NFSUTimeEventArgs = NFSUTimeEventArgs::new(par, cbpar);
            callable/*.on_u_time*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_u_time(&self) -> &'a dyn NFSUTimeEvent
    pub fn on_u_time(&self) -> Option<fn (sender : &NFS, e : &mut NFSUTimeEventArgs)>
    {
        self.onUTime
    }

    //pub fn set_on_u_time(&mut self, value : &'a dyn NFSUTimeEvent)
    pub fn set_on_u_time(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSUTimeEventArgs)>)
    {
        self.onUTime = value;
    }

    fn fire_write(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onWrite
        {
            let mut args : NFSWriteEventArgs = NFSWriteEventArgs::new(par, cbpar);
            callable/*.on_write*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_write(&self) -> &'a dyn NFSWriteEvent
    pub fn on_write(&self) -> Option<fn (sender : &NFS, e : &mut NFSWriteEventArgs)>
    {
        self.onWrite
    }

    //pub fn set_on_write(&mut self, value : &'a dyn NFSWriteEvent)
    pub fn set_on_write(&mut self, value : Option<fn (sender : &NFS, e : &mut NFSWriteEventArgs)>)
    {
        self.onWrite = value;
    }


    pub(crate) fn report_error_info(&self, error_info : &str)
    {
        if let Some(callable) = self.onError
        {
            let mut args : NFSErrorEventArgs = NFSErrorEventArgs
                {
                    ConnectionId: 0,
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
            let callable = CBFSConnect_NFS_GetLastError.clone().unwrap();
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
            let callable = CBFSConnect_NFS_GetLastErrorCode.clone().unwrap();
            result = callable(self.Handle) as i32;
        }
        result
    }

    // GetRuntimeLicense returns the runtime license key set for NFS.
    pub fn get_runtime_license(&self) -> Result<String, CBFSConnectError>
    {
        let result : String;
        //let length : c_long;
        unsafe
        {
            let callable = CBFSConnect_NFS_GetAsCStr.clone().unwrap();

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

    // SetRuntimeLicense sets the runtime license key for NFS.
    pub fn set_runtime_license(&self, value : String) -> Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let oem_key = CString::new(value).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();
        unsafe
        {
            let callable = CBFSConnect_NFS_SetCStr.clone().unwrap();
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

    // Gets the value of the NFSConnectionCount property: The number of records in the NFSConnection arrays.
    pub fn nfs_connection_count(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_NFS_GetAsInt.clone().unwrap();
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


    // Gets the value of the NFSConnectionAuxGIDs property: This property contains a comma-separated list of Auxiliary GIDs (or groups the client is a part of) associated with the connection making a request.
    pub fn nfs_connection_aux_gi_ds(&self, ConnectionId : i32) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_NFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 2, ConnectionId as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value ConnectionId for NFSConnectionAuxGIDs"));
            }
        }
        unsafe
        {
            let callable = CBFSConnect_NFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 2, ConnectionId as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the NFSConnectionClientId property: This property represents the client identifier, as assigned by the server, of the client making the current request.
    pub fn nfs_connection_client_id(&self, ConnectionId : i32) -> Result<i64, CBFSConnectError>
    {
        let ret_val : i64; // = 0;
        unsafe
        {
            let callable = CBFSConnect_NFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 3, ConnectionId as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value ConnectionId for NFSConnectionClientId"));
            }
        }
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSConnect_NFS_GetAsInt64.clone().unwrap();
            callable(self.Handle, 3, ConnectionId as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the NFSConnectionConnected property: This property shows the status of a particular connection (connected/disconnected).
    pub fn nfs_connection_connected(&self, ConnectionId : i32) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe
        {
            let callable = CBFSConnect_NFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 4, ConnectionId as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value ConnectionId for NFSConnectionConnected"));
            }
        }
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_NFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 4, ConnectionId as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the NFSConnectionCurrentFile property: This property represents the current file opened by the connection.
    pub fn nfs_connection_current_file(&self, ConnectionId : i32) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_NFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 6, ConnectionId as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value ConnectionId for NFSConnectionCurrentFile"));
            }
        }
        unsafe
        {
            let callable = CBFSConnect_NFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 6, ConnectionId as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the NFSConnectionGID property: This property represents the current group identifier (GID) of the connection making a request.
    pub fn nfs_connection_gid(&self, ConnectionId : i32) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSConnect_NFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 7, ConnectionId as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value ConnectionId for NFSConnectionGID"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_NFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 7, ConnectionId as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the NFSConnectionRemoteHost property: This property indicates the IP address of the remote host through which the connection is coming.
    pub fn nfs_connection_remote_host(&self, ConnectionId : i32) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_NFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 8, ConnectionId as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value ConnectionId for NFSConnectionRemoteHost"));
            }
        }
        unsafe
        {
            let callable = CBFSConnect_NFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 8, ConnectionId as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the NFSConnectionRemotePort property: This property indicates the TCP port on the remote host through which the connection is coming.
    pub fn nfs_connection_remote_port(&self, ConnectionId : i32) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSConnect_NFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 9, ConnectionId as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value ConnectionId for NFSConnectionRemotePort"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_NFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 9, ConnectionId as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the NFSConnectionUID property: This property represents the current user identifier (UID) of the connection making a request.
    pub fn nfs_connection_uid(&self, ConnectionId : i32) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSConnect_NFS_CheckIndex.clone().unwrap();
            let ret_code = callable(self.Handle, 10, ConnectionId as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(-1, "Invalid array index value ConnectionId for NFSConnectionUID"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_NFS_GetAsInt.clone().unwrap();
            val = callable(self.Handle, 10, ConnectionId as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the Listening property: This property indicates whether the struct is listening for incoming connections.
    pub fn listening(&self) -> Result<bool, CBFSConnectError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSConnect_NFS_GetAsInt.clone().unwrap();
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


    // Gets the value of the LocalHost property: The name of the local host or user-assigned IP interface through which connections are initiated or accepted.
    pub fn local_host(&self) -> Result<String, CBFSConnectError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSConnect_NFS_GetAsCStr.clone().unwrap();

            let cptr = callable(self.Handle, 12, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the LocalHost property: The name of the local host or user-assigned IP interface through which connections are initiated or accepted.
    pub fn set_local_host(&self, value : &str) -> Result<(), CBFSConnectError> {
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

            let callable = CBFSConnect_NFS_SetCStr.clone().unwrap();
            ret_code = callable(self.Handle, 12, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the LocalPort property: The TCP port in the local host where the struct listens.
    pub fn local_port(&self) -> Result<i32, CBFSConnectError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSConnect_NFS_GetAsInt.clone().unwrap();
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


    // Sets the value of the LocalPort property: The TCP port in the local host where the struct listens.
    pub fn set_local_port(&self, value : i32) -> Result<(), CBFSConnectError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSConnect_NFS_SetInt.clone().unwrap();
            ret_code = callable(self.Handle, 13, 0 as c_long, value as isize, 0) as i32;
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
            let callable = CBFSConnect_NFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 2, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrConfigurationStringPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn config

    // Disconnect: This method disconnects the specified client.
    pub fn disconnect(&self, connection_id : i32) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        CParams[0] = (connection_id as isize) as usize;

        unsafe
        {
            let callable = CBFSConnect_NFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 3, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn disconnect

    // DoEvents: This method processes events from the internal message queue.
    pub fn do_events(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_NFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 4, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn do_events

    // FillDir: This method fills the buffer with information about a directory entry.
    pub fn fill_dir(&self, connection_id : i32, name : &str, file_id : i64, cookie : i64, mode : i32, user : &str, group : &str, link_count : i32, size : i64, a_time : &chrono::DateTime<Utc>, m_time : &chrono::DateTime<Utc>, c_time : &chrono::DateTime<Utc>) ->  Result<i32, CBFSConnectError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 12 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        CParams[0] = (connection_id as isize) as usize;
        let CStrNamePtr : *mut c_char;
        match CString::new(name)
        {
            Ok(CStrValueName) => { CStrNamePtr = CStrValueName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrNamePtr as usize;
        let FileIdArr : Vec<i64> = vec![file_id];
        CParams[2] = FileIdArr.as_ptr() as usize;
        let CookieArr : Vec<i64> = vec![cookie];
        CParams[3] = CookieArr.as_ptr() as usize;
        CParams[4] = (mode as isize) as usize;
        let CStrUserPtr : *mut c_char;
        match CString::new(user)
        {
            Ok(CStrValueUser) => { CStrUserPtr = CStrValueUser.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[5] = CStrUserPtr as usize;
        let CStrGroupPtr : *mut c_char;
        match CString::new(group)
        {
            Ok(CStrValueGroup) => { CStrGroupPtr = CStrValueGroup.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[6] = CStrGroupPtr as usize;
        CParams[7] = (link_count as isize) as usize;
        let SizeArr : Vec<i64> = vec![size];
        CParams[8] = SizeArr.as_ptr() as usize;
        let ATimeUnixDate : i64 = chrono_time_to_file_time(a_time);
        let ATimeArr : Vec<i64> = vec![ATimeUnixDate];
        CParams[9] = ATimeArr.as_ptr() as usize;
        let MTimeUnixDate : i64 = chrono_time_to_file_time(m_time);
        let MTimeArr : Vec<i64> = vec![MTimeUnixDate];
        CParams[10] = MTimeArr.as_ptr() as usize;
        let CTimeUnixDate : i64 = chrono_time_to_file_time(c_time);
        let CTimeArr : Vec<i64> = vec![CTimeUnixDate];
        CParams[11] = CTimeArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSConnect_NFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 5, 12, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrNamePtr);
            let _ = CString::from_raw(CStrUserPtr);
            let _ = CString::from_raw(CStrGroupPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[12] as i32;
        return Result::Ok(ret_val);
    } // fn fill_dir

    // Shutdown: This method shuts down the server.
    pub fn shutdown(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_NFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 6, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn shutdown

    // StartListening: This method starts listening for incoming connections.
    pub fn start_listening(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_NFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 7, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn start_listening

    // StopListening: This method stops listening for new connections.
    pub fn stop_listening(&self, ) ->  Result<(), CBFSConnectError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSConnect_NFS_Do.clone().unwrap();
            ret_code = callable(self.Handle, 8, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSConnectError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn stop_listening

} // NFS

extern "system" fn NFSEventDispatcher(pObj : usize, event_id : c_long, _cparam : c_long, param : IntPtrArrayType, cbparam : IntArrayType) -> c_long
{
    let obj: &'static NFS;
    // Lock the Mutex to get access to the HashMap
    unsafe
    {
        let _map = NFSDictMutex.lock().unwrap();
        let objOpt = NFSDict.get(&pObj);
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

            4 /* Close */=> obj.fire_close(/*cparam as i32, */param, cbparam),

            5 /* Commit */=> obj.fire_commit(/*cparam as i32, */param, cbparam),

            6 /* Connected */=> obj.fire_connected(/*cparam as i32, */param, cbparam),

            7 /* ConnectionRequest */=> obj.fire_connection_request(/*cparam as i32, */param, cbparam),

            8 /* CreateLink */=> obj.fire_create_link(/*cparam as i32, */param, cbparam),

            9 /* Disconnected */=> obj.fire_disconnected(/*cparam as i32, */param, cbparam),

            10 /* Error */=> obj.fire_error(/*cparam as i32, */param, cbparam),

            11 /* GetAttr */=> obj.fire_get_attr(/*cparam as i32, */param, cbparam),

            12 /* Lock */=> obj.fire_lock(/*cparam as i32, */param, cbparam),

            13 /* Log */=> obj.fire_log(/*cparam as i32, */param, cbparam),

            14 /* Lookup */=> obj.fire_lookup(/*cparam as i32, */param, cbparam),

            15 /* MkDir */=> obj.fire_mk_dir(/*cparam as i32, */param, cbparam),

            16 /* Open */=> obj.fire_open(/*cparam as i32, */param, cbparam),

            17 /* Read */=> obj.fire_read(/*cparam as i32, */param, cbparam),

            18 /* ReadDir */=> obj.fire_read_dir(/*cparam as i32, */param, cbparam),

            19 /* ReadLink */=> obj.fire_read_link(/*cparam as i32, */param, cbparam),

            20 /* Rename */=> obj.fire_rename(/*cparam as i32, */param, cbparam),

            21 /* RmDir */=> obj.fire_rm_dir(/*cparam as i32, */param, cbparam),

            22 /* Truncate */=> obj.fire_truncate(/*cparam as i32, */param, cbparam),

            23 /* Unlink */=> obj.fire_unlink(/*cparam as i32, */param, cbparam),

            24 /* Unlock */=> obj.fire_unlock(/*cparam as i32, */param, cbparam),

            25 /* UTime */=> obj.fire_u_time(/*cparam as i32, */param, cbparam),

            26 /* Write */=> obj.fire_write(/*cparam as i32, */param, cbparam),

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


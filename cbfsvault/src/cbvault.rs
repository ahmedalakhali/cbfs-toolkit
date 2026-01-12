


#![allow(non_snake_case)]

extern crate libloading as lib;

use std::{collections::HashMap, ffi::{c_char, c_long, c_longlong, c_ulong, c_void, CStr, CString}, panic::catch_unwind, sync::{atomic::{AtomicUsize, Ordering::SeqCst}, Mutex, OnceLock} };
use lib::{Library, Symbol, Error};
use std::fmt::Write;
use chrono::Utc;
use once_cell::sync::Lazy;

use crate::{*, cbfsvaultkey};

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVault_StaticInit)(void *hInst);
type CBFSVaultCBVaultStaticInitType = unsafe extern "system" fn(hInst : *mut c_void) -> i32;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVault_StaticDestroy)();
type CBFSVaultCBVaultStaticDestroyType = unsafe extern "system" fn()-> i32;

// typedef void* (CBFSVAULT_CALL *lpCBFSVault_CBVault_Create)(PCBFSVAULT_CALLBACK lpSink, void *lpContext, char *lpOemKey, int opts);
type CBFSVaultCBVaultCreateType = unsafe extern "system" fn(lpSink : CBFSVaultSinkDelegateType, lpContext : usize, lpOemKey : *const c_char, opts : i32) -> usize;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVault_Destroy)(void *lpObj);
type CBFSVaultCBVaultDestroyType = unsafe extern "system" fn(lpObj: usize)-> c_long;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVault_CheckIndex)(void *lpObj, int propid, int arridx);
type CBFSVaultCBVaultCheckIndexType = unsafe extern "system" fn(lpObj: usize, propid: c_long, arridx: c_long)-> c_long;

// typedef void* (CBFSVAULT_CALL *lpCBFSVault_CBVault_Get)(void *lpObj, int propid, int arridx, int *lpcbVal, int64 *lpllVal);
type CBFSVaultCBVaultGetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *mut c_longlong) -> *mut c_void;
type CBFSVaultCBVaultGetAsCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *const c_longlong) -> *const c_char;
type CBFSVaultCBVaultGetAsIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *const c_void) -> usize;
type CBFSVaultCBVaultGetAsInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *mut i64) -> usize;
type CBFSVaultCBVaultGetAsBSTRType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut i32, llVal: *const c_void) -> *const u8;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVault_Set)(void *lpObj, int propid, int arridx, const void *val, int cbVal);
type CBFSVaultCBVaultSetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_void, len: c_ulong)-> c_long;
type CBFSVaultCBVaultSetCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_char, len: c_ulong)-> c_long;
type CBFSVaultCBVaultSetIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: isize, len: c_ulong)-> c_long;
type CBFSVaultCBVaultSetInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const i64, len: c_ulong)-> c_long;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVault_Do)(void *lpObj, int methid, int cparam, void *param[], int cbparam[], int64 *lpllVal);
type CBFSVaultCBVaultDoType = unsafe extern "system" fn(p: usize, method_id: c_long, cparam: c_long, params: UIntPtrArrayType, cbparam: IntArrayType, llVal: *mut c_longlong)-> c_long;

// typedef char* (CBFSVAULT_CALL *lpCBFSVault_CBVault_GetLastError)(void *lpObj);
type CBFSVaultCBVaultGetLastErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVault_GetLastErrorCode)(void *lpObj);
type CBFSVaultCBVaultGetLastErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVault_SetLastErrorAndCode)(void *lpObj, int code, char *message);
type CBFSVaultCBVaultSetLastErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

// typedef char* (CBFSVAULT_CALL *lpCBFSVault_CBVault_GetEventError)(void *lpObj);
type CBFSVaultCBVaultGetEventErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVault_GetEventErrorCode)(void *lpObj);
type CBFSVaultCBVaultGetEventErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVault_SetEventErrorAndCode)(void *lpObj, int code, char *message);
type CBFSVaultCBVaultSetEventErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

static CBFSVault_CBVault_StaticInit : OnceLock<Symbol<'static, CBFSVaultCBVaultStaticInitType>> = OnceLock::new();
static CBFSVault_CBVault_StaticDestroy : OnceLock<Symbol<'static, CBFSVaultCBVaultStaticDestroyType>> = OnceLock::new();

static CBFSVault_CBVault_Create: OnceLock<Symbol<'static, CBFSVaultCBVaultCreateType>> = OnceLock::new();
static CBFSVault_CBVault_Destroy: OnceLock<Symbol<'static, CBFSVaultCBVaultDestroyType>> = OnceLock::new();
static CBFSVault_CBVault_Set: OnceLock<Symbol<'static, CBFSVaultCBVaultSetType>> = OnceLock::new();
static CBFSVault_CBVault_SetCStr: OnceLock<Symbol<'static, CBFSVaultCBVaultSetCStrType>> = OnceLock::new();
static CBFSVault_CBVault_SetInt: OnceLock<Symbol<'static, CBFSVaultCBVaultSetIntType>> = OnceLock::new();
static CBFSVault_CBVault_SetInt64: OnceLock<Symbol<'static, CBFSVaultCBVaultSetInt64Type>> = OnceLock::new();
static CBFSVault_CBVault_Get: OnceLock<Symbol<'static, CBFSVaultCBVaultGetType>> = OnceLock::new();
static CBFSVault_CBVault_GetAsCStr: OnceLock<Symbol<'static, CBFSVaultCBVaultGetAsCStrType>> = OnceLock::new();
static CBFSVault_CBVault_GetAsInt: OnceLock<Symbol<'static, CBFSVaultCBVaultGetAsIntType>> = OnceLock::new();
static CBFSVault_CBVault_GetAsInt64: OnceLock<Symbol<'static, CBFSVaultCBVaultGetAsInt64Type>> = OnceLock::new();
static CBFSVault_CBVault_GetAsBSTR: OnceLock<Symbol<'static, CBFSVaultCBVaultGetAsBSTRType>> = OnceLock::new();
static CBFSVault_CBVault_GetLastError: OnceLock<Symbol<'static, CBFSVaultCBVaultGetLastErrorType>> = OnceLock::new();
static CBFSVault_CBVault_GetLastErrorCode: OnceLock<Symbol<'static, CBFSVaultCBVaultGetLastErrorCodeType>> = OnceLock::new();
static CBFSVault_CBVault_SetLastErrorAndCode: OnceLock<Symbol<'static, CBFSVaultCBVaultSetLastErrorAndCodeType>> = OnceLock::new();
static CBFSVault_CBVault_GetEventError: OnceLock<Symbol<'static, CBFSVaultCBVaultGetEventErrorType>> = OnceLock::new();
static CBFSVault_CBVault_GetEventErrorCode: OnceLock<Symbol<'static, CBFSVaultCBVaultGetEventErrorCodeType>> = OnceLock::new();
static CBFSVault_CBVault_SetEventErrorAndCode: OnceLock<Symbol<'static, CBFSVaultCBVaultSetEventErrorAndCodeType>> = OnceLock::new();
static CBFSVault_CBVault_CheckIndex: OnceLock<Symbol<'static, CBFSVaultCBVaultCheckIndexType>> = OnceLock::new();
static CBFSVault_CBVault_Do: OnceLock<Symbol<'static, CBFSVaultCBVaultDoType>> = OnceLock::new();

static CBVaultIDSeed : AtomicUsize = AtomicUsize::new(1);

static CBVaultDict : Lazy<Mutex<HashMap<usize, usize>>> = Lazy::new(|| Mutex::new(HashMap::new()) );

const CBVaultCreateOpt : i32 = 0;


pub type CBVaultStream = crate::CBFSVaultStream;


pub(crate) fn get_lib_funcs( lib_hand : &'static Library) -> bool
{
    // CBFSVault_CBVault_StaticInit
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultStaticInitType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_StaticInit") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_StaticInit.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_StaticDestroy
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultStaticDestroyType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_StaticDestroy") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_StaticDestroy.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_Create
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultCreateType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Create") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_Create.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_Destroy
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultDestroyType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Destroy") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_Destroy.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_Get
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultGetType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Get") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_Get.set(func);
    } else {
        return false;
    }
    // CBFSVault_CBVault_GetAsCStr
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultGetAsCStrType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Get") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_GetAsCStr.set(func);
    } else {
        return false;
    }
    // CBFSVault_CBVault_GetAsInt
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultGetAsIntType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Get") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_GetAsInt.set(func);
    } else {
        return false;
    }
    // CBFSVault_CBVault_GetAsInt64
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultGetAsInt64Type>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Get") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_GetAsInt64.set(func);
    } else {
        return false;
    }
    // CBFSVault_CBVault_GetAsBSTR
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultGetAsBSTRType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Get") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_GetAsBSTR.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_Set
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultSetType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Set") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_Set.set(func);
    } else {
        return false;
    }
    // CBFSVault_CBVault_SetCStr
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultSetCStrType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Set") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_SetCStr.set(func);
    } else {
        return false;
    }
    // CBFSVault_CBVault_SetInt
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultSetIntType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Set") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_SetInt.set(func);
    } else {
        return false;
    }
    // CBFSVault_CBVault_SetInt64
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultSetInt64Type>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Set") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_SetInt64.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_GetLastError
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultGetLastErrorType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_GetLastError") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_GetLastError.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_GetLastErrorCode
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultGetLastErrorCodeType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_GetLastErrorCode") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_GetLastErrorCode.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_SetLastErrorAndCode
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultSetLastErrorAndCodeType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_SetLastErrorAndCode") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_SetLastErrorAndCode.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_GetEventError
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultGetEventErrorType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_GetEventError") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_GetEventError.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_GetEventErrorCode
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultGetEventErrorCodeType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_GetEventErrorCode") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_GetEventErrorCode.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_SetEventErrorAndCode
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultSetEventErrorAndCodeType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_SetEventErrorAndCode") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_SetEventErrorAndCode.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_CheckIndex
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultCheckIndexType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_CheckIndex") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_CheckIndex.set(func);
    } else {
        return false;
    }

    // CBFSVault_CBVault_Do
    let func_ptr_res : Result<Symbol<'static, CBFSVaultCBVaultDoType>, Error> = unsafe { lib_hand.get(b"CBFSVault_CBVault_Do") };
    if let Ok(func) = func_ptr_res {
        let _ = CBFSVault_CBVault_Do.set(func);
    } else {
        return false;
    }

    return true;
}

//////////////
// Event Types
//////////////


// CBVaultDataCompressEventArgs carries the parameters of the DataCompress event of CBVault
pub struct CBVaultDataCompressEventArgs
{
    hInDataPtr : *mut u8,
    InSize : i32,
    hOutDataPtr : *mut u8,
    OutSize : i32,
    CompressionLevel : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of DataCompressEventArgs
impl CBVaultDataCompressEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDataCompressEventArgs
    {

        let lvhInDataPtr : *mut u8;
        unsafe
        {
            lvhInDataPtr = *par.add(0) as *mut u8;
        }

        let lvInSize : i32;
        unsafe
        {
            lvInSize = *par.add(1) as i32;
        }
        // lvhInDataLen = lvInSize;

        let lvhOutDataPtr : *mut u8;
        unsafe
        {
            lvhOutDataPtr = *par.add(2) as *mut u8;
        }

        let lvOutSize : i32;
        unsafe
        {
            lvOutSize = *par.add(3) as i32;
        }
        // lvhOutDataLen = lvOutSize;

        let lvCompressionLevel : i32;
        unsafe
        {
            lvCompressionLevel = *par.add(4) as i32;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBVaultDataCompressEventArgs
        {
            hInDataPtr: lvhInDataPtr,
            InSize: lvInSize,
            hOutDataPtr: lvhOutDataPtr,
            OutSize: lvOutSize,
            CompressionLevel: lvCompressionLevel,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.OutSize as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn in_data(&self) -> *mut u8
    {
        self.hInDataPtr
    }
    pub fn in_size(&self) -> i32
    {
        self.InSize
    }
    pub fn out_data(&self) -> *mut u8
    {
        self.hOutDataPtr
    }
    pub fn out_size(&self) -> i32
    {
        self.OutSize
    }
    pub fn set_out_size(&mut self, value: i32)
    {
        self.OutSize = value;
    }
    pub fn compression_level(&self) -> i32
    {
        self.CompressionLevel
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

pub trait CBVaultDataCompressEvent
{
    fn on_data_compress(&self, sender : &CBVault, e : &mut CBVaultDataCompressEventArgs);
}


// CBVaultDataDecompressEventArgs carries the parameters of the DataDecompress event of CBVault
pub struct CBVaultDataDecompressEventArgs
{
    hInDataPtr : *mut u8,
    InSize : i32,
    hOutDataPtr : *mut u8,
    OutSize : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of DataDecompressEventArgs
impl CBVaultDataDecompressEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDataDecompressEventArgs
    {

        let lvhInDataPtr : *mut u8;
        unsafe
        {
            lvhInDataPtr = *par.add(0) as *mut u8;
        }

        let lvInSize : i32;
        unsafe
        {
            lvInSize = *par.add(1) as i32;
        }
        // lvhInDataLen = lvInSize;

        let lvhOutDataPtr : *mut u8;
        unsafe
        {
            lvhOutDataPtr = *par.add(2) as *mut u8;
        }

        let lvOutSize : i32;
        unsafe
        {
            lvOutSize = *par.add(3) as i32;
        }
        // lvhOutDataLen = lvOutSize;

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(4) as i32;
        }
        
        CBVaultDataDecompressEventArgs
        {
            hInDataPtr: lvhInDataPtr,
            InSize: lvInSize,
            hOutDataPtr: lvhOutDataPtr,
            OutSize: lvOutSize,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(3)) = self.OutSize as isize;
            *(self._Params.add(4)) = self.ResultCode as isize;
        }
    }

    pub fn in_data(&self) -> *mut u8
    {
        self.hInDataPtr
    }
    pub fn in_size(&self) -> i32
    {
        self.InSize
    }
    pub fn out_data(&self) -> *mut u8
    {
        self.hOutDataPtr
    }
    pub fn out_size(&self) -> i32
    {
        self.OutSize
    }
    pub fn set_out_size(&mut self, value: i32)
    {
        self.OutSize = value;
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

pub trait CBVaultDataDecompressEvent
{
    fn on_data_decompress(&self, sender : &CBVault, e : &mut CBVaultDataDecompressEventArgs);
}


// CBVaultDataDecryptEventArgs carries the parameters of the DataDecrypt event of CBVault
pub struct CBVaultDataDecryptEventArgs
{
    hKeyPtr : *mut u8,
    KeyLength : i32,
    hSalt1Ptr : *mut u8,
    Salt1Size : i32,
    hSalt2Ptr : *mut u8,
    Salt2Size : i32,
    hDataPtr : *mut u8,
    DataSize : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of DataDecryptEventArgs
impl CBVaultDataDecryptEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDataDecryptEventArgs
    {

        let lvhKeyPtr : *mut u8;
        unsafe
        {
            lvhKeyPtr = *par.add(0) as *mut u8;
        }

        let lvKeyLength : i32;
        unsafe
        {
            lvKeyLength = *par.add(1) as i32;
        }
        // lvhKeyLen = lvKeyLength;

        let lvhSalt1Ptr : *mut u8;
        unsafe
        {
            lvhSalt1Ptr = *par.add(2) as *mut u8;
        }

        let lvSalt1Size : i32;
        unsafe
        {
            lvSalt1Size = *par.add(3) as i32;
        }
        // lvhSalt1Len = lvSalt1Size;

        let lvhSalt2Ptr : *mut u8;
        unsafe
        {
            lvhSalt2Ptr = *par.add(4) as *mut u8;
        }

        let lvSalt2Size : i32;
        unsafe
        {
            lvSalt2Size = *par.add(5) as i32;
        }
        // lvhSalt2Len = lvSalt2Size;

        let lvhDataPtr : *mut u8;
        unsafe
        {
            lvhDataPtr = *par.add(6) as *mut u8;
        }

        let lvDataSize : i32;
        unsafe
        {
            lvDataSize = *par.add(7) as i32;
        }
        // lvhDataLen = lvDataSize;

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(8) as i32;
        }
        
        CBVaultDataDecryptEventArgs
        {
            hKeyPtr: lvhKeyPtr,
            KeyLength: lvKeyLength,
            hSalt1Ptr: lvhSalt1Ptr,
            Salt1Size: lvSalt1Size,
            hSalt2Ptr: lvhSalt2Ptr,
            Salt2Size: lvSalt2Size,
            hDataPtr: lvhDataPtr,
            DataSize: lvDataSize,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(8)) = self.ResultCode as isize;
        }
    }

    pub fn key(&self) -> *mut u8
    {
        self.hKeyPtr
    }
    pub fn key_length(&self) -> i32
    {
        self.KeyLength
    }
    pub fn salt1(&self) -> *mut u8
    {
        self.hSalt1Ptr
    }
    pub fn salt1_size(&self) -> i32
    {
        self.Salt1Size
    }
    pub fn salt2(&self) -> *mut u8
    {
        self.hSalt2Ptr
    }
    pub fn salt2_size(&self) -> i32
    {
        self.Salt2Size
    }
    pub fn data(&self) -> *mut u8
    {
        self.hDataPtr
    }
    pub fn data_size(&self) -> i32
    {
        self.DataSize
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

pub trait CBVaultDataDecryptEvent
{
    fn on_data_decrypt(&self, sender : &CBVault, e : &mut CBVaultDataDecryptEventArgs);
}


// CBVaultDataEncryptEventArgs carries the parameters of the DataEncrypt event of CBVault
pub struct CBVaultDataEncryptEventArgs
{
    hKeyPtr : *mut u8,
    KeyLength : i32,
    hSalt1Ptr : *mut u8,
    Salt1Size : i32,
    hSalt2Ptr : *mut u8,
    Salt2Size : i32,
    hDataPtr : *mut u8,
    DataSize : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of DataEncryptEventArgs
impl CBVaultDataEncryptEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDataEncryptEventArgs
    {

        let lvhKeyPtr : *mut u8;
        unsafe
        {
            lvhKeyPtr = *par.add(0) as *mut u8;
        }

        let lvKeyLength : i32;
        unsafe
        {
            lvKeyLength = *par.add(1) as i32;
        }
        // lvhKeyLen = lvKeyLength;

        let lvhSalt1Ptr : *mut u8;
        unsafe
        {
            lvhSalt1Ptr = *par.add(2) as *mut u8;
        }

        let lvSalt1Size : i32;
        unsafe
        {
            lvSalt1Size = *par.add(3) as i32;
        }
        // lvhSalt1Len = lvSalt1Size;

        let lvhSalt2Ptr : *mut u8;
        unsafe
        {
            lvhSalt2Ptr = *par.add(4) as *mut u8;
        }

        let lvSalt2Size : i32;
        unsafe
        {
            lvSalt2Size = *par.add(5) as i32;
        }
        // lvhSalt2Len = lvSalt2Size;

        let lvhDataPtr : *mut u8;
        unsafe
        {
            lvhDataPtr = *par.add(6) as *mut u8;
        }

        let lvDataSize : i32;
        unsafe
        {
            lvDataSize = *par.add(7) as i32;
        }
        // lvhDataLen = lvDataSize;

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(8) as i32;
        }
        
        CBVaultDataEncryptEventArgs
        {
            hKeyPtr: lvhKeyPtr,
            KeyLength: lvKeyLength,
            hSalt1Ptr: lvhSalt1Ptr,
            Salt1Size: lvSalt1Size,
            hSalt2Ptr: lvhSalt2Ptr,
            Salt2Size: lvSalt2Size,
            hDataPtr: lvhDataPtr,
            DataSize: lvDataSize,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(8)) = self.ResultCode as isize;
        }
    }

    pub fn key(&self) -> *mut u8
    {
        self.hKeyPtr
    }
    pub fn key_length(&self) -> i32
    {
        self.KeyLength
    }
    pub fn salt1(&self) -> *mut u8
    {
        self.hSalt1Ptr
    }
    pub fn salt1_size(&self) -> i32
    {
        self.Salt1Size
    }
    pub fn salt2(&self) -> *mut u8
    {
        self.hSalt2Ptr
    }
    pub fn salt2_size(&self) -> i32
    {
        self.Salt2Size
    }
    pub fn data(&self) -> *mut u8
    {
        self.hDataPtr
    }
    pub fn data_size(&self) -> i32
    {
        self.DataSize
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

pub trait CBVaultDataEncryptEvent
{
    fn on_data_encrypt(&self, sender : &CBVault, e : &mut CBVaultDataEncryptEventArgs);
}


// CBVaultErrorEventArgs carries the parameters of the Error event of CBVault
pub struct CBVaultErrorEventArgs
{
    ErrorCode : i32,
    Description : String,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ErrorEventArgs
impl CBVaultErrorEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultErrorEventArgs
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
                lvDescription = CStr::from_ptr(lvhDescriptionPtr).to_str().expect("Valid UTF8 not received for the parameter 'Description' in the Error event of a CBVault instance").to_owned();
            }
        }

        CBVaultErrorEventArgs
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

pub trait CBVaultErrorEvent
{
    fn on_error(&self, sender : &CBVault, e : &mut CBVaultErrorEventArgs);
}


// CBVaultFileAfterCopyEventArgs carries the parameters of the FileAfterCopy event of CBVault
pub struct CBVaultFileAfterCopyEventArgs
{
    SourcePath : String,
    DestinationPath : String,
    Attributes : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of FileAfterCopyEventArgs
impl CBVaultFileAfterCopyEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultFileAfterCopyEventArgs
    {

        let lvhSourcePathPtr : *mut c_char;
        let lvSourcePath : String;
        unsafe
        {
            lvhSourcePathPtr = *par.add(0) as *mut c_char;
            if lvhSourcePathPtr == std::ptr::null_mut()
            {
                lvSourcePath = String::default();
            }
            else
            {
                lvSourcePath = CStr::from_ptr(lvhSourcePathPtr).to_str().expect("Valid UTF8 not received for the parameter 'SourcePath' in the FileAfterCopy event of a CBVault instance").to_owned();
            }
        }

        let lvhDestinationPathPtr : *mut c_char;
        let lvDestinationPath : String;
        unsafe
        {
            lvhDestinationPathPtr = *par.add(1) as *mut c_char;
            if lvhDestinationPathPtr == std::ptr::null_mut()
            {
                lvDestinationPath = String::default();
            }
            else
            {
                lvDestinationPath = CStr::from_ptr(lvhDestinationPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'DestinationPath' in the FileAfterCopy event of a CBVault instance").to_owned();
            }
        }

        let lvAttributes : i32;
        unsafe
        {
            lvAttributes = *par.add(2) as i32;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(3) as i32;
        }
        
        CBVaultFileAfterCopyEventArgs
        {
            SourcePath: lvSourcePath,
            DestinationPath: lvDestinationPath,
            Attributes: lvAttributes,
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

    pub fn source_path(&self) -> &String
    {
        &self.SourcePath
    }
    pub fn destination_path(&self) -> &String
    {
        &self.DestinationPath
    }
    pub fn attributes(&self) -> i32
    {
        self.Attributes
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

pub trait CBVaultFileAfterCopyEvent
{
    fn on_file_after_copy(&self, sender : &CBVault, e : &mut CBVaultFileAfterCopyEventArgs);
}


// CBVaultFileBeforeCopyEventArgs carries the parameters of the FileBeforeCopy event of CBVault
pub struct CBVaultFileBeforeCopyEventArgs
{
    SourcePath : String,
    DestinationPath : String,
    Attributes : i32,
    DestinationExists : bool,
    Skip : bool,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of FileBeforeCopyEventArgs
impl CBVaultFileBeforeCopyEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultFileBeforeCopyEventArgs
    {

        let lvhSourcePathPtr : *mut c_char;
        let lvSourcePath : String;
        unsafe
        {
            lvhSourcePathPtr = *par.add(0) as *mut c_char;
            if lvhSourcePathPtr == std::ptr::null_mut()
            {
                lvSourcePath = String::default();
            }
            else
            {
                lvSourcePath = CStr::from_ptr(lvhSourcePathPtr).to_str().expect("Valid UTF8 not received for the parameter 'SourcePath' in the FileBeforeCopy event of a CBVault instance").to_owned();
            }
        }

        let lvhDestinationPathPtr : *mut c_char;
        let lvDestinationPath : String;
        unsafe
        {
            lvhDestinationPathPtr = *par.add(1) as *mut c_char;
            if lvhDestinationPathPtr == std::ptr::null_mut()
            {
                lvDestinationPath = String::default();
            }
            else
            {
                lvDestinationPath = CStr::from_ptr(lvhDestinationPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'DestinationPath' in the FileBeforeCopy event of a CBVault instance").to_owned();
            }
        }

        let lvAttributes : i32;
        unsafe
        {
            lvAttributes = *par.add(2) as i32;
        }
        
        let lvDestinationExists : bool;
        unsafe
        {
            lvDestinationExists = (*par.add(3) as i32) != 0;
        }

        let lvSkip : bool;
        unsafe
        {
            lvSkip = (*par.add(4) as i32) != 0;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(5) as i32;
        }
        
        CBVaultFileBeforeCopyEventArgs
        {
            SourcePath: lvSourcePath,
            DestinationPath: lvDestinationPath,
            Attributes: lvAttributes,
            DestinationExists: lvDestinationExists,
            Skip: lvSkip,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(2)) = self.Attributes as isize;
            let intValOfSkip : i32;
            if self.Skip
            {
                intValOfSkip = 1;
            }
            else
            {
                intValOfSkip = 0;
            }
            *(self._Params.add(4)) = intValOfSkip as isize;
            *(self._Params.add(5)) = self.ResultCode as isize;
        }
    }

    pub fn source_path(&self) -> &String
    {
        &self.SourcePath
    }
    pub fn destination_path(&self) -> &String
    {
        &self.DestinationPath
    }
    pub fn attributes(&self) -> i32
    {
        self.Attributes
    }
    pub fn set_attributes(&mut self, value: i32)
    {
        self.Attributes = value;
    }
    pub fn destination_exists(&self) -> bool
    {
        self.DestinationExists
    }
    pub fn skip(&self) -> bool
    {
        self.Skip
    }
    pub fn set_skip(&mut self, value: bool)
    {
        self.Skip = value;
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

pub trait CBVaultFileBeforeCopyEvent
{
    fn on_file_before_copy(&self, sender : &CBVault, e : &mut CBVaultFileBeforeCopyEventArgs);
}


// CBVaultFilePasswordNeededEventArgs carries the parameters of the FilePasswordNeeded event of CBVault
pub struct CBVaultFilePasswordNeededEventArgs
{
    FileName : String,
    hPasswordPtr : *mut c_char,
    hPasswordLen : i32,
    Password : String,
    TTLInCache : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType,
    _Cbparam : IntArrayType
}

// Constructor and marshalRefParams() of FilePasswordNeededEventArgs
impl CBVaultFilePasswordNeededEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultFilePasswordNeededEventArgs
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
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the FilePasswordNeeded event of a CBVault instance").to_owned();
            }
        }

        let lvhPasswordPtr : *mut c_char;
        let lvhPasswordLen : i32;
        let lvPassword : String;
        unsafe
        {
            lvhPasswordPtr = *par.add(1) as *mut c_char;
            lvhPasswordLen = *_cbpar.add(1);
            if lvhPasswordPtr == std::ptr::null_mut()
            {
                lvPassword = String::default();
            }
            else
            {
                lvPassword = CStr::from_ptr(lvhPasswordPtr).to_str().expect("Valid UTF8 not received for the parameter 'Password' in the FilePasswordNeeded event of a CBVault instance").to_owned();
            }
        }

        let lvTTLInCache : i32;
        unsafe
        {
            lvTTLInCache = *par.add(2) as i32;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(3) as i32;
        }
        
        CBVaultFilePasswordNeededEventArgs
        {
            FileName: lvFileName,
            hPasswordPtr: lvhPasswordPtr,
            hPasswordLen: lvhPasswordLen,
            Password: lvPassword,
            TTLInCache: lvTTLInCache,
            ResultCode: lvResultCode,
            _Params: par,
            _Cbparam: _cbpar
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let bytesPassword = self.Password.as_bytes();
            let to_copy : usize;
            let bytesPasswordLen = bytesPassword.len();
            if bytesPasswordLen + 1 < self.hPasswordLen as usize
            {
                to_copy = bytesPasswordLen;
            }
            else
            {
                to_copy = self.hPasswordLen as usize - 1;
            }
            if to_copy > 0
            {
                std::ptr::copy_nonoverlapping(bytesPassword.as_ptr(), self.hPasswordPtr as *mut u8, to_copy);
            }
            *(self.hPasswordPtr.add(to_copy)) = 0;
            *(self._Cbparam.add(1)) = to_copy as i32;
            *(self._Params.add(2)) = self.TTLInCache as isize;
            *(self._Params.add(3)) = self.ResultCode as isize;
        }
    }

    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn password(&self) -> &String
    {
        &self.Password
    }
    pub fn set_password_ref(&mut self, value: &String)
    {
        self.Password = value.clone();
    }
    pub fn set_password(&mut self, value: String)
    {
        self.Password = value;
    }
    pub fn ttl_in_cache(&self) -> i32
    {
        self.TTLInCache
    }
    pub fn set_ttl_in_cache(&mut self, value: i32)
    {
        self.TTLInCache = value;
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

pub trait CBVaultFilePasswordNeededEvent
{
    fn on_file_password_needed(&self, sender : &CBVault, e : &mut CBVaultFilePasswordNeededEventArgs);
}


// CBVaultHashCalculateEventArgs carries the parameters of the HashCalculate event of CBVault
pub struct CBVaultHashCalculateEventArgs
{
    hPasswordPtr : *mut u8,
    PasswordSize : i32,
    hSaltPtr : *mut u8,
    SaltSize : i32,
    hHashPtr : *mut u8,
    HashSize : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of HashCalculateEventArgs
impl CBVaultHashCalculateEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultHashCalculateEventArgs
    {

        let lvhPasswordPtr : *mut u8;
        unsafe
        {
            lvhPasswordPtr = *par.add(0) as *mut u8;
        }

        let lvPasswordSize : i32;
        unsafe
        {
            lvPasswordSize = *par.add(1) as i32;
        }
        // lvhPasswordLen = lvPasswordSize;

        let lvhSaltPtr : *mut u8;
        unsafe
        {
            lvhSaltPtr = *par.add(2) as *mut u8;
        }

        let lvSaltSize : i32;
        unsafe
        {
            lvSaltSize = *par.add(3) as i32;
        }
        // lvhSaltLen = lvSaltSize;

        let lvhHashPtr : *mut u8;
        unsafe
        {
            lvhHashPtr = *par.add(4) as *mut u8;
        }

        let lvHashSize : i32;
        unsafe
        {
            lvHashSize = *par.add(5) as i32;
        }
        // lvhHashLen = lvHashSize;

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(6) as i32;
        }
        
        CBVaultHashCalculateEventArgs
        {
            hPasswordPtr: lvhPasswordPtr,
            PasswordSize: lvPasswordSize,
            hSaltPtr: lvhSaltPtr,
            SaltSize: lvSaltSize,
            hHashPtr: lvhHashPtr,
            HashSize: lvHashSize,
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

    pub fn password(&self) -> *mut u8
    {
        self.hPasswordPtr
    }
    pub fn password_size(&self) -> i32
    {
        self.PasswordSize
    }
    pub fn salt(&self) -> *mut u8
    {
        self.hSaltPtr
    }
    pub fn salt_size(&self) -> i32
    {
        self.SaltSize
    }
    pub fn hash(&self) -> *mut u8
    {
        self.hHashPtr
    }
    pub fn hash_size(&self) -> i32
    {
        self.HashSize
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

pub trait CBVaultHashCalculateEvent
{
    fn on_hash_calculate(&self, sender : &CBVault, e : &mut CBVaultHashCalculateEventArgs);
}


// CBVaultKeyDeriveEventArgs carries the parameters of the KeyDerive event of CBVault
pub struct CBVaultKeyDeriveEventArgs
{
    hPasswordPtr : *mut u8,
    PasswordSize : i32,
    hSaltPtr : *mut u8,
    SaltSize : i32,
    hKeyPtr : *mut u8,
    KeySize : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of KeyDeriveEventArgs
impl CBVaultKeyDeriveEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultKeyDeriveEventArgs
    {

        let lvhPasswordPtr : *mut u8;
        unsafe
        {
            lvhPasswordPtr = *par.add(0) as *mut u8;
        }

        let lvPasswordSize : i32;
        unsafe
        {
            lvPasswordSize = *par.add(1) as i32;
        }
        // lvhPasswordLen = lvPasswordSize;

        let lvhSaltPtr : *mut u8;
        unsafe
        {
            lvhSaltPtr = *par.add(2) as *mut u8;
        }

        let lvSaltSize : i32;
        unsafe
        {
            lvSaltSize = *par.add(3) as i32;
        }
        // lvhSaltLen = lvSaltSize;

        let lvhKeyPtr : *mut u8;
        unsafe
        {
            lvhKeyPtr = *par.add(4) as *mut u8;
        }

        let lvKeySize : i32;
        unsafe
        {
            lvKeySize = *par.add(5) as i32;
        }
        // lvhKeyLen = lvKeySize;

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(6) as i32;
        }
        
        CBVaultKeyDeriveEventArgs
        {
            hPasswordPtr: lvhPasswordPtr,
            PasswordSize: lvPasswordSize,
            hSaltPtr: lvhSaltPtr,
            SaltSize: lvSaltSize,
            hKeyPtr: lvhKeyPtr,
            KeySize: lvKeySize,
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

    pub fn password(&self) -> *mut u8
    {
        self.hPasswordPtr
    }
    pub fn password_size(&self) -> i32
    {
        self.PasswordSize
    }
    pub fn salt(&self) -> *mut u8
    {
        self.hSaltPtr
    }
    pub fn salt_size(&self) -> i32
    {
        self.SaltSize
    }
    pub fn key(&self) -> *mut u8
    {
        self.hKeyPtr
    }
    pub fn key_size(&self) -> i32
    {
        self.KeySize
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

pub trait CBVaultKeyDeriveEvent
{
    fn on_key_derive(&self, sender : &CBVault, e : &mut CBVaultKeyDeriveEventArgs);
}


// CBVaultProgressEventArgs carries the parameters of the Progress event of CBVault
pub struct CBVaultProgressEventArgs
{
    Operation : i32,
    FileName : String,
    Progress : i32,
    Total : i32,
    CanStop : bool,
    Stop : bool,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ProgressEventArgs
impl CBVaultProgressEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultProgressEventArgs
    {

        let lvOperation : i32;
        unsafe
        {
            lvOperation = *par.add(0) as i32;
        }
        
        let lvhFileNamePtr : *mut c_char;
        let lvFileName : String;
        unsafe
        {
            lvhFileNamePtr = *par.add(1) as *mut c_char;
            if lvhFileNamePtr == std::ptr::null_mut()
            {
                lvFileName = String::default();
            }
            else
            {
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the Progress event of a CBVault instance").to_owned();
            }
        }

        let lvProgress : i32;
        unsafe
        {
            lvProgress = *par.add(2) as i32;
        }
        
        let lvTotal : i32;
        unsafe
        {
            lvTotal = *par.add(3) as i32;
        }
        
        let lvCanStop : bool;
        unsafe
        {
            lvCanStop = (*par.add(4) as i32) != 0;
        }

        let lvStop : bool;
        unsafe
        {
            lvStop = (*par.add(5) as i32) != 0;
        }

        CBVaultProgressEventArgs
        {
            Operation: lvOperation,
            FileName: lvFileName,
            Progress: lvProgress,
            Total: lvTotal,
            CanStop: lvCanStop,
            Stop: lvStop,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let intValOfStop : i32;
            if self.Stop
            {
                intValOfStop = 1;
            }
            else
            {
                intValOfStop = 0;
            }
            *(self._Params.add(5)) = intValOfStop as isize;
        }
    }

    pub fn operation(&self) -> i32
    {
        self.Operation
    }
    pub fn file_name(&self) -> &String
    {
        &self.FileName
    }
    pub fn progress(&self) -> i32
    {
        self.Progress
    }
    pub fn total(&self) -> i32
    {
        self.Total
    }
    pub fn can_stop(&self) -> bool
    {
        self.CanStop
    }
    pub fn stop(&self) -> bool
    {
        self.Stop
    }
    pub fn set_stop(&mut self, value: bool)
    {
        self.Stop = value;
    }
}

pub trait CBVaultProgressEvent
{
    fn on_progress(&self, sender : &CBVault, e : &mut CBVaultProgressEventArgs);
}


// CBVaultVaultCloseEventArgs carries the parameters of the VaultClose event of CBVault
pub struct CBVaultVaultCloseEventArgs
{
    VaultHandle : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultCloseEventArgs
impl CBVaultVaultCloseEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultVaultCloseEventArgs
    {

        let lvVaultHandle : i64;
        unsafe
        {
            let lvVaultHandleLPtr : *mut i64 = *par.add(0) as *mut i64;
            lvVaultHandle = *lvVaultHandleLPtr;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(1) as i32;
        }
        
        CBVaultVaultCloseEventArgs
        {
            VaultHandle: lvVaultHandle,
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

    pub fn vault_handle(&self) -> i64
    {
        self.VaultHandle
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

pub trait CBVaultVaultCloseEvent
{
    fn on_vault_close(&self, sender : &CBVault, e : &mut CBVaultVaultCloseEventArgs);
}


// CBVaultVaultDeleteEventArgs carries the parameters of the VaultDelete event of CBVault
pub struct CBVaultVaultDeleteEventArgs
{
    Vault : String,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultDeleteEventArgs
impl CBVaultVaultDeleteEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultVaultDeleteEventArgs
    {

        let lvhVaultPtr : *mut c_char;
        let lvVault : String;
        unsafe
        {
            lvhVaultPtr = *par.add(0) as *mut c_char;
            if lvhVaultPtr == std::ptr::null_mut()
            {
                lvVault = String::default();
            }
            else
            {
                lvVault = CStr::from_ptr(lvhVaultPtr).to_str().expect("Valid UTF8 not received for the parameter 'Vault' in the VaultDelete event of a CBVault instance").to_owned();
            }
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(1) as i32;
        }
        
        CBVaultVaultDeleteEventArgs
        {
            Vault: lvVault,
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

    pub fn vault(&self) -> &String
    {
        &self.Vault
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

pub trait CBVaultVaultDeleteEvent
{
    fn on_vault_delete(&self, sender : &CBVault, e : &mut CBVaultVaultDeleteEventArgs);
}


// CBVaultVaultFlushEventArgs carries the parameters of the VaultFlush event of CBVault
pub struct CBVaultVaultFlushEventArgs
{
    VaultHandle : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultFlushEventArgs
impl CBVaultVaultFlushEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultVaultFlushEventArgs
    {

        let lvVaultHandle : i64;
        unsafe
        {
            let lvVaultHandleLPtr : *mut i64 = *par.add(0) as *mut i64;
            lvVaultHandle = *lvVaultHandleLPtr;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(1) as i32;
        }
        
        CBVaultVaultFlushEventArgs
        {
            VaultHandle: lvVaultHandle,
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

    pub fn vault_handle(&self) -> i64
    {
        self.VaultHandle
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

pub trait CBVaultVaultFlushEvent
{
    fn on_vault_flush(&self, sender : &CBVault, e : &mut CBVaultVaultFlushEventArgs);
}


// CBVaultVaultGetParentSizeEventArgs carries the parameters of the VaultGetParentSize event of CBVault
pub struct CBVaultVaultGetParentSizeEventArgs
{
    Vault : String,
    VaultHandle : i64,
    FreeSpace : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultGetParentSizeEventArgs
impl CBVaultVaultGetParentSizeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultVaultGetParentSizeEventArgs
    {

        let lvhVaultPtr : *mut c_char;
        let lvVault : String;
        unsafe
        {
            lvhVaultPtr = *par.add(0) as *mut c_char;
            if lvhVaultPtr == std::ptr::null_mut()
            {
                lvVault = String::default();
            }
            else
            {
                lvVault = CStr::from_ptr(lvhVaultPtr).to_str().expect("Valid UTF8 not received for the parameter 'Vault' in the VaultGetParentSize event of a CBVault instance").to_owned();
            }
        }

        let lvVaultHandle : i64;
        unsafe
        {
            let lvVaultHandleLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvVaultHandle = *lvVaultHandleLPtr;
        }
        
        let lvFreeSpace : i64;
        unsafe
        {
            let lvFreeSpaceLPtr : *mut i64 = *par.add(2) as *mut i64;
            lvFreeSpace = *lvFreeSpaceLPtr;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(3) as i32;
        }
        
        CBVaultVaultGetParentSizeEventArgs
        {
            Vault: lvVault,
            VaultHandle: lvVaultHandle,
            FreeSpace: lvFreeSpace,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvFreeSpaceLPtr : *mut i64 = *self._Params.add(2) as *mut i64;
            *lvFreeSpaceLPtr = self.FreeSpace ;
            *(self._Params.add(3)) = self.ResultCode as isize;
        }
    }

    pub fn vault(&self) -> &String
    {
        &self.Vault
    }
    pub fn vault_handle(&self) -> i64
    {
        self.VaultHandle
    }
    pub fn free_space(&self) -> i64
    {
        self.FreeSpace
    }
    pub fn set_free_space(&mut self, value: i64)
    {
        self.FreeSpace = value;
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

pub trait CBVaultVaultGetParentSizeEvent
{
    fn on_vault_get_parent_size(&self, sender : &CBVault, e : &mut CBVaultVaultGetParentSizeEventArgs);
}


// CBVaultVaultGetSizeEventArgs carries the parameters of the VaultGetSize event of CBVault
pub struct CBVaultVaultGetSizeEventArgs
{
    VaultHandle : i64,
    Size : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultGetSizeEventArgs
impl CBVaultVaultGetSizeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultVaultGetSizeEventArgs
    {

        let lvVaultHandle : i64;
        unsafe
        {
            let lvVaultHandleLPtr : *mut i64 = *par.add(0) as *mut i64;
            lvVaultHandle = *lvVaultHandleLPtr;
        }
        
        let lvSize : i64;
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvSize = *lvSizeLPtr;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(2) as i32;
        }
        
        CBVaultVaultGetSizeEventArgs
        {
            VaultHandle: lvVaultHandle,
            Size: lvSize,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvSizeLPtr : *mut i64 = *self._Params.add(1) as *mut i64;
            *lvSizeLPtr = self.Size ;
            *(self._Params.add(2)) = self.ResultCode as isize;
        }
    }

    pub fn vault_handle(&self) -> i64
    {
        self.VaultHandle
    }
    pub fn size(&self) -> i64
    {
        self.Size
    }
    pub fn set_size(&mut self, value: i64)
    {
        self.Size = value;
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

pub trait CBVaultVaultGetSizeEvent
{
    fn on_vault_get_size(&self, sender : &CBVault, e : &mut CBVaultVaultGetSizeEventArgs);
}


// CBVaultVaultOpenEventArgs carries the parameters of the VaultOpen event of CBVault
pub struct CBVaultVaultOpenEventArgs
{
    Vault : String,
    VaultHandle : i64,
    OpenMode : i32,
    ReadOnly : bool,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultOpenEventArgs
impl CBVaultVaultOpenEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultVaultOpenEventArgs
    {

        let lvhVaultPtr : *mut c_char;
        let lvVault : String;
        unsafe
        {
            lvhVaultPtr = *par.add(0) as *mut c_char;
            if lvhVaultPtr == std::ptr::null_mut()
            {
                lvVault = String::default();
            }
            else
            {
                lvVault = CStr::from_ptr(lvhVaultPtr).to_str().expect("Valid UTF8 not received for the parameter 'Vault' in the VaultOpen event of a CBVault instance").to_owned();
            }
        }

        let lvVaultHandle : i64;
        unsafe
        {
            let lvVaultHandleLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvVaultHandle = *lvVaultHandleLPtr;
        }
        
        let lvOpenMode : i32;
        unsafe
        {
            lvOpenMode = *par.add(2) as i32;
        }
        
        let lvReadOnly : bool;
        unsafe
        {
            lvReadOnly = (*par.add(3) as i32) != 0;
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(4) as i32;
        }
        
        CBVaultVaultOpenEventArgs
        {
            Vault: lvVault,
            VaultHandle: lvVaultHandle,
            OpenMode: lvOpenMode,
            ReadOnly: lvReadOnly,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            let lvVaultHandleLPtr : *mut i64 = *self._Params.add(1) as *mut i64;
            *lvVaultHandleLPtr = self.VaultHandle ;
            let intValOfReadOnly : i32;
            if self.ReadOnly
            {
                intValOfReadOnly = 1;
            }
            else
            {
                intValOfReadOnly = 0;
            }
            *(self._Params.add(3)) = intValOfReadOnly as isize;
            *(self._Params.add(4)) = self.ResultCode as isize;
        }
    }

    pub fn vault(&self) -> &String
    {
        &self.Vault
    }
    pub fn vault_handle(&self) -> i64
    {
        self.VaultHandle
    }
    pub fn set_vault_handle(&mut self, value: i64)
    {
        self.VaultHandle = value;
    }
    pub fn open_mode(&self) -> i32
    {
        self.OpenMode
    }
    pub fn read_only(&self) -> bool
    {
        self.ReadOnly
    }
    pub fn set_read_only(&mut self, value: bool)
    {
        self.ReadOnly = value;
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

pub trait CBVaultVaultOpenEvent
{
    fn on_vault_open(&self, sender : &CBVault, e : &mut CBVaultVaultOpenEventArgs);
}


// CBVaultVaultReadEventArgs carries the parameters of the VaultRead event of CBVault
pub struct CBVaultVaultReadEventArgs
{
    VaultHandle : i64,
    Offset : i64,
    hBufferPtr : *mut u8,
    Count : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultReadEventArgs
impl CBVaultVaultReadEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultVaultReadEventArgs
    {

        let lvVaultHandle : i64;
        unsafe
        {
            let lvVaultHandleLPtr : *mut i64 = *par.add(0) as *mut i64;
            lvVaultHandle = *lvVaultHandleLPtr;
        }
        
        let lvOffset : i64;
        unsafe
        {
            let lvOffsetLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvOffset = *lvOffsetLPtr;
        }
        
        let lvhBufferPtr : *mut u8;
        unsafe
        {
            lvhBufferPtr = *par.add(2) as *mut u8;
        }

        let lvCount : i32;
        unsafe
        {
            lvCount = *par.add(3) as i32;
        }
        // lvhBufferLen = lvCount;

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(4) as i32;
        }
        
        CBVaultVaultReadEventArgs
        {
            VaultHandle: lvVaultHandle,
            Offset: lvOffset,
            hBufferPtr: lvhBufferPtr,
            Count: lvCount,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(4)) = self.ResultCode as isize;
        }
    }

    pub fn vault_handle(&self) -> i64
    {
        self.VaultHandle
    }
    pub fn offset(&self) -> i64
    {
        self.Offset
    }
    pub fn buffer(&self) -> *mut u8
    {
        self.hBufferPtr
    }
    pub fn count(&self) -> i32
    {
        self.Count
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

pub trait CBVaultVaultReadEvent
{
    fn on_vault_read(&self, sender : &CBVault, e : &mut CBVaultVaultReadEventArgs);
}


// CBVaultVaultSetSizeEventArgs carries the parameters of the VaultSetSize event of CBVault
pub struct CBVaultVaultSetSizeEventArgs
{
    VaultHandle : i64,
    NewSize : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultSetSizeEventArgs
impl CBVaultVaultSetSizeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultVaultSetSizeEventArgs
    {

        let lvVaultHandle : i64;
        unsafe
        {
            let lvVaultHandleLPtr : *mut i64 = *par.add(0) as *mut i64;
            lvVaultHandle = *lvVaultHandleLPtr;
        }
        
        let lvNewSize : i64;
        unsafe
        {
            let lvNewSizeLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvNewSize = *lvNewSizeLPtr;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(2) as i32;
        }
        
        CBVaultVaultSetSizeEventArgs
        {
            VaultHandle: lvVaultHandle,
            NewSize: lvNewSize,
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

    pub fn vault_handle(&self) -> i64
    {
        self.VaultHandle
    }
    pub fn new_size(&self) -> i64
    {
        self.NewSize
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

pub trait CBVaultVaultSetSizeEvent
{
    fn on_vault_set_size(&self, sender : &CBVault, e : &mut CBVaultVaultSetSizeEventArgs);
}


// CBVaultVaultWriteEventArgs carries the parameters of the VaultWrite event of CBVault
pub struct CBVaultVaultWriteEventArgs
{
    VaultHandle : i64,
    Offset : i64,
    hBufferPtr : *mut u8,
    Count : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultWriteEventArgs
impl CBVaultVaultWriteEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultVaultWriteEventArgs
    {

        let lvVaultHandle : i64;
        unsafe
        {
            let lvVaultHandleLPtr : *mut i64 = *par.add(0) as *mut i64;
            lvVaultHandle = *lvVaultHandleLPtr;
        }
        
        let lvOffset : i64;
        unsafe
        {
            let lvOffsetLPtr : *mut i64 = *par.add(1) as *mut i64;
            lvOffset = *lvOffsetLPtr;
        }
        
        let lvhBufferPtr : *mut u8;
        unsafe
        {
            lvhBufferPtr = *par.add(2) as *mut u8;
        }

        let lvCount : i32;
        unsafe
        {
            lvCount = *par.add(3) as i32;
        }
        // lvhBufferLen = lvCount;

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(4) as i32;
        }
        
        CBVaultVaultWriteEventArgs
        {
            VaultHandle: lvVaultHandle,
            Offset: lvOffset,
            hBufferPtr: lvhBufferPtr,
            Count: lvCount,
            ResultCode: lvResultCode,
            _Params: par
        }
    }

    fn marshalRefParams(&self) 
    {
        unsafe
        {
            *(self._Params.add(4)) = self.ResultCode as isize;
        }
    }

    pub fn vault_handle(&self) -> i64
    {
        self.VaultHandle
    }
    pub fn offset(&self) -> i64
    {
        self.Offset
    }
    pub fn buffer(&self) -> *mut u8
    {
        self.hBufferPtr
    }
    pub fn count(&self) -> i32
    {
        self.Count
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

pub trait CBVaultVaultWriteEvent
{
    fn on_vault_write(&self, sender : &CBVault, e : &mut CBVaultVaultWriteEventArgs);
}


////////////////////////////
// Main Class Implementation
////////////////////////////

/* The CBVault component lets applications create a vault and manipulate its contents. */
//pub struct CBVault<'a>
pub struct CBVault
{

    // onDataCompress : Option<&'a dyn CBVaultDataCompressEvent>,
    onDataCompress : Option<fn (sender : &CBVault, e : &mut CBVaultDataCompressEventArgs) >,
    // onDataDecompress : Option<&'a dyn CBVaultDataDecompressEvent>,
    onDataDecompress : Option<fn (sender : &CBVault, e : &mut CBVaultDataDecompressEventArgs) >,
    // onDataDecrypt : Option<&'a dyn CBVaultDataDecryptEvent>,
    onDataDecrypt : Option<fn (sender : &CBVault, e : &mut CBVaultDataDecryptEventArgs) >,
    // onDataEncrypt : Option<&'a dyn CBVaultDataEncryptEvent>,
    onDataEncrypt : Option<fn (sender : &CBVault, e : &mut CBVaultDataEncryptEventArgs) >,
    // onError : Option<&'a dyn CBVaultErrorEvent>,
    onError : Option<fn (sender : &CBVault, e : &mut CBVaultErrorEventArgs) >,
    // onFileAfterCopy : Option<&'a dyn CBVaultFileAfterCopyEvent>,
    onFileAfterCopy : Option<fn (sender : &CBVault, e : &mut CBVaultFileAfterCopyEventArgs) >,
    // onFileBeforeCopy : Option<&'a dyn CBVaultFileBeforeCopyEvent>,
    onFileBeforeCopy : Option<fn (sender : &CBVault, e : &mut CBVaultFileBeforeCopyEventArgs) >,
    // onFilePasswordNeeded : Option<&'a dyn CBVaultFilePasswordNeededEvent>,
    onFilePasswordNeeded : Option<fn (sender : &CBVault, e : &mut CBVaultFilePasswordNeededEventArgs) >,
    // onHashCalculate : Option<&'a dyn CBVaultHashCalculateEvent>,
    onHashCalculate : Option<fn (sender : &CBVault, e : &mut CBVaultHashCalculateEventArgs) >,
    // onKeyDerive : Option<&'a dyn CBVaultKeyDeriveEvent>,
    onKeyDerive : Option<fn (sender : &CBVault, e : &mut CBVaultKeyDeriveEventArgs) >,
    // onProgress : Option<&'a dyn CBVaultProgressEvent>,
    onProgress : Option<fn (sender : &CBVault, e : &mut CBVaultProgressEventArgs) >,
    // onVaultClose : Option<&'a dyn CBVaultVaultCloseEvent>,
    onVaultClose : Option<fn (sender : &CBVault, e : &mut CBVaultVaultCloseEventArgs) >,
    // onVaultDelete : Option<&'a dyn CBVaultVaultDeleteEvent>,
    onVaultDelete : Option<fn (sender : &CBVault, e : &mut CBVaultVaultDeleteEventArgs) >,
    // onVaultFlush : Option<&'a dyn CBVaultVaultFlushEvent>,
    onVaultFlush : Option<fn (sender : &CBVault, e : &mut CBVaultVaultFlushEventArgs) >,
    // onVaultGetParentSize : Option<&'a dyn CBVaultVaultGetParentSizeEvent>,
    onVaultGetParentSize : Option<fn (sender : &CBVault, e : &mut CBVaultVaultGetParentSizeEventArgs) >,
    // onVaultGetSize : Option<&'a dyn CBVaultVaultGetSizeEvent>,
    onVaultGetSize : Option<fn (sender : &CBVault, e : &mut CBVaultVaultGetSizeEventArgs) >,
    // onVaultOpen : Option<&'a dyn CBVaultVaultOpenEvent>,
    onVaultOpen : Option<fn (sender : &CBVault, e : &mut CBVaultVaultOpenEventArgs) >,
    // onVaultRead : Option<&'a dyn CBVaultVaultReadEvent>,
    onVaultRead : Option<fn (sender : &CBVault, e : &mut CBVaultVaultReadEventArgs) >,
    // onVaultSetSize : Option<&'a dyn CBVaultVaultSetSizeEvent>,
    onVaultSetSize : Option<fn (sender : &CBVault, e : &mut CBVaultVaultSetSizeEventArgs) >,
    // onVaultWrite : Option<&'a dyn CBVaultVaultWriteEvent>,
    onVaultWrite : Option<fn (sender : &CBVault, e : &mut CBVaultVaultWriteEventArgs) >,

    Id : usize,
    Handle : usize 
}

//impl<'a> Drop for CBVault<'a>
impl Drop for CBVault
{
    fn drop(&mut self)
    {
        self.dispose();
    }
}

impl CBVault
{
    pub fn new() -> &'static mut CBVault
    {        
        if !crate::is_lib_loaded()
        {
            crate::init_on_demand();
        }
        
        let ret_code : i32;
        let lId : usize = CBVaultIDSeed.fetch_add(1, SeqCst) as usize;

        let lHandle : isize;
        unsafe
        {
            let callable = CBFSVault_CBVault_Create.get().unwrap();
            lHandle = callable(CBVaultEventDispatcher, lId, std::ptr::null(), CBVaultCreateOpt) as isize;
        }
        if lHandle < 0
        {
            panic!("Failed to instantiate CBVault. Please verify that it is supported on this platform");
        }

        let result : CBVault = CBVault
        {
            onDataCompress: None,
            onDataDecompress: None,
            onDataDecrypt: None,
            onDataEncrypt: None,
            onError: None,
            onFileAfterCopy: None,
            onFileBeforeCopy: None,
            onFilePasswordNeeded: None,
            onHashCalculate: None,
            onKeyDerive: None,
            onProgress: None,
            onVaultClose: None,
            onVaultDelete: None,
            onVaultFlush: None,
            onVaultGetParentSize: None,
            onVaultGetSize: None,
            onVaultOpen: None,
            onVaultRead: None,
            onVaultSetSize: None,
            onVaultWrite: None,
            Id: lId,
            Handle: lHandle as usize
        };

        let oem_key = CString::new(cbfsvaultkey::rtkCBFSVault).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();

        unsafe
        {
            let callable = CBFSVault_CBVault_SetCStr.get().unwrap();
            ret_code = callable(lHandle as usize, 8012/*PID_KEYCHECK_RUST*/, 0, oem_key_ptr, 0) as i32;
            let _ = CString::from_raw(oem_key_ptr);
        }
        if ret_code != 0
        {
            panic!("Initialization of CBVault has failed with error {}", ret_code);
        }

        // Lock the Mutex to get access to the HashMap
        let result_ptr = Box::into_raw(Box::new(result));
        let result_addr = result_ptr as usize;

        // Lock the Mutex to get access to the HashMap
        {
            let mut map = CBVaultDict.lock().unwrap();
            map.insert(lId, result_addr);
        }
        return unsafe { &mut *result_ptr };
    }

    pub fn dispose(&self)
    {
        unsafe
        {
             let mut map = CBVaultDict.lock().unwrap();
            
            if !map.contains_key(&self.Id)
            {
                return;
            }

            // Remove reference from the list
            if let Some(addr) = map.remove(&self.Id) {
                // finalize the ctlclass
                let callable = CBFSVault_CBVault_Destroy.get().unwrap();
                callable(self.Handle);
                
                // Convert back to Box to drop
                let _boxed = Box::from_raw(addr as *mut CBVault);
            }
        }
    }

/////////
// Events
/////////

    fn fire_data_compress(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDataCompress
        {
            let mut args : CBVaultDataCompressEventArgs = CBVaultDataCompressEventArgs::new(par, cbpar);
            callable/*.on_data_compress*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_data_compress(&self) -> &'a dyn CBVaultDataCompressEvent
    pub fn on_data_compress(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultDataCompressEventArgs)>
    {
        self.onDataCompress
    }

    //pub fn set_on_data_compress(&mut self, value : &'a dyn CBVaultDataCompressEvent)
    pub fn set_on_data_compress(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultDataCompressEventArgs)>)
    {
        self.onDataCompress = value;
    }

    fn fire_data_decompress(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDataDecompress
        {
            let mut args : CBVaultDataDecompressEventArgs = CBVaultDataDecompressEventArgs::new(par, cbpar);
            callable/*.on_data_decompress*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_data_decompress(&self) -> &'a dyn CBVaultDataDecompressEvent
    pub fn on_data_decompress(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultDataDecompressEventArgs)>
    {
        self.onDataDecompress
    }

    //pub fn set_on_data_decompress(&mut self, value : &'a dyn CBVaultDataDecompressEvent)
    pub fn set_on_data_decompress(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultDataDecompressEventArgs)>)
    {
        self.onDataDecompress = value;
    }

    fn fire_data_decrypt(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDataDecrypt
        {
            let mut args : CBVaultDataDecryptEventArgs = CBVaultDataDecryptEventArgs::new(par, cbpar);
            callable/*.on_data_decrypt*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_data_decrypt(&self) -> &'a dyn CBVaultDataDecryptEvent
    pub fn on_data_decrypt(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultDataDecryptEventArgs)>
    {
        self.onDataDecrypt
    }

    //pub fn set_on_data_decrypt(&mut self, value : &'a dyn CBVaultDataDecryptEvent)
    pub fn set_on_data_decrypt(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultDataDecryptEventArgs)>)
    {
        self.onDataDecrypt = value;
    }

    fn fire_data_encrypt(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDataEncrypt
        {
            let mut args : CBVaultDataEncryptEventArgs = CBVaultDataEncryptEventArgs::new(par, cbpar);
            callable/*.on_data_encrypt*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_data_encrypt(&self) -> &'a dyn CBVaultDataEncryptEvent
    pub fn on_data_encrypt(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultDataEncryptEventArgs)>
    {
        self.onDataEncrypt
    }

    //pub fn set_on_data_encrypt(&mut self, value : &'a dyn CBVaultDataEncryptEvent)
    pub fn set_on_data_encrypt(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultDataEncryptEventArgs)>)
    {
        self.onDataEncrypt = value;
    }

    fn fire_error(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onError
        {
            let mut args : CBVaultErrorEventArgs = CBVaultErrorEventArgs::new(par, cbpar);
            callable/*.on_error*/(&self, &mut args);
        }
    }

    //pub fn on_error(&self) -> &'a dyn CBVaultErrorEvent
    pub fn on_error(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultErrorEventArgs)>
    {
        self.onError
    }

    //pub fn set_on_error(&mut self, value : &'a dyn CBVaultErrorEvent)
    pub fn set_on_error(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultErrorEventArgs)>)
    {
        self.onError = value;
    }

    fn fire_file_after_copy(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFileAfterCopy
        {
            let mut args : CBVaultFileAfterCopyEventArgs = CBVaultFileAfterCopyEventArgs::new(par, cbpar);
            callable/*.on_file_after_copy*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_file_after_copy(&self) -> &'a dyn CBVaultFileAfterCopyEvent
    pub fn on_file_after_copy(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultFileAfterCopyEventArgs)>
    {
        self.onFileAfterCopy
    }

    //pub fn set_on_file_after_copy(&mut self, value : &'a dyn CBVaultFileAfterCopyEvent)
    pub fn set_on_file_after_copy(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultFileAfterCopyEventArgs)>)
    {
        self.onFileAfterCopy = value;
    }

    fn fire_file_before_copy(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFileBeforeCopy
        {
            let mut args : CBVaultFileBeforeCopyEventArgs = CBVaultFileBeforeCopyEventArgs::new(par, cbpar);
            callable/*.on_file_before_copy*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_file_before_copy(&self) -> &'a dyn CBVaultFileBeforeCopyEvent
    pub fn on_file_before_copy(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultFileBeforeCopyEventArgs)>
    {
        self.onFileBeforeCopy
    }

    //pub fn set_on_file_before_copy(&mut self, value : &'a dyn CBVaultFileBeforeCopyEvent)
    pub fn set_on_file_before_copy(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultFileBeforeCopyEventArgs)>)
    {
        self.onFileBeforeCopy = value;
    }

    fn fire_file_password_needed(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFilePasswordNeeded
        {
            let mut args : CBVaultFilePasswordNeededEventArgs = CBVaultFilePasswordNeededEventArgs::new(par, cbpar);
            callable/*.on_file_password_needed*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_file_password_needed(&self) -> &'a dyn CBVaultFilePasswordNeededEvent
    pub fn on_file_password_needed(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultFilePasswordNeededEventArgs)>
    {
        self.onFilePasswordNeeded
    }

    //pub fn set_on_file_password_needed(&mut self, value : &'a dyn CBVaultFilePasswordNeededEvent)
    pub fn set_on_file_password_needed(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultFilePasswordNeededEventArgs)>)
    {
        self.onFilePasswordNeeded = value;
    }

    fn fire_hash_calculate(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onHashCalculate
        {
            let mut args : CBVaultHashCalculateEventArgs = CBVaultHashCalculateEventArgs::new(par, cbpar);
            callable/*.on_hash_calculate*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_hash_calculate(&self) -> &'a dyn CBVaultHashCalculateEvent
    pub fn on_hash_calculate(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultHashCalculateEventArgs)>
    {
        self.onHashCalculate
    }

    //pub fn set_on_hash_calculate(&mut self, value : &'a dyn CBVaultHashCalculateEvent)
    pub fn set_on_hash_calculate(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultHashCalculateEventArgs)>)
    {
        self.onHashCalculate = value;
    }

    fn fire_key_derive(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onKeyDerive
        {
            let mut args : CBVaultKeyDeriveEventArgs = CBVaultKeyDeriveEventArgs::new(par, cbpar);
            callable/*.on_key_derive*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_key_derive(&self) -> &'a dyn CBVaultKeyDeriveEvent
    pub fn on_key_derive(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultKeyDeriveEventArgs)>
    {
        self.onKeyDerive
    }

    //pub fn set_on_key_derive(&mut self, value : &'a dyn CBVaultKeyDeriveEvent)
    pub fn set_on_key_derive(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultKeyDeriveEventArgs)>)
    {
        self.onKeyDerive = value;
    }

    fn fire_progress(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onProgress
        {
            let mut args : CBVaultProgressEventArgs = CBVaultProgressEventArgs::new(par, cbpar);
            callable/*.on_progress*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_progress(&self) -> &'a dyn CBVaultProgressEvent
    pub fn on_progress(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultProgressEventArgs)>
    {
        self.onProgress
    }

    //pub fn set_on_progress(&mut self, value : &'a dyn CBVaultProgressEvent)
    pub fn set_on_progress(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultProgressEventArgs)>)
    {
        self.onProgress = value;
    }

    fn fire_vault_close(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultClose
        {
            let mut args : CBVaultVaultCloseEventArgs = CBVaultVaultCloseEventArgs::new(par, cbpar);
            callable/*.on_vault_close*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_close(&self) -> &'a dyn CBVaultVaultCloseEvent
    pub fn on_vault_close(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultVaultCloseEventArgs)>
    {
        self.onVaultClose
    }

    //pub fn set_on_vault_close(&mut self, value : &'a dyn CBVaultVaultCloseEvent)
    pub fn set_on_vault_close(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultVaultCloseEventArgs)>)
    {
        self.onVaultClose = value;
    }

    fn fire_vault_delete(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultDelete
        {
            let mut args : CBVaultVaultDeleteEventArgs = CBVaultVaultDeleteEventArgs::new(par, cbpar);
            callable/*.on_vault_delete*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_delete(&self) -> &'a dyn CBVaultVaultDeleteEvent
    pub fn on_vault_delete(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultVaultDeleteEventArgs)>
    {
        self.onVaultDelete
    }

    //pub fn set_on_vault_delete(&mut self, value : &'a dyn CBVaultVaultDeleteEvent)
    pub fn set_on_vault_delete(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultVaultDeleteEventArgs)>)
    {
        self.onVaultDelete = value;
    }

    fn fire_vault_flush(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultFlush
        {
            let mut args : CBVaultVaultFlushEventArgs = CBVaultVaultFlushEventArgs::new(par, cbpar);
            callable/*.on_vault_flush*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_flush(&self) -> &'a dyn CBVaultVaultFlushEvent
    pub fn on_vault_flush(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultVaultFlushEventArgs)>
    {
        self.onVaultFlush
    }

    //pub fn set_on_vault_flush(&mut self, value : &'a dyn CBVaultVaultFlushEvent)
    pub fn set_on_vault_flush(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultVaultFlushEventArgs)>)
    {
        self.onVaultFlush = value;
    }

    fn fire_vault_get_parent_size(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultGetParentSize
        {
            let mut args : CBVaultVaultGetParentSizeEventArgs = CBVaultVaultGetParentSizeEventArgs::new(par, cbpar);
            callable/*.on_vault_get_parent_size*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_get_parent_size(&self) -> &'a dyn CBVaultVaultGetParentSizeEvent
    pub fn on_vault_get_parent_size(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultVaultGetParentSizeEventArgs)>
    {
        self.onVaultGetParentSize
    }

    //pub fn set_on_vault_get_parent_size(&mut self, value : &'a dyn CBVaultVaultGetParentSizeEvent)
    pub fn set_on_vault_get_parent_size(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultVaultGetParentSizeEventArgs)>)
    {
        self.onVaultGetParentSize = value;
    }

    fn fire_vault_get_size(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultGetSize
        {
            let mut args : CBVaultVaultGetSizeEventArgs = CBVaultVaultGetSizeEventArgs::new(par, cbpar);
            callable/*.on_vault_get_size*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_get_size(&self) -> &'a dyn CBVaultVaultGetSizeEvent
    pub fn on_vault_get_size(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultVaultGetSizeEventArgs)>
    {
        self.onVaultGetSize
    }

    //pub fn set_on_vault_get_size(&mut self, value : &'a dyn CBVaultVaultGetSizeEvent)
    pub fn set_on_vault_get_size(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultVaultGetSizeEventArgs)>)
    {
        self.onVaultGetSize = value;
    }

    fn fire_vault_open(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultOpen
        {
            let mut args : CBVaultVaultOpenEventArgs = CBVaultVaultOpenEventArgs::new(par, cbpar);
            callable/*.on_vault_open*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_open(&self) -> &'a dyn CBVaultVaultOpenEvent
    pub fn on_vault_open(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultVaultOpenEventArgs)>
    {
        self.onVaultOpen
    }

    //pub fn set_on_vault_open(&mut self, value : &'a dyn CBVaultVaultOpenEvent)
    pub fn set_on_vault_open(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultVaultOpenEventArgs)>)
    {
        self.onVaultOpen = value;
    }

    fn fire_vault_read(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultRead
        {
            let mut args : CBVaultVaultReadEventArgs = CBVaultVaultReadEventArgs::new(par, cbpar);
            callable/*.on_vault_read*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_read(&self) -> &'a dyn CBVaultVaultReadEvent
    pub fn on_vault_read(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultVaultReadEventArgs)>
    {
        self.onVaultRead
    }

    //pub fn set_on_vault_read(&mut self, value : &'a dyn CBVaultVaultReadEvent)
    pub fn set_on_vault_read(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultVaultReadEventArgs)>)
    {
        self.onVaultRead = value;
    }

    fn fire_vault_set_size(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultSetSize
        {
            let mut args : CBVaultVaultSetSizeEventArgs = CBVaultVaultSetSizeEventArgs::new(par, cbpar);
            callable/*.on_vault_set_size*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_set_size(&self) -> &'a dyn CBVaultVaultSetSizeEvent
    pub fn on_vault_set_size(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultVaultSetSizeEventArgs)>
    {
        self.onVaultSetSize
    }

    //pub fn set_on_vault_set_size(&mut self, value : &'a dyn CBVaultVaultSetSizeEvent)
    pub fn set_on_vault_set_size(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultVaultSetSizeEventArgs)>)
    {
        self.onVaultSetSize = value;
    }

    fn fire_vault_write(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultWrite
        {
            let mut args : CBVaultVaultWriteEventArgs = CBVaultVaultWriteEventArgs::new(par, cbpar);
            callable/*.on_vault_write*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_write(&self) -> &'a dyn CBVaultVaultWriteEvent
    pub fn on_vault_write(&self) -> Option<fn (sender : &CBVault, e : &mut CBVaultVaultWriteEventArgs)>
    {
        self.onVaultWrite
    }

    //pub fn set_on_vault_write(&mut self, value : &'a dyn CBVaultVaultWriteEvent)
    pub fn set_on_vault_write(&mut self, value : Option<fn (sender : &CBVault, e : &mut CBVaultVaultWriteEventArgs)>)
    {
        self.onVaultWrite = value;
    }


    pub(crate) fn report_error_info(&self, error_info : &str)
    {
        if let Some(callable) = self.onError
        {
            let mut args : CBVaultErrorEventArgs = CBVaultErrorEventArgs
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
            let callable = CBFSVault_CBVault_GetLastError.get().unwrap();
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
            let callable = CBFSVault_CBVault_GetLastErrorCode.get().unwrap();
            result = callable(self.Handle) as i32;
        }
        result
    }

    // GetRuntimeLicense returns the runtime license key set for CBVault.
    pub fn get_runtime_license(&self) -> Result<String, CBFSVaultError>
    {
        let result : String;
        //let length : c_long;
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 8001 /*PID_RUNTIME_LICENSE*/, 0, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            result = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            Result::Ok(result)
        }
    }

    // SetRuntimeLicense sets the runtime license key for CBVault.
    pub fn set_runtime_license(&self, value : String) -> Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let oem_key = CString::new(value).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();
        unsafe
        {
            let callable = CBFSVault_CBVault_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 8001/*PID_RUNTIME_LICENSE*/, 0, oem_key_ptr, 0) as i32;
            let _ = CString::from_raw(oem_key_ptr);
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }
        else
        {
            return Result::Ok(());
        }
    }

    // Gets the value of the Active property: This property reflects whether a vault has been opened.
    pub fn active(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 1, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AutoCompactAt property: This property specifies the free space percentage threshold a vault must reach to be eligible for automatic compaction.
    pub fn auto_compact_at(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 2, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the AutoCompactAt property: This property specifies the free space percentage threshold a vault must reach to be eligible for automatic compaction.
    pub fn set_auto_compact_at(&self, value : i32) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSVault_CBVault_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 2, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the CallbackMode property: This property specifies whether the struct should operate in callback mode.
    pub fn callback_mode(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 3, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the CallbackMode property: This property specifies whether the struct should operate in callback mode.
    pub fn set_callback_mode(&self, value : bool) -> Result<(), CBFSVaultError> {
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
            let callable = CBFSVault_CBVault_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 3, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the CaseSensitive property: This property specifies whether the struct should open a vault in case-sensitive mode.
    pub fn case_sensitive(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 4, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the CaseSensitive property: This property specifies whether the struct should open a vault in case-sensitive mode.
    pub fn set_case_sensitive(&self, value : bool) -> Result<(), CBFSVaultError> {
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
            let callable = CBFSVault_CBVault_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 4, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the DefaultFileAccessPassword property: This property specifies the default encryption password to use when opening files and alternate streams.
    pub fn default_file_access_password(&self) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 5, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the DefaultFileAccessPassword property: This property specifies the default encryption password to use when opening files and alternate streams.
    pub fn set_default_file_access_password(&self, value : &str) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            //let CStrValue : CString = CString::from_vec_unchecked(value./*clone().*/into_bytes());
            //let cstrvalue_ptr: *mut c_char = CStrValue.into_raw();

            let cstrvalue_ptr: *mut c_char;
            match CString::new(value)
            {
                Ok(CStrValue) => { cstrvalue_ptr = CStrValue.into_raw(); }
                Err(_) => { return Result::Err(CBFSVaultError::new(-1, "String conversion error")); }
            }

            let callable = CBFSVault_CBVault_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 5, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the DefaultFileCompression property: This property specifies the default compression mode to use when creating files and alternate streams.
    pub fn default_file_compression(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 6, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the DefaultFileCompression property: This property specifies the default compression mode to use when creating files and alternate streams.
    pub fn set_default_file_compression(&self, value : i32) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSVault_CBVault_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 6, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the DefaultFileCreatePassword property: This property specifies the default encryption password to use when creating new files and alternate streams.
    pub fn default_file_create_password(&self) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 7, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the DefaultFileCreatePassword property: This property specifies the default encryption password to use when creating new files and alternate streams.
    pub fn set_default_file_create_password(&self, value : &str) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            //let CStrValue : CString = CString::from_vec_unchecked(value./*clone().*/into_bytes());
            //let cstrvalue_ptr: *mut c_char = CStrValue.into_raw();

            let cstrvalue_ptr: *mut c_char;
            match CString::new(value)
            {
                Ok(CStrValue) => { cstrvalue_ptr = CStrValue.into_raw(); }
                Err(_) => { return Result::Err(CBFSVaultError::new(-1, "String conversion error")); }
            }

            let callable = CBFSVault_CBVault_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 7, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the DefaultFileEncryption property: This property specifies the default encryption mode to use when creating files and alternate streams.
    pub fn default_file_encryption(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 8, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the DefaultFileEncryption property: This property specifies the default encryption mode to use when creating files and alternate streams.
    pub fn set_default_file_encryption(&self, value : i32) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSVault_CBVault_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 8, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the IsCorrupted property: This property specifies whether the vault is corrupted.
    pub fn is_corrupted(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 9, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the LastWriteTime property: This property specifies the last modification time of the vault.
    pub fn last_write_time(&self) -> Result<chrono::DateTime<Utc>, CBFSVaultError>
    {
        let ret_val : chrono::DateTime<Utc>; // = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsInt64.get().unwrap();
            callable(self.Handle, 10, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = file_time_to_chrono_time(val_buf);
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the Logo property: This property specifies an application-defined text-based logo stored in the second page of a vault.
    pub fn logo(&self) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 11, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the Logo property: This property specifies an application-defined text-based logo stored in the second page of a vault.
    pub fn set_logo(&self, value : &str) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            //let CStrValue : CString = CString::from_vec_unchecked(value./*clone().*/into_bytes());
            //let cstrvalue_ptr: *mut c_char = CStrValue.into_raw();

            let cstrvalue_ptr: *mut c_char;
            match CString::new(value)
            {
                Ok(CStrValue) => { cstrvalue_ptr = CStrValue.into_raw(); }
                Err(_) => { return Result::Err(CBFSVaultError::new(-1, "String conversion error")); }
            }

            let callable = CBFSVault_CBVault_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 11, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the PageSize property: This property specifies the vault's page size.
    pub fn page_size(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 12, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the PageSize property: This property specifies the vault's page size.
    pub fn set_page_size(&self, value : i32) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSVault_CBVault_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 12, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the PathSeparator property: This property specifies the path separator character to use when returning vault paths.
    pub fn path_separator(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 13, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the PathSeparator property: This property specifies the path separator character to use when returning vault paths.
    pub fn set_path_separator(&self, value : i32) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSVault_CBVault_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 13, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the PossibleFreeSpace property: This property specifies the maximum amount of free space the vault could possibly have available.
    pub fn possible_free_space(&self) -> Result<i64, CBFSVaultError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsInt64.get().unwrap();
            callable(self.Handle, 14, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the PossibleSize property: This property specifies the maximum size the vault could possibly be.
    pub fn possible_size(&self) -> Result<i64, CBFSVaultError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsInt64.get().unwrap();
            callable(self.Handle, 15, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the ReadOnly property: This property specifies whether the struct should open a vault in read-only mode.
    pub fn read_only(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 16, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the ReadOnly property: This property specifies whether the struct should open a vault in read-only mode.
    pub fn set_read_only(&self, value : bool) -> Result<(), CBFSVaultError> {
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
            let callable = CBFSVault_CBVault_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 16, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the Tag property: This property stores application-defined data specific to a particular instance of the struct.
    pub fn tag(&self) -> Result<i64, CBFSVaultError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsInt64.get().unwrap();
            callable(self.Handle, 17, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the Tag property: This property stores application-defined data specific to a particular instance of the struct.
    pub fn set_tag(&self, value : i64) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe 
        {
            let ValuePtr = &value;
            let callable = CBFSVault_CBVault_SetInt64.get().unwrap();
            ret_code = callable(self.Handle, 17, 0 as c_long, ValuePtr as *const i64, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseAccessTime property: This property specifies whether the struct should keep track of last access times for vault items.
    pub fn use_access_time(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 18, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseAccessTime property: This property specifies whether the struct should keep track of last access times for vault items.
    pub fn set_use_access_time(&self, value : bool) -> Result<(), CBFSVaultError> {
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
            let callable = CBFSVault_CBVault_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 18, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UseSystemCache property: This property specifies whether the operating system's cache is used.
    pub fn use_system_cache(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 19, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UseSystemCache property: This property specifies whether the operating system's cache is used.
    pub fn set_use_system_cache(&self, value : bool) -> Result<(), CBFSVaultError> {
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
            let callable = CBFSVault_CBVault_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 19, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the VaultEncryption property: This property specifies the whole-vault encryption mode.
    pub fn vault_encryption(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 20, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the VaultEncryption property: This property specifies the whole-vault encryption mode.
    pub fn set_vault_encryption(&self, value : i32) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSVault_CBVault_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 20, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the VaultFile property: This property specifies the vault to create or open.
    pub fn vault_file(&self) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 21, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the VaultFile property: This property specifies the vault to create or open.
    pub fn set_vault_file(&self, value : &str) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            //let CStrValue : CString = CString::from_vec_unchecked(value./*clone().*/into_bytes());
            //let cstrvalue_ptr: *mut c_char = CStrValue.into_raw();

            let cstrvalue_ptr: *mut c_char;
            match CString::new(value)
            {
                Ok(CStrValue) => { cstrvalue_ptr = CStrValue.into_raw(); }
                Err(_) => { return Result::Err(CBFSVaultError::new(-1, "String conversion error")); }
            }

            let callable = CBFSVault_CBVault_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 21, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the VaultFreeSpace property: This property reflects the actual amount of free space the vault has available.
    pub fn vault_free_space(&self) -> Result<i64, CBFSVaultError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsInt64.get().unwrap();
            callable(self.Handle, 22, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the VaultPassword property: This property specifies the whole-vault encryption password.
    pub fn vault_password(&self) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 23, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the VaultPassword property: This property specifies the whole-vault encryption password.
    pub fn set_vault_password(&self, value : &str) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            //let CStrValue : CString = CString::from_vec_unchecked(value./*clone().*/into_bytes());
            //let cstrvalue_ptr: *mut c_char = CStrValue.into_raw();

            let cstrvalue_ptr: *mut c_char;
            match CString::new(value)
            {
                Ok(CStrValue) => { cstrvalue_ptr = CStrValue.into_raw(); }
                Err(_) => { return Result::Err(CBFSVaultError::new(-1, "String conversion error")); }
            }

            let callable = CBFSVault_CBVault_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 23, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the VaultSize property: This property specifies the actual size of the vault.
    pub fn vault_size(&self) -> Result<i64, CBFSVaultError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsInt64.get().unwrap();
            callable(self.Handle, 24, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the VaultSize property: This property specifies the actual size of the vault.
    pub fn set_vault_size(&self, value : i64) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe 
        {
            let ValuePtr = &value;
            let callable = CBFSVault_CBVault_SetInt64.get().unwrap();
            ret_code = callable(self.Handle, 24, 0 as c_long, ValuePtr as *const i64, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the VaultSizeMax property: This property specifies the maximum size a vault can be.
    pub fn vault_size_max(&self) -> Result<i64, CBFSVaultError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsInt64.get().unwrap();
            callable(self.Handle, 25, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the VaultSizeMax property: This property specifies the maximum size a vault can be.
    pub fn set_vault_size_max(&self, value : i64) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe 
        {
            let ValuePtr = &value;
            let callable = CBFSVault_CBVault_SetInt64.get().unwrap();
            ret_code = callable(self.Handle, 25, 0 as c_long, ValuePtr as *const i64, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the VaultSizeMin property: This property specifies the minimum size a vault can be.
    pub fn vault_size_min(&self) -> Result<i64, CBFSVaultError>
    {
        let ret_val : i64; // = 0;
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_GetAsInt64.get().unwrap();
            callable(self.Handle, 26, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the VaultSizeMin property: This property specifies the minimum size a vault can be.
    pub fn set_vault_size_min(&self, value : i64) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe 
        {
            let ValuePtr = &value;
            let callable = CBFSVault_CBVault_SetInt64.get().unwrap();
            ret_code = callable(self.Handle, 26, 0 as c_long, ValuePtr as *const i64, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the VaultState property: This property specifies information about the state of the vault.
    pub fn vault_state(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVault_GetAsInt.get().unwrap();
            val = callable(self.Handle, 27, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }



//////////
// Methods
//////////

    // CacheFilePassword: This method caches an encryption password to use the next time a file or alternate stream is accessed or removes the cached password.
    pub fn cache_file_password(&self, file_name : &str, password : &str, ttl_in_cache : i32, remove_from_cache : bool) ->  Result<(), CBFSVaultError>
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
        let CStrPasswordPtr : *mut c_char;
        match CString::new(password)
        {
            Ok(CStrValuePassword) => { CStrPasswordPtr = CStrValuePassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrPasswordPtr as usize;
        CParams[2] = (ttl_in_cache as isize) as usize;
        let RemoveFromCacheBool : i32;
        if remove_from_cache
        {
            RemoveFromCacheBool = 1;
        } else {
            RemoveFromCacheBool = 0;
        }
        CParams[3] = RemoveFromCacheBool as usize;


        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 2, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn cache_file_password

    // CheckAndRepair: This method checks a vault's consistency and repairs it as necessary.
    pub fn check_and_repair(&self, flags : i32) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        CParams[0] = (flags as isize) as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 3, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn check_and_repair

    // CheckFilePassword: This method verifies whether a particular file password is correct.
    pub fn check_file_password(&self, file_name : &str, password : &str) ->  Result<bool, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrPasswordPtr : *mut c_char;
        match CString::new(password)
        {
            Ok(CStrValuePassword) => { CStrPasswordPtr = CStrValuePassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrPasswordPtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 4, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] != 0;
        return Result::Ok(ret_val);
    } // fn check_file_password

    // CheckVaultPassword: This method verifies whether a particular vault password is correct.
    pub fn check_vault_password(&self, password : &str) ->  Result<bool, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrPasswordPtr : *mut c_char;
        match CString::new(password)
        {
            Ok(CStrValuePassword) => { CStrPasswordPtr = CStrValuePassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrPasswordPtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 5, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn check_vault_password

    // CloseVault: This method closes the vault.
    pub fn close_vault(&self, ) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 6, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn close_vault

    // CompactVault: This method compacts the vault.
    pub fn compact_vault(&self, ) ->  Result<bool, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 7, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[0] != 0;
        return Result::Ok(ret_val);
    } // fn compact_vault

    // Config: Sets or retrieves a configuration setting.
    pub fn config(&self, configuration_string : &str) ->  Result<String, CBFSVaultError>
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
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 8, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrConfigurationStringPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn config

    // CopyFromVault: This method copies files and directories from the vault to a physical filesystem.
    pub fn copy_from_vault(&self, vault_path : &str, system_path : &str, mask : &str, flags : i32, password : &str) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 5 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0, 0];

        let CStrVaultPathPtr : *mut c_char;
        match CString::new(vault_path)
        {
            Ok(CStrValueVaultPath) => { CStrVaultPathPtr = CStrValueVaultPath.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrVaultPathPtr as usize;
        let CStrSystemPathPtr : *mut c_char;
        match CString::new(system_path)
        {
            Ok(CStrValueSystemPath) => { CStrSystemPathPtr = CStrValueSystemPath.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrSystemPathPtr as usize;
        let CStrMaskPtr : *mut c_char;
        match CString::new(mask)
        {
            Ok(CStrValueMask) => { CStrMaskPtr = CStrValueMask.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrMaskPtr as usize;
        CParams[3] = (flags as isize) as usize;
        let CStrPasswordPtr : *mut c_char;
        match CString::new(password)
        {
            Ok(CStrValuePassword) => { CStrPasswordPtr = CStrValuePassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[4] = CStrPasswordPtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 9, 5, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrVaultPathPtr);
            let _ = CString::from_raw(CStrSystemPathPtr);
            let _ = CString::from_raw(CStrMaskPtr);
            let _ = CString::from_raw(CStrPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn copy_from_vault

    // CopyToVault: This method copies files and directories from a physical filesystem to the vault.
    pub fn copy_to_vault(&self, system_path : &str, vault_path : &str, mask : &str, flags : i32, encryption : i32, password : &str, compression : i32, compression_level : i32, pages_per_block : i32) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 9 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let CStrSystemPathPtr : *mut c_char;
        match CString::new(system_path)
        {
            Ok(CStrValueSystemPath) => { CStrSystemPathPtr = CStrValueSystemPath.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrSystemPathPtr as usize;
        let CStrVaultPathPtr : *mut c_char;
        match CString::new(vault_path)
        {
            Ok(CStrValueVaultPath) => { CStrVaultPathPtr = CStrValueVaultPath.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrVaultPathPtr as usize;
        let CStrMaskPtr : *mut c_char;
        match CString::new(mask)
        {
            Ok(CStrValueMask) => { CStrMaskPtr = CStrValueMask.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrMaskPtr as usize;
        CParams[3] = (flags as isize) as usize;
        CParams[4] = (encryption as isize) as usize;
        let CStrPasswordPtr : *mut c_char;
        match CString::new(password)
        {
            Ok(CStrValuePassword) => { CStrPasswordPtr = CStrValuePassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[5] = CStrPasswordPtr as usize;
        CParams[6] = (compression as isize) as usize;
        CParams[7] = (compression_level as isize) as usize;
        CParams[8] = (pages_per_block as isize) as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 10, 9, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrSystemPathPtr);
            let _ = CString::from_raw(CStrVaultPathPtr);
            let _ = CString::from_raw(CStrMaskPtr);
            let _ = CString::from_raw(CStrPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn copy_to_vault

    // CreateDirectory: This method creates a new directory in the vault.
    pub fn create_directory(&self, directory : &str, create_parents : bool) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrDirectoryPtr : *mut c_char;
        match CString::new(directory)
        {
            Ok(CStrValueDirectory) => { CStrDirectoryPtr = CStrValueDirectory.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrDirectoryPtr as usize;
        let CreateParentsBool : i32;
        if create_parents
        {
            CreateParentsBool = 1;
        } else {
            CreateParentsBool = 0;
        }
        CParams[1] = CreateParentsBool as usize;


        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 11, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrDirectoryPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn create_directory

    // CreateLink: This method creates a symbolic link to another file in the vault.
    pub fn create_link(&self, link_name : &str, destination_name : &str) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrLinkNamePtr : *mut c_char;
        match CString::new(link_name)
        {
            Ok(CStrValueLinkName) => { CStrLinkNamePtr = CStrValueLinkName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrLinkNamePtr as usize;
        let CStrDestinationNamePtr : *mut c_char;
        match CString::new(destination_name)
        {
            Ok(CStrValueDestinationName) => { CStrDestinationNamePtr = CStrValueDestinationName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrDestinationNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 12, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrLinkNamePtr);
            let _ = CString::from_raw(CStrDestinationNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn create_link

    // DeleteFile: This method deletes a vault item.
    pub fn delete_file(&self, file_name : &str) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 13, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn delete_file

    // DeleteFileTag: This method deletes a file tag.
    pub fn delete_file_tag(&self, file_name : &str, tag_id : i32, tag_name : &str) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        CParams[1] = (tag_id as isize) as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrTagNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 14, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn delete_file_tag

    // FileExists: This method checks whether a vault item exists.
    pub fn file_exists(&self, file_name : &str) ->  Result<bool, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 15, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn file_exists

    // FileMatchesMask: This method checks whether a particular file or directory name matches the specified mask.
    pub fn file_matches_mask(&self, mask : &str, file_name : &str, case_sensitive : bool) ->  Result<bool, CBFSVaultError>
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
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 16, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrMaskPtr);
            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[3] != 0;
        return Result::Ok(ret_val);
    } // fn file_matches_mask

    // FileTagExists: This method checks whether a file tag exists.
    pub fn file_tag_exists(&self, file_name : &str, tag_id : i32, tag_name : &str) ->  Result<bool, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        CParams[1] = (tag_id as isize) as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrTagNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 17, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[3] != 0;
        return Result::Ok(ret_val);
    } // fn file_tag_exists

    // FileTimeToNanoseconds: This method returns the subsecond part of the time expressed in nanoseconds.
    pub fn file_time_to_nanoseconds(&self, file_time : &chrono::DateTime<Utc>) ->  Result<i32, CBFSVaultError>
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
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 18, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] as i32;
        return Result::Ok(ret_val);
    } // fn file_time_to_nanoseconds

    // FileTimeToUnixTime: This method converts FileTime to Unix time format.
    pub fn file_time_to_unix_time(&self, file_time : &chrono::DateTime<Utc>) ->  Result<i64, CBFSVaultError>
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
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 19, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn file_time_to_unix_time

    // FindClose: This method closes a search operation and releases any associated resources.
    pub fn find_close(&self, search_id : i64) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 20, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn find_close

    // FindFirst: This method searches for the first vault item that matches the specified name and attributes.
    pub fn find_first(&self, file_mask : &str, attributes : i32, flags : i32) ->  Result<i64, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrFileMaskPtr : *mut c_char;
        match CString::new(file_mask)
        {
            Ok(CStrValueFileMask) => { CStrFileMaskPtr = CStrValueFileMask.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileMaskPtr as usize;
        CParams[1] = (attributes as isize) as usize;
        CParams[2] = (flags as isize) as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 21, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileMaskPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn find_first

    // FindFirstByQuery: This method searches for the first file or directory whose file tags match the specified query.
    pub fn find_first_by_query(&self, directory : &str, query : &str, flags : i32) ->  Result<i64, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrDirectoryPtr : *mut c_char;
        match CString::new(directory)
        {
            Ok(CStrValueDirectory) => { CStrDirectoryPtr = CStrValueDirectory.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrDirectoryPtr as usize;
        let CStrQueryPtr : *mut c_char;
        match CString::new(query)
        {
            Ok(CStrValueQuery) => { CStrQueryPtr = CStrValueQuery.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrQueryPtr as usize;
        CParams[2] = (flags as isize) as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 22, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrDirectoryPtr);
            let _ = CString::from_raw(CStrQueryPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn find_first_by_query

    // FindNext: This method searches for the next vault item that matches an ongoing search operation.
    pub fn find_next(&self, search_id : i64) ->  Result<bool, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 23, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn find_next

    // GetFileAttributes: This method retrieves the attributes of a vault item.
    pub fn get_file_attributes(&self, file_name : &str) ->  Result<i32, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 24, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] as i32;
        return Result::Ok(ret_val);
    } // fn get_file_attributes

    // GetFileCompression: This method retrieves the compression mode of a file or alternate stream.
    pub fn get_file_compression(&self, file_name : &str) ->  Result<i32, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 25, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] as i32;
        return Result::Ok(ret_val);
    } // fn get_file_compression

    // GetFileCreationTime: This method retrieves the creation time of a vault item.
    pub fn get_file_creation_time(&self, file_name : &str) ->  Result<chrono::DateTime<Utc>, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : chrono::DateTime<Utc> ; // = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 26, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = file_time_to_chrono_time(ll_buf);
        return Result::Ok(ret_val);
    } // fn get_file_creation_time

    // GetFileEncryption: This method retrieves the encryption mode of a file or alternate stream.
    pub fn get_file_encryption(&self, file_name : &str) ->  Result<i32, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 27, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] as i32;
        return Result::Ok(ret_val);
    } // fn get_file_encryption

    // GetFileLastAccessTime: This method retrieves the last access time of a vault item.
    pub fn get_file_last_access_time(&self, file_name : &str) ->  Result<chrono::DateTime<Utc>, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : chrono::DateTime<Utc> ; // = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 28, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = file_time_to_chrono_time(ll_buf);
        return Result::Ok(ret_val);
    } // fn get_file_last_access_time

    // GetFileMetadataSize: This method retrieves the size of the metadata associated with a vault item.
    pub fn get_file_metadata_size(&self, file_name : &str) ->  Result<i64, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 29, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_file_metadata_size

    // GetFileModificationTime: This method retrieves the modification time of a vault item.
    pub fn get_file_modification_time(&self, file_name : &str) ->  Result<chrono::DateTime<Utc>, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : chrono::DateTime<Utc> ; // = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 30, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = file_time_to_chrono_time(ll_buf);
        return Result::Ok(ret_val);
    } // fn get_file_modification_time

    // GetFileSize: This method retrieves the size of a file or alternate stream.
    pub fn get_file_size(&self, file_name : &str) ->  Result<i64, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 31, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_file_size

    // GetFileTag: This method retrieves the binary data held by a raw file tag attached to the specified vault item.
    pub fn get_file_tag(&self, file_name : &str, tag_id : i32) ->  Result<Vec<u8>, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : Vec<u8> ; // = Vec::new();
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        CParams[1] = (tag_id as isize) as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 32, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
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
    } // fn get_file_tag

    // GetFileTagAsAnsiString: This method retrieves the value of an AnsiString-typed file tag attached to the specified vault item.
    pub fn get_file_tag_as_ansi_string(&self, file_name : &str, tag_name : &str) ->  Result<String, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrTagNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 33, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[2] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn get_file_tag_as_ansi_string

    // GetFileTagAsBoolean: This method retrieves the value of a Boolean-typed file tag attached to the specified vault item.
    pub fn get_file_tag_as_boolean(&self, file_name : &str, tag_name : &str) ->  Result<bool, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrTagNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 34, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] != 0;
        return Result::Ok(ret_val);
    } // fn get_file_tag_as_boolean

    // GetFileTagAsDateTime: This method retrieves the value of a DateTime-typed file tag attached to the specified vault item.
    pub fn get_file_tag_as_date_time(&self, file_name : &str, tag_name : &str) ->  Result<chrono::DateTime<Utc>, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : chrono::DateTime<Utc> ; // = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrTagNamePtr as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 35, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = file_time_to_chrono_time(ll_buf);
        return Result::Ok(ret_val);
    } // fn get_file_tag_as_date_time

    // GetFileTagAsNumber: This method retrieves the value of a Number-typed file tag attached to the specified vault item.
    pub fn get_file_tag_as_number(&self, file_name : &str, tag_name : &str) ->  Result<i64, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrTagNamePtr as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 36, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_file_tag_as_number

    // GetFileTagAsString: This method retrieves the value of a String-typed file tag attached to the specified vault item.
    pub fn get_file_tag_as_string(&self, file_name : &str, tag_name : &str) ->  Result<String, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrTagNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 37, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[2] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn get_file_tag_as_string

    // GetFileTagDataType: This method retrieves the data type of a typed file tag attached to a specific vault item.
    pub fn get_file_tag_data_type(&self, file_name : &str, tag_name : &str) ->  Result<i32, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrTagNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 38, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] as i32;
        return Result::Ok(ret_val);
    } // fn get_file_tag_data_type

    // GetFileTagSize: This method retrieves the size of a raw file tag attached to the specified vault item.
    pub fn get_file_tag_size(&self, file_name : &str, tag_id : i32) ->  Result<i32, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        CParams[1] = (tag_id as isize) as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 39, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] as i32;
        return Result::Ok(ret_val);
    } // fn get_file_tag_size

    // GetSearchResultAttributes: This method retrieves the attributes of a vault item found during a search operation.
    pub fn get_search_result_attributes(&self, search_id : i64) ->  Result<i32, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 40, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] as i32;
        return Result::Ok(ret_val);
    } // fn get_search_result_attributes

    // GetSearchResultCreationTime: This method retrieves the creation time of a vault item found during a search operation.
    pub fn get_search_result_creation_time(&self, search_id : i64) ->  Result<chrono::DateTime<Utc>, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : chrono::DateTime<Utc> ; // = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 41, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = file_time_to_chrono_time(ll_buf);
        return Result::Ok(ret_val);
    } // fn get_search_result_creation_time

    // GetSearchResultFullName: This method retrieves the fully qualified name of a vault item found during a search operation.
    pub fn get_search_result_full_name(&self, search_id : i64) ->  Result<String, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 42, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn get_search_result_full_name

    // GetSearchResultLastAccessTime: This method retrieves the last access time of a vault item found during a search operation.
    pub fn get_search_result_last_access_time(&self, search_id : i64) ->  Result<chrono::DateTime<Utc>, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : chrono::DateTime<Utc> ; // = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 43, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = file_time_to_chrono_time(ll_buf);
        return Result::Ok(ret_val);
    } // fn get_search_result_last_access_time

    // GetSearchResultLinkDestination: This method retrieves the destination of a symbolic link found during a search operation.
    pub fn get_search_result_link_destination(&self, search_id : i64) ->  Result<String, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 44, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn get_search_result_link_destination

    // GetSearchResultMetadataSize: This method retrieves the size of the metadata associated with a vault item found during a search operation.
    pub fn get_search_result_metadata_size(&self, search_id : i64) ->  Result<i64, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 45, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_search_result_metadata_size

    // GetSearchResultModificationTime: This method retrieves the modification time of a vault item found during a search operation.
    pub fn get_search_result_modification_time(&self, search_id : i64) ->  Result<chrono::DateTime<Utc>, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : chrono::DateTime<Utc> ; // = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 46, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = file_time_to_chrono_time(ll_buf);
        return Result::Ok(ret_val);
    } // fn get_search_result_modification_time

    // GetSearchResultName: This method retrieves the name of a vault item found during a search operation.
    pub fn get_search_result_name(&self, search_id : i64) ->  Result<String, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 47, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn get_search_result_name

    // GetSearchResults: This method retrieves all information about a vault item found during a search operation.
    pub fn get_search_results(&self, search_id : i64, name : &mut String, full_name : &mut String, attributes : &mut i32, size : &mut i64, creation_time : &mut chrono::DateTime<Utc>, modification_time : &mut chrono::DateTime<Utc>, last_access_time : &mut chrono::DateTime<Utc>, link_destination : &mut String, metadata_size : &mut i64) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 10 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;
        let NameStr : String = name.clone();
        let CStrValueName : CString;
        unsafe
        {
            CStrValueName = CString::from_vec_unchecked(NameStr.into_bytes());
        }
        let CStrNamePtr : *mut c_char = CStrValueName.into_raw();
        let NameArr : Vec<*mut c_char> = vec![CStrNamePtr];
        CParams[1] = NameArr.as_ptr() as usize;
        let FullNameStr : String = full_name.clone();
        let CStrValueFullName : CString;
        unsafe
        {
            CStrValueFullName = CString::from_vec_unchecked(FullNameStr.into_bytes());
        }
        let CStrFullNamePtr : *mut c_char = CStrValueFullName.into_raw();
        let FullNameArr : Vec<*mut c_char> = vec![CStrFullNamePtr];
        CParams[2] = FullNameArr.as_ptr() as usize;
        let AttributesArr : Vec<i32> = vec![*attributes];
        CParams[3] = AttributesArr.as_ptr() as usize;
        let SizeArr : Vec<i64> = vec![*size];
        CParams[4] = SizeArr.as_ptr() as usize;
        let CreationTimeUnixDate : i64 = chrono_time_to_file_time(creation_time);
        let CreationTimeArr : Vec<i64> = vec![CreationTimeUnixDate];
        CParams[5] = CreationTimeArr.as_ptr() as usize;
        let ModificationTimeUnixDate : i64 = chrono_time_to_file_time(modification_time);
        let ModificationTimeArr : Vec<i64> = vec![ModificationTimeUnixDate];
        CParams[6] = ModificationTimeArr.as_ptr() as usize;
        let LastAccessTimeUnixDate : i64 = chrono_time_to_file_time(last_access_time);
        let LastAccessTimeArr : Vec<i64> = vec![LastAccessTimeUnixDate];
        CParams[7] = LastAccessTimeArr.as_ptr() as usize;
        let LinkDestinationStr : String = link_destination.clone();
        let CStrValueLinkDestination : CString;
        unsafe
        {
            CStrValueLinkDestination = CString::from_vec_unchecked(LinkDestinationStr.into_bytes());
        }
        let CStrLinkDestinationPtr : *mut c_char = CStrValueLinkDestination.into_raw();
        let LinkDestinationArr : Vec<*mut c_char> = vec![CStrLinkDestinationPtr];
        CParams[8] = LinkDestinationArr.as_ptr() as usize;
        let MetadataSizeArr : Vec<i64> = vec![*metadata_size];
        CParams[9] = MetadataSizeArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 48, 10, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrNamePtr);
            let _ = CString::from_raw(CStrFullNamePtr);
            let _ = CString::from_raw(CStrLinkDestinationPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        *name = charptr_to_string(NameArr[0] as *const i8).unwrap_or_else(|_| String::default());
        *full_name = charptr_to_string(FullNameArr[0] as *const i8).unwrap_or_else(|_| String::default());
        *attributes = AttributesArr[0];
        *size = SizeArr[0];
        *creation_time = file_time_to_chrono_time(CreationTimeArr[0]);
        *modification_time = file_time_to_chrono_time(ModificationTimeArr[0]);
        *last_access_time = file_time_to_chrono_time(LastAccessTimeArr[0]);
        *link_destination = charptr_to_string(LinkDestinationArr[0] as *const i8).unwrap_or_else(|_| String::default());
        *metadata_size = MetadataSizeArr[0];
        return Result::Ok(());
    } // fn get_search_results

    // GetSearchResultSize: This method retrieves the size of a vault item found during a search operation.
    pub fn get_search_result_size(&self, search_id : i64) ->  Result<i64, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let SearchIdArr : Vec<i64> = vec![search_id];
        CParams[0] = SearchIdArr.as_ptr() as usize;

        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 49, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_search_result_size

    // IsDirectoryEmpty: This method checks whether a directory is empty.
    pub fn is_directory_empty(&self, directory : &str) ->  Result<bool, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrDirectoryPtr : *mut c_char;
        match CString::new(directory)
        {
            Ok(CStrValueDirectory) => { CStrDirectoryPtr = CStrValueDirectory.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrDirectoryPtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 50, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrDirectoryPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn is_directory_empty

    // IsValidVault: This method checks whether a local file is a CBFS Vault vault.
    pub fn is_valid_vault(&self, ) ->  Result<bool, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 51, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[0] != 0;
        return Result::Ok(ret_val);
    } // fn is_valid_vault

    // MoveFile: This method renames or moves a vault item.
    pub fn move_file(&self, old_file_name : &str, new_file_name : &str, overwrite : bool) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrOldFileNamePtr : *mut c_char;
        match CString::new(old_file_name)
        {
            Ok(CStrValueOldFileName) => { CStrOldFileNamePtr = CStrValueOldFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrOldFileNamePtr as usize;
        let CStrNewFileNamePtr : *mut c_char;
        match CString::new(new_file_name)
        {
            Ok(CStrValueNewFileName) => { CStrNewFileNamePtr = CStrValueNewFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrNewFileNamePtr as usize;
        let OverwriteBool : i32;
        if overwrite
        {
            OverwriteBool = 1;
        } else {
            OverwriteBool = 0;
        }
        CParams[2] = OverwriteBool as usize;


        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 52, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrOldFileNamePtr);
            let _ = CString::from_raw(CStrNewFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn move_file

    // OpenFile: This method opens a new or existing file or alternate stream in the vault.
    pub fn open_file(&self, file_name : &str, open_mode : i32, read_enabled : bool, write_enabled : bool, password : &str) ->  Result<CBFSVaultStream, CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 5 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        CParams[1] = (open_mode as isize) as usize;
        let ReadEnabledBool : i32;
        if read_enabled
        {
            ReadEnabledBool = 1;
        } else {
            ReadEnabledBool = 0;
        }
        CParams[2] = ReadEnabledBool as usize;

        let WriteEnabledBool : i32;
        if write_enabled
        {
            WriteEnabledBool = 1;
        } else {
            WriteEnabledBool = 0;
        }
        CParams[3] = WriteEnabledBool as usize;

        let CStrPasswordPtr : *mut c_char;
        match CString::new(password)
        {
            Ok(CStrValuePassword) => { CStrPasswordPtr = CStrValuePassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[4] = CStrPasswordPtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 53, 5, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

    if CParams[5] == 0
    {
        return Result::Err(CBFSVaultError::new(-1, "No result was returned by the method implementation and no error was reported either"));
    }
    else
    {
        return Result::Ok(CBFSVaultStream::new(CParams[5]));
    }
    } // fn open_file

    // OpenFileEx: This method opens a new or existing file or alternate stream in the vault.
    pub fn open_file_ex(&self, file_name : &str, open_mode : i32, read_enabled : bool, write_enabled : bool, share_deny_read : bool, share_deny_write : bool, encryption : i32, password : &str, compression : i32, compression_level : i32, pages_per_block : i32) ->  Result<CBFSVaultStream, CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 11 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        CParams[1] = (open_mode as isize) as usize;
        let ReadEnabledBool : i32;
        if read_enabled
        {
            ReadEnabledBool = 1;
        } else {
            ReadEnabledBool = 0;
        }
        CParams[2] = ReadEnabledBool as usize;

        let WriteEnabledBool : i32;
        if write_enabled
        {
            WriteEnabledBool = 1;
        } else {
            WriteEnabledBool = 0;
        }
        CParams[3] = WriteEnabledBool as usize;

        let ShareDenyReadBool : i32;
        if share_deny_read
        {
            ShareDenyReadBool = 1;
        } else {
            ShareDenyReadBool = 0;
        }
        CParams[4] = ShareDenyReadBool as usize;

        let ShareDenyWriteBool : i32;
        if share_deny_write
        {
            ShareDenyWriteBool = 1;
        } else {
            ShareDenyWriteBool = 0;
        }
        CParams[5] = ShareDenyWriteBool as usize;

        CParams[6] = (encryption as isize) as usize;
        let CStrPasswordPtr : *mut c_char;
        match CString::new(password)
        {
            Ok(CStrValuePassword) => { CStrPasswordPtr = CStrValuePassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[7] = CStrPasswordPtr as usize;
        CParams[8] = (compression as isize) as usize;
        CParams[9] = (compression_level as isize) as usize;
        CParams[10] = (pages_per_block as isize) as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 54, 11, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

    if CParams[11] == 0
    {
        return Result::Err(CBFSVaultError::new(-1, "No result was returned by the method implementation and no error was reported either"));
    }
    else
    {
        return Result::Ok(CBFSVaultStream::new(CParams[11]));
    }
    } // fn open_file_ex

    // OpenRootData: This method opens the vault's root data stream.
    pub fn open_root_data(&self, ) ->  Result<CBFSVaultStream, CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 55, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

    if CParams[0] == 0
    {
        return Result::Err(CBFSVaultError::new(-1, "No result was returned by the method implementation and no error was reported either"));
    }
    else
    {
        return Result::Ok(CBFSVaultStream::new(CParams[0]));
    }
    } // fn open_root_data

    // OpenVault: This method opens a new or existing vault.
    pub fn open_vault(&self, open_mode : i32, journaling_mode : i32) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        CParams[0] = (open_mode as isize) as usize;
        CParams[1] = (journaling_mode as isize) as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 56, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn open_vault

    // ResolveLink: This method retrieves the destination of a symbolic link.
    pub fn resolve_link(&self, link_name : &str, normalize : bool) ->  Result<String, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrLinkNamePtr : *mut c_char;
        match CString::new(link_name)
        {
            Ok(CStrValueLinkName) => { CStrLinkNamePtr = CStrValueLinkName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrLinkNamePtr as usize;
        let NormalizeBool : i32;
        if normalize
        {
            NormalizeBool = 1;
        } else {
            NormalizeBool = 0;
        }
        CParams[1] = NormalizeBool as usize;


        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 57, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrLinkNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[2] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn resolve_link

    // SetFileAttributes: This method sets the attributes of a vault item.
    pub fn set_file_attributes(&self, file_name : &str, attributes : i32) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        CParams[1] = (attributes as isize) as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 58, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_attributes

    // SetFileCompression: This method compresses or decompresses a file or alternate stream.
    pub fn set_file_compression(&self, file_name : &str, compression : i32, compression_level : i32, pages_per_block : i32, password : &str) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 5 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        CParams[1] = (compression as isize) as usize;
        CParams[2] = (compression_level as isize) as usize;
        CParams[3] = (pages_per_block as isize) as usize;
        let CStrPasswordPtr : *mut c_char;
        match CString::new(password)
        {
            Ok(CStrValuePassword) => { CStrPasswordPtr = CStrValuePassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[4] = CStrPasswordPtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 59, 5, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_compression

    // SetFileCreationTime: This method sets the creation time of a vault item.
    pub fn set_file_creation_time(&self, file_name : &str, creation_time : &chrono::DateTime<Utc>) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CreationTimeUnixDate : i64 = chrono_time_to_file_time(creation_time);
        let CreationTimeArr : Vec<i64> = vec![CreationTimeUnixDate];
        CParams[1] = CreationTimeArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 60, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_creation_time

    // SetFileEncryption: This method encrypts, decrypts, or changes the encryption password of a file or alternate stream.
    pub fn set_file_encryption(&self, file_name : &str, encryption : i32, old_password : &str, new_password : &str) ->  Result<(), CBFSVaultError>
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
        CParams[1] = (encryption as isize) as usize;
        let CStrOldPasswordPtr : *mut c_char;
        match CString::new(old_password)
        {
            Ok(CStrValueOldPassword) => { CStrOldPasswordPtr = CStrValueOldPassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrOldPasswordPtr as usize;
        let CStrNewPasswordPtr : *mut c_char;
        match CString::new(new_password)
        {
            Ok(CStrValueNewPassword) => { CStrNewPasswordPtr = CStrValueNewPassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[3] = CStrNewPasswordPtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 61, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrOldPasswordPtr);
            let _ = CString::from_raw(CStrNewPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_encryption

    // SetFileLastAccessTime: This method sets the last access time of a vault item.
    pub fn set_file_last_access_time(&self, file_name : &str, last_access_time : &chrono::DateTime<Utc>) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let LastAccessTimeUnixDate : i64 = chrono_time_to_file_time(last_access_time);
        let LastAccessTimeArr : Vec<i64> = vec![LastAccessTimeUnixDate];
        CParams[1] = LastAccessTimeArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 62, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_last_access_time

    // SetFileModificationTime: This method sets the modification time of a vault item.
    pub fn set_file_modification_time(&self, file_name : &str, modification_time : &chrono::DateTime<Utc>) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let ModificationTimeUnixDate : i64 = chrono_time_to_file_time(modification_time);
        let ModificationTimeArr : Vec<i64> = vec![ModificationTimeUnixDate];
        CParams[1] = ModificationTimeArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 63, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_modification_time

    // SetFileSize: This method sets the size of a file or alternate stream.
    pub fn set_file_size(&self, file_name : &str, size : i64, password : &str) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let SizeArr : Vec<i64> = vec![size];
        CParams[1] = SizeArr.as_ptr() as usize;
        let CStrPasswordPtr : *mut c_char;
        match CString::new(password)
        {
            Ok(CStrValuePassword) => { CStrPasswordPtr = CStrValuePassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrPasswordPtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 64, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_size

    // SetFileTag: This method attaches a raw file tag with binary data to the specified vault item.
    pub fn set_file_tag(&self, file_name : &str, tag_id : i32, data : &[u8]) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, data.len() as i32, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        CParams[1] = (tag_id as isize) as usize;
        CParams[2] = data.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 65, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_tag

    // SetFileTagAsAnsiString: This method attaches an AnsiString-typed file tag to the specified vault item.
    pub fn set_file_tag_as_ansi_string(&self, file_name : &str, tag_name : &str, value : &str) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrTagNamePtr as usize;
        let CStrValuePtr : *mut c_char;
        match CString::new(value)
        {
            Ok(CStrValueValue) => { CStrValuePtr = CStrValueValue.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrValuePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 66, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
            let _ = CString::from_raw(CStrValuePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_tag_as_ansi_string

    // SetFileTagAsBoolean: This method attaches a Boolean-typed file tag to the specified vault item.
    pub fn set_file_tag_as_boolean(&self, file_name : &str, tag_name : &str, value : bool) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrTagNamePtr as usize;
        let ValueBool : i32;
        if value
        {
            ValueBool = 1;
        } else {
            ValueBool = 0;
        }
        CParams[2] = ValueBool as usize;


        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 67, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_tag_as_boolean

    // SetFileTagAsDateTime: This method attaches a DateTime-typed file tag to the specified vault item.
    pub fn set_file_tag_as_date_time(&self, file_name : &str, tag_name : &str, value : &chrono::DateTime<Utc>) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrTagNamePtr as usize;
        let ValueUnixDate : i64 = chrono_time_to_file_time(value);
        let ValueArr : Vec<i64> = vec![ValueUnixDate];
        CParams[2] = ValueArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 68, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_tag_as_date_time

    // SetFileTagAsNumber: This method attaches a Number-typed file tag to the specified vault item.
    pub fn set_file_tag_as_number(&self, file_name : &str, tag_name : &str, value : i64) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrTagNamePtr as usize;
        let ValueArr : Vec<i64> = vec![value];
        CParams[2] = ValueArr.as_ptr() as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 69, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_tag_as_number

    // SetFileTagAsString: This method attaches a String-typed file tag to the specified vault item.
    pub fn set_file_tag_as_string(&self, file_name : &str, tag_name : &str, value : &str) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        let CStrFileNamePtr : *mut c_char;
        match CString::new(file_name)
        {
            Ok(CStrValueFileName) => { CStrFileNamePtr = CStrValueFileName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrFileNamePtr as usize;
        let CStrTagNamePtr : *mut c_char;
        match CString::new(tag_name)
        {
            Ok(CStrValueTagName) => { CStrTagNamePtr = CStrValueTagName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrTagNamePtr as usize;
        let CStrValuePtr : *mut c_char;
        match CString::new(value)
        {
            Ok(CStrValueValue) => { CStrValuePtr = CStrValueValue.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrValuePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 70, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
            let _ = CString::from_raw(CStrValuePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_file_tag_as_string

    // UnixTimeToFileTime: This method converts the date/time in Unix format to the Windows FileTime format.
    pub fn unix_time_to_file_time(&self, unix_time : i64, nanoseconds : i32) ->  Result<chrono::DateTime<Utc>, CBFSVaultError>
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
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 71, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = file_time_to_chrono_time(ll_buf);
        return Result::Ok(ret_val);
    } // fn unix_time_to_file_time

    // UpdateVaultEncryption: This method encrypts, decrypts, or changes the encryption password of the vault.
    pub fn update_vault_encryption(&self, encryption : i32, old_password : &str, new_password : &str) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 3 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0, 0];

        CParams[0] = (encryption as isize) as usize;
        let CStrOldPasswordPtr : *mut c_char;
        match CString::new(old_password)
        {
            Ok(CStrValueOldPassword) => { CStrOldPasswordPtr = CStrValueOldPassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[1] = CStrOldPasswordPtr as usize;
        let CStrNewPasswordPtr : *mut c_char;
        match CString::new(new_password)
        {
            Ok(CStrValueNewPassword) => { CStrNewPasswordPtr = CStrValueNewPassword.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[2] = CStrNewPasswordPtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVault_Do.get().unwrap();
            ret_code = callable(self.Handle, 72, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrOldPasswordPtr);
            let _ = CString::from_raw(CStrNewPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn update_vault_encryption

} // CBVault

extern "system" fn CBVaultEventDispatcher(pObj : usize, event_id : c_long, _cparam : c_long, param : IntPtrArrayType, cbparam : IntArrayType) -> c_long
{
    let obj: &CBVault;
    // Lock the Mutex to get access to the HashMap
    let map = CBVaultDict.lock().unwrap();
    let objOpt = map.get(&pObj);
    if objOpt.is_none()
    {
        return -1;
    }
    let addr = *objOpt.unwrap();
    drop(map);
    obj = unsafe { & *(addr as *mut CBVault) };

    let event_result: Result<(), Box<dyn std::any::Any + Send>> = catch_unwind (||
    {
        match event_id
        {
            1 /* DataCompress */=> obj.fire_data_compress(/*cparam as i32, */param, cbparam),

            2 /* DataDecompress */=> obj.fire_data_decompress(/*cparam as i32, */param, cbparam),

            3 /* DataDecrypt */=> obj.fire_data_decrypt(/*cparam as i32, */param, cbparam),

            4 /* DataEncrypt */=> obj.fire_data_encrypt(/*cparam as i32, */param, cbparam),

            5 /* Error */=> obj.fire_error(/*cparam as i32, */param, cbparam),

            6 /* FileAfterCopy */=> obj.fire_file_after_copy(/*cparam as i32, */param, cbparam),

            7 /* FileBeforeCopy */=> obj.fire_file_before_copy(/*cparam as i32, */param, cbparam),

            8 /* FilePasswordNeeded */=> obj.fire_file_password_needed(/*cparam as i32, */param, cbparam),

            9 /* HashCalculate */=> obj.fire_hash_calculate(/*cparam as i32, */param, cbparam),

            10 /* KeyDerive */=> obj.fire_key_derive(/*cparam as i32, */param, cbparam),

            11 /* Progress */=> obj.fire_progress(/*cparam as i32, */param, cbparam),

            12 /* VaultClose */=> obj.fire_vault_close(/*cparam as i32, */param, cbparam),

            13 /* VaultDelete */=> obj.fire_vault_delete(/*cparam as i32, */param, cbparam),

            14 /* VaultFlush */=> obj.fire_vault_flush(/*cparam as i32, */param, cbparam),

            15 /* VaultGetParentSize */=> obj.fire_vault_get_parent_size(/*cparam as i32, */param, cbparam),

            16 /* VaultGetSize */=> obj.fire_vault_get_size(/*cparam as i32, */param, cbparam),

            17 /* VaultOpen */=> obj.fire_vault_open(/*cparam as i32, */param, cbparam),

            18 /* VaultRead */=> obj.fire_vault_read(/*cparam as i32, */param, cbparam),

            19 /* VaultSetSize */=> obj.fire_vault_set_size(/*cparam as i32, */param, cbparam),

            20 /* VaultWrite */=> obj.fire_vault_write(/*cparam as i32, */param, cbparam),

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


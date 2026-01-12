


#![allow(non_snake_case)]

extern crate libloading as lib;

use std::{collections::HashMap, ffi::{c_char, c_long, c_longlong, c_ulong, c_void, CStr, CString}, panic::catch_unwind, sync::{atomic::{AtomicUsize, Ordering::SeqCst}, Mutex, OnceLock} };
use lib::{Library, Symbol};
use std::fmt::Write;
use chrono::Utc;
use once_cell::sync::Lazy;

use crate::{*, cbfsvaultkey};

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_StaticInit)(void *hInst);
type CBFSVaultCBVaultDriveStaticInitType = unsafe extern "system" fn(hInst : *mut c_void) -> i32;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_StaticDestroy)();
type CBFSVaultCBVaultDriveStaticDestroyType = unsafe extern "system" fn()-> i32;

// typedef void* (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_Create)(PCBFSVAULT_CALLBACK lpSink, void *lpContext, char *lpOemKey, int opts);
type CBFSVaultCBVaultDriveCreateType = unsafe extern "system" fn(lpSink : CBFSVaultSinkDelegateType, lpContext : usize, lpOemKey : *const c_char, opts : i32) -> usize;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_Destroy)(void *lpObj);
type CBFSVaultCBVaultDriveDestroyType = unsafe extern "system" fn(lpObj: usize)-> c_long;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_CheckIndex)(void *lpObj, int propid, int arridx);
type CBFSVaultCBVaultDriveCheckIndexType = unsafe extern "system" fn(lpObj: usize, propid: c_long, arridx: c_long)-> c_long;

// typedef void* (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_Get)(void *lpObj, int propid, int arridx, int *lpcbVal, int64 *lpllVal);
type CBFSVaultCBVaultDriveGetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *mut c_longlong) -> *mut c_void;
type CBFSVaultCBVaultDriveGetAsCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut c_long, llVal: *const c_longlong) -> *const c_char;
type CBFSVaultCBVaultDriveGetAsIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *const c_void) -> usize;
type CBFSVaultCBVaultDriveGetAsInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *const c_void, llVal: *mut i64) -> usize;
type CBFSVaultCBVaultDriveGetAsBSTRType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, len: *mut i32, llVal: *const c_void) -> *const u8;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_Set)(void *lpObj, int propid, int arridx, const void *val, int cbVal);
type CBFSVaultCBVaultDriveSetType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_void, len: c_ulong)-> c_long;
type CBFSVaultCBVaultDriveSetCStrType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const c_char, len: c_ulong)-> c_long;
type CBFSVaultCBVaultDriveSetIntType = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: isize, len: c_ulong)-> c_long;
type CBFSVaultCBVaultDriveSetInt64Type = unsafe extern "system" fn(p: usize, propid: c_long, arridx: c_long, value: *const i64, len: c_ulong)-> c_long;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_Do)(void *lpObj, int methid, int cparam, void *param[], int cbparam[], int64 *lpllVal);
type CBFSVaultCBVaultDriveDoType = unsafe extern "system" fn(p: usize, method_id: c_long, cparam: c_long, params: UIntPtrArrayType, cbparam: IntArrayType, llVal: *mut c_longlong)-> c_long;

// typedef char* (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_GetLastError)(void *lpObj);
type CBFSVaultCBVaultDriveGetLastErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_GetLastErrorCode)(void *lpObj);
type CBFSVaultCBVaultDriveGetLastErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_SetLastErrorAndCode)(void *lpObj, int code, char *message);
type CBFSVaultCBVaultDriveSetLastErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

// typedef char* (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_GetEventError)(void *lpObj);
type CBFSVaultCBVaultDriveGetEventErrorType = unsafe extern "system" fn(p: usize) -> *const c_char; /*PLXAnsiChar, */

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_GetEventErrorCode)(void *lpObj);
type CBFSVaultCBVaultDriveGetEventErrorCodeType = unsafe extern "system" fn(p: usize)-> c_long;

// typedef int   (CBFSVAULT_CALL *lpCBFSVault_CBVaultDrive_SetEventErrorAndCode)(void *lpObj, int code, char *message);
type CBFSVaultCBVaultDriveSetEventErrorAndCodeType = unsafe extern "system" fn(p: usize, code: c_long, message: *mut c_void)-> c_long;

static CBFSVault_CBVaultDrive_StaticInit : OnceLock<Symbol<CBFSVaultCBVaultDriveStaticInitType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_StaticDestroy : OnceLock<Symbol<CBFSVaultCBVaultDriveStaticDestroyType>> = OnceLock::new();

static CBFSVault_CBVaultDrive_Create: OnceLock<Symbol<CBFSVaultCBVaultDriveCreateType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_Destroy: OnceLock<Symbol<CBFSVaultCBVaultDriveDestroyType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_Set: OnceLock<Symbol<CBFSVaultCBVaultDriveSetType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_SetCStr: OnceLock<Symbol<CBFSVaultCBVaultDriveSetCStrType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_SetInt: OnceLock<Symbol<CBFSVaultCBVaultDriveSetIntType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_SetInt64: OnceLock<Symbol<CBFSVaultCBVaultDriveSetInt64Type>> = OnceLock::new();
static CBFSVault_CBVaultDrive_Get: OnceLock<Symbol<CBFSVaultCBVaultDriveGetType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_GetAsCStr: OnceLock<Symbol<CBFSVaultCBVaultDriveGetAsCStrType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_GetAsInt: OnceLock<Symbol<CBFSVaultCBVaultDriveGetAsIntType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_GetAsInt64: OnceLock<Symbol<CBFSVaultCBVaultDriveGetAsInt64Type>> = OnceLock::new();
static CBFSVault_CBVaultDrive_GetAsBSTR: OnceLock<Symbol<CBFSVaultCBVaultDriveGetAsBSTRType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_GetLastError: OnceLock<Symbol<CBFSVaultCBVaultDriveGetLastErrorType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_GetLastErrorCode: OnceLock<Symbol<CBFSVaultCBVaultDriveGetLastErrorCodeType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_SetLastErrorAndCode: OnceLock<Symbol<CBFSVaultCBVaultDriveSetLastErrorAndCodeType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_GetEventError: OnceLock<Symbol<CBFSVaultCBVaultDriveGetEventErrorType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_GetEventErrorCode: OnceLock<Symbol<CBFSVaultCBVaultDriveGetEventErrorCodeType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_SetEventErrorAndCode: OnceLock<Symbol<CBFSVaultCBVaultDriveSetEventErrorAndCodeType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_CheckIndex: OnceLock<Symbol<CBFSVaultCBVaultDriveCheckIndexType>> = OnceLock::new();
static CBFSVault_CBVaultDrive_Do: OnceLock<Symbol<CBFSVaultCBVaultDriveDoType>> = OnceLock::new();

static CBVaultDriveIDSeed : AtomicUsize = AtomicUsize::new(1);

static CBVaultDriveDict : Lazy<Mutex<HashMap<usize, Box<CBVaultDrive>>>> = Lazy::new(|| Mutex::new(HashMap::new()) );
// static CBVaultDriveDictMutex : Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0) ); // Removed as map is now mutexed

const CBVaultDriveCreateOpt : i32 = 0;


pub type CBVaultDriveStream = crate::CBFSVaultStream;


pub(crate) fn get_lib_funcs( lib_hand : &'static Library) -> bool
{
    #[cfg(target_os = "android")]
    return true;
    #[cfg(target_os = "ios")]
    return true;

    unsafe
    {
        // CBFSVault_CBVaultDrive_StaticInit
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_StaticInit") {
            let _ = CBFSVault_CBVaultDrive_StaticInit.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_StaticDestroy
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_StaticDestroy") {
            let _ = CBFSVault_CBVaultDrive_StaticDestroy.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_Create
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Create") {
            let _ = CBFSVault_CBVaultDrive_Create.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_Destroy
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Destroy") {
            let _ = CBFSVault_CBVaultDrive_Destroy.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_Get
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Get") {
            let _ = CBFSVault_CBVaultDrive_Get.set(func);
        } else { return false; }
        // CBFSVault_CBVaultDrive_GetAsCStr
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Get") {
            let _ = CBFSVault_CBVaultDrive_GetAsCStr.set(func);
        } else { return false; }
        // CBFSVault_CBVaultDrive_GetAsInt
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Get") {
            let _ = CBFSVault_CBVaultDrive_GetAsInt.set(func);
        } else { return false; }
        // CBFSVault_CBVaultDrive_GetAsInt64
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Get") {
            let _ = CBFSVault_CBVaultDrive_GetAsInt64.set(func);
        } else { return false; }
        // CBFSVault_CBVaultDrive_GetAsBSTR
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Get") {
            let _ = CBFSVault_CBVaultDrive_GetAsBSTR.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_Set
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Set") {
            let _ = CBFSVault_CBVaultDrive_Set.set(func);
        } else { return false; }
        // CBFSVault_CBVaultDrive_SetCStr
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Set") {
            let _ = CBFSVault_CBVaultDrive_SetCStr.set(func);
        } else { return false; }
        // CBFSVault_CBVaultDrive_SetInt
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Set") {
            let _ = CBFSVault_CBVaultDrive_SetInt.set(func);
        } else { return false; }
        // CBFSVault_CBVaultDrive_SetInt64
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Set") {
            let _ = CBFSVault_CBVaultDrive_SetInt64.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_GetLastError
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_GetLastError") {
            let _ = CBFSVault_CBVaultDrive_GetLastError.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_GetLastErrorCode
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_GetLastErrorCode") {
            let _ = CBFSVault_CBVaultDrive_GetLastErrorCode.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_SetLastErrorAndCode
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_SetLastErrorAndCode") {
            let _ = CBFSVault_CBVaultDrive_SetLastErrorAndCode.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_GetEventError
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_GetEventError") {
            let _ = CBFSVault_CBVaultDrive_GetEventError.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_GetEventErrorCode
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_GetEventErrorCode") {
            let _ = CBFSVault_CBVaultDrive_GetEventErrorCode.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_SetEventErrorAndCode
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_SetEventErrorAndCode") {
            let _ = CBFSVault_CBVaultDrive_SetEventErrorAndCode.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_CheckIndex
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_CheckIndex") {
            let _ = CBFSVault_CBVaultDrive_CheckIndex.set(func);
        } else { return false; }

        // CBFSVault_CBVaultDrive_Do
        if let Ok(func) = lib_hand.get(b"CBFSVault_CBVaultDrive_Do") {
            let _ = CBFSVault_CBVaultDrive_Do.set(func);
        } else { return false; }

    }
    return true;
}

//////////////
// Event Types
//////////////


// CBVaultDriveDataCompressEventArgs carries the parameters of the DataCompress event of CBVaultDrive
pub struct CBVaultDriveDataCompressEventArgs
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
impl CBVaultDriveDataCompressEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveDataCompressEventArgs
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
        
        CBVaultDriveDataCompressEventArgs
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

pub trait CBVaultDriveDataCompressEvent
{
    fn on_data_compress(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveDataCompressEventArgs);
}


// CBVaultDriveDataDecompressEventArgs carries the parameters of the DataDecompress event of CBVaultDrive
pub struct CBVaultDriveDataDecompressEventArgs
{
    hInDataPtr : *mut u8,
    InSize : i32,
    hOutDataPtr : *mut u8,
    OutSize : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of DataDecompressEventArgs
impl CBVaultDriveDataDecompressEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveDataDecompressEventArgs
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
        
        CBVaultDriveDataDecompressEventArgs
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

pub trait CBVaultDriveDataDecompressEvent
{
    fn on_data_decompress(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveDataDecompressEventArgs);
}


// CBVaultDriveDataDecryptEventArgs carries the parameters of the DataDecrypt event of CBVaultDrive
pub struct CBVaultDriveDataDecryptEventArgs
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
impl CBVaultDriveDataDecryptEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveDataDecryptEventArgs
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
        
        CBVaultDriveDataDecryptEventArgs
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

pub trait CBVaultDriveDataDecryptEvent
{
    fn on_data_decrypt(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveDataDecryptEventArgs);
}


// CBVaultDriveDataEncryptEventArgs carries the parameters of the DataEncrypt event of CBVaultDrive
pub struct CBVaultDriveDataEncryptEventArgs
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
impl CBVaultDriveDataEncryptEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveDataEncryptEventArgs
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
        
        CBVaultDriveDataEncryptEventArgs
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

pub trait CBVaultDriveDataEncryptEvent
{
    fn on_data_encrypt(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveDataEncryptEventArgs);
}


// CBVaultDriveEjectedEventArgs carries the parameters of the Ejected event of CBVaultDrive
pub struct CBVaultDriveEjectedEventArgs
{
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of EjectedEventArgs
impl CBVaultDriveEjectedEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveEjectedEventArgs
    {

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(0) as i32;
        }
        
        CBVaultDriveEjectedEventArgs
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

pub trait CBVaultDriveEjectedEvent
{
    fn on_ejected(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveEjectedEventArgs);
}


// CBVaultDriveErrorEventArgs carries the parameters of the Error event of CBVaultDrive
pub struct CBVaultDriveErrorEventArgs
{
    ErrorCode : i32,
    Description : String,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of ErrorEventArgs
impl CBVaultDriveErrorEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveErrorEventArgs
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
                lvDescription = CStr::from_ptr(lvhDescriptionPtr).to_str().expect("Valid UTF8 not received for the parameter 'Description' in the Error event of a CBVaultDrive instance").to_owned();
            }
        }

        CBVaultDriveErrorEventArgs
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

pub trait CBVaultDriveErrorEvent
{
    fn on_error(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveErrorEventArgs);
}


// CBVaultDriveFileAccessEventArgs carries the parameters of the FileAccess event of CBVaultDrive
pub struct CBVaultDriveFileAccessEventArgs
{
    FileName : String,
    ExistingAttributes : i32,
    DesiredAccess : i32,
    Attributes : i32,
    Options : i32,
    ShareMode : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of FileAccessEventArgs
impl CBVaultDriveFileAccessEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveFileAccessEventArgs
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
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the FileAccess event of a CBVaultDrive instance").to_owned();
            }
        }

        let lvExistingAttributes : i32;
        unsafe
        {
            lvExistingAttributes = *par.add(1) as i32;
        }
        
        let lvDesiredAccess : i32;
        unsafe
        {
            lvDesiredAccess = *par.add(2) as i32;
        }
        
        let lvAttributes : i32;
        unsafe
        {
            lvAttributes = *par.add(3) as i32;
        }
        
        let lvOptions : i32;
        unsafe
        {
            lvOptions = *par.add(4) as i32;
        }
        
        let lvShareMode : i32;
        unsafe
        {
            lvShareMode = *par.add(5) as i32;
        }
        
        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(6) as i32;
        }
        
        CBVaultDriveFileAccessEventArgs
        {
            FileName: lvFileName,
            ExistingAttributes: lvExistingAttributes,
            DesiredAccess: lvDesiredAccess,
            Attributes: lvAttributes,
            Options: lvOptions,
            ShareMode: lvShareMode,
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
    pub fn existing_attributes(&self) -> i32
    {
        self.ExistingAttributes
    }
    pub fn desired_access(&self) -> i32
    {
        self.DesiredAccess
    }
    pub fn attributes(&self) -> i32
    {
        self.Attributes
    }
    pub fn options(&self) -> i32
    {
        self.Options
    }
    pub fn share_mode(&self) -> i32
    {
        self.ShareMode
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

pub trait CBVaultDriveFileAccessEvent
{
    fn on_file_access(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveFileAccessEventArgs);
}


// CBVaultDriveFileAfterCopyEventArgs carries the parameters of the FileAfterCopy event of CBVaultDrive
pub struct CBVaultDriveFileAfterCopyEventArgs
{
    SourcePath : String,
    DestinationPath : String,
    Attributes : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of FileAfterCopyEventArgs
impl CBVaultDriveFileAfterCopyEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveFileAfterCopyEventArgs
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
                lvSourcePath = CStr::from_ptr(lvhSourcePathPtr).to_str().expect("Valid UTF8 not received for the parameter 'SourcePath' in the FileAfterCopy event of a CBVaultDrive instance").to_owned();
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
                lvDestinationPath = CStr::from_ptr(lvhDestinationPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'DestinationPath' in the FileAfterCopy event of a CBVaultDrive instance").to_owned();
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
        
        CBVaultDriveFileAfterCopyEventArgs
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

pub trait CBVaultDriveFileAfterCopyEvent
{
    fn on_file_after_copy(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveFileAfterCopyEventArgs);
}


// CBVaultDriveFileBeforeCopyEventArgs carries the parameters of the FileBeforeCopy event of CBVaultDrive
pub struct CBVaultDriveFileBeforeCopyEventArgs
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
impl CBVaultDriveFileBeforeCopyEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveFileBeforeCopyEventArgs
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
                lvSourcePath = CStr::from_ptr(lvhSourcePathPtr).to_str().expect("Valid UTF8 not received for the parameter 'SourcePath' in the FileBeforeCopy event of a CBVaultDrive instance").to_owned();
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
                lvDestinationPath = CStr::from_ptr(lvhDestinationPathPtr).to_str().expect("Valid UTF8 not received for the parameter 'DestinationPath' in the FileBeforeCopy event of a CBVaultDrive instance").to_owned();
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
        
        CBVaultDriveFileBeforeCopyEventArgs
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

pub trait CBVaultDriveFileBeforeCopyEvent
{
    fn on_file_before_copy(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveFileBeforeCopyEventArgs);
}


// CBVaultDriveFilePasswordNeededEventArgs carries the parameters of the FilePasswordNeeded event of CBVaultDrive
pub struct CBVaultDriveFilePasswordNeededEventArgs
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
impl CBVaultDriveFilePasswordNeededEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveFilePasswordNeededEventArgs
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
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the FilePasswordNeeded event of a CBVaultDrive instance").to_owned();
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
                lvPassword = CStr::from_ptr(lvhPasswordPtr).to_str().expect("Valid UTF8 not received for the parameter 'Password' in the FilePasswordNeeded event of a CBVaultDrive instance").to_owned();
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
        
        CBVaultDriveFilePasswordNeededEventArgs
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

pub trait CBVaultDriveFilePasswordNeededEvent
{
    fn on_file_password_needed(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveFilePasswordNeededEventArgs);
}


// CBVaultDriveHashCalculateEventArgs carries the parameters of the HashCalculate event of CBVaultDrive
pub struct CBVaultDriveHashCalculateEventArgs
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
impl CBVaultDriveHashCalculateEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveHashCalculateEventArgs
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
        
        CBVaultDriveHashCalculateEventArgs
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

pub trait CBVaultDriveHashCalculateEvent
{
    fn on_hash_calculate(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveHashCalculateEventArgs);
}


// CBVaultDriveKeyDeriveEventArgs carries the parameters of the KeyDerive event of CBVaultDrive
pub struct CBVaultDriveKeyDeriveEventArgs
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
impl CBVaultDriveKeyDeriveEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveKeyDeriveEventArgs
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
        
        CBVaultDriveKeyDeriveEventArgs
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

pub trait CBVaultDriveKeyDeriveEvent
{
    fn on_key_derive(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveKeyDeriveEventArgs);
}


// CBVaultDriveProgressEventArgs carries the parameters of the Progress event of CBVaultDrive
pub struct CBVaultDriveProgressEventArgs
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
impl CBVaultDriveProgressEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveProgressEventArgs
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
                lvFileName = CStr::from_ptr(lvhFileNamePtr).to_str().expect("Valid UTF8 not received for the parameter 'FileName' in the Progress event of a CBVaultDrive instance").to_owned();
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

        CBVaultDriveProgressEventArgs
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

pub trait CBVaultDriveProgressEvent
{
    fn on_progress(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveProgressEventArgs);
}


// CBVaultDriveVaultCloseEventArgs carries the parameters of the VaultClose event of CBVaultDrive
pub struct CBVaultDriveVaultCloseEventArgs
{
    VaultHandle : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultCloseEventArgs
impl CBVaultDriveVaultCloseEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveVaultCloseEventArgs
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
        
        CBVaultDriveVaultCloseEventArgs
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

pub trait CBVaultDriveVaultCloseEvent
{
    fn on_vault_close(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveVaultCloseEventArgs);
}


// CBVaultDriveVaultDeleteEventArgs carries the parameters of the VaultDelete event of CBVaultDrive
pub struct CBVaultDriveVaultDeleteEventArgs
{
    Vault : String,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultDeleteEventArgs
impl CBVaultDriveVaultDeleteEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveVaultDeleteEventArgs
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
                lvVault = CStr::from_ptr(lvhVaultPtr).to_str().expect("Valid UTF8 not received for the parameter 'Vault' in the VaultDelete event of a CBVaultDrive instance").to_owned();
            }
        }

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(1) as i32;
        }
        
        CBVaultDriveVaultDeleteEventArgs
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

pub trait CBVaultDriveVaultDeleteEvent
{
    fn on_vault_delete(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveVaultDeleteEventArgs);
}


// CBVaultDriveVaultFlushEventArgs carries the parameters of the VaultFlush event of CBVaultDrive
pub struct CBVaultDriveVaultFlushEventArgs
{
    VaultHandle : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultFlushEventArgs
impl CBVaultDriveVaultFlushEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveVaultFlushEventArgs
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
        
        CBVaultDriveVaultFlushEventArgs
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

pub trait CBVaultDriveVaultFlushEvent
{
    fn on_vault_flush(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveVaultFlushEventArgs);
}


// CBVaultDriveVaultGetParentSizeEventArgs carries the parameters of the VaultGetParentSize event of CBVaultDrive
pub struct CBVaultDriveVaultGetParentSizeEventArgs
{
    Vault : String,
    VaultHandle : i64,
    FreeSpace : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultGetParentSizeEventArgs
impl CBVaultDriveVaultGetParentSizeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveVaultGetParentSizeEventArgs
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
                lvVault = CStr::from_ptr(lvhVaultPtr).to_str().expect("Valid UTF8 not received for the parameter 'Vault' in the VaultGetParentSize event of a CBVaultDrive instance").to_owned();
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
        
        CBVaultDriveVaultGetParentSizeEventArgs
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

pub trait CBVaultDriveVaultGetParentSizeEvent
{
    fn on_vault_get_parent_size(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveVaultGetParentSizeEventArgs);
}


// CBVaultDriveVaultGetSizeEventArgs carries the parameters of the VaultGetSize event of CBVaultDrive
pub struct CBVaultDriveVaultGetSizeEventArgs
{
    VaultHandle : i64,
    Size : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultGetSizeEventArgs
impl CBVaultDriveVaultGetSizeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveVaultGetSizeEventArgs
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
        
        CBVaultDriveVaultGetSizeEventArgs
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

pub trait CBVaultDriveVaultGetSizeEvent
{
    fn on_vault_get_size(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveVaultGetSizeEventArgs);
}


// CBVaultDriveVaultOpenEventArgs carries the parameters of the VaultOpen event of CBVaultDrive
pub struct CBVaultDriveVaultOpenEventArgs
{
    Vault : String,
    VaultHandle : i64,
    OpenMode : i32,
    ReadOnly : bool,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultOpenEventArgs
impl CBVaultDriveVaultOpenEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveVaultOpenEventArgs
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
                lvVault = CStr::from_ptr(lvhVaultPtr).to_str().expect("Valid UTF8 not received for the parameter 'Vault' in the VaultOpen event of a CBVaultDrive instance").to_owned();
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
        
        CBVaultDriveVaultOpenEventArgs
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

pub trait CBVaultDriveVaultOpenEvent
{
    fn on_vault_open(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveVaultOpenEventArgs);
}


// CBVaultDriveVaultReadEventArgs carries the parameters of the VaultRead event of CBVaultDrive
pub struct CBVaultDriveVaultReadEventArgs
{
    VaultHandle : i64,
    Offset : i64,
    hBufferPtr : *mut u8,
    Count : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultReadEventArgs
impl CBVaultDriveVaultReadEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveVaultReadEventArgs
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
        
        CBVaultDriveVaultReadEventArgs
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

pub trait CBVaultDriveVaultReadEvent
{
    fn on_vault_read(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveVaultReadEventArgs);
}


// CBVaultDriveVaultSetSizeEventArgs carries the parameters of the VaultSetSize event of CBVaultDrive
pub struct CBVaultDriveVaultSetSizeEventArgs
{
    VaultHandle : i64,
    NewSize : i64,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultSetSizeEventArgs
impl CBVaultDriveVaultSetSizeEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveVaultSetSizeEventArgs
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
        
        CBVaultDriveVaultSetSizeEventArgs
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

pub trait CBVaultDriveVaultSetSizeEvent
{
    fn on_vault_set_size(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveVaultSetSizeEventArgs);
}


// CBVaultDriveVaultWriteEventArgs carries the parameters of the VaultWrite event of CBVaultDrive
pub struct CBVaultDriveVaultWriteEventArgs
{
    VaultHandle : i64,
    Offset : i64,
    hBufferPtr : *mut u8,
    Count : i32,
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of VaultWriteEventArgs
impl CBVaultDriveVaultWriteEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveVaultWriteEventArgs
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
        
        CBVaultDriveVaultWriteEventArgs
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

pub trait CBVaultDriveVaultWriteEvent
{
    fn on_vault_write(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveVaultWriteEventArgs);
}


// CBVaultDriveWorkerThreadCreationEventArgs carries the parameters of the WorkerThreadCreation event of CBVaultDrive
pub struct CBVaultDriveWorkerThreadCreationEventArgs
{
    ResultCode : i32,

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of WorkerThreadCreationEventArgs
impl CBVaultDriveWorkerThreadCreationEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveWorkerThreadCreationEventArgs
    {

        let lvResultCode : i32;
        unsafe
        {
            lvResultCode = *par.add(0) as i32;
        }
        
        CBVaultDriveWorkerThreadCreationEventArgs
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

pub trait CBVaultDriveWorkerThreadCreationEvent
{
    fn on_worker_thread_creation(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveWorkerThreadCreationEventArgs);
}


// CBVaultDriveWorkerThreadTerminationEventArgs carries the parameters of the WorkerThreadTermination event of CBVaultDrive
pub struct CBVaultDriveWorkerThreadTerminationEventArgs
{

    _Params  : IntPtrArrayType
}

// Constructor and marshalRefParams() of WorkerThreadTerminationEventArgs
impl CBVaultDriveWorkerThreadTerminationEventArgs
{
    fn new(par : IntPtrArrayType, _cbpar : IntArrayType) -> CBVaultDriveWorkerThreadTerminationEventArgs
    {

        CBVaultDriveWorkerThreadTerminationEventArgs
        {
            _Params: par
        }
    }


}

pub trait CBVaultDriveWorkerThreadTerminationEvent
{
    fn on_worker_thread_termination(&self, sender : &CBVaultDrive, e : &mut CBVaultDriveWorkerThreadTerminationEventArgs);
}


////////////////////////////
// Main Class Implementation
////////////////////////////

/* The CBVaultDrive component lets applications create a vault, manipulate its contents, and mount it as a virtual drive. */
//pub struct CBVaultDrive<'a>
pub struct CBVaultDrive
{

    // onDataCompress : Option<&'a dyn CBVaultDriveDataCompressEvent>,
    onDataCompress : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataCompressEventArgs) >,
    // onDataDecompress : Option<&'a dyn CBVaultDriveDataDecompressEvent>,
    onDataDecompress : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataDecompressEventArgs) >,
    // onDataDecrypt : Option<&'a dyn CBVaultDriveDataDecryptEvent>,
    onDataDecrypt : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataDecryptEventArgs) >,
    // onDataEncrypt : Option<&'a dyn CBVaultDriveDataEncryptEvent>,
    onDataEncrypt : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataEncryptEventArgs) >,
    // onEjected : Option<&'a dyn CBVaultDriveEjectedEvent>,
    onEjected : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveEjectedEventArgs) >,
    // onError : Option<&'a dyn CBVaultDriveErrorEvent>,
    onError : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveErrorEventArgs) >,
    // onFileAccess : Option<&'a dyn CBVaultDriveFileAccessEvent>,
    onFileAccess : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFileAccessEventArgs) >,
    // onFileAfterCopy : Option<&'a dyn CBVaultDriveFileAfterCopyEvent>,
    onFileAfterCopy : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFileAfterCopyEventArgs) >,
    // onFileBeforeCopy : Option<&'a dyn CBVaultDriveFileBeforeCopyEvent>,
    onFileBeforeCopy : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFileBeforeCopyEventArgs) >,
    // onFilePasswordNeeded : Option<&'a dyn CBVaultDriveFilePasswordNeededEvent>,
    onFilePasswordNeeded : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFilePasswordNeededEventArgs) >,
    // onHashCalculate : Option<&'a dyn CBVaultDriveHashCalculateEvent>,
    onHashCalculate : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveHashCalculateEventArgs) >,
    // onKeyDerive : Option<&'a dyn CBVaultDriveKeyDeriveEvent>,
    onKeyDerive : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveKeyDeriveEventArgs) >,
    // onProgress : Option<&'a dyn CBVaultDriveProgressEvent>,
    onProgress : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveProgressEventArgs) >,
    // onVaultClose : Option<&'a dyn CBVaultDriveVaultCloseEvent>,
    onVaultClose : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultCloseEventArgs) >,
    // onVaultDelete : Option<&'a dyn CBVaultDriveVaultDeleteEvent>,
    onVaultDelete : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultDeleteEventArgs) >,
    // onVaultFlush : Option<&'a dyn CBVaultDriveVaultFlushEvent>,
    onVaultFlush : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultFlushEventArgs) >,
    // onVaultGetParentSize : Option<&'a dyn CBVaultDriveVaultGetParentSizeEvent>,
    onVaultGetParentSize : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultGetParentSizeEventArgs) >,
    // onVaultGetSize : Option<&'a dyn CBVaultDriveVaultGetSizeEvent>,
    onVaultGetSize : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultGetSizeEventArgs) >,
    // onVaultOpen : Option<&'a dyn CBVaultDriveVaultOpenEvent>,
    onVaultOpen : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultOpenEventArgs) >,
    // onVaultRead : Option<&'a dyn CBVaultDriveVaultReadEvent>,
    onVaultRead : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultReadEventArgs) >,
    // onVaultSetSize : Option<&'a dyn CBVaultDriveVaultSetSizeEvent>,
    onVaultSetSize : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultSetSizeEventArgs) >,
    // onVaultWrite : Option<&'a dyn CBVaultDriveVaultWriteEvent>,
    onVaultWrite : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultWriteEventArgs) >,
    // onWorkerThreadCreation : Option<&'a dyn CBVaultDriveWorkerThreadCreationEvent>,
    onWorkerThreadCreation : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveWorkerThreadCreationEventArgs) >,
    // onWorkerThreadTermination : Option<&'a dyn CBVaultDriveWorkerThreadTerminationEvent>,
    onWorkerThreadTermination : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveWorkerThreadTerminationEventArgs) >,

    Id : usize,
    Handle : usize 
}

//impl<'a> Drop for CBVaultDrive<'a>
impl Drop for CBVaultDrive
{
    fn drop(&mut self)
    {
        self.dispose();
    }
}

impl CBVaultDrive
{
    pub fn new() -> &'static mut CBVaultDrive
    {        
         #[cfg(target_os = "android")]
         panic!("CBVaultDrive is not available on Android");
        #[cfg(target_os = "ios")]
        panic!("CBVaultDrive is not available on iOS");
        if !crate::is_lib_loaded()
        {
            crate::init_on_demand();
        }
        
        let ret_code : i32;
        let lId : usize = CBVaultDriveIDSeed.fetch_add(1, SeqCst) as usize;

        let lHandle : isize;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Create.get().unwrap();
            lHandle = callable(CBVaultDriveEventDispatcher, lId, std::ptr::null(), CBVaultDriveCreateOpt) as isize;
        }
        if lHandle < 0
        {
            panic!("Failed to instantiate CBVaultDrive. Please verify that it is supported on this platform");
        }

        let result : CBVaultDrive = CBVaultDrive
        {
            onDataCompress: None,
            onDataDecompress: None,
            onDataDecrypt: None,
            onDataEncrypt: None,
            onEjected: None,
            onError: None,
            onFileAccess: None,
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
            onWorkerThreadCreation: None,
            onWorkerThreadTermination: None,
            Id: lId,
            Handle: lHandle as usize
        };

        let oem_key = CString::new(cbfsvaultkey::rtkCBFSVault).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();

        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_SetCStr.get().unwrap();
            ret_code = callable(lHandle as usize, 8012/*PID_KEYCHECK_RUST*/, 0, oem_key_ptr, 0) as i32;
            let _ = CString::from_raw(oem_key_ptr);
        }
        if ret_code != 0
        {
            panic!("Initialization of CBVaultDrive has failed with error {}", ret_code);
        }

        let mut boxed = Box::new(result);
        let ptr = &mut *boxed as *mut CBVaultDrive;
        CBVaultDriveDict.lock().unwrap().insert(lId, boxed);
        return unsafe { &mut *ptr };
    }

    pub fn dispose(&self)
    {
        let mut _aself : Option<Box<CBVaultDrive>>;
        unsafe
        {


            if !CBVaultDriveDict.lock().unwrap().contains_key(&self.Id)
            {
                return;
            }

            // Remove itself from the list
            _aself = CBVaultDriveDict.lock().unwrap().remove(&self.Id);

            // finalize the ctlclass
            let callable = CBFSVault_CBVaultDrive_Destroy.get().unwrap();
            callable(self.Handle);
        }
    }

/////////
// Events
/////////

    fn fire_data_compress(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDataCompress
        {
            let mut args : CBVaultDriveDataCompressEventArgs = CBVaultDriveDataCompressEventArgs::new(par, cbpar);
            callable/*.on_data_compress*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_data_compress(&self) -> &'a dyn CBVaultDriveDataCompressEvent
    pub fn on_data_compress(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataCompressEventArgs)>
    {
        self.onDataCompress
    }

    //pub fn set_on_data_compress(&mut self, value : &'a dyn CBVaultDriveDataCompressEvent)
    pub fn set_on_data_compress(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataCompressEventArgs)>)
    {
        self.onDataCompress = value;
    }

    fn fire_data_decompress(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDataDecompress
        {
            let mut args : CBVaultDriveDataDecompressEventArgs = CBVaultDriveDataDecompressEventArgs::new(par, cbpar);
            callable/*.on_data_decompress*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_data_decompress(&self) -> &'a dyn CBVaultDriveDataDecompressEvent
    pub fn on_data_decompress(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataDecompressEventArgs)>
    {
        self.onDataDecompress
    }

    //pub fn set_on_data_decompress(&mut self, value : &'a dyn CBVaultDriveDataDecompressEvent)
    pub fn set_on_data_decompress(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataDecompressEventArgs)>)
    {
        self.onDataDecompress = value;
    }

    fn fire_data_decrypt(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDataDecrypt
        {
            let mut args : CBVaultDriveDataDecryptEventArgs = CBVaultDriveDataDecryptEventArgs::new(par, cbpar);
            callable/*.on_data_decrypt*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_data_decrypt(&self) -> &'a dyn CBVaultDriveDataDecryptEvent
    pub fn on_data_decrypt(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataDecryptEventArgs)>
    {
        self.onDataDecrypt
    }

    //pub fn set_on_data_decrypt(&mut self, value : &'a dyn CBVaultDriveDataDecryptEvent)
    pub fn set_on_data_decrypt(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataDecryptEventArgs)>)
    {
        self.onDataDecrypt = value;
    }

    fn fire_data_encrypt(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onDataEncrypt
        {
            let mut args : CBVaultDriveDataEncryptEventArgs = CBVaultDriveDataEncryptEventArgs::new(par, cbpar);
            callable/*.on_data_encrypt*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_data_encrypt(&self) -> &'a dyn CBVaultDriveDataEncryptEvent
    pub fn on_data_encrypt(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataEncryptEventArgs)>
    {
        self.onDataEncrypt
    }

    //pub fn set_on_data_encrypt(&mut self, value : &'a dyn CBVaultDriveDataEncryptEvent)
    pub fn set_on_data_encrypt(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveDataEncryptEventArgs)>)
    {
        self.onDataEncrypt = value;
    }

    fn fire_ejected(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onEjected
        {
            let mut args : CBVaultDriveEjectedEventArgs = CBVaultDriveEjectedEventArgs::new(par, cbpar);
            callable/*.on_ejected*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_ejected(&self) -> &'a dyn CBVaultDriveEjectedEvent
    pub fn on_ejected(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveEjectedEventArgs)>
    {
        self.onEjected
    }

    //pub fn set_on_ejected(&mut self, value : &'a dyn CBVaultDriveEjectedEvent)
    pub fn set_on_ejected(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveEjectedEventArgs)>)
    {
        self.onEjected = value;
    }

    fn fire_error(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onError
        {
            let mut args : CBVaultDriveErrorEventArgs = CBVaultDriveErrorEventArgs::new(par, cbpar);
            callable/*.on_error*/(&self, &mut args);
        }
    }

    //pub fn on_error(&self) -> &'a dyn CBVaultDriveErrorEvent
    pub fn on_error(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveErrorEventArgs)>
    {
        self.onError
    }

    //pub fn set_on_error(&mut self, value : &'a dyn CBVaultDriveErrorEvent)
    pub fn set_on_error(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveErrorEventArgs)>)
    {
        self.onError = value;
    }

    fn fire_file_access(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFileAccess
        {
            let mut args : CBVaultDriveFileAccessEventArgs = CBVaultDriveFileAccessEventArgs::new(par, cbpar);
            callable/*.on_file_access*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_file_access(&self) -> &'a dyn CBVaultDriveFileAccessEvent
    pub fn on_file_access(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFileAccessEventArgs)>
    {
        self.onFileAccess
    }

    //pub fn set_on_file_access(&mut self, value : &'a dyn CBVaultDriveFileAccessEvent)
    pub fn set_on_file_access(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFileAccessEventArgs)>)
    {
        self.onFileAccess = value;
    }

    fn fire_file_after_copy(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFileAfterCopy
        {
            let mut args : CBVaultDriveFileAfterCopyEventArgs = CBVaultDriveFileAfterCopyEventArgs::new(par, cbpar);
            callable/*.on_file_after_copy*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_file_after_copy(&self) -> &'a dyn CBVaultDriveFileAfterCopyEvent
    pub fn on_file_after_copy(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFileAfterCopyEventArgs)>
    {
        self.onFileAfterCopy
    }

    //pub fn set_on_file_after_copy(&mut self, value : &'a dyn CBVaultDriveFileAfterCopyEvent)
    pub fn set_on_file_after_copy(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFileAfterCopyEventArgs)>)
    {
        self.onFileAfterCopy = value;
    }

    fn fire_file_before_copy(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFileBeforeCopy
        {
            let mut args : CBVaultDriveFileBeforeCopyEventArgs = CBVaultDriveFileBeforeCopyEventArgs::new(par, cbpar);
            callable/*.on_file_before_copy*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_file_before_copy(&self) -> &'a dyn CBVaultDriveFileBeforeCopyEvent
    pub fn on_file_before_copy(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFileBeforeCopyEventArgs)>
    {
        self.onFileBeforeCopy
    }

    //pub fn set_on_file_before_copy(&mut self, value : &'a dyn CBVaultDriveFileBeforeCopyEvent)
    pub fn set_on_file_before_copy(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFileBeforeCopyEventArgs)>)
    {
        self.onFileBeforeCopy = value;
    }

    fn fire_file_password_needed(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onFilePasswordNeeded
        {
            let mut args : CBVaultDriveFilePasswordNeededEventArgs = CBVaultDriveFilePasswordNeededEventArgs::new(par, cbpar);
            callable/*.on_file_password_needed*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_file_password_needed(&self) -> &'a dyn CBVaultDriveFilePasswordNeededEvent
    pub fn on_file_password_needed(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFilePasswordNeededEventArgs)>
    {
        self.onFilePasswordNeeded
    }

    //pub fn set_on_file_password_needed(&mut self, value : &'a dyn CBVaultDriveFilePasswordNeededEvent)
    pub fn set_on_file_password_needed(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveFilePasswordNeededEventArgs)>)
    {
        self.onFilePasswordNeeded = value;
    }

    fn fire_hash_calculate(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onHashCalculate
        {
            let mut args : CBVaultDriveHashCalculateEventArgs = CBVaultDriveHashCalculateEventArgs::new(par, cbpar);
            callable/*.on_hash_calculate*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_hash_calculate(&self) -> &'a dyn CBVaultDriveHashCalculateEvent
    pub fn on_hash_calculate(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveHashCalculateEventArgs)>
    {
        self.onHashCalculate
    }

    //pub fn set_on_hash_calculate(&mut self, value : &'a dyn CBVaultDriveHashCalculateEvent)
    pub fn set_on_hash_calculate(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveHashCalculateEventArgs)>)
    {
        self.onHashCalculate = value;
    }

    fn fire_key_derive(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onKeyDerive
        {
            let mut args : CBVaultDriveKeyDeriveEventArgs = CBVaultDriveKeyDeriveEventArgs::new(par, cbpar);
            callable/*.on_key_derive*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_key_derive(&self) -> &'a dyn CBVaultDriveKeyDeriveEvent
    pub fn on_key_derive(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveKeyDeriveEventArgs)>
    {
        self.onKeyDerive
    }

    //pub fn set_on_key_derive(&mut self, value : &'a dyn CBVaultDriveKeyDeriveEvent)
    pub fn set_on_key_derive(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveKeyDeriveEventArgs)>)
    {
        self.onKeyDerive = value;
    }

    fn fire_progress(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onProgress
        {
            let mut args : CBVaultDriveProgressEventArgs = CBVaultDriveProgressEventArgs::new(par, cbpar);
            callable/*.on_progress*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_progress(&self) -> &'a dyn CBVaultDriveProgressEvent
    pub fn on_progress(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveProgressEventArgs)>
    {
        self.onProgress
    }

    //pub fn set_on_progress(&mut self, value : &'a dyn CBVaultDriveProgressEvent)
    pub fn set_on_progress(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveProgressEventArgs)>)
    {
        self.onProgress = value;
    }

    fn fire_vault_close(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultClose
        {
            let mut args : CBVaultDriveVaultCloseEventArgs = CBVaultDriveVaultCloseEventArgs::new(par, cbpar);
            callable/*.on_vault_close*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_close(&self) -> &'a dyn CBVaultDriveVaultCloseEvent
    pub fn on_vault_close(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultCloseEventArgs)>
    {
        self.onVaultClose
    }

    //pub fn set_on_vault_close(&mut self, value : &'a dyn CBVaultDriveVaultCloseEvent)
    pub fn set_on_vault_close(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultCloseEventArgs)>)
    {
        self.onVaultClose = value;
    }

    fn fire_vault_delete(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultDelete
        {
            let mut args : CBVaultDriveVaultDeleteEventArgs = CBVaultDriveVaultDeleteEventArgs::new(par, cbpar);
            callable/*.on_vault_delete*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_delete(&self) -> &'a dyn CBVaultDriveVaultDeleteEvent
    pub fn on_vault_delete(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultDeleteEventArgs)>
    {
        self.onVaultDelete
    }

    //pub fn set_on_vault_delete(&mut self, value : &'a dyn CBVaultDriveVaultDeleteEvent)
    pub fn set_on_vault_delete(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultDeleteEventArgs)>)
    {
        self.onVaultDelete = value;
    }

    fn fire_vault_flush(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultFlush
        {
            let mut args : CBVaultDriveVaultFlushEventArgs = CBVaultDriveVaultFlushEventArgs::new(par, cbpar);
            callable/*.on_vault_flush*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_flush(&self) -> &'a dyn CBVaultDriveVaultFlushEvent
    pub fn on_vault_flush(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultFlushEventArgs)>
    {
        self.onVaultFlush
    }

    //pub fn set_on_vault_flush(&mut self, value : &'a dyn CBVaultDriveVaultFlushEvent)
    pub fn set_on_vault_flush(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultFlushEventArgs)>)
    {
        self.onVaultFlush = value;
    }

    fn fire_vault_get_parent_size(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultGetParentSize
        {
            let mut args : CBVaultDriveVaultGetParentSizeEventArgs = CBVaultDriveVaultGetParentSizeEventArgs::new(par, cbpar);
            callable/*.on_vault_get_parent_size*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_get_parent_size(&self) -> &'a dyn CBVaultDriveVaultGetParentSizeEvent
    pub fn on_vault_get_parent_size(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultGetParentSizeEventArgs)>
    {
        self.onVaultGetParentSize
    }

    //pub fn set_on_vault_get_parent_size(&mut self, value : &'a dyn CBVaultDriveVaultGetParentSizeEvent)
    pub fn set_on_vault_get_parent_size(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultGetParentSizeEventArgs)>)
    {
        self.onVaultGetParentSize = value;
    }

    fn fire_vault_get_size(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultGetSize
        {
            let mut args : CBVaultDriveVaultGetSizeEventArgs = CBVaultDriveVaultGetSizeEventArgs::new(par, cbpar);
            callable/*.on_vault_get_size*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_get_size(&self) -> &'a dyn CBVaultDriveVaultGetSizeEvent
    pub fn on_vault_get_size(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultGetSizeEventArgs)>
    {
        self.onVaultGetSize
    }

    //pub fn set_on_vault_get_size(&mut self, value : &'a dyn CBVaultDriveVaultGetSizeEvent)
    pub fn set_on_vault_get_size(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultGetSizeEventArgs)>)
    {
        self.onVaultGetSize = value;
    }

    fn fire_vault_open(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultOpen
        {
            let mut args : CBVaultDriveVaultOpenEventArgs = CBVaultDriveVaultOpenEventArgs::new(par, cbpar);
            callable/*.on_vault_open*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_open(&self) -> &'a dyn CBVaultDriveVaultOpenEvent
    pub fn on_vault_open(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultOpenEventArgs)>
    {
        self.onVaultOpen
    }

    //pub fn set_on_vault_open(&mut self, value : &'a dyn CBVaultDriveVaultOpenEvent)
    pub fn set_on_vault_open(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultOpenEventArgs)>)
    {
        self.onVaultOpen = value;
    }

    fn fire_vault_read(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultRead
        {
            let mut args : CBVaultDriveVaultReadEventArgs = CBVaultDriveVaultReadEventArgs::new(par, cbpar);
            callable/*.on_vault_read*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_read(&self) -> &'a dyn CBVaultDriveVaultReadEvent
    pub fn on_vault_read(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultReadEventArgs)>
    {
        self.onVaultRead
    }

    //pub fn set_on_vault_read(&mut self, value : &'a dyn CBVaultDriveVaultReadEvent)
    pub fn set_on_vault_read(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultReadEventArgs)>)
    {
        self.onVaultRead = value;
    }

    fn fire_vault_set_size(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultSetSize
        {
            let mut args : CBVaultDriveVaultSetSizeEventArgs = CBVaultDriveVaultSetSizeEventArgs::new(par, cbpar);
            callable/*.on_vault_set_size*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_set_size(&self) -> &'a dyn CBVaultDriveVaultSetSizeEvent
    pub fn on_vault_set_size(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultSetSizeEventArgs)>
    {
        self.onVaultSetSize
    }

    //pub fn set_on_vault_set_size(&mut self, value : &'a dyn CBVaultDriveVaultSetSizeEvent)
    pub fn set_on_vault_set_size(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultSetSizeEventArgs)>)
    {
        self.onVaultSetSize = value;
    }

    fn fire_vault_write(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onVaultWrite
        {
            let mut args : CBVaultDriveVaultWriteEventArgs = CBVaultDriveVaultWriteEventArgs::new(par, cbpar);
            callable/*.on_vault_write*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_vault_write(&self) -> &'a dyn CBVaultDriveVaultWriteEvent
    pub fn on_vault_write(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultWriteEventArgs)>
    {
        self.onVaultWrite
    }

    //pub fn set_on_vault_write(&mut self, value : &'a dyn CBVaultDriveVaultWriteEvent)
    pub fn set_on_vault_write(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveVaultWriteEventArgs)>)
    {
        self.onVaultWrite = value;
    }

    fn fire_worker_thread_creation(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onWorkerThreadCreation
        {
            let mut args : CBVaultDriveWorkerThreadCreationEventArgs = CBVaultDriveWorkerThreadCreationEventArgs::new(par, cbpar);
            callable/*.on_worker_thread_creation*/(&self, &mut args);
            args.marshalRefParams();
        }
    }

    //pub fn on_worker_thread_creation(&self) -> &'a dyn CBVaultDriveWorkerThreadCreationEvent
    pub fn on_worker_thread_creation(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveWorkerThreadCreationEventArgs)>
    {
        self.onWorkerThreadCreation
    }

    //pub fn set_on_worker_thread_creation(&mut self, value : &'a dyn CBVaultDriveWorkerThreadCreationEvent)
    pub fn set_on_worker_thread_creation(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveWorkerThreadCreationEventArgs)>)
    {
        self.onWorkerThreadCreation = value;
    }

    fn fire_worker_thread_termination(&self, /*cparam : i32, */ par: IntPtrArrayType, cbpar : IntArrayType)
    {
        if let Some(callable) = self.onWorkerThreadTermination
        {
            let mut args : CBVaultDriveWorkerThreadTerminationEventArgs = CBVaultDriveWorkerThreadTerminationEventArgs::new(par, cbpar);
            callable/*.on_worker_thread_termination*/(&self, &mut args);
        }
    }

    //pub fn on_worker_thread_termination(&self) -> &'a dyn CBVaultDriveWorkerThreadTerminationEvent
    pub fn on_worker_thread_termination(&self) -> Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveWorkerThreadTerminationEventArgs)>
    {
        self.onWorkerThreadTermination
    }

    //pub fn set_on_worker_thread_termination(&mut self, value : &'a dyn CBVaultDriveWorkerThreadTerminationEvent)
    pub fn set_on_worker_thread_termination(&mut self, value : Option<fn (sender : &CBVaultDrive, e : &mut CBVaultDriveWorkerThreadTerminationEventArgs)>)
    {
        self.onWorkerThreadTermination = value;
    }


    pub(crate) fn report_error_info(&self, error_info : &str)
    {
        if let Some(callable) = self.onError
        {
            let mut args : CBVaultDriveErrorEventArgs = CBVaultDriveErrorEventArgs
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
            let callable = CBFSVault_CBVaultDrive_GetLastError.get().unwrap();
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
            let callable = CBFSVault_CBVaultDrive_GetLastErrorCode.get().unwrap();
            result = callable(self.Handle) as i32;
        }
        result
    }

    // GetRuntimeLicense returns the runtime license key set for CBVaultDrive.
    pub fn get_runtime_license(&self) -> Result<String, CBFSVaultError>
    {
        let result : String;
        //let length : c_long;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

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

    // SetRuntimeLicense sets the runtime license key for CBVaultDrive.
    pub fn set_runtime_license(&self, value : String) -> Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let oem_key = CString::new(value).expect("Failed to create CString");
        let oem_key_ptr: *mut c_char = oem_key.into_raw();
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_SetCStr.get().unwrap();
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

    // Gets the value of the AccessDeniedProcessCount property: The number of records in the AccessDeniedProcess arrays.
    pub fn access_denied_process_count(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 1, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessDeniedProcessDesiredAccess property: The kind of access granted or denied.
    pub fn access_denied_process_desired_access(&self, AccessDeniedProcessIndex : i32) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 2, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessDesiredAccess"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 2, AccessDeniedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessDeniedProcessIncludeChildren property: Whether child processes are affected.
    pub fn access_denied_process_include_children(&self, AccessDeniedProcessIndex : i32) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 3, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessIncludeChildren"));
            }
        }
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 3, AccessDeniedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessDeniedProcessId property: The Id of the target process.
    pub fn access_denied_process_id(&self, AccessDeniedProcessIndex : i32) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 4, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessId"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 4, AccessDeniedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessDeniedProcessName property: The filename of the target process's executable.
    pub fn access_denied_process_name(&self, AccessDeniedProcessIndex : i32) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 5, AccessDeniedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value AccessDeniedProcessIndex for AccessDeniedProcessName"));
            }
        }
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 5, AccessDeniedProcessIndex as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the AccessGrantedProcessCount property: The number of records in the AccessGrantedProcess arrays.
    pub fn access_granted_process_count(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
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


    // Gets the value of the AccessGrantedProcessDesiredAccess property: The kind of access granted or denied.
    pub fn access_granted_process_desired_access(&self, AccessGrantedProcessIndex : i32) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 7, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessDesiredAccess"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 7, AccessGrantedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessGrantedProcessIncludeChildren property: Whether child processes are affected.
    pub fn access_granted_process_include_children(&self, AccessGrantedProcessIndex : i32) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 8, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessIncludeChildren"));
            }
        }
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 8, AccessGrantedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessGrantedProcessId property: The Id of the target process.
    pub fn access_granted_process_id(&self, AccessGrantedProcessIndex : i32) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 9, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessId"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 9, AccessGrantedProcessIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the AccessGrantedProcessName property: The filename of the target process's executable.
    pub fn access_granted_process_name(&self, AccessGrantedProcessIndex : i32) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 10, AccessGrantedProcessIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value AccessGrantedProcessIndex for AccessGrantedProcessName"));
            }
        }
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 10, AccessGrantedProcessIndex as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the Active property: Whether a vault has been opened and mounted as a virtual drive.
    pub fn active(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 11, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
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


    // Sets the value of the AutoCompactAt property: This property specifies the free space percentage threshold a vault must reach to be eligible for automatic compaction.
    pub fn set_auto_compact_at(&self, value : i32) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 12, 0 as c_long, value as isize, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 13, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 13, 0 as c_long, IntValue as isize, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 14, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 14, 0 as c_long, IntValue as isize, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 15, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
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

            let callable = CBFSVault_CBVaultDrive_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 15, 0 as c_long, cstrvalue_ptr, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 16, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 16, 0 as c_long, value as isize, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 17, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
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

            let callable = CBFSVault_CBVaultDrive_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 17, 0 as c_long, cstrvalue_ptr, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 18, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 18, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the FileSystemName property: The name of the virtual filesystem.
    pub fn file_system_name(&self) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 19, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the FileSystemName property: The name of the virtual filesystem.
    pub fn set_file_system_name(&self, value : &str) -> Result<(), CBFSVaultError> {
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

            let callable = CBFSVault_CBVaultDrive_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 19, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 20, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt64.get().unwrap();
            callable(self.Handle, 21, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
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
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 22, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
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

            let callable = CBFSVault_CBVaultDrive_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 22, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the MountingPointCount property: The number of records in the MountingPoint arrays.
    pub fn mounting_point_count(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 23, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the MountingPointAuthenticationId property: The Authentication ID used when creating the mounting point, if applicable.
    pub fn mounting_point_authentication_id(&self, MountingPointIndex : i32) -> Result<i64, CBFSVaultError>
    {
        let ret_val : i64; // = 0;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 24, MountingPointIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value MountingPointIndex for MountingPointAuthenticationId"));
            }
        }
        let mut val_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_GetAsInt64.get().unwrap();
            callable(self.Handle, 24, MountingPointIndex as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the MountingPointFlags property: The flags used to create the mounting point.
    pub fn mounting_point_flags(&self, MountingPointIndex : i32) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 25, MountingPointIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value MountingPointIndex for MountingPointFlags"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 25, MountingPointIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the MountingPointName property: The mounting point name.
    pub fn mounting_point_name(&self, MountingPointIndex : i32) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 26, MountingPointIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value MountingPointIndex for MountingPointName"));
            }
        }
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 26, MountingPointIndex as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the OpenFilesCount property: The number of records in the OpenFile arrays.
    pub fn open_files_count(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
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


    // Gets the value of the OpenFileName property: The name of the open file.
    pub fn open_file_name(&self, OpenFileIndex : i32) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 28, OpenFileIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value OpenFileIndex for OpenFileName"));
            }
        }
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 28, OpenFileIndex as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the OpenFileProcessId property: The Id of the process that opened the file.
    pub fn open_file_process_id(&self, OpenFileIndex : i32) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 29, OpenFileIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value OpenFileIndex for OpenFileProcessId"));
            }
        }
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 29, OpenFileIndex as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the OpenFileProcessName property: The name of the process that opened the file.
    pub fn open_file_process_name(&self, OpenFileIndex : i32) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_CheckIndex.get().unwrap();
            let ret_code = callable(self.Handle, 30, OpenFileIndex as c_long) as i32;
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(-1, "Invalid array index value OpenFileIndex for OpenFileProcessName"));
            }
        }
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 30, OpenFileIndex as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Gets the value of the PageSize property: This property specifies the vault's page size.
    pub fn page_size(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 31, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 31, 0 as c_long, value as isize, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 32, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 32, 0 as c_long, value as isize, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt64.get().unwrap();
            callable(self.Handle, 33, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt64.get().unwrap();
            callable(self.Handle, 34, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
        }
        ret_val = val_buf;
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Gets the value of the ProcessRestrictionsEnabled property: Whether process access restrictions are enabled (Windows and Linux).
    pub fn process_restrictions_enabled(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 35, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the ProcessRestrictionsEnabled property: Whether process access restrictions are enabled (Windows and Linux).
    pub fn set_process_restrictions_enabled(&self, value : bool) -> Result<(), CBFSVaultError> {
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 35, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the ReadOnly property: This property specifies whether the struct should open a vault in read-only mode.
    pub fn read_only(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 36, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 36, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the ReportPossibleSize property: How the struct should report the virtual drive's size and free space to the OS.
    pub fn report_possible_size(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 37, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the ReportPossibleSize property: How the struct should report the virtual drive's size and free space to the OS.
    pub fn set_report_possible_size(&self, value : bool) -> Result<(), CBFSVaultError> {
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 37, 0 as c_long, IntValue as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the SerializeEvents property: Whether events should be fired on a single worker thread, or many.
    pub fn serialize_events(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 38, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the SerializeEvents property: Whether events should be fired on a single worker thread, or many.
    pub fn set_serialize_events(&self, value : i32) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 38, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the StorageCharacteristics property: The characteristic flags to create the virtual drive with (Windows only).
    pub fn storage_characteristics(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 39, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the StorageCharacteristics property: The characteristic flags to create the virtual drive with (Windows only).
    pub fn set_storage_characteristics(&self, value : i32) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 39, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the StorageGUID property: The GUID to create the virtual drive with.
    pub fn storage_guid(&self) -> Result<String, CBFSVaultError>
    {
        let ret_val : String; // = String::default();
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 40, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
            let ret_code = self.get_last_error_code();
            if ret_code != 0
            {
                return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
            }
            ret_val = charptr_to_string(cptr).unwrap_or_else(|_| String::default());
            return Result::Ok(ret_val);
        }
    }


    // Sets the value of the StorageGUID property: The GUID to create the virtual drive with.
    pub fn set_storage_guid(&self, value : &str) -> Result<(), CBFSVaultError> {
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

            let callable = CBFSVault_CBVaultDrive_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 40, 0 as c_long, cstrvalue_ptr, 0) as i32;
            let _ = CString::from_raw(cstrvalue_ptr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the StorageType property: The type of virtual drive to create (Windows only).
    pub fn storage_type(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 41, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the StorageType property: The type of virtual drive to create (Windows only).
    pub fn set_storage_type(&self, value : i32) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 41, 0 as c_long, value as isize, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt64.get().unwrap();
            callable(self.Handle, 42, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
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
            let callable = CBFSVault_CBVaultDrive_SetInt64.get().unwrap();
            ret_code = callable(self.Handle, 42, 0 as c_long, ValuePtr as *const i64, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the Timeout property: How long vault events may execute before timing out.
    pub fn timeout(&self) -> Result<i32, CBFSVaultError>
    {
        let ret_val : i32; // = 0;
        unsafe
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 43, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val as i32;
        }
        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the Timeout property: How long vault events may execute before timing out.
    pub fn set_timeout(&self, value : i32) -> Result<(), CBFSVaultError> {
        let ret_code : i32;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 43, 0 as c_long, value as isize, 0) as i32;
        }
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(()); // no error occurred
    }


    // Gets the value of the UnmountOnTermination property: Whether the virtual drive should be unmounted if the application terminates (Windows only).
    pub fn unmount_on_termination(&self) -> Result<bool, CBFSVaultError>
    {
        let ret_val : bool; // = false;
        unsafe 
        {
            let val : usize;
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 44, 0 as c_long, std::ptr::null(), std::ptr::null());
            ret_val = val != 0;
        }

        let ret_code = self.get_last_error_code();
        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        Result::Ok(ret_val)
    }


    // Sets the value of the UnmountOnTermination property: Whether the virtual drive should be unmounted if the application terminates (Windows only).
    pub fn set_unmount_on_termination(&self, value : bool) -> Result<(), CBFSVaultError> {
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 44, 0 as c_long, IntValue as isize, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 45, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 45, 0 as c_long, IntValue as isize, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 46, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 46, 0 as c_long, IntValue as isize, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 47, 0 as c_long, std::ptr::null(), std::ptr::null());
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
            let callable = CBFSVault_CBVaultDrive_SetInt.get().unwrap();
            ret_code = callable(self.Handle, 47, 0 as c_long, value as isize, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 48, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
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

            let callable = CBFSVault_CBVaultDrive_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 48, 0 as c_long, cstrvalue_ptr, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt64.get().unwrap();
            callable(self.Handle, 49, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
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
            let callable = CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap();

            let cptr = callable(self.Handle, 50, 0 as c_long, std::ptr::null_mut(), std::ptr::null());
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

            let callable = CBFSVault_CBVaultDrive_SetCStr.get().unwrap();
            ret_code = callable(self.Handle, 50, 0 as c_long, cstrvalue_ptr, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt64.get().unwrap();
            callable(self.Handle, 51, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
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
            let callable = CBFSVault_CBVaultDrive_SetInt64.get().unwrap();
            ret_code = callable(self.Handle, 51, 0 as c_long, ValuePtr as *const i64, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt64.get().unwrap();
            callable(self.Handle, 52, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
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
            let callable = CBFSVault_CBVaultDrive_SetInt64.get().unwrap();
            ret_code = callable(self.Handle, 52, 0 as c_long, ValuePtr as *const i64, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt64.get().unwrap();
            callable(self.Handle, 53, 0 as c_long, std::ptr::null_mut(), &mut val_buf as *mut i64) as usize;
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
            let callable = CBFSVault_CBVaultDrive_SetInt64.get().unwrap();
            ret_code = callable(self.Handle, 53, 0 as c_long, ValuePtr as *const i64, 0) as i32;
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
            let callable = CBFSVault_CBVaultDrive_GetAsInt.get().unwrap();
            val = callable(self.Handle, 54, 0 as c_long, std::ptr::null(), std::ptr::null());
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

    // AddDeniedProcess: Adds a rule that prevents a process from accessing the virtual drive (Windows and Linux).
    pub fn add_denied_process(&self, process_file_name : &str, process_id : i32, child_processes : bool, desired_access : i32) ->  Result<(), CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 2, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProcessFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn add_denied_process

    // AddGrantedProcess: Adds a rule that allows a process to access the virtual drive (Windows and Linux).
    pub fn add_granted_process(&self, process_file_name : &str, process_id : i32, child_processes : bool, desired_access : i32) ->  Result<(), CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 3, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProcessFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn add_granted_process

    // AddMountingPoint: Adds a mounting point for the virtual drive.
    pub fn add_mounting_point(&self, mounting_point : &str, flags : i32, authentication_id : i64) ->  Result<(), CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 4, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrMountingPointPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn add_mounting_point

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 5, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 6, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 7, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 8, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn check_vault_password

    // CloseOpenedFilesSnapshot: Closes the previously-created opened files snapshot.
    pub fn close_opened_files_snapshot(&self, ) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 9, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn close_opened_files_snapshot

    // CloseVault: Closes the vault.
    pub fn close_vault(&self, force : bool) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let ForceBool : i32;
        if force
        {
            ForceBool = 1;
        } else {
            ForceBool = 0;
        }
        CParams[0] = ForceBool as usize;


        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 10, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 11, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 12, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrConfigurationStringPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn config

    // ConvertToDrivePath: Converts a vault-local vault item path to a virtual drive file path (Windows only).
    pub fn convert_to_drive_path(&self, vault_file_path : &str) ->  Result<String, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrVaultFilePathPtr : *mut c_char;
        match CString::new(vault_file_path)
        {
            Ok(CStrValueVaultFilePath) => { CStrVaultFilePathPtr = CStrValueVaultFilePath.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrVaultFilePathPtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 13, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrVaultFilePathPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn convert_to_drive_path

    // ConvertToVaultPath: Converts a virtual drive file path to a vault-local vault item path (Windows only).
    pub fn convert_to_vault_path(&self, virtual_file_path : &str) ->  Result<String, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrVirtualFilePathPtr : *mut c_char;
        match CString::new(virtual_file_path)
        {
            Ok(CStrValueVirtualFilePath) => { CStrVirtualFilePathPtr = CStrValueVirtualFilePath.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrVirtualFilePathPtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 14, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrVirtualFilePathPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn convert_to_vault_path

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 15, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 16, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrLinkNamePtr);
            let _ = CString::from_raw(CStrDestinationNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn create_link

    // CreateMemoryVault: Creates an in-memory vault (Windows only).
    pub fn create_memory_vault(&self, ) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 17, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn create_memory_vault

    // CreateOpenedFilesSnapshot: Creates a snapshot of information about files that are currently open.
    pub fn create_opened_files_snapshot(&self, ) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 18, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn create_opened_files_snapshot

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 19, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 20, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
            let _ = CString::from_raw(CStrTagNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn delete_file_tag

    // EjectVolume: Ejects a removable storage volume formatted with the CBFS Vault filesystem (Windows only).
    pub fn eject_volume(&self, force : bool) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let ForceBool : i32;
        if force
        {
            ForceBool = 1;
        } else {
            ForceBool = 0;
        }
        CParams[0] = ForceBool as usize;


        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 21, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn eject_volume

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 22, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 23, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 24, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 25, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 26, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 27, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 28, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 29, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 30, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn find_next

    // ForceUnmount: Forcefully unmounts the virtual drive associated with the specified vault (Windows only).
    pub fn force_unmount(&self, vault_file : &str) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrVaultFilePtr : *mut c_char;
        match CString::new(vault_file)
        {
            Ok(CStrValueVaultFile) => { CStrVaultFilePtr = CStrValueVaultFile.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrVaultFilePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 31, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrVaultFilePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn force_unmount

    // FormatVolume: Formats a storage volume or partition with the CBFS Vault filesystem (Windows only).
    pub fn format_volume(&self, volume_name : &str, flags : i32) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrVolumeNamePtr : *mut c_char;
        match CString::new(volume_name)
        {
            Ok(CStrValueVolumeName) => { CStrVolumeNamePtr = CStrValueVolumeName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrVolumeNamePtr as usize;
        CParams[1] = (flags as isize) as usize;

        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 32, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrVolumeNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn format_volume

    // GetDriverStatus: Retrieves the status of the system driver.
    pub fn get_driver_status(&self, product_guid : &str, module : i32) ->  Result<i32, CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 33, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProductGUIDPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] as i32;
        return Result::Ok(ret_val);
    } // fn get_driver_status

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 34, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 35, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 36, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 37, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 38, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = file_time_to_chrono_time(ll_buf);
        return Result::Ok(ret_val);
    } // fn get_file_last_access_time

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 39, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 40, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 41, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 42, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 43, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 44, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 45, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 46, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 47, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 48, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] as i32;
        return Result::Ok(ret_val);
    } // fn get_file_tag_size

    // GetModuleVersion: Retrieves the version of a given product module.
    pub fn get_module_version(&self, product_guid : &str, module : i32) ->  Result<i64, CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 49, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;

            let _ = CString::from_raw(CStrProductGUIDPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_module_version

    // GetOriginatorProcessId: Retrieves the Id of the process (PID) that initiated the operation (Windows and Linux).
    pub fn get_originator_process_id(&self, ) ->  Result<i32, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 50, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[0] as i32;
        return Result::Ok(ret_val);
    } // fn get_originator_process_id

    // GetOriginatorProcessName: Retrieves the name of the process that initiated the operation (Windows and Linux).
    pub fn get_originator_process_name(&self, ) ->  Result<String, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : String ; // = String::default();
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 51, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[0] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn get_originator_process_name

    // GetOriginatorThreadId: Retrieves the Id of the thread that initiated the operation (Windows only).
    pub fn get_originator_thread_id(&self, ) ->  Result<i32, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i32 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 52, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[0] as i32;
        return Result::Ok(ret_val);
    } // fn get_originator_thread_id

    // GetOriginatorToken: Retrieves the security token associated with the process that initiated the operation (Windows only).
    pub fn get_originator_token(&self, ) ->  Result<i64, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : i64 ; // = 0;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        let mut ll_buf : i64 = 0;
        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 53, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_originator_token

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 54, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 55, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 56, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 57, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 58, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 59, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 60, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 61, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[1] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn get_search_result_name

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 62, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = ll_buf;
        return Result::Ok(ret_val);
    } // fn get_search_result_size

    // Initialize: This method initializes the struct.
    pub fn initialize(&self, product_guid : &str) ->  Result<(), CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 63, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProductGUIDPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn initialize

    // Install: Installs (or upgrades) the product's system drivers and/or the helper DLL (Windows only).
    pub fn install(&self, cab_file_name : &str, product_guid : &str, path_to_install : &str, modules_to_install : i32, flags : i32) ->  Result<i32, CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 64, 5, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrCabFileNamePtr);
            let _ = CString::from_raw(CStrProductGUIDPtr);
            let _ = CString::from_raw(CStrPathToInstallPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[5] as i32;
        return Result::Ok(ret_val);
    } // fn install

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 65, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrDirectoryPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn is_directory_empty

    // IsIconRegistered: Checks whether the specified icon is registered (Windows only).
    pub fn is_icon_registered(&self, icon_id : &str) ->  Result<bool, CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 66, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrIconIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn is_icon_registered

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 67, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[0] != 0;
        return Result::Ok(ret_val);
    } // fn is_valid_vault

    // IsValidVaultVolume: Checks whether a storage partition or volume is formatted with the CBFS Vault filesystem (Windows only).
    pub fn is_valid_vault_volume(&self, volume_name : &str) ->  Result<bool, CBFSVaultError>
    {
        let ret_code : i32;
        let ret_val : bool ; // = false;
        let mut CParams : Vec<usize> = vec![0 as usize; 1 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0];

        let CStrVolumeNamePtr : *mut c_char;
        match CString::new(volume_name)
        {
            Ok(CStrValueVolumeName) => { CStrVolumeNamePtr = CStrValueVolumeName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrVolumeNamePtr as usize;

        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 68, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrVolumeNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[1] != 0;
        return Result::Ok(ret_val);
    } // fn is_valid_vault_volume

    // LoadMemoryVault: Loads contents of a file-based vault into the in-memory vault (Windows only).
    pub fn load_memory_vault(&self, file_name : &str) ->  Result<(), CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 69, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn load_memory_vault

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 70, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 71, 5, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 72, 11, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 73, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 74, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn open_vault

    // OpenVolume: Opens a storage volume or partition formatted with the CBFS Vault filesystem as a vault (Windows only).
    pub fn open_volume(&self, volume_name : &str, journaling_mode : i32) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 2 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0, 0, 0];

        let CStrVolumeNamePtr : *mut c_char;
        match CString::new(volume_name)
        {
            Ok(CStrValueVolumeName) => { CStrVolumeNamePtr = CStrValueVolumeName.into_raw(); }
            Err(_) => { panic!("Parameter conversion error"); }
        }
        CParams[0] = CStrVolumeNamePtr as usize;
        CParams[1] = (journaling_mode as isize) as usize;

        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 75, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrVolumeNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn open_volume

    // RegisterIcon: Registers an icon that can be displayed as an overlay on the virtual drive in Windows File Explorer (Windows only).
    pub fn register_icon(&self, icon_path : &str, product_guid : &str, icon_id : &str) ->  Result<bool, CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 76, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrIconPathPtr);
            let _ = CString::from_raw(CStrProductGUIDPtr);
            let _ = CString::from_raw(CStrIconIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[3] != 0;
        return Result::Ok(ret_val);
    } // fn register_icon

    // RemoveDeniedProcess: Removes a rule that prevents a process from accessing the virtual drive (Windows and Linux).
    pub fn remove_denied_process(&self, process_file_name : &str, process_id : i32) ->  Result<(), CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 77, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProcessFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn remove_denied_process

    // RemoveGrantedProcess: Removes a rule that allows a process to access the virtual drive (Windows and Linux).
    pub fn remove_granted_process(&self, process_file_name : &str, process_id : i32) ->  Result<(), CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 78, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProcessFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn remove_granted_process

    // RemoveMountingPoint: Removes a mounting point for the virtual drive.
    pub fn remove_mounting_point(&self, index : i32, mounting_point : &str, flags : i32, authentication_id : i64) ->  Result<(), CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 79, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrMountingPointPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn remove_mounting_point

    // ResetIcon: Resets the virtual drive's icon back to default by deselecting the active overlay icon (Windows only).
    pub fn reset_icon(&self, ) ->  Result<(), CBFSVaultError>
    {
        let ret_code : i32;
        let mut CParams : Vec<usize> = vec![0 as usize; 0 + 1];
        // let mut CParamPtr : UIntPtrArrayType = CParams.as_mut_ptr();
        let mut CBParams : Vec<i32> = vec![0];


        unsafe
        {
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 80, 0, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn reset_icon

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 81, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrLinkNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = charptr_to_string(CParams[2] as *const i8).unwrap_or_else(|_| String::default());
        return Result::Ok(ret_val);
    } // fn resolve_link

    // SaveMemoryVault: Copies contents of the in-memory vault into a file-based vault (Windows only).
    pub fn save_memory_vault(&self, file_name : &str) ->  Result<(), CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 82, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrFileNamePtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn save_memory_vault

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 83, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 84, 5, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 85, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 86, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 87, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 88, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 89, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 90, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 91, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 92, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 93, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 94, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 95, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

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

    // SetIcon: Selects a registered overlay icon for display on the virtual drive in Windows File Explorer (Windows only).
    pub fn set_icon(&self, icon_id : &str) ->  Result<(), CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 96, 1, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrIconIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn set_icon

    // ShutdownSystem: Shuts down or reboots the operating system.
    pub fn shutdown_system(&self, shutdown_prompt : &str, timeout : i32, force_close_apps : bool, reboot : bool) ->  Result<bool, CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 97, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrShutdownPromptPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[4] != 0;
        return Result::Ok(ret_val);
    } // fn shutdown_system

    // Uninstall: Uninstalls the product's system drivers and/or helper DLL (Windows only).
    pub fn uninstall(&self, cab_file_name : &str, product_guid : &str, installed_path : &str, flags : i32) ->  Result<i32, CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 98, 4, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrCabFileNamePtr);
            let _ = CString::from_raw(CStrProductGUIDPtr);
            let _ = CString::from_raw(CStrInstalledPathPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[4] as i32;
        return Result::Ok(ret_val);
    } // fn uninstall

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 99, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), &mut ll_buf as *mut i64) as i32;
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = file_time_to_chrono_time(ll_buf);
        return Result::Ok(ret_val);
    } // fn unix_time_to_file_time

    // UnregisterIcon: Unregisters an existing overlay icon (Windows only).
    pub fn unregister_icon(&self, product_guid : &str, icon_id : &str) ->  Result<bool, CBFSVaultError>
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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 100, 2, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrProductGUIDPtr);
            let _ = CString::from_raw(CStrIconIdPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        ret_val = CParams[2] != 0;
        return Result::Ok(ret_val);
    } // fn unregister_icon

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
            let callable = CBFSVault_CBVaultDrive_Do.get().unwrap();
            ret_code = callable(self.Handle, 101, 3, CParams.as_mut_ptr(), CBParams.as_mut_ptr(), std::ptr::null_mut() as *mut i64) as i32;

            let _ = CString::from_raw(CStrOldPasswordPtr);
            let _ = CString::from_raw(CStrNewPasswordPtr);
        }

        if ret_code != 0
        {
            return Result::Err(CBFSVaultError::new(ret_code, self.get_last_error().as_str()));
        }

        return Result::Ok(());
    } // fn update_vault_encryption

} // CBVaultDrive

extern "system" fn CBVaultDriveEventDispatcher(pObj : usize, event_id : c_long, _cparam : c_long, param : IntPtrArrayType, cbparam : IntArrayType) -> c_long
{
    let obj: &'static CBVaultDrive;
    // Lock the Mutex to get access to the HashMap
    let ptr = {
        let map = CBVaultDriveDict.lock().unwrap();
         if let Some(boxed) = map.get(&pObj) {
            &**boxed as *const CBVaultDrive
        } else {
            return -1;
        }
    };
    unsafe { obj = &*ptr; }

    let event_result: Result<(), Box<dyn std::any::Any + Send>> = catch_unwind (||
    {
        match event_id
        {
            1 /* DataCompress */=> obj.fire_data_compress(/*cparam as i32, */param, cbparam),

            2 /* DataDecompress */=> obj.fire_data_decompress(/*cparam as i32, */param, cbparam),

            3 /* DataDecrypt */=> obj.fire_data_decrypt(/*cparam as i32, */param, cbparam),

            4 /* DataEncrypt */=> obj.fire_data_encrypt(/*cparam as i32, */param, cbparam),

            5 /* Ejected */=> obj.fire_ejected(/*cparam as i32, */param, cbparam),

            6 /* Error */=> obj.fire_error(/*cparam as i32, */param, cbparam),

            7 /* FileAccess */=> obj.fire_file_access(/*cparam as i32, */param, cbparam),

            8 /* FileAfterCopy */=> obj.fire_file_after_copy(/*cparam as i32, */param, cbparam),

            9 /* FileBeforeCopy */=> obj.fire_file_before_copy(/*cparam as i32, */param, cbparam),

            10 /* FilePasswordNeeded */=> obj.fire_file_password_needed(/*cparam as i32, */param, cbparam),

            11 /* HashCalculate */=> obj.fire_hash_calculate(/*cparam as i32, */param, cbparam),

            12 /* KeyDerive */=> obj.fire_key_derive(/*cparam as i32, */param, cbparam),

            13 /* Progress */=> obj.fire_progress(/*cparam as i32, */param, cbparam),

            14 /* VaultClose */=> obj.fire_vault_close(/*cparam as i32, */param, cbparam),

            15 /* VaultDelete */=> obj.fire_vault_delete(/*cparam as i32, */param, cbparam),

            16 /* VaultFlush */=> obj.fire_vault_flush(/*cparam as i32, */param, cbparam),

            17 /* VaultGetParentSize */=> obj.fire_vault_get_parent_size(/*cparam as i32, */param, cbparam),

            18 /* VaultGetSize */=> obj.fire_vault_get_size(/*cparam as i32, */param, cbparam),

            19 /* VaultOpen */=> obj.fire_vault_open(/*cparam as i32, */param, cbparam),

            20 /* VaultRead */=> obj.fire_vault_read(/*cparam as i32, */param, cbparam),

            21 /* VaultSetSize */=> obj.fire_vault_set_size(/*cparam as i32, */param, cbparam),

            22 /* VaultWrite */=> obj.fire_vault_write(/*cparam as i32, */param, cbparam),

            23 /* WorkerThreadCreation */=> obj.fire_worker_thread_creation(/*cparam as i32, */param, cbparam),

            24 /* WorkerThreadTermination */=> obj.fire_worker_thread_termination(/*cparam as i32, */param, cbparam),

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


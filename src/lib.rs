//! Rust wrapper of VirtDisk APIs
//!
//! # Overview
//!
//! This project is a collection of Rust libraries that wrap functionality exposed by [VirtDisk](https://docs.microsoft.com/en-us/windows/desktop/api/virtdisk/).
//!
//! VirtDisk APIs are part of the [Windows 10 SDK](https://developer.microsoft.com/en-us/windows/downloads/windows-10-sdk).
//!
//! # Requirements
//!
//! For this wrapper to build properly, the following requirements need to be met by the building machine:
//!
//! - Windows 10 SDK version **10.0.17763.132**.
//! - **amd64** architecture.
//!   - This Rust wrapper, for now, expects to build only in amd64.
//!
//! # Wrapped Windows 10 SDK APIs
//!
//! **_Note: This section includes the paths in the Windows SDK for the header and lib files based on the default installation path `c:\Program Files (x86)\Windows Kits\10`._**
//!
//! The relevant Windows 10 SDK files that this project is wrapping are:
//! - C:\Program Files (x86)\Windows Kits\10\Include\10.0.17763.0\um\virtdisk.h
//! - C:\Program Files (x86)\Windows Kits\10\Lib\10.0.17763.0\um\x64\virtdisk.lib
//! - C:\Windows\System32\virtdisk.dll
//!

pub mod virtdisk;
pub (crate) mod virtdisk_bindings;
pub mod virtdiskdefs;

pub mod diskutilities;
pub mod vhdutilities;

#[allow(dead_code)]
pub(crate) mod winutilities;

pub mod windefs {
    //! Defines type aliases for Windows Definitions to user Rust naming conventions
    //! throughout the crate.

    pub type Bool = winapi::shared::minwindef::BOOL;
    pub type Boolean = winapi::shared::ntdef::BOOLEAN;
    pub type Byte = winapi::shared::minwindef::BYTE;
    pub type ULong = winapi::shared::minwindef::ULONG;
    pub type UShort = winapi::shared::minwindef::USHORT;
    pub type UInt = winapi::shared::minwindef::UINT;
    pub type ULongPtr = winapi::shared::basetsd::ULONG_PTR;
    pub type DWord = winapi::shared::minwindef::DWORD;
    pub type DWordLong = winapi::shared::ntdef::DWORDLONG;
    pub type LongLong = winapi::shared::ntdef::LONGLONG;
    pub type LargeInteger = winapi::shared::ntdef::LARGE_INTEGER;
    pub type Handle = winapi::shared::ntdef::HANDLE;
    pub type PCWStr = winapi::shared::ntdef::PCWSTR;
    pub type PWStr = winapi::shared::ntdef::PWSTR;
    pub type UChar = winapi::shared::ntdef::UCHAR;
    pub type Void = winapi::shared::ntdef::VOID;
    pub type PVoid = winapi::shared::ntdef::PVOID;
    pub type LPVoid = winapi::shared::minwindef::LPVOID;
    pub type WChar = winapi::shared::ntdef::WCHAR;
    pub type Word = winapi::shared::minwindef::WORD;

    pub type Guid = winapi::shared::guiddef::GUID;
    pub type Acl = winapi::um::winnt::ACL;
    pub type SecurityDescriptor = winapi::um::winnt::SECURITY_DESCRIPTOR;
    pub type Overlapped = winapi::um::minwinbase::OVERLAPPED;

    pub const GUID_NULL: Guid = Guid {
        Data1: 0x00000000,
        Data2: 0x0000,
        Data3: 0x0000,
        Data4: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    };
}

/// Enumeration of common error codes returned from the virtdisk APIs.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ResultCode {
    Success,
    InvalidParameter,
    UnsupportedCompression,
    FileEncrypted,
    FileSystemLimitation,
    FileCorrupt,
    FileNotFound,
    InsufficientBuffer,
    Timeout,
    GenFailure,
    WindowsErrorCode(windefs::DWord),
}

pub(crate) fn error_code_to_result_code(error_code: windefs::DWord) -> ResultCode {
    match error_code {
        0 => ResultCode::Success,
        87 => ResultCode::InvalidParameter,
        618 => ResultCode::UnsupportedCompression,
        6002 => ResultCode::FileEncrypted,
        665 => ResultCode::FileSystemLimitation,
        1392 => ResultCode::FileCorrupt,
        2 => ResultCode::FileNotFound,
        122 => ResultCode::InsufficientBuffer,
        1460 => ResultCode::Timeout,
        31 => ResultCode::GenFailure,
        error_code => ResultCode::WindowsErrorCode(error_code),
    }
}

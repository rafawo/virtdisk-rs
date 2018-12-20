//! Rust wrapper of VirtDisk APIs
//!
//! # Overview
//!
//! This project is a collection of Rust libraries that wrap functionality exposed by [VirtDisk](https://docs.microsoft.com/en-us/windows/desktop/api/virtdisk/).
//!
//! VirtDisk APIs are part of the [Windows 10 SDK](https://developer.microsoft.com/en-us/windows/downloads/windows-10-sdk).
//!
//! Both the FFI bindings and Rust wrappers are public to this crate, to give flexibility
//! to consumer code to use the bindings directly as they see fit or the rust wrappers for safe abstractions.
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
pub mod virtdisk_bindings;
pub mod virtdiskdefs;

pub mod windefs {
    //! Defines type aliases for Windows Definitions to user Rust naming conventions
    //! throughout the crate.

    pub type Bool = winapi::shared::minwindef::BOOL;
    pub type Byte = winapi::shared::minwindef::BYTE;
    pub type DWord = winapi::shared::minwindef::DWORD;
    pub type Handle = winapi::shared::ntdef::HANDLE;
    pub type PCWStr = winapi::shared::ntdef::PCWSTR;
    pub type PWStr = winapi::shared::ntdef::PWSTR;
    pub type UChar = winapi::shared::ntdef::UCHAR;
    pub type Void = winapi::shared::ntdef::VOID;
    pub type WChar = winapi::shared::ntdef::WCHAR;
    pub type Word = winapi::shared::minwindef::WORD;

    pub type Guid = winapi::shared::guiddef::GUID;
    pub type Acl = winapi::um::winnt::ACL;
    pub type SecurityDescriptor = winapi::um::winnt::SECURITY_DESCRIPTOR;
    pub type Overlapped = winapi::um::minwinbase::OVERLAPPED;
}

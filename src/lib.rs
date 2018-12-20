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
    use libc;

    pub type Bool = libc::c_int;
    pub type Byte = libc::c_uchar;
    pub type DWord = libc::c_ulong;
    pub type Handle = *mut libc::c_void;
    pub type PCWStr = *const libc::wchar_t;
    pub type PWStr = *mut libc::wchar_t;
    pub type PSId = *mut Void;
    pub type SecurityDescriptorControl = libc::c_ushort;
    pub type Word = libc::c_ushort;
    pub type Void = libc::c_void;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Guid {
        pub data1: libc::c_ulong,
        pub data2: libc::c_ushort,
        pub data3: libc::c_ushort,
        pub data4: [libc::c_uchar; 8],
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Acl {
        pub acl_revision: Byte,
        pub sbz1: Byte,
        pub acl_size: Word,
        pub ace_count: Word,
        pub sbz2: Word,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct SecurityDescriptor {
        pub revision: Byte,
        pub sbz1: Byte,
        pub control: SecurityDescriptorControl,
        pub owner: PSId,
        pub group: PSId,
        pub sacl: *mut Acl,
        pub dacl: *mut Acl,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct OverlappedPointerDetails {
        pub offset: DWord,
        pub offset_high: DWord,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union OverlappedPointer {
        pub details: OverlappedPointerDetails,
        pub pointer: *mut Void,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Overlapped {
        pub internal: *mut u64,
        pub internal_high: *mut u64,
        pub pointer: OverlappedPointer,
        pub h_event: Handle,
    }
}

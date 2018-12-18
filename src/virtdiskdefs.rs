//! This module contains the type definitions used by the VirtDisk APIs.

use crate::windefs::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VirtualStorageType {
    device_id: u64,
    vendor_id: Guid,
}

/// {00000000-0000-0000-0000-000000000000}
pub const VIRTUAL_STORAGE_TYPE_VENDOR_UNKNOWN: Guid = Guid {
    data1: 0x00000000,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
};

/// {EC984AEC-A0F9-47e9-901F-71415A66345B}
pub const VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT: Guid = Guid {
    data1: 0xec984aec,
    data2: 0xa0f9,
    data3: 0x47e9,
    data4: [0x90, 0x1f, 0x71, 0x41, 0x5a, 0x66, 0x34, 0x5b],
};

pub const VIRTUAL_STORAGE_TYPE_DEVICE_UNKNOWN: u32 = 0;
pub const VIRTUAL_STORAGE_TYPE_DEVICE_ISO: u32 = 1;
pub const VIRTUAL_STORAGE_TYPE_DEVICE_VHD: u32 = 2;
pub const VIRTUAL_STORAGE_TYPE_DEVICE_VHDX: u32 = 3;
pub const VIRTUAL_STORAGE_TYPE_DEVICE_VHDSET: u32 = 4;

/// The default RW Depth parameter value
pub const OPEN_VIRTUAL_DISK_RW_DEPTH_DEFAULT: u32 = 1;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OpenVirtualDiskVersion {
    VersionUnspecified = 0,
    Version1 = 1,
    Version2 = 2,
    Version3 = 3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OpenVirtualDiskVersion1 {
    rw_depth: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OpenVirtualDiskVersion2 {
    get_info_only: bool,
    read_only: bool,
    resiliency_guid: Guid,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OpenVirtualDiskVersion3 {
    get_info_only: bool,
    read_only: bool,
    resiliency_guid: Guid,
    snapshot_id: Guid,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union OpenVirtualDiskVersionDetails {
    version1: OpenVirtualDiskVersion1,
    version2: OpenVirtualDiskVersion2,
    version3: OpenVirtualDiskVersion3,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OpenVirtualDiskParameters {
    version: OpenVirtualDiskVersion,
    version_details: OpenVirtualDiskVersionDetails,
}

/// Access Mask for OpenVirtualDisk and CreateVirtualDisk.  The virtual
/// disk drivers expose file objects as handles therefore we map
/// it into that AccessMask space.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VirtualDiskAccessMask {
    None = 0x00000000,
    AttachRo = 0x00010000,
    AttachRw = 0x00020000,
    AccessDetach = 0x00040000,
    GetInfo = 0x00080000,
    Create = 0x00100000,
    MetaOps = 0x00200000,
    Read = 0x000d0000,
    All = 0x003f0000,

    /// A special flag to be used to test if the virtual disk needs to be
    /// opened for write.
    Writable = 0x00320000,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OpenVirtualDiskFlag {
    None = 0x00000000,

    /// Open the backing store without opening any differencing chain parents.
    /// This allows one to fixup broken parent links.
    NoParents = 0x00000001,

    /// The backing store being opened is an empty file. Do not perform virtual
    /// disk verification.
    BlankFile = 0x00000002,

    /// This flag is only specified at boot time to load the system disk
    /// during virtual disk boot.  Must be kernel mode to specify this flag.
    BootDrive = 0x00000004,

    /// This flag causes the backing file to be opened in cached mode.
    CachedIo = 0x00000008,

    /// Open the backing store without opening any differencing chain parents.
    /// This allows one to fixup broken parent links temporarily without updating
    /// the parent locator.
    CustomDiffChain = 0x00000010,

    /// This flag causes all backing stores except the leaf backing store to
    /// be opened in cached mode.
    ParentCachedIo = 0x00000020,

    /// This flag causes a Vhd Set file to be opened without any virtual disk.
    VhdsetFileOnly = 0x00000040,

    /// For differencing disks, relative parent locators are not used when
    /// determining the path of a parent VHD.
    IgnoreRelativeParentLocator = 0x00000080,

    /// Disable flushing and FUA (both for payload data and for metadata)
    /// for backing files associated with this virtual disk.
    NoWriteHardening = 0x00000100,
}

/// This value causes the implementation defaults to be used for block size:
///
/// Fixed VHDs: 0 is the only valid value since block size is N/A.
/// Dynamic VHDs: The default block size will be used (2 MB, subject to change).
/// Differencing VHDs: 0 causes the parent VHD's block size to be used.
///
pub const CREATE_VIRTUAL_DISK_PARAMETERS_DEFAULT_BLOCK_SIZE: u32 = 0;

/// Default logical sector size is 512B
pub const CREATE_VIRTUAL_DISK_PARAMETERS_DEFAULT_SECTOR_SIZE: u32 = 0;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CreateVirtualDiskVersion {
    VersionUnspecified = 0,
    Version1 = 1,
    Version2 = 2,
    Version3 = 3,
    Version4 = 4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CreateVirtualDiskVersion1 {
    unique_id: Guid,
    maximum_size: u64,
    block_size_in_bytes: u64,
    sector_size_in_bytes: u64,
    parent_path: PCWStr,
    source_path: PCWStr,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CreateVirtualDiskVersion2 {
    unique_id: Guid,
    maximum_size: u64,
    block_size_in_bytes: u64,
    sector_size_in_bytes: u64,
    parent_path: PCWStr,
    source_path: PCWStr,
    open_flags: OpenVirtualDiskFlag,
}

//! This module contains the type definitions used by the VirtDisk APIs.

use crate::windefs::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VirtualStorageType {
    pub device_id: u64,
    pub vendor_id: Guid,
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
    pub rw_depth: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OpenVirtualDiskVersion2 {
    pub get_info_only: bool,
    pub read_only: bool,
    pub resiliency_guid: Guid,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OpenVirtualDiskVersion3 {
    pub get_info_only: bool,
    pub read_only: bool,
    pub resiliency_guid: Guid,
    pub snapshot_id: Guid,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union OpenVirtualDiskVersionDetails {
    pub version1: OpenVirtualDiskVersion1,
    pub version2: OpenVirtualDiskVersion2,
    pub version3: OpenVirtualDiskVersion3,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OpenVirtualDiskParameters {
    pub version: OpenVirtualDiskVersion,
    pub version_details: OpenVirtualDiskVersionDetails,
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
    pub unique_id: Guid,
    pub maximum_size: u64,
    pub block_size_in_bytes: u64,
    pub sector_size_in_bytes: u64,
    pub parent_path: PCWStr,
    pub source_path: PCWStr,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CreateVirtualDiskVersion2 {
    pub unique_id: Guid,
    pub maximum_size: u64,
    pub block_size_in_bytes: u64,
    pub sector_size_in_bytes: u64,
    pub parent_path: PCWStr,
    pub source_path: PCWStr,
    pub open_flags: u32, // OpenVirtualDiskFlag
    pub parent_virtual_storage_type: VirtualStorageType,
    pub source_virtual_storage_type: VirtualStorageType,
    pub resiliency_guid: Guid,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CreateVirtualDiskVersion3 {
    pub unique_id: Guid,
    pub maximum_size: u64,
    pub block_size_in_bytes: u64,
    pub sector_size_in_bytes: u64,
    pub parent_path: PCWStr,
    pub source_path: PCWStr,
    pub open_flags: u32, // OpenVirtualDiskFlag
    pub parent_virtual_storage_type: VirtualStorageType,
    pub source_virtual_storage_type: VirtualStorageType,
    pub resiliency_guid: Guid,
    pub source_limit_path: PCWStr,
    pub backing_storage_type: VirtualStorageType,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CreateVirtualDiskVersion4 {
    pub unique_id: Guid,
    pub maximum_size: u64,
    pub block_size_in_bytes: u64,
    pub sector_size_in_bytes: u64,
    pub parent_path: PCWStr,
    pub source_path: PCWStr,
    pub open_flags: u32, // OpenVirtualDiskFlag
    pub parent_virtual_storage_type: VirtualStorageType,
    pub source_virtual_storage_type: VirtualStorageType,
    pub resiliency_guid: Guid,
    pub source_limit_path: PCWStr,
    pub backing_storage_type: VirtualStorageType,
    pub pmem_address_abstraction_type: Guid,
    pub data_alignment: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union CreateVirtualDiskVersionDetails {
    pub version1: CreateVirtualDiskVersion1,
    pub version2: CreateVirtualDiskVersion2,
    pub version3: CreateVirtualDiskVersion3,
    pub version4: CreateVirtualDiskVersion4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CreateVirtualDiskParameters {
    pub version: CreateVirtualDiskVersion,
    pub version_details: CreateVirtualDiskVersionDetails,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CreateVirtualDiskFlag {
    None = 0x0,

    /// Pre-allocate all physical space necessary for the virtual
    /// size of the disk (e.g. a fixed VHD).
    FullPhysicalAllocation = 0x1,

    /// Take ownership of the source disk during create from source disk, to
    /// insure the source disk does not change during the create operation.  The
    /// source disk must also already be offline or read-only (or both).
    /// Ownership is released when create is done.  This also has a side-effect
    /// of disallowing concurrent create from same source disk.  Create will fail
    /// if ownership cannot be obtained or if the source disk is not already
    /// offline or read-only.  This flag is optional, but highly recommended for
    /// creates from source disk.  No effect for other types of create (no effect
    /// for create from source VHD; no effect for create without SourcePath).
    PreventWritesToSourceDisk = 0x2,

    /// Do not copy initial virtual disk metadata or block states from the
    /// parent VHD; this is useful if the parent VHD is a stand-in file and the
    /// real parent will be explicitly set later.
    DoNotCopyMetadataFromParent = 0x4,

    /// Create the backing storage disk.
    CreateBackingStore = 0x8,

    /// If set, the SourceLimitPath is an change tracking ID, and all data that has changed
    /// since that change tracking ID will be copied from the source. If clear, the
    /// SourceLimitPath is a VHD file path in the source VHD's chain, and
    /// all data that is present in the children of that VHD in the chain
    /// will be copied from the source.
    UseChangeTrackingSourceLimit = 0x10,

    /// If set and the parent VHD has change tracking enabled, the child will
    /// have change tracking enabled and will recognize all change tracking
    /// IDs that currently exist in the parent. If clear or if the parent VHD
    /// does not have change tracking available, then change tracking will
    /// not be enabled in the new VHD.
    PreserveParentChangeTrackingState = 0x20,

    /// When creating a VHD Set from source, don't copy the data in the original
    /// backing store, but intsead use the file as is. If this flag is not specified
    /// and a source file is passed to CreateVirtualDisk for a VHDSet file, the data
    /// in the source file is copied. If this flag is set the data is moved. The
    /// name of the file may change.
    VhdSetUseOriginalBackingStorage = 0x40,

    /// When creating a fixed virtual disk, take advantage of an underlying sparse file.
    /// Only supported on file systems that support sparse VDLs.
    SparseFile = 0x80,

    /// Creates a VHD suitable as the backing store for a virtual persistent memory device.
    PmemCompatible = 0x100,
}

pub const CREATE_VIRTUAL_DISK_FLAG_USE_RCT_SOURCE_LIMIT: u32 = CreateVirtualDiskFlag::UseChangeTrackingSourceLimit as u32;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AttachVirtualDiskVersion {
    Unspecified = 0,
    Version1 = 1,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AttachVirtualDiskVersion1 {
    pub reserved: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union AttachVirtualDiskVersionDetails {
    pub version1: AttachVirtualDiskVersion1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AttachVirtualDiskParameters {
    pub version: AttachVirtualDiskVersion,
    pub version_details: AttachVirtualDiskVersionDetails,
}


#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AttachVirtualDiskFlag {
    None = 0x00000000,

    /// Attach the disk as read only
    ReadOnly = 0x00000001,

    /// Will cause all volumes on the disk to be mounted
    /// without drive letters.
    NoDriveLetter = 0x00000002,

    /// Will decouple the disk lifetime from that of the VirtualDiskHandle.
    /// The disk will be attached until an explicit call is made to
    /// DetachVirtualDisk, even if all handles are closed.
    PermanentLifetime = 0x00000004,

    /// Indicates that the drive will not be attached to
    /// the local system (but rather to a VM).
    NoLocalHost = 0x00000008,

    /// Do not assign a custom security descriptor to the disk; use the
    /// system default.
    NoSecurityDescriptor = 0x00000010,

    /// Default volume encryption policies should not be applied to the
    /// disk when attached to the local system.
    BypassDefaultEncryptionPolicy = 0x00000020,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DetachVirtualDiskFlag {
    None = 0x00000000,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DependentDiskFlag {
    None = 0x00000000,

    /// Multiple files backing the virtual storage device
    MultipleBackingFiles = 0x00000001,

    FullyAllocated = 0x00000002,
    ReadOnly = 0x00000004,

    /// Backing file of the virtual storage device is not local to the machine
    Remote = 0x00000008,

    /// Volume is the system volume
    SystemVolume = 0x00000010,

    /// Volume backing the virtual storage device file is the system volume
    SystemVolumeParent = 0x00000020,

    Removable = 0x00000040,

    /// Drive letters are not assigned to the volumes
    /// on the virtual disk automatically.
    NoDriveLetter = 0x00000080,

    Parent = 0x00000100,

    /// Virtual disk is not attached on the local host
    /// (instead attached on a guest VM for instance)
    NoHostDisk = 0x00000200,

    /// Indicates the lifetime of the disk is not tied
    /// to any system handles
    PermanentLifetime = 0x00000400,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StorageDependencyInfoVersion {
    Unspecified = 0,
    Version1 = 1,
    Version2 = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct StorageDependencyInfoVersion1 {
    pub dependency_type_flags: u64, // DependentDiskFlag
    pub provider_specific_flags: u64,
    pub virtual_storage_type: VirtualStorageType,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct StorageDependencyInfoVersion2 {
    pub dependency_type_flags: u64, // DependentDiskFlag
    pub provider_specific_flags: u64,
    pub virtual_storage_type: VirtualStorageType,
    pub ancestor_level: u64,
    pub dependency_device_name: PWStr,
    pub host_volume_name: PWStr,
    pub dependent_volume_name: PWStr,
    pub dependent_volume_relative_path: PWStr,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union StorageDependencyInfoVersionDetails {
    pub version1: StorageDependencyInfoVersion1,
    pub version2: StorageDependencyInfoVersion2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StorageDependencyInfo {
    pub version: StorageDependencyInfoVersion,
    pub number_entries: u64,
    pub version_details: *mut StorageDependencyInfoVersionDetails,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GetStorageDependencyFlag {
    None = 0x00000000,

    /// Return information for volumes or disks hosting the volume specified
    /// If not set, returns info about volumes or disks being hosted by
    /// the volume or disk specified
    HostVolumes = 0x00000001,

    /// The handle provided is to a disk, not volume or file
    DiskHandle = 0x00000002,
}

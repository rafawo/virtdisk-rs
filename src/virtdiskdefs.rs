//! This module contains the type definitions used by the VirtDisk APIs.

use crate::windefs::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VirtualStorageType {
    pub device_id: u32,
    pub vendor_id: Guid,
}

/// {00000000-0000-0000-0000-000000000000}
pub const VIRTUAL_STORAGE_TYPE_VENDOR_UNKNOWN: Guid = Guid {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
};

/// {EC984AEC-A0F9-47e9-901F-71415A66345B}
pub const VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT: Guid = Guid {
    Data1: 0xec984aec,
    Data2: 0xa0f9,
    Data3: 0x47e9,
    Data4: [0x90, 0x1f, 0x71, 0x41, 0x5a, 0x66, 0x34, 0x5b],
};

pub const VIRTUAL_STORAGE_TYPE_DEVICE_UNKNOWN: u32 = 0;
pub const VIRTUAL_STORAGE_TYPE_DEVICE_ISO: u32 = 1;
pub const VIRTUAL_STORAGE_TYPE_DEVICE_VHD: u32 = 2;
pub const VIRTUAL_STORAGE_TYPE_DEVICE_VHDX: u32 = 3;
pub const VIRTUAL_STORAGE_TYPE_DEVICE_VHDSET: u32 = 4;

/// Access Mask for OpenVirtualDisk and CreateVirtualDisk. The virtual
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

pub mod open_virtual_disk {
    use super::*;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        VersionUnspecified = 0,
        Version1 = 1,
        Version2 = 2,
        Version3 = 3,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Version1 {
        pub rw_depth: u32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Version2 {
        pub get_info_only: Bool,
        pub read_only: Bool,
        pub resiliency_guid: Guid,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Version3 {
        pub get_info_only: Bool,
        pub read_only: Bool,
        pub resiliency_guid: Guid,
        pub snapshot_id: Guid,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
        pub version2: Version2,
        pub version3: Version3,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
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
    pub const PARAMETERS_DEFAULT_BLOCK_SIZE: u32 = 0;

    /// Default logical sector size is 512B
    pub const PARAMETERS_DEFAULT_SECTOR_SIZE: u32 = 0;

    /// The default RW Depth parameter value
    pub const RW_DEPTH_DEFAULT: u32 = 1;
}

pub mod create_virtual_disk {
    use super::*;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        VersionUnspecified = 0,
        Version1 = 1,
        Version2 = 2,
        Version3 = 3,
        Version4 = 4,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Version1 {
        pub unique_id: Guid,
        pub maximum_size: u64,
        pub block_size_in_bytes: u32,
        pub sector_size_in_bytes: u32,
        pub parent_path: PCWStr,
        pub source_path: PCWStr,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Version2 {
        pub unique_id: Guid,
        pub maximum_size: u64,
        pub block_size_in_bytes: u32,
        pub sector_size_in_bytes: u32,
        pub parent_path: PCWStr,
        pub source_path: PCWStr,
        pub open_flags: u32, // OpenVirtualDiskFlag
        pub parent_virtual_storage_type: VirtualStorageType,
        pub source_virtual_storage_type: VirtualStorageType,
        pub resiliency_guid: Guid,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Version3 {
        pub unique_id: Guid,
        pub maximum_size: u64,
        pub block_size_in_bytes: u32,
        pub sector_size_in_bytes: u32,
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
    #[derive(Copy, Clone)]
    pub struct Version4 {
        pub unique_id: Guid,
        pub maximum_size: u64,
        pub block_size_in_bytes: u32,
        pub sector_size_in_bytes: u32,
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
    pub union VersionDetails {
        pub version1: Version1,
        pub version2: Version2,
        pub version3: Version3,
        pub version4: Version4,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
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

    pub const FLAG_USE_RCT_SOURCE_LIMIT: u32 = Flag::UseChangeTrackingSourceLimit as u32;
}

pub mod attach_virtual_disk {
    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        Version1 = 1,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Version1 {
        pub reserved: u32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
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
}

pub mod detach_virtual_disk {
    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
    }
}

pub mod storage_dependency {
    use super::*;

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
    pub enum InfoVersion {
        Unspecified = 0,
        Version1 = 1,
        Version2 = 2,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct InfoVersion1 {
        pub dependency_type_flags: u32, // DependentDiskFlag
        pub provider_specific_flags: u32,
        pub virtual_storage_type: VirtualStorageType,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct InfoVersion2 {
        pub dependency_type_flags: u32, // DependentDiskFlag
        pub provider_specific_flags: u32,
        pub virtual_storage_type: VirtualStorageType,
        pub ancestor_level: u32,
        pub dependency_device_name: PWStr,
        pub host_volume_name: PWStr,
        pub dependent_volume_name: PWStr,
        pub dependent_volume_relative_path: PWStr,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union InfoVersionDetails {
        pub version1: InfoVersion1,
        pub version2: InfoVersion2,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Info {
        pub version: InfoVersion,
        pub number_entries: u32,
        pub version_details: *mut InfoVersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum GetFlag {
        None = 0x00000000,

        /// Return information for volumes or disks hosting the volume specified
        /// If not set, returns info about volumes or disks being hosted by
        /// the volume or disk specified
        HostVolumes = 0x00000001,

        /// The handle provided is to a disk, not volume or file
        DiskHandle = 0x00000002,
    }
}

pub mod get_virtual_disk {
    use super::*;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum InfoVersion {
        Unspecified = 0,
        Size = 1,
        Identifier = 2,
        ParentLocation = 3,
        ParentIdentifier = 4,
        ParentTimeStamp = 5,
        VirtualStorageType = 6,
        ProviderSubType = 7,
        Is4KAligned = 8,
        PhysicalDisk = 9,
        VhdPhysicalSectorSize = 10,
        SmallestSafeVirtualSize = 11,
        Fragmentation = 12,
        IsLoaded = 13,
        VirtualDiskId = 14,
        ChangeTrackingState = 15,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct InfoSize {
        pub virtual_size: u64,
        pub physical_size: u64,
        pub block_size: u32,
        pub sector_size: u32,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct InfoParentLocation {
        pub parent_resolved: Bool,
        pub parent_location_buffer: [WChar; 1],
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct InfoPhysicalDisk {
        pub logical_sector_size: u32,
        pub physical_sector_size: u32,
        pub is_remote: Bool,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct InfoChangeTrackingState {
        pub enabled: Bool,
        pub newer_changes: Bool,
        pub most_recent_id: [WChar; 1],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union InfoVersionDetails {
        pub size: InfoSize,
        pub parent_location: InfoParentLocation,
        pub parent_identifier: Guid,
        pub parent_time_stamp: u32,
        pub virtual_storage_type: VirtualStorageType,
        pub provider_sub_type: u32,
        pub is_4k_aligned: Bool,
        pub is_loaded: Bool,
        pub physical_disk: InfoPhysicalDisk,
        pub vhd_physical_sector_size: u32,
        pub smallest_safe_virtual_size: u64,
        pub fragmentation_percentage: u32,
        pub virtual_disk_id: Guid,
        pub change_tracking_state: InfoChangeTrackingState,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Info {
        pub version: InfoVersion,
        pub version_details: InfoVersionDetails,
    }
}

pub const VIRTUAL_DISK_MAXIMUM_CHANGE_TRACKING_ID_LENGTH: u32 = 256;

pub mod set_virtual_disk {
    use super::*;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum InfoVersion {
        Unspecified = 0,
        ParentPath = 1,
        Identifier = 2,
        ParentPathWithDepth = 3,
        PhysicalSectorSize = 4,
        VirtualDiskId = 5,
        ChangeTrackingState = 6,
        ParentLocator = 7,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct InfoParentPathWithDepthInfo {
        pub child_depth: u32,
        pub parent_file_path: PCWStr,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct InfoParentLocator {
        pub linkage_id: Guid,
        pub parent_file_path: PCWStr,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union InfoVersionDetails {
        pub parent_file_path: PCWStr,
        pub unique_identifier: Guid,
        pub parent_with_depth_info: InfoParentPathWithDepthInfo,
        pub vhd_physical_sector_size: u32,
        pub virtual_disk_id: Guid,
        pub change_tracking_enabled: Bool,
        pub parent_locator: InfoParentLocator,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Info {
        pub version: InfoVersion,
        pub version_details: InfoVersionDetails,
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VirtualDiskProgress {
    pub operation_status: DWord,
    pub current_value: u64,
    pub completion_value: u64,
}

pub mod compact_virtual_disk {
    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        Version1 = 1,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Version1 {
        pub reserved: u32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
        NoZeroScan = 0x00000001,
        NoBlockMoves = 0x00000002,
    }
}

pub mod merge_virtual_disk {
    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        Version1 = 1,
        Version2 = 2,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Version1 {
        pub merge_depth: u32,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Version2 {
        pub merge_source_path: u32,
        pub merge_target_path: u32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
        pub version2: Version2,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
    }
}

pub mod expand_virtual_disk {
    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        Version1 = 1,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Version1 {
        pub new_size: u64,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
    }
}

pub mod resize_virtual_disk {
    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        Version1 = 1,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Version1 {
        pub new_size: u64,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x0,

        /// If this flag is set, skip checking the virtual disk's partition table
        /// to ensure that this truncation is safe. Setting this flag can cause
        /// unrecoverable data loss; use with care.
        AllowUnsafeVirtualSize = 0x1,

        /// If this flag is set, resize the disk to the smallest virtual size
        /// possible without truncating past any existing partitions. If this
        /// is set, NewSize in RESIZE_VIRTUAL_DISK_PARAMETERS must be zero.
        ResizeToSmallestSafeVirtualSize = 0x2,
    }
}

pub mod mirror_virtual_disk {
    use crate::windefs::*;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        Version1 = 1,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Version1 {
        pub mirror_virtual_disk_path: PCWStr,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
        ExistingFile = 0x00000001,
        SkipMirrorActivation = 0x00000002,
    }
}

pub mod query_changes_virtual_disk {
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Range {
        pub byte_offset: u64,
        pub byte_length: u64,
        pub reserved: u64,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
    }
}

pub mod take_snapshot_vhdset {
    use crate::windefs::*;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        Version1 = 1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Version1 {
        pub snapshot_id: Guid,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
        Writable = 0x00000001,
    }
}

pub mod delete_snapshot_vhdset {
    use crate::windefs::*;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        Version1 = 1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Version1 {
        pub snapshot_id: Guid,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
        PersistRct = 0x00000001,
    }
}

pub mod modify_vhdset {
    use crate::windefs::*;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        SnapshotPath = 1,
        RemoveSnapshot = 2,
        DefaultSnapshotPath = 3,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct SnapshotPath {
        pub snapshot_id: Guid,
        pub snapshot_file_path: PCWStr,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub snapshot_path: SnapshotPath,
        pub snapshot_id: Guid,
        pub default_file_path: PCWStr,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
        WritableSnapshot = 0x00000001,
    }
}

pub mod apply_snapshot_vhdset {
    use crate::windefs::*;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        Version1 = 1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Version1 {
        pub snapshot_id: Guid,
        pub leaf_snapshot_id: Guid,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
        Writable = 0x00000001,
    }
}

pub mod raw_scsi_virtual_disk {
    use crate::windefs::*;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        Version1 = 1,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Version1 {
        pub rsvd_handle: Bool,
        pub data_in: UChar,
        pub cdb_length: UChar,
        pub sense_info_length: UChar,
        pub srb_flags: u32,
        pub data_transfer_length: u32,
        pub data_buffer: *mut Void,
        pub sense_info: *mut UChar,
        pub cdb: *mut UChar,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct ResponseVersion1 {
        pub scsi_status: UChar,
        pub sense_info_length: UChar,
        pub data_transfer_length: u32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union ResponseVersionDetails {
        pub version1: ResponseVersion1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Response {
        pub version: Version,
        pub version_details: ResponseVersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
    }
}

pub mod fork_virtual_disk {
    use crate::windefs::*;

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Version {
        Unspecified = 0,
        Version1 = 1,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct Version1 {
        pub forked_virtual_disk_path: PCWStr,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union VersionDetails {
        pub version1: Version1,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Parameters {
        pub version: Version,
        pub version_details: VersionDetails,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Flag {
        None = 0x00000000,
        ExistingFile = 0x00000001,
    }
}

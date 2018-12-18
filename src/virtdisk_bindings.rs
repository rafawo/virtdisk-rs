//! This module contains the C bindings to the VirtDisk APIs.

use crate::virtdiskdefs::*;
use crate::windefs::*;

#[link(name = "virtdisk")]
extern "C" {
    pub fn OpenVirtualDisk(
        virtualStorageType: *const VirtualStorageType,
        path: PCWStr,
        virtualDiskAccessMask: VirtualDiskAccessMask,
        flags: u32, // OpenVirtualDiskFlag
        parameters: *const OpenVirtualDiskParameters,
        handle: Handle,
    ) -> DWord;

    pub fn CreateVirtualDisk(
        virtualStorageType: *const VirtualStorageType,
        path: PCWStr,
        virtualDiskAccessMask: VirtualDiskAccessMask,
        securityDescriptor: *const SecurityDescriptor,
        flags: u32, // CreateVirtualDiskFlag
        providerSpecificFlags: u64,
        parameters: *const CreateVirtualDiskParameters,
        overlapped: *const Overlapped,
        handle: Handle,
    ) -> DWord;

    pub fn AttachVirtualDisk(
        virtualDiskHandle: Handle,
        securityDescriptor: *const SecurityDescriptor,
        flags: u32, // AttachVirtualDiskFlag
        providerSpecificFlags: u64,
        parameters: *const AttachVirtualDiskParameters,
        overlapped: *const Overlapped,
    ) -> DWord;

    pub fn DetachVirtualDisk(
        virtualDiskHandle: Handle,
        flags: u32, // DetachVirtualDiskFlag
        providerSpecificFlags: u64,
    ) -> DWord;

    pub fn GetVirtualDiskPhysicalPath(
        virtualDiskHandle: Handle,
        diskPathSizeInBytes: *mut u64,
        diskPath: PWStr,
    ) -> DWord;

    pub fn GetAllAttachedVirtualDiskPhysicalPaths(
        pathsBufferSizeInBytes: *const u64,
        pathsBuffer: PWStr,
    ) -> DWord;

    pub fn GetStorageDependencyInformation(
        objectHandle: Handle,
        flags: u32, // GetStorageDependencyFlag
        storageDependencyInfoSize: u64,
        storageDependencyInfo: *const StorageDependencyInfo,
        sizeUsed: *const u64
    ) -> DWord;

    pub fn GetVirtualDiskInformation(
        virtualDiskHandle: Handle,
        virtualDiskInfoSize: *const u64,
        virtualDiskInfo: *mut GetVirtualDiskInfo,
        sizeUsed: *mut u64,
    ) -> DWord;

    pub fn SetVirtualDiskInformation(
        virtualDiskHandle: Handle,
        virtualDiskInfo: *const SetVirtualDiskInfo,
    ) -> DWord;

    pub fn EnumerateVirtualDiskMetadata(
        virtualDiskHandle: Handle,
        numberOfItems: *mut u64,
        items: *mut Guid,
    ) -> DWord;

    pub fn GetVirtualDiskMetadata(
        VirtualDiskHandle: Handle,
        item: *const Guid,
        metaDataSize: *mut u64,
        metaData: *mut Void,
    ) -> DWord;

    pub fn SetVirtualDiskMetadata(
        virtualDiskHandle: Handle,
        item: *const Guid,
        metaDataSize: u64,
        metaData: *const Void,
    ) -> DWord;

    pub fn DeleteVirtualDiskMetadata(
        virtualDiskHandle: Handle,
        item: *const Guid,
    ) -> DWord;

} /*

//
// GetVirtualDiskOperationProgress
//

typedef struct _VIRTUAL_DISK_PROGRESS
{
    DWord      OperationStatus;
    ULONGLONG  CurrentValue;
    ULONGLONG  CompletionValue;
} VIRTUAL_DISK_PROGRESS, *PVIRTUAL_DISK_PROGRESS;


DWord WINAPI
GetVirtualDiskOperationProgress(
    _In_      HANDLE                 VirtualDiskHandle,
    _In_      LPOVERLAPPED           Overlapped,
    _Out_     PVIRTUAL_DISK_PROGRESS Progress
    ) -> DWord;



//
// CompactVirtualDisk
//

// Version definitions
typedef enum _COMPACT_VIRTUAL_DISK_VERSION
{
    COMPACT_VIRTUAL_DISK_VERSION_UNSPECIFIED    = 0,
    COMPACT_VIRTUAL_DISK_VERSION_1              = 1,

} COMPACT_VIRTUAL_DISK_VERSION;


// Versioned structure for CompactVirtualDisk
typedef struct _COMPACT_VIRTUAL_DISK_PARAMETERS
{
    COMPACT_VIRTUAL_DISK_VERSION Version;

    union
    {
        struct
        {
            ULONG Reserved;
        } Version1;
    };
} COMPACT_VIRTUAL_DISK_PARAMETERS, *PCOMPACT_VIRTUAL_DISK_PARAMETERS;


// Flags for CompactVirtualDisk
typedef enum _COMPACT_VIRTUAL_DISK_FLAG
{
    COMPACT_VIRTUAL_DISK_FLAG_NONE                 = 0x00000000,
    COMPACT_VIRTUAL_DISK_FLAG_NO_ZERO_SCAN         = 0x00000001,
    COMPACT_VIRTUAL_DISK_FLAG_NO_BLOCK_MOVES       = 0x00000002,

} COMPACT_VIRTUAL_DISK_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(COMPACT_VIRTUAL_DISK_FLAG) -> DWord;
#endif

DWord
WINAPI
CompactVirtualDisk(
    _In_     HANDLE                           VirtualDiskHandle,
    _In_     COMPACT_VIRTUAL_DISK_FLAG        Flags,
    _In_opt_ PCOMPACT_VIRTUAL_DISK_PARAMETERS Parameters,
    _In_opt_ LPOVERLAPPED                     Overlapped
    ) -> DWord;



//
// MergeVirtualDisk
//

// Version definitions
typedef enum _MERGE_VIRTUAL_DISK_VERSION
{
    MERGE_VIRTUAL_DISK_VERSION_UNSPECIFIED    = 0,
    MERGE_VIRTUAL_DISK_VERSION_1              = 1,
    MERGE_VIRTUAL_DISK_VERSION_2              = 2,

} MERGE_VIRTUAL_DISK_VERSION;



// Versioned parameter structure for MergeVirtualDisk
#define MERGE_VIRTUAL_DISK_DEFAULT_MERGE_DEPTH 1

typedef struct _MERGE_VIRTUAL_DISK_PARAMETERS
{
    MERGE_VIRTUAL_DISK_VERSION Version;

    union
    {
        struct
        {
            ULONG MergeDepth;
        } Version1;
        struct
        {
            ULONG MergeSourceDepth;
            ULONG MergeTargetDepth;
        } Version2;
    };
} MERGE_VIRTUAL_DISK_PARAMETERS, *PMERGE_VIRTUAL_DISK_PARAMETERS;


// Flags for MergeVirtualDisk
typedef enum _MERGE_VIRTUAL_DISK_FLAG
{
    MERGE_VIRTUAL_DISK_FLAG_NONE                 = 0x00000000,

} MERGE_VIRTUAL_DISK_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(MERGE_VIRTUAL_DISK_FLAG) -> DWord;
#endif

DWord
WINAPI
MergeVirtualDisk(
    _In_     HANDLE                         VirtualDiskHandle,
    _In_     MERGE_VIRTUAL_DISK_FLAG        Flags,
    _In_     PMERGE_VIRTUAL_DISK_PARAMETERS Parameters,
    _In_opt_ LPOVERLAPPED                   Overlapped
    ) -> DWord;



//
// ExpandVirtualDisk
//

// Version definitions
typedef enum _EXPAND_VIRTUAL_DISK_VERSION
{
    EXPAND_VIRTUAL_DISK_VERSION_UNSPECIFIED    = 0,
    EXPAND_VIRTUAL_DISK_VERSION_1              = 1,

} EXPAND_VIRTUAL_DISK_VERSION;


// Versioned parameter structure for ExpandVirtualDisk
typedef struct _EXPAND_VIRTUAL_DISK_PARAMETERS
{
    EXPAND_VIRTUAL_DISK_VERSION Version;

    union
    {
        struct
        {
            ULONGLONG NewSize;
        } Version1;
    };
} EXPAND_VIRTUAL_DISK_PARAMETERS, *PEXPAND_VIRTUAL_DISK_PARAMETERS;


// Flags for ExpandVirtualDisk
typedef enum _EXPAND_VIRTUAL_DISK_FLAG
{
    EXPAND_VIRTUAL_DISK_FLAG_NONE                 = 0x00000000,

} EXPAND_VIRTUAL_DISK_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(EXPAND_VIRTUAL_DISK_FLAG) -> DWord;
#endif

DWord
WINAPI
ExpandVirtualDisk(
    _In_     HANDLE                          VirtualDiskHandle,
    _In_     EXPAND_VIRTUAL_DISK_FLAG        Flags,
    _In_     PEXPAND_VIRTUAL_DISK_PARAMETERS Parameters,
    _In_opt_ LPOVERLAPPED                    Overlapped
    ) -> DWord;


//
// ResizeVirtualDisk
//

// Version definitions
typedef enum _RESIZE_VIRTUAL_DISK_VERSION
{
    RESIZE_VIRTUAL_DISK_VERSION_UNSPECIFIED    = 0,
    RESIZE_VIRTUAL_DISK_VERSION_1              = 1,

} RESIZE_VIRTUAL_DISK_VERSION;


// Versioned parameter structure for ResizeVirtualDisk
typedef struct _RESIZE_VIRTUAL_DISK_PARAMETERS
{
    RESIZE_VIRTUAL_DISK_VERSION Version;

    union
    {
        struct
        {
            ULONGLONG NewSize;
        } Version1;
    };
} RESIZE_VIRTUAL_DISK_PARAMETERS, *PRESIZE_VIRTUAL_DISK_PARAMETERS;


// Flags for ResizeVirtualDisk
typedef enum _RESIZE_VIRTUAL_DISK_FLAG
{
    RESIZE_VIRTUAL_DISK_FLAG_NONE                                 = 0x0,

    // If this flag is set, skip checking the virtual disk's partition table
    // to ensure that this truncation is safe. Setting this flag can cause
    // unrecoverable data loss; use with care.
    RESIZE_VIRTUAL_DISK_FLAG_ALLOW_UNSAFE_VIRTUAL_SIZE            = 0x1,

    // If this flag is set, resize the disk to the smallest virtual size
    // possible without truncating past any existing partitions. If this
    // is set, NewSize in RESIZE_VIRTUAL_DISK_PARAMETERS must be zero.
    RESIZE_VIRTUAL_DISK_FLAG_RESIZE_TO_SMALLEST_SAFE_VIRTUAL_SIZE = 0x2,

} RESIZE_VIRTUAL_DISK_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(RESIZE_VIRTUAL_DISK_FLAG) -> DWord;
#endif

#if (NTDDI_VERSION >= NTDDI_WIN8)

DWord
WINAPI
ResizeVirtualDisk(
    _In_     HANDLE                          VirtualDiskHandle,
    _In_     RESIZE_VIRTUAL_DISK_FLAG        Flags,
    _In_     PRESIZE_VIRTUAL_DISK_PARAMETERS Parameters,
    _In_opt_ LPOVERLAPPED                    Overlapped
    ) -> DWord;

#endif // NTDDI_VERSION >= NTDDI_WIN8

//
// MirrorVirtualDisk
//

// Version definitions
typedef enum _MIRROR_VIRTUAL_DISK_VERSION
{
    MIRROR_VIRTUAL_DISK_VERSION_UNSPECIFIED    = 0,
    MIRROR_VIRTUAL_DISK_VERSION_1              = 1,

} MIRROR_VIRTUAL_DISK_VERSION;


// Versioned parameter structure for MirrorVirtualDisk
typedef struct _MIRROR_VIRTUAL_DISK_PARAMETERS
{
    MIRROR_VIRTUAL_DISK_VERSION Version;

    union
    {
        struct
        {
            PCWSTR MirrorVirtualDiskPath;
        } Version1;
    };
} MIRROR_VIRTUAL_DISK_PARAMETERS, *PMIRROR_VIRTUAL_DISK_PARAMETERS;


// Flags for MirrorVirtualDisk
typedef enum _MIRROR_VIRTUAL_DISK_FLAG
{
    MIRROR_VIRTUAL_DISK_FLAG_NONE                   = 0x00000000,
    MIRROR_VIRTUAL_DISK_FLAG_EXISTING_FILE          = 0x00000001,
    MIRROR_VIRTUAL_DISK_FLAG_SKIP_MIRROR_ACTIVATION = 0x00000002

} MIRROR_VIRTUAL_DISK_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(MIRROR_VIRTUAL_DISK_FLAG) -> DWord;
#endif

#if (NTDDI_VERSION >= NTDDI_WIN8)

DWord
WINAPI
MirrorVirtualDisk(
    _In_     HANDLE                          VirtualDiskHandle,
    _In_     MIRROR_VIRTUAL_DISK_FLAG        Flags,
    _In_     PMIRROR_VIRTUAL_DISK_PARAMETERS Parameters,
    _In_     LPOVERLAPPED                    Overlapped
    ) -> DWord;

#endif // NTDDI_VERSION >= NTDDI_WIN8


//
// BreakMirrorVirtualDisk
//

#if (NTDDI_VERSION >= NTDDI_WIN8)

DWord
WINAPI
BreakMirrorVirtualDisk(
    _In_ HANDLE VirtualDiskHandle
    ) -> DWord;

#endif // NTDDI_VERSION >= NTDDI_WIN8

//
// AddVirtualDiskParent
//

#if (NTDDI_VERSION >= NTDDI_WIN8)

DWord
WINAPI
AddVirtualDiskParent(
    _In_ HANDLE VirtualDiskHandle,
    _In_ PCWSTR ParentPath
    ) -> DWord;

#endif // NTDDI_VERSION >= NTDDI_WIN8

typedef struct _QUERY_CHANGES_VIRTUAL_DISK_RANGE {
    ULONG64 ByteOffset;
    ULONG64 ByteLength;
    ULONG64 Reserved;
} QUERY_CHANGES_VIRTUAL_DISK_RANGE, *PQUERY_CHANGES_VIRTUAL_DISK_RANGE;

// Flags for QueryChangesVirtualDisk
typedef enum _QUERY_CHANGES_VIRTUAL_DISK_FLAG
{
    QUERY_CHANGES_VIRTUAL_DISK_FLAG_NONE          = 0x00000000,

} QUERY_CHANGES_VIRTUAL_DISK_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(QUERY_CHANGES_VIRTUAL_DISK_FLAG) -> DWord;
#endif

typedef enum _TAKE_SNAPSHOT_VHDSET_FLAG
{
    TAKE_SNAPSHOT_VHDSET_FLAG_NONE          = 0x00000000,
    TAKE_SNAPSHOT_VHDSET_FLAG_WRITEABLE     = 0x00000001,

} TAKE_SNAPSHOT_VHDSET_FLAG, *PTAKE_SNAPSHOT_VHDSET_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(TAKE_SNAPSHOT_VHDSET_FLAG) -> DWord;
#endif

typedef enum _TAKE_SNAPSHOT_VHDSET_VERSION
{
    TAKE_SNAPSHOT_VHDSET_VERSION_UNSPECIFIED = 0,
    TAKE_SNAPSHOT_VHDSET_VERSION_1           = 1,

} TAKE_SNAPSHOT_VHDSET_VERSION;

typedef struct _TAKE_SNAPSHOT_VHDSET_PARAMETERS
{
    TAKE_SNAPSHOT_VHDSET_VERSION Version;

    union
    {
        struct
        {
            GUID SnapshotId;
        } Version1;
    };
} TAKE_SNAPSHOT_VHDSET_PARAMETERS, *PTAKE_SNAPSHOT_VHDSET_PARAMETERS;

typedef enum _DELETE_SNAPSHOT_VHDSET_FLAG
{
    DELETE_SNAPSHOT_VHDSET_FLAG_NONE           = 0x00000000,
    DELETE_SNAPSHOT_VHDSET_FLAG_PERSIST_RCT    = 0x00000001,

} DELETE_SNAPSHOT_VHDSET_FLAG, *PDELETE_SNAPSHOT_VHDSET_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(DELETE_SNAPSHOT_VHDSET_FLAG) -> DWord;
#endif

typedef enum _DELETE_SNAPSHOT_VHDSET_VERSION
{
    DELETE_SNAPSHOT_VHDSET_VERSION_UNSPECIFIED = 0,
    DELETE_SNAPSHOT_VHDSET_VERSION_1           = 1,

} DELETE_SNAPSHOT_VHDSET_VERSION;

typedef struct _DELETE_SNAPSHOT_VHDSET_PARAMETERS
{
    DELETE_SNAPSHOT_VHDSET_VERSION Version;

    union
    {
        struct
        {
            GUID SnapshotId;
        } Version1;
    };
} DELETE_SNAPSHOT_VHDSET_PARAMETERS, *PDELETE_SNAPSHOT_VHDSET_PARAMETERS;

typedef enum _MODIFY_VHDSET_VERSION
{
    MODIFY_VHDSET_UNSPECIFIED              = 0,
    MODIFY_VHDSET_SNAPSHOT_PATH            = 1,
    MODIFY_VHDSET_REMOVE_SNAPSHOT          = 2,
    MODIFY_VHDSET_DEFAULT_SNAPSHOT_PATH    = 3,

} MODIFY_VHDSET_VERSION, *PMODIFY_VHDSET_VERSION;

typedef enum _MODIFY_VHDSET_FLAG
{
    MODIFY_VHDSET_FLAG_NONE               = 0x00000000,
    MODIFY_VHDSET_FLAG_WRITEABLE_SNAPSHOT = 0x00000001,

} MODIFY_VHDSET_FLAG, *PMODIFY_VHDSET_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(MODIFY_VHDSET_FLAG) -> DWord;
#endif

typedef struct _MODIFY_VHDSET_PARAMETERS
{
    MODIFY_VHDSET_VERSION Version;

    union
    {
        struct
        {
            GUID SnapshotId;
            PCWSTR SnapshotFilePath;
        } SnapshotPath;

        GUID SnapshotId;

        PCWSTR DefaultFilePath;
    };
} MODIFY_VHDSET_PARAMETERS, *PMODIFY_VHDSET_PARAMETERS;

typedef enum _APPLY_SNAPSHOT_VHDSET_FLAG
{
    APPLY_SNAPSHOT_VHDSET_FLAG_NONE      = 0x00000000,
    APPLY_SNAPSHOT_VHDSET_FLAG_WRITEABLE = 0x00000001,

} APPLY_SNAPSHOT_VHDSET_FLAG, *PAPPLY_SNAPSHOT_VHDSET_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(APPLY_SNAPSHOT_VHDSET_FLAG) -> DWord;
#endif

typedef enum _APPLY_SNAPSHOT_VHDSET_VERSION
{
    APPLY_SNAPSHOT_VHDSET_VERSION_UNSPECIFIED = 0,
    APPLY_SNAPSHOT_VHDSET_VERSION_1           = 1,

} APPLY_SNAPSHOT_VHDSET_VERSION;

typedef struct _APPLY_SNAPSHOT_VHDSET_PARAMETERS
{
    APPLY_SNAPSHOT_VHDSET_VERSION Version;

    union
    {
        struct
        {
            GUID SnapshotId;
            GUID LeafSnapshotId;
        } Version1;
    };

} APPLY_SNAPSHOT_VHDSET_PARAMETERS, *PAPPLY_SNAPSHOT_VHDSET_PARAMETERS;

typedef enum _RAW_SCSI_VIRTUAL_DISK_FLAG
{
    RAW_SCSI_VIRTUAL_DISK_FLAG_NONE   = 0X00000000

} RAW_SCSI_VIRTUAL_DISK_FLAG, *PRAW_SCSI_VIRTUAL_DISK_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(RAW_SCSI_VIRTUAL_DISK_FLAG) -> DWord;
#endif

typedef enum _RAW_SCSI_VIRTUAL_DISK_VERSION
{
    RAW_SCSI_VIRTUAL_DISK_VERSION_UNSPECIFIED = 0,
    RAW_SCSI_VIRTUAL_DISK_VERSION_1           = 1,

} RAW_SCSI_VIRTUAL_DISK_VERSION;

typedef struct _RAW_SCSI_VIRTUAL_DISK_PARAMETERS
{
    RAW_SCSI_VIRTUAL_DISK_VERSION Version;

    union
    {
        struct
        {
            BOOL    RSVDHandle;
            UCHAR   DataIn;

            UCHAR   CdbLength;
            UCHAR   SenseInfoLength;
            ULONG   SrbFlags;
            ULONG   DataTransferLength;

            _Field_size_bytes_full_(DataTransferLength)
            PVOID   DataBuffer;

            _Field_size_bytes_full_(SenseInfoLength)
            UCHAR*  SenseInfo;

            _Field_size_bytes_full_(CdbLength)
            UCHAR*  Cdb;
        } Version1;
    };

} RAW_SCSI_VIRTUAL_DISK_PARAMETERS, *PRAW_SCSI_VIRTUAL_DISK_PARAMETERS;


typedef struct _RAW_SCSI_VIRTUAL_DISK_RESPONSE
{
    RAW_SCSI_VIRTUAL_DISK_VERSION Version;

    union
    {
        struct
        {
            UCHAR ScsiStatus;
            UCHAR SenseInfoLength;    // bytes transferred to SenseInfo pointed to by the parameters.
            ULONG DataTransferLength; // bytes transferred to DataBuffer pointed to by the parameters.
        } Version1;
    };

} RAW_SCSI_VIRTUAL_DISK_RESPONSE, *PRAW_SCSI_VIRTUAL_DISK_RESPONSE;


#if (NTDDI_VERSION >= NTDDI_WINTHRESHOLD)

DWord
WINAPI
QueryChangesVirtualDisk (
    _In_ HANDLE VirtualDiskHandle,
    _In_ PCWSTR ChangeTrackingId,
    _In_ ULONG64 ByteOffset,
    _In_ ULONG64 ByteLength,
    _In_ QUERY_CHANGES_VIRTUAL_DISK_FLAG Flags,
    _Out_writes_to_(*RangeCount, *RangeCount) PQUERY_CHANGES_VIRTUAL_DISK_RANGE Ranges,
    _Inout_ PULONG RangeCount,
    _Out_ PULONG64 ProcessedLength
    ) -> DWord;

DWord
WINAPI
TakeSnapshotVhdSet (
    _In_ HANDLE VirtualDiskHandle,
    _In_ const PTAKE_SNAPSHOT_VHDSET_PARAMETERS Parameters,
    _In_ TAKE_SNAPSHOT_VHDSET_FLAG Flags
    ) -> DWord;

DWord
WINAPI
DeleteSnapshotVhdSet (
    _In_ HANDLE VirtualDiskHandle,
    _In_ const PDELETE_SNAPSHOT_VHDSET_PARAMETERS Parameters,
    _In_ DELETE_SNAPSHOT_VHDSET_FLAG Flags
    ) -> DWord;

DWord
WINAPI
ModifyVhdSet (
    _In_ HANDLE VirtualDiskHandle,
    _In_ const PMODIFY_VHDSET_PARAMETERS Parameters,
    _In_ MODIFY_VHDSET_FLAG Flags
    ) -> DWord;

DWord
WINAPI
ApplySnapshotVhdSet (
    _In_ HANDLE VirtualDiskHandle,
    _In_ const PAPPLY_SNAPSHOT_VHDSET_PARAMETERS Parameters,
    _In_ APPLY_SNAPSHOT_VHDSET_FLAG Flags
    ) -> DWord;

DWord
WINAPI
RawSCSIVirtualDisk(
    _In_ HANDLE VirtualDiskHandle,
    _In_ const PRAW_SCSI_VIRTUAL_DISK_PARAMETERS Parameters,
    _In_ RAW_SCSI_VIRTUAL_DISK_FLAG Flags,
    _Out_ PRAW_SCSI_VIRTUAL_DISK_RESPONSE Response
    ) -> DWord;


#endif // NTDDI_VERSION >= NTDDI_WINTHRESHOLD

#if (NTDDI_VERSION >= NTDDI_WIN10_RS5)

//
// ForkVirtualDisk
//

// Version definitions
typedef enum _FORK_VIRTUAL_DISK_VERSION
{
    FORK_VIRTUAL_DISK_VERSION_UNSPECIFIED = 0,
    FORK_VIRTUAL_DISK_VERSION_1           = 1,

} FORK_VIRTUAL_DISK_VERSION;

// Versioned parameter structure for ForkVirtualDisk
typedef struct _FORK_VIRTUAL_DISK_PARAMETERS
{
    FORK_VIRTUAL_DISK_VERSION Version;

    union
    {
        struct
        {
            PCWSTR ForkedVirtualDiskPath;

        } Version1;
    };

} FORK_VIRTUAL_DISK_PARAMETERS, *PFORK_VIRTUAL_DISK_PARAMETERS;

// Flags for ForkVirtualDisk
typedef enum _FORK_VIRTUAL_DISK_FLAG
{
    FORK_VIRTUAL_DISK_FLAG_NONE          = 0x00000000,
    FORK_VIRTUAL_DISK_FLAG_EXISTING_FILE = 0x00000001,

} FORK_VIRTUAL_DISK_FLAG;

#ifdef DEFINE_ENUM_FLAG_OPERATORS
DEFINE_ENUM_FLAG_OPERATORS(FORK_VIRTUAL_DISK_FLAG) -> DWord;
#endif

DWord
WINAPI
ForkVirtualDisk(
    _In_ HANDLE VirtualDiskHandle,
    _In_ FORK_VIRTUAL_DISK_FLAG Flags,
    _In_ const FORK_VIRTUAL_DISK_PARAMETERS* Parameters,
    _Inout_ LPOVERLAPPED Overlapped
    ) -> DWord;

DWord
WINAPI
CompleteForkVirtualDisk(
    _In_ HANDLE VirtualDiskHandle
    ) -> DWord;

}
*/

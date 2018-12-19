//! This module contains the C bindings to the VirtDisk APIs.

use crate::virtdiskdefs::*;
use crate::windefs::*;

#[link(name = "virtdisk")]
extern "C" {
    pub fn OpenVirtualDisk(
        virtualStorageType: *const VirtualStorageType,
        path: PCWStr,
        virtualDiskAccessMask: VirtualDiskAccessMask,
        flags: u32, // open_virtual_disk::Flag
        parameters: *const open_virtual_disk::Parameters,
        handle: *mut Handle,
    ) -> DWord;

    pub fn CreateVirtualDisk(
        virtualStorageType: *const VirtualStorageType,
        path: PCWStr,
        virtualDiskAccessMask: VirtualDiskAccessMask,
        securityDescriptor: *const SecurityDescriptor,
        flags: u32, // create_virtual_disk::Flag
        providerSpecificFlags: u64,
        parameters: *const create_virtual_disk::Parameters,
        overlapped: *const Overlapped,
        handle: *mut Handle,
    ) -> DWord;

    pub fn AttachVirtualDisk(
        virtualDiskHandle: Handle,
        securityDescriptor: *const SecurityDescriptor,
        flags: u32, // attach_virtual_disk::Flag
        providerSpecificFlags: u64,
        parameters: *const attach_virtual_disk::Parameters,
        overlapped: *const Overlapped,
    ) -> DWord;

    pub fn DetachVirtualDisk(
        virtualDiskHandle: Handle,
        flags: u32, // detach_virtual_disk::Flag
        providerSpecificFlags: u64,
    ) -> DWord;

    pub fn GetVirtualDiskPhysicalPath(
        virtualDiskHandle: Handle,
        diskPathSizeInBytes: *const u64,
        diskPath: PWStr,
    ) -> DWord;

    pub fn GetAllAttachedVirtualDiskPhysicalPaths(
        pathsBufferSizeInBytes: *const u64,
        pathsBuffer: PWStr,
    ) -> DWord;

    pub fn GetStorageDependencyInformation(
        objectHandle: Handle,
        flags: u32, // storage_dependency::GetFlag
        storageDependencyInfoSize: u64,
        storageDependencyInfo: *const storage_dependency::Info,
        sizeUsed: *const u64,
    ) -> DWord;

    pub fn GetVirtualDiskInformation(
        virtualDiskHandle: Handle,
        virtualDiskInfoSize: *const u64,
        virtualDiskInfo: *mut get_virtual_disk::Info,
        sizeUsed: *mut u64,
    ) -> DWord;

    pub fn SetVirtualDiskInformation(
        virtualDiskHandle: Handle,
        virtualDiskInfo: *const set_virtual_disk::Info,
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

    pub fn DeleteVirtualDiskMetadata(virtualDiskHandle: Handle, item: *const Guid) -> DWord;

    pub fn GetVirtualDiskOperationProgress(
        virtualDiskHandle: Handle,
        overlapped: *const Overlapped,
        progress: *mut VirtualDiskProgress,
    ) -> DWord;

    pub fn CompactVirtualDisk(
        virtualDiskHandle: Handle,
        flags: u32, // compact_virtual_disk::Flag
        parameters: *const compact_virtual_disk::Parameters,
        overlapped: *const Overlapped,
    ) -> DWord;

    pub fn MergeVirtualDisk(
        virtualDiskHandle: Handle,
        flags: u32, // merge_virtual_disk::Flag
        parameters: *const merge_virtual_disk::Parameters,
        overlapped: *const Overlapped,
    ) -> DWord;

    pub fn ExpandVirtualDisk(
        virtualDiskHandle: Handle,
        flags: u32, // expand_virtual_disk::Flag
        parameters: *const expand_virtual_disk::Parameters,
        overlapped: *const Overlapped,
    ) -> DWord;

    pub fn ResizeVirtualDisk(
        virtualDiskHandle: Handle,
        flags: u32, // resize_virtual_disk::Flag
        parameters: *const resize_virtual_disk::Parameters,
        overlapped: *const Overlapped,
    ) -> DWord;

    pub fn MirrorVirtualDisk(
        virtualDiskHandle: Handle,
        flags: u32, // mirror_virtual_disk::Flag
        parameters: *const mirror_virtual_disk::Parameters,
        overlapped: *const Overlapped,
    ) -> DWord;

    pub fn BreakMirrorVirtualDisk(virtualDiskHandle: Handle) -> DWord;

    pub fn AddVirtualDiskParent(virtualDiskHandle: Handle, parentPath: PCWStr) -> DWord;

    pub fn QueryChangesVirtualDisk(
        virtualDiskHandle: Handle,
        changeTrackingId: PCWStr,
        byteOffset: u64,
        byteLength: u64,
        flags: u32, // query_changes_virtual_disk::Flag
        ranges: *mut query_changes_virtual_disk::Range,
        rangeCount: *mut u64,
        processedLength: *mut u64,
    ) -> DWord;

    pub fn TakeSnapshotVhdSet(
        virtualDiskHandle: Handle,
        parameters: *const take_snapshot_vhdset::Parameters,
        flags: u32, // take_snapshot_vhdset::Flag
    ) -> DWord;

    pub fn DeleteSnapshotVhdSet(
        virtualDiskHandle: Handle,
        parameters: *const delete_snapshot_vhdset::Parameters,
        flags: u32, // delete_snapshot_vhdset::Flag
    ) -> DWord;

    pub fn ModifyVhdSet(
        virtualDiskHandle: Handle,
        parameters: *const modify_vhdset::Parameters,
        flags: u32, // modify_vhdset::Flag
    ) -> DWord;

    pub fn ApplySnapshotVhdSet(
        virtualDiskHandle: Handle,
        parameters: *const apply_snapshot_vhdset::Parameters,
        flags: u32, // apply_snapshot_vhdset::Flag
    ) -> DWord;

    pub fn RawSCSIVirtualDisk(
        virtualDiskHandle: Handle,
        parameters: *const raw_scsi_virtual_disk::Parameters,
        flags: u32, // raw_scsi_virtual_disk::Flag
        response: *mut raw_scsi_virtual_disk::Response,
    ) -> DWord;

    pub fn ForkVirtualDisk(
        virtualDiskHandle: Handle,
        flags: u32, // fork_virtual_disk::Flag
        parameters: *const fork_virtual_disk::Parameters,
        overlapped: *mut Overlapped,
    ) -> DWord;

    pub fn CompleteForkVirtualDisk(virtualDiskHandle: Handle) -> DWord;
}

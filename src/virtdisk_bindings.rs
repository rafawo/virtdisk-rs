// Copyright © rafawo (rafawo1@hotmail.com). All rights reserved.
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
// THE SOURCE CODE IS AVAILABLE UNDER THE ABOVE CHOSEN LICENSE "AS IS", WITH NO WARRANTIES.

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
        providerSpecificFlags: u32,
        parameters: *const create_virtual_disk::Parameters,
        overlapped: *const Overlapped,
        handle: *mut Handle,
    ) -> DWord;

    pub fn AttachVirtualDisk(
        virtualDiskHandle: Handle,
        securityDescriptor: *const SecurityDescriptor,
        flags: u32, // attach_virtual_disk::Flag
        providerSpecificFlags: u32,
        parameters: *const attach_virtual_disk::Parameters,
        overlapped: *const Overlapped,
    ) -> DWord;

    pub fn DetachVirtualDisk(
        virtualDiskHandle: Handle,
        flags: u32, // detach_virtual_disk::Flag
        providerSpecificFlags: u32,
    ) -> DWord;

    pub fn GetVirtualDiskPhysicalPath(
        virtualDiskHandle: Handle,
        diskPathSizeInBytes: *const u32,
        diskPath: PWStr,
    ) -> DWord;

    pub fn GetAllAttachedVirtualDiskPhysicalPaths(
        pathsBufferSizeInBytes: *mut u32,
        pathsBuffer: PWStr,
    ) -> DWord;

    pub fn GetStorageDependencyInformation(
        objectHandle: Handle,
        flags: u32, // storage_dependency::GetFlag
        storageDependencyInfoSize: u32,
        storageDependencyInfo: *mut storage_dependency::Info,
        sizeUsed: *mut u32,
    ) -> DWord;

    pub fn GetVirtualDiskInformation(
        virtualDiskHandle: Handle,
        virtualDiskInfoSize: *mut u32,
        virtualDiskInfo: *mut get_virtual_disk::Info,
        sizeUsed: *mut u32,
    ) -> DWord;

    pub fn SetVirtualDiskInformation(
        virtualDiskHandle: Handle,
        virtualDiskInfo: *const set_virtual_disk::Info,
    ) -> DWord;

    pub fn EnumerateVirtualDiskMetadata(
        virtualDiskHandle: Handle,
        numberOfItems: *mut u32,
        items: *mut Guid,
    ) -> DWord;

    pub fn GetVirtualDiskMetadata(
        VirtualDiskHandle: Handle,
        item: *const Guid,
        metaDataSize: *mut u32,
        metaData: *mut Void,
    ) -> DWord;

    pub fn SetVirtualDiskMetadata(
        virtualDiskHandle: Handle,
        item: *const Guid,
        metaDataSize: u32,
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
        rangeCount: *mut u32,
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
        overlapped: *const Overlapped,
    ) -> DWord;

    pub fn CompleteForkVirtualDisk(virtualDiskHandle: Handle) -> DWord;
}

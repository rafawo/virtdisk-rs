//! This module provides Rust idiomatic abstractions to the C bindings of VirtDisk.

use crate::virtdisk_bindings::*;
use crate::virtdiskdefs::*;
use crate::windefs::*;
use crate::{error_code_to_result_code, ResultCode};
use widestring::{WideCString, WideStr, WideString};

/// Wrapper of a get_virtual_disk::Info struct that can be of a variable heap allocated length.
pub struct GetVirtualDiskInfoWrapper {
    raw_buffer: Vec<Byte>,
}

impl GetVirtualDiskInfoWrapper {
    /// Gets a reference to a get_virtual_disk::Info struct,
    /// using the internal raw buffer.
    pub fn info(&self) -> &get_virtual_disk::Info {
        unsafe { std::mem::transmute(self.raw_buffer.as_ptr()) }
    }

    /// Gets a mut reference to a get_virtual_disk::Info struct,
    /// using the internal raw buffer.
    pub fn info_mut(&mut self) -> &mut get_virtual_disk::Info {
        unsafe { std::mem::transmute(self.raw_buffer.as_mut_ptr()) }
    }
}

/// Wrapper of a storage_dependency::Info struct that can be of a variable heap allocated length.
pub struct GetStorageDependencyInformationWrapper {
    raw_buffer: Vec<Byte>,
}

impl GetStorageDependencyInformationWrapper {
    /// Gets a reference to a storage_dependency::Info struct,
    /// using the internal raw buffer.
    pub fn info(&self) -> &storage_dependency::Info {
        unsafe { std::mem::transmute(self.raw_buffer.as_ptr()) }
    }

    /// Gets a mut reference to a storage_dependency::Info struct,
    /// using the internal raw buffer.
    pub fn info_mut(&mut self) -> &mut storage_dependency::Info {
        unsafe { std::mem::transmute(self.raw_buffer.as_mut_ptr()) }
    }
}

/// Safe abstraction to a virtual hard disk handle.
/// Additionally, provides the entry point to all safe wrappers to the virtdisk C bindings.
pub struct VirtualDisk {
    handle: Handle,
}

impl std::ops::Drop for VirtualDisk {
    fn drop(&mut self) {
        crate::winutilities::close_handle(&mut self.handle);
    }
}

impl VirtualDisk {
    /// Wraps the supplied virtual hard disk handle, providing a safe drop implementation that will close the handle
    /// on the end of its lifetime.
    pub fn wrap_handle(handle: Handle) -> Result<VirtualDisk, ResultCode> {
        match handle {
            handle if handle == std::ptr::null_mut() => Err(ResultCode::InvalidParameter),
            handle => Ok(VirtualDisk { handle }),
        }
    }

    /// Releases the wrapped handle to ensure that at the end of the lifetime of this VirtualDisk instance
    /// the handle is not closed.
    ///
    /// # Unsafe
    ///
    /// Marked as unsafe because of the possibility of leaking a handle.
    pub unsafe fn release_handle(&mut self) -> Handle {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }

    /// Returns a cloned value of the internally stored handle to the virtual disk.
    /// This is useful so that the virtual hard disk handle can be used on other Windows APIs.
    /// Be careful and do not close the handle returned here because the code will panic at the
    /// end of the lifetime of this VirtualDisk instance if CloseHandle fails.
    pub fn get_handle(&self) -> Handle {
        self.handle.clone()
    }

    /// Opens a virtual hard disk (VHD) or CD or DVD image file (ISO) for use, and returns a safe wrapper to its handle.
    /// The returned object can be used to call any virtdisk API that operates over an open
    /// handle to a virtual disk.
    /// The flags are a u32 representation of any valid combination from `open_virtual_disk::Flag` values.
    pub fn open(
        virtual_storage_type: VirtualStorageType,
        path: &str,
        virtual_disk_access_mask: VirtualDiskAccessMask,
        flags: u32,
        parameters: Option<&open_virtual_disk::Parameters>,
    ) -> Result<VirtualDisk, ResultCode> {
        let mut handle: Handle = std::ptr::null_mut();

        let parameters_ptr = match parameters {
            Some(parameters) => parameters,
            None => std::ptr::null(),
        };

        unsafe {
            match OpenVirtualDisk(
                &virtual_storage_type,
                WideCString::from_str(path).unwrap().as_ptr(),
                virtual_disk_access_mask,
                flags,
                parameters_ptr,
                &mut handle,
            ) {
                result if result == 0 => Ok(VirtualDisk { handle }),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Creates a virtual hard disk, either using default paramters or using an existing virtual disk
    /// or physical disk.
    /// The returned object can be used to call any virtdisk API that operates over an open
    /// handle to a virtual disk.
    /// The flags are a u32 representation of any valid combination from `create_virtual_disk::Flag` values.
    pub fn create(
        virtual_storage_type: VirtualStorageType,
        path: &str,
        virtual_disk_access_mask: VirtualDiskAccessMask,
        security_descriptor: Option<SecurityDescriptor>,
        flags: u32,
        provider_specific_flags: u32,
        parameters: &create_virtual_disk::Parameters,
        overlapped: Option<&Overlapped>,
    ) -> Result<VirtualDisk, ResultCode> {
        let mut handle: Handle = std::ptr::null_mut();

        let security_descriptor_ptr = match security_descriptor {
            Some(security_descriptor) => &security_descriptor,
            None => std::ptr::null(),
        };

        let overlapped_ptr = match overlapped {
            Some(overlapped) => overlapped,
            None => std::ptr::null(),
        };

        unsafe {
            match CreateVirtualDisk(
                &virtual_storage_type,
                WideCString::from_str(path).unwrap().as_ptr(),
                virtual_disk_access_mask,
                security_descriptor_ptr,
                flags,
                provider_specific_flags,
                parameters,
                overlapped_ptr,
                &mut handle,
            ) {
                result if result == 0 => Ok(VirtualDisk { handle }),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Attaches a virtual hard disk (VHD) or CD or DVD image file (ISO)
    /// by locating an appropriate VHD provider to accomplish the attachment.
    /// The flags are a u32 representation of any valid combination from `attach_virtual_disk::Flag` values.
    pub fn attach(
        &self,
        security_descriptor: Option<SecurityDescriptor>,
        flags: u32,
        provider_specific_flags: u32,
        parameters: &attach_virtual_disk::Parameters,
        overlapped: Option<&Overlapped>,
    ) -> Result<(), ResultCode> {
        let security_descriptor_ptr = match security_descriptor {
            Some(security_descriptor) => &security_descriptor,
            None => std::ptr::null(),
        };

        let overlapped_ptr = match overlapped {
            Some(overlapped) => overlapped,
            None => std::ptr::null(),
        };

        unsafe {
            match AttachVirtualDisk(
                self.handle,
                security_descriptor_ptr,
                flags,
                provider_specific_flags,
                parameters,
                overlapped_ptr,
            ) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Detaches a virtual hard disk (VHD) or CD or DVD image file (ISO)
    /// by locating an appropriate virtual disk provider to accomplish the operation.
    /// The flags are a u32 representation of any valid combination from `detach_virtual_disk::Flag` values.
    pub fn detach(&self, flags: u32, provider_specific_flags: u32) -> Result<(), ResultCode> {
        unsafe {
            match DetachVirtualDisk(self.handle, flags, provider_specific_flags) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Retrieves the path to the physical device object that contains a virtual hard disk (VHD) or CD or DVD image file (ISO).
    pub fn get_physical_path(&self) -> Result<String, ResultCode> {
        const PATH_SIZE: u32 = 256; // MAX_PATH
        let mut disk_path_wstr: [WChar; PATH_SIZE as usize] = [0; PATH_SIZE as usize];

        unsafe {
            match GetVirtualDiskPhysicalPath(self.handle, &PATH_SIZE, disk_path_wstr.as_mut_ptr()) {
                result if result == 0 => Ok(WideString::from_ptr(
                    disk_path_wstr.as_ptr(),
                    PATH_SIZE as usize,
                )
                .to_string_lossy()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Retrieves the physical paths to all attached virtual disks and returns it in a vector of strings.
    pub fn get_all_attached_physical_paths() -> Result<Vec<String>, ResultCode> {
        let mut paths_buffer: Vec<WChar> = Vec::new();
        let mut buffer_size_bytes: u32 = 0;

        let mut paths: Vec<String> = Vec::new();

        unsafe {
            // First figure out the required size to fit all paths
            let result = GetAllAttachedVirtualDiskPhysicalPaths(
                &mut buffer_size_bytes,
                paths_buffer.as_mut_ptr(),
            );

            match error_code_to_result_code(result) {
                ResultCode::InsufficientBuffer => {
                    let buffer_size = buffer_size_bytes as usize / std::mem::size_of::<WChar>();
                    paths_buffer.resize(buffer_size, 0);

                    match GetAllAttachedVirtualDiskPhysicalPaths(
                        &mut buffer_size_bytes,
                        paths_buffer.as_mut_ptr(),
                    ) {
                        result if result == 0 => {
                            assert_eq!(
                                buffer_size * std::mem::size_of::<WChar>(),
                                buffer_size_bytes as usize
                            );

                            for string in paths_buffer.as_slice().split(|element| *element == 0) {
                                if !string.is_empty() {
                                    paths.push(WideStr::from_slice(string).to_string_lossy());
                                }
                            }

                            Ok(paths)
                        }
                        result => Err(error_code_to_result_code(result)),
                    }
                }
                ResultCode::Success => Ok(paths),
                error => Err(error),
            }
        }
    }

    /// Retrieves on the supplied info structure the storage dependency information of a virtual disk.
    /// On success, returns the size used in the info structure.
    /// The flags are a u32 representation of any valid combination from storage_dependency::GetFlag values.
    pub fn get_storage_dependency_information(
        &self,
        flags: u32,
        version: storage_dependency::InfoVersion,
    ) -> Result<GetStorageDependencyInformationWrapper, ResultCode> {
        let mut raw_buffer: Vec<Byte> = Vec::new();
        let size: u32 = std::mem::size_of::<storage_dependency::Info>() as u32;
        let mut buffer_size: u32 = size;
        raw_buffer.reserve(buffer_size as usize);

        let info_ptr = raw_buffer.as_mut_ptr() as *mut storage_dependency::Info;

        unsafe {
            (*info_ptr).version = version;

            let result = GetStorageDependencyInformation(
                self.handle,
                flags,
                size,
                info_ptr,
                &mut buffer_size,
            );

            match error_code_to_result_code(result) {
                ResultCode::InsufficientBuffer => {
                    raw_buffer.reserve(buffer_size as usize);

                    let result = GetStorageDependencyInformation(
                        self.handle,
                        flags,
                        size,
                        info_ptr,
                        &mut buffer_size,
                    );

                    match error_code_to_result_code(result) {
                        ResultCode::Success => {
                            Ok(GetStorageDependencyInformationWrapper { raw_buffer })
                        }
                        error => Err(error),
                    }
                }
                ResultCode::Success => Ok(GetStorageDependencyInformationWrapper { raw_buffer }),
                error => Err(error),
            }
        }
    }

    /// Retrieves information of a virtual disk wrapped on a safe structure on top of a raw buffer.
    pub fn get_information(
        &self,
        version: get_virtual_disk::InfoVersion,
    ) -> Result<GetVirtualDiskInfoWrapper, ResultCode> {
        let mut size_used: u32 = 0;
        let mut raw_buffer: Vec<Byte> = Vec::new();
        let mut size: u32 = std::mem::size_of::<get_virtual_disk::Info>() as u32;
        raw_buffer.reserve(size as usize);

        let info_ptr = raw_buffer.as_mut_ptr() as *mut get_virtual_disk::Info;

        unsafe {
            (*info_ptr).version = version;

            let result =
                GetVirtualDiskInformation(self.handle, &mut size, info_ptr, &mut size_used);

            match error_code_to_result_code(result) {
                ResultCode::InsufficientBuffer => {
                    raw_buffer.reserve(size as usize);

                    let result =
                        GetVirtualDiskInformation(self.handle, &mut size, info_ptr, &mut size_used);

                    match error_code_to_result_code(result) {
                        ResultCode::Success => Ok(GetVirtualDiskInfoWrapper { raw_buffer }),
                        error => Err(error),
                    }
                }
                ResultCode::Success => Ok(GetVirtualDiskInfoWrapper { raw_buffer }),
                error => Err(error),
            }
        }
    }

    /// Sets information about a virtual hard disk.
    pub fn set_information(&self, info: &set_virtual_disk::Info) -> Result<(), ResultCode> {
        unsafe {
            match SetVirtualDiskInformation(self.handle, info) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Enumerates the metadata associated with a virtual disk.
    /// The returned vector of GUID refer to a set of metadata that can be retrieved
    /// using function `VirtualHardDisk::get_metadata`.
    pub fn enumerate_metadata(&self) -> Result<Vec<Guid>, ResultCode> {
        let mut guids: Vec<Guid> = Vec::new();
        let mut vector_size: u32 = 0;

        unsafe {
            let result =
                EnumerateVirtualDiskMetadata(self.handle, &mut vector_size, guids.as_mut_ptr());

            match error_code_to_result_code(result) {
                ResultCode::InsufficientBuffer => {
                    guids.resize(
                        vector_size as usize,
                        Guid {
                            Data1: 0,
                            Data2: 0,
                            Data3: 0,
                            Data4: [0; 8],
                        },
                    );

                    match EnumerateVirtualDiskMetadata(
                        self.handle,
                        &mut vector_size,
                        guids.as_mut_ptr(),
                    ) {
                        result if result == 0 => {
                            assert_eq!(vector_size as usize, guids.len());
                            Ok(guids)
                        }
                        result => Err(error_code_to_result_code(result)),
                    }
                }
                ResultCode::Success => Ok(guids),
                error => Err(error),
            }
        }
    }

    /// Retrieves the specified metadata from the virtual disk as an u8 byte buffer.
    pub fn get_metadata(&self, item: &Guid) -> Result<Vec<u8>, ResultCode> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut buffer_size: u32 = 0;

        unsafe {
            let result = GetVirtualDiskMetadata(
                self.handle,
                item,
                &mut buffer_size,
                buffer.as_mut_ptr() as *mut Void,
            );

            match error_code_to_result_code(result) {
                ResultCode::InsufficientBuffer => {
                    buffer.resize(buffer_size as usize, 0);

                    match GetVirtualDiskMetadata(
                        self.handle,
                        item,
                        &mut buffer_size,
                        buffer.as_mut_ptr() as *mut Void,
                    ) {
                        result if result == 0 => {
                            assert_eq!(buffer_size as usize, buffer.len());
                            Ok(buffer)
                        }
                        result => Err(error_code_to_result_code(result)),
                    }
                }
                ResultCode::Success => Ok(buffer),
                error => Err(error),
            }
        }
    }

    /// Sets a metadata item for a virtual disk.
    pub fn set_metadata(&self, item: &Guid, buffer: &[u8]) -> Result<(), ResultCode> {
        unsafe {
            match SetVirtualDiskMetadata(
                self.handle,
                item,
                buffer.len() as u32,
                buffer.as_ptr() as *const Void,
            ) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Deletes metadata from a virtual disk.
    pub fn delete_metadata(&self, item: &Guid) -> Result<(), ResultCode> {
        unsafe {
            match DeleteVirtualDiskMetadata(self.handle, item) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Checks the progress of an asynchronous virtual disk operation.
    pub fn get_operation_progress(
        &self,
        overlapped: &Overlapped,
    ) -> Result<VirtualDiskProgress, ResultCode> {
        let mut progress = VirtualDiskProgress {
            operation_status: 0,
            current_value: 0,
            completion_value: 0,
        };

        unsafe {
            match GetVirtualDiskOperationProgress(self.handle, overlapped, &mut progress) {
                result if result == 0 => Ok(progress),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Reduces the size of a virtual disk backing store file.
    /// The flags are a u32 representation of any valid combination from `compact_virtual_disk::Flag` values.
    pub fn compact(
        &self,
        flags: u32,
        parameters: &compact_virtual_disk::Parameters,
        overlapped: Option<&Overlapped>,
    ) -> Result<(), ResultCode> {
        let overlapped_ptr = match overlapped {
            Some(overlapped) => overlapped,
            None => std::ptr::null(),
        };

        unsafe {
            match CompactVirtualDisk(self.handle, flags, parameters, overlapped_ptr) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Merges a child virtual hard disk in a differencing chain with one or more parent virtual disks in the chain.
    /// The flags are a u32 representation of any valid combination from `merge_virtual_disk::Flag` values.
    pub fn merge(
        &self,
        flags: u32,
        parameters: &merge_virtual_disk::Parameters,
        overlapped: Option<&Overlapped>,
    ) -> Result<(), ResultCode> {
        let overlapped_ptr = match overlapped {
            Some(overlapped) => overlapped,
            None => std::ptr::null(),
        };

        unsafe {
            match MergeVirtualDisk(self.handle, flags, parameters, overlapped_ptr) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Increases the size of a fixed or dynamically expandable virtual hard disk.
    /// The flags are a u32 representation of any valid combination from `expand_virtual_disk::Flag` values.
    pub fn expand(
        &self,
        flags: u32,
        parameters: &expand_virtual_disk::Parameters,
        overlapped: Option<&Overlapped>,
    ) -> Result<(), ResultCode> {
        let overlapped_ptr = match overlapped {
            Some(overlapped) => overlapped,
            None => std::ptr::null(),
        };

        unsafe {
            match ExpandVirtualDisk(self.handle, flags, parameters, overlapped_ptr) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Resizes virtual hard disk.
    /// The flags are a u32 representation of any valid combination from `resize_virtual_disk::Flag` values.
    pub fn resize(
        &self,
        flags: u32,
        parameters: &resize_virtual_disk::Parameters,
        overlapped: Option<&Overlapped>,
    ) -> Result<(), ResultCode> {
        let overlapped_ptr = match overlapped {
            Some(overlapped) => overlapped,
            None => std::ptr::null(),
        };

        unsafe {
            match ResizeVirtualDisk(self.handle, flags, parameters, overlapped_ptr) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Initiates a mirror operation for a virtual disk. Once the mirroring operation is initiated it will
    /// not complete until either [CancelIo](https://docs.microsoft.com/en-us/windows/desktop/FileIO/cancelio)
    /// or [CancelIoEx](https://docs.microsoft.com/en-us/windows/desktop/FileIO/cancelioex-func) is called
    /// to cancel all I-O on the VirtualHardDiskHandle, leaving the original file as the current or
    /// `VirtualHardDisk::break_mirror` is called to stop using the original file and only use the mirror.
    /// `VirtualHardDisk::get_operation_progress` can be used to determine if the disks are fully mirrored
    /// and writes go to both virtual disks.
    /// The flags are a u32 representation of any valid combination from `mirror_virtual_disk::Flag` values.
    pub fn mirror(
        &self,
        flags: u32,
        parameters: &mirror_virtual_disk::Parameters,
        overlapped: &Overlapped,
    ) -> Result<(), ResultCode> {
        unsafe {
            match MirrorVirtualDisk(self.handle, flags, parameters, overlapped) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Breaks a previously initiated mirror operation and sets the mirror to be the active virtual disk.
    pub fn break_mirror(&self) -> Result<(), ResultCode> {
        unsafe {
            match BreakMirrorVirtualDisk(self.handle) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Attaches a parent to a virtual disk opened with the `open_virtual_disk::Flag::CustomDiffChain` flag.
    pub fn add_parent(&self, parent_path: &str) -> Result<(), ResultCode> {
        unsafe {
            match AddVirtualDiskParent(
                self.handle,
                WideCString::from_str(parent_path).unwrap().as_ptr(),
            ) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Retrieves information about changes to the specified areas of a virtual hard disk
    /// that are tracked by resilient change tracking (RCT).
    /// Returns a tuple with the number of `query_changes_virtual_disk::Range` structures that the method
    /// placed in the array and the processed length in bytes, which indicates for how much of the area that the `byte_length`
    /// parameter specifies that changes were captured in the available space of the array that the `ranges`
    /// parameter specifies.
    /// The flags are a u32 representation of any valid combination from `query_changes_virtual_disk::Flag` values.
    pub fn query_changes(
        &self,
        change_tracking_id: &str,
        byte_offset: u64,
        byte_length: u64,
        flags: u32,
        ranges: &mut [query_changes_virtual_disk::Range],
    ) -> Result<(u32, u64), ResultCode> {
        let mut range_count: u32 = ranges.len() as u32;
        let mut processed_length: u64 = 0;

        unsafe {
            match QueryChangesVirtualDisk(
                self.handle,
                WideCString::from_str(change_tracking_id).unwrap().as_ptr(),
                byte_offset,
                byte_length,
                flags,
                ranges.as_mut_ptr(),
                &mut range_count,
                &mut processed_length,
            ) {
                result if result == 0 => Ok((range_count, processed_length)),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Creates a snapshot of the current virtual disk for VHD Set files.
    /// The flags are a u32 representation of any valid combination from `take_snapshot_vhdset::Flag` values.
    pub fn take_snapshot_vhdset(
        &self,
        parameters: &take_snapshot_vhdset::Parameters,
        flags: u32,
    ) -> Result<(), ResultCode> {
        unsafe {
            match TakeSnapshotVhdSet(self.handle, parameters, flags) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Deletes a snapshot from a VHD Set file.
    /// The flags are a u32 representation of any valid combination from `delete_snapshot_vhdset::Flag` values.
    pub fn delete_snapshot_vhdset(
        &self,
        parameters: &delete_snapshot_vhdset::Parameters,
        flags: u32,
    ) -> Result<(), ResultCode> {
        unsafe {
            match DeleteSnapshotVhdSet(self.handle, parameters, flags) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Modifies the internal contents of a virtual disk file. Can be used to set the active leaf,
    /// or to fix up snapshot entries.
    /// The flags are a u32 representation of any valid combination from `modify_vhdset::Flag` values.
    pub fn modify_vhdset(
        &self,
        parameters: &modify_vhdset::Parameters,
        flags: u32,
    ) -> Result<(), ResultCode> {
        unsafe {
            match ModifyVhdSet(self.handle, parameters, flags) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Applies a snapshot of the current virtual disk for VHD Set files.
    /// The flags are a u32 representation of any valid combination from `apply_snapshot_vhdset::Flag` values.
    pub fn apply_snapshot_vhdset(
        &self,
        parameters: &apply_snapshot_vhdset::Parameters,
        flags: u32,
    ) -> Result<(), ResultCode> {
        unsafe {
            match ApplySnapshotVhdSet(self.handle, parameters, flags) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Issues an embedded SCSI request directly to a virtual hard disk.
    /// The flags are a u32 representation of any valid combination from `raw_scsi_virtual_disk::Flag` values.
    pub fn raw_scsi_virtual_disk(
        &self,
        parameters: &raw_scsi_virtual_disk::Parameters,
        flags: u32,
    ) -> Result<raw_scsi_virtual_disk::Response, ResultCode> {
        let mut response = raw_scsi_virtual_disk::Response {
            version: raw_scsi_virtual_disk::Version::Unspecified,
            version_details: raw_scsi_virtual_disk::ResponseVersionDetails {
                version1: raw_scsi_virtual_disk::ResponseVersion1 {
                    scsi_status: 0,
                    sense_info_length: 0,
                    data_transfer_length: 0,
                },
            },
        };

        unsafe {
            match RawSCSIVirtualDisk(self.handle, parameters, flags, &mut response) {
                result if result == 0 => Ok(response),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Forks a virtual hard disk.
    /// `VirtualHardDisk::get_operation_progress` can be used to determine if the disk has been fully forked.
    /// The flags are a u32 representation of any valid combination from `fork_virtual_disk::Flag` values.
    pub fn fork(
        &self,
        flags: u32,
        parameters: &fork_virtual_disk::Parameters,
        overlapped: &Overlapped,
    ) -> Result<(), ResultCode> {
        unsafe {
            match ForkVirtualDisk(self.handle, flags, parameters, overlapped) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Completes a virtual hard disk fork initiated with `VirtualHardDisk::fork`.
    pub fn complete_fork(&self) -> Result<(), ResultCode> {
        unsafe {
            match CompleteForkVirtualDisk(self.handle) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }
}

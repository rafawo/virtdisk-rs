//! This module provides Rust idiomatic abstractions to the C bindings of VirtDisk.
//! Both the FFI bindings and Rust wrappers are public to this crate, to give flexibility
//! to consumer code to use the bindings directly as they see fit.
//!

use crate::virtdisk_bindings::*;
use crate::virtdiskdefs::*;
use crate::windefs::*;
use widestring::{WideCString, WideStr, WideString};

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
    WindowsErrorCode(DWord),
}

fn error_code_to_result_code(error_code: DWord) -> ResultCode {
    match error_code {
        0 => ResultCode::Success,
        87 => ResultCode::InvalidParameter,
        618 => ResultCode::UnsupportedCompression,
        6002 => ResultCode::FileEncrypted,
        665 => ResultCode::FileSystemLimitation,
        1392 => ResultCode::FileCorrupt,
        2 => ResultCode::FileNotFound,
        122 => ResultCode::InsufficientBuffer,
        error_code => ResultCode::WindowsErrorCode(error_code),
    }
}

const MAX_PATH: u64 = 256;

/// Safe abstraction to a virtual hard disk handle.
/// Additionally, provides the entry point to all save wrappers to the virtdisk C bindings.
pub struct VirtualDisk {
    handle: Handle,
}

impl std::ops::Drop for VirtualDisk {
    fn drop(&mut self) {
        #[allow(unused_assignments)]
        let mut result: Bool = 0;

        unsafe {
            result = kernel32::CloseHandle(self.handle);
        }

        match result {
            result if result == 0 => {
                panic!("Closing handle failed with error code {}", unsafe {
                    kernel32::GetLastError()
                });
            }
            _ => {}
        }
    }
}

impl VirtualDisk {
    /// Opens a virtual hard disk (VHD) or CD or DVD image file (ISO) for use, and returns a safe wrapper to its handle.
    /// The returned object can be used to call any virtdisk API that operates over an open
    /// handle to a virtual disk.
    /// The flags are a u32 representation of any valid combination from open_virtual_disk::Flag values.
    pub fn open(
        virtual_storage_type: VirtualStorageType,
        path: &str,
        virtual_disk_access_mask: VirtualDiskAccessMask,
        flags: u32,
        parameters: Option<open_virtual_disk::Parameters>,
    ) -> Result<VirtualDisk, ResultCode> {
        let mut handle: Handle = std::ptr::null_mut();

        let parameters_ptr = match parameters {
            Some(parameters) => &parameters,
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
    /// The flags are a u32 representation of any valid combination from create_virtual_disk::Flag values.
    pub fn create(
        virtual_storage_type: VirtualStorageType,
        path: &str,
        virtual_disk_access_mask: VirtualDiskAccessMask,
        security_descriptor: Option<SecurityDescriptor>,
        flags: u32,
        provider_specific_flags: u64,
        parameters: &create_virtual_disk::Parameters,
        overlapped: Option<Overlapped>,
    ) -> Result<VirtualDisk, ResultCode> {
        let mut handle: Handle = std::ptr::null_mut();

        let security_descriptor_ptr = match security_descriptor {
            Some(security_descriptor) => &security_descriptor,
            None => std::ptr::null(),
        };

        let overlapped_ptr = match overlapped {
            Some(overlapped) => &overlapped,
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

    /// Returns a cloned value of the internally stored handle to the virtual disk.
    /// This is useful so that the virtual hard disk handle can be used on other Windows APIs.
    pub fn get_handle(&self) -> Handle {
        self.handle.clone()
    }

    /// Attaches a virtual hard disk (VHD) or CD or DVD image file (ISO)
    /// by locating an appropriate VHD provider to accomplish the attachment.
    /// The flags are a u32 representation of any valid combination from attach_virtual_disk::Flag values.
    pub fn attach(
        &self,
        security_descriptor: Option<SecurityDescriptor>,
        flags: u32,
        provider_specific_flags: u64,
        parameters: &attach_virtual_disk::Parameters,
        overlapped: Option<Overlapped>,
    ) -> Result<(), ResultCode> {
        let security_descriptor_ptr = match security_descriptor {
            Some(security_descriptor) => &security_descriptor,
            None => std::ptr::null(),
        };

        let overlapped_ptr = match overlapped {
            Some(overlapped) => &overlapped,
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
    /// The flags are a u32 representation of any valid combination from detach_virtual_disk::Flag values.
    pub fn detach(&self, flags: u32, provider_specific_flags: u64) -> Result<(), ResultCode> {
        unsafe {
            match DetachVirtualDisk(self.handle, flags, provider_specific_flags) {
                result if result == 0 => Ok(()),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Retrieves the path to the physical device object that contains a virtual hard disk (VHD) or CD or DVD image file (ISO).
    pub fn get_physical_path(&self) -> Result<String, ResultCode> {
        const PATH_SIZE: u64 = MAX_PATH;
        let mut disk_path_wstr: [libc::wchar_t; PATH_SIZE as usize] = [0; PATH_SIZE as usize];

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
        let mut paths_buffer: Vec<libc::wchar_t> = Vec::new();
        let mut buffer_size_bytes: u64 = 0;

        let mut paths: Vec<String> = Vec::new();

        unsafe {
            // First figure out the required size to fit all paths
            let result = GetAllAttachedVirtualDiskPhysicalPaths(
                &mut buffer_size_bytes,
                paths_buffer.as_mut_ptr(),
            );

            match error_code_to_result_code(result) {
                ResultCode::InsufficientBuffer => {
                    let buffer_size =
                        buffer_size_bytes as usize / std::mem::size_of::<libc::wchar_t>();
                    paths_buffer.resize(buffer_size, 0);

                    match GetAllAttachedVirtualDiskPhysicalPaths(
                        &mut buffer_size_bytes,
                        paths_buffer.as_mut_ptr(),
                    ) {
                        result if result == 0 => {
                            assert_eq!(
                                buffer_size * std::mem::size_of::<libc::wchar_t>(),
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
        info_size: u64,
        info: *mut storage_dependency::Info,
    ) -> Result<u64, ResultCode> {
        let mut size_used: u64 = 0;

        unsafe {
            match GetStorageDependencyInformation(
                self.handle,
                flags,
                info_size,
                info,
                &mut size_used,
            ) {
                result if result == 0 => Ok(size_used),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }

    /// Retrieves on the supplied info structure information of a virtual disk.
    /// On success, returns the size used in the info structure.
    pub fn get_information(
        &self,
        info_size: u64,
        info: &mut get_virtual_disk::Info,
    ) -> Result<u64, ResultCode> {
        let mut size_used: u64 = 0;

        unsafe {
            match GetVirtualDiskInformation(self.handle, &info_size, info, &mut size_used) {
                result if result == 0 => Ok(size_used),
                result => Err(error_code_to_result_code(result)),
            }
        }
    }
}

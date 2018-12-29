// Copyright © rafawo (rafawo1@hotmail.com). All rights reserved.
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
// THE SOURCE CODE IS AVAILABLE UNDER THE ABOVE CHOSEN LICENSE "AS IS", WITH NO WARRANTIES.

//! Wrappers around basic VHD functions used to setup container storage.

use crate::diskutilities::*;
use crate::virtdisk::*;
use crate::virtdiskdefs::*;
use crate::windefs::*;
use crate::winutilities::*;
use crate::{error_code_to_result_code, ResultCode};

pub struct MountedVolume {
    pub vhd: VirtualDisk,
    pub disk: Disk,
    pub partition: PartitionInfo,
}

/// Mounts the given VHD into the host.
/// The flags are a u32 representation of any valid combination from `attach_virtual_disk::Flag` values.
pub fn mount_vhd(
    virtual_disk: &VirtualDisk,
    flags: u32,
    cache_mode: u16,
) -> Result<(), ResultCode> {
    use winapi::um::{errhandlingapi, ioapiset, winnt};

    let manage_volume = TemporaryPrivilege::new(winnt::SE_MANAGE_VOLUME_NAME);

    #[repr(C)]
    pub struct StorageSurfaceVirtualDiskLev1Request {
        request_level: ULong, // 1 is currently only value supported
        flags: u32,
        provider_flags: ULong,
        security_descriptor_offset: ULong,
        security_descriptor_length: ULong,
        internal_reserved_flags: UShort,
        cache_mode: UShort,
        qos_flow_id: Guid,
    }

    unsafe {
        let mut request = std::mem::zeroed::<StorageSurfaceVirtualDiskLev1Request>();
        request.request_level = 1;
        request.flags = flags;
        request.cache_mode = cache_mode;

        if ioapiset::DeviceIoControl(
            virtual_disk.get_handle(),
            2955548, // IOCTL_STORAGE_SURFACE_VIRTUAL_DISK
            &mut request as *mut _ as PVoid,
            std::mem::size_of::<StorageSurfaceVirtualDiskLev1Request>() as DWord,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) == 0
        {
            return Err(error_code_to_result_code(errhandlingapi::GetLastError()));
        }
    }

    // Make sure we revert the temporary privilege to manage volumes
    drop(manage_volume);

    let disk = open_vhd_backed_disk(&virtual_disk)?;
    match disk.force_online() {
        Err(error) => {
            virtual_disk.detach(detach_virtual_disk::Flag::None as u32, 0)?;
            Err(error)
        }
        _ => Ok(()),
    }
}

/// Mounts a VHD with temporarily lifetime and without respecting flushes.
/// The expectation is that this is only called during setup, where if there is
/// a power failure the file would be deleted anyway.
pub fn mount_vhd_temporarily_for_setup(virtual_disk: &VirtualDisk) -> Result<(), ResultCode> {
    mount_vhd(
        virtual_disk,
        attach_virtual_disk::Flag::NoDriveLetter as u32
            | attach_virtual_disk::Flag::BypassDefaultEncryptionPolicy as u32,
        4, // VHD_WRITE_CACHE_MODE_DISABLE_FLUSHING
    )
}

/// Attaches a VHD with permanent lifetime, respecting all flushes (but cache metadata in VHDX),
/// and ensure there is no extra security descriptor applied to the volume object.
pub fn mount_vhd_permanently_for_use(virtual_disk: &VirtualDisk) -> Result<(), ResultCode> {
    mount_vhd(
        virtual_disk,
        attach_virtual_disk::Flag::NoDriveLetter as u32
            | attach_virtual_disk::Flag::PermanentLifetime as u32
            | attach_virtual_disk::Flag::NoSecurityDescriptor as u32
            | attach_virtual_disk::Flag::BypassDefaultEncryptionPolicy as u32,
        0, // VHD_WRITE_CACHE_MODE_CACHE_METADATA
    )
}

/// Dismounts the given VHD from the host.
pub fn dismount_vhd(virtual_disk: &VirtualDisk) -> Result<(), ResultCode> {
    virtual_disk.detach(detach_virtual_disk::Flag::None as u32, 0)
}

/// Opens a VHD for use as a container sandbox and returns a safe wrapper over the handle.
pub fn open_vhd(filename: &str, read_only: bool) -> Result<VirtualDisk, ResultCode> {
    let default_storage_type = VirtualStorageType {
        device_id: 0,
        vendor_id: VIRTUAL_STORAGE_TYPE_VENDOR_UNKNOWN,
    };

    let parameters = open_virtual_disk::Parameters {
        version: open_virtual_disk::Version::Version2,
        version_details: open_virtual_disk::VersionDetails {
            version2: open_virtual_disk::Version2 {
                get_info_only: 0,
                read_only: read_only as Bool,
                resiliency_guid: GUID_NULL,
            },
        },
    };

    VirtualDisk::open(
        default_storage_type,
        filename,
        VirtualDiskAccessMask::None,
        open_virtual_disk::Flag::ParentCachedIo as u32
            | open_virtual_disk::Flag::IgnoreRelativeParentLocator as u32,
        Some(&parameters),
    )
}

/// Creates a new base VHD specified by filename.
pub fn create_base_vhd(
    filename: &str,
    disk_size_gb: u64,
    block_size_mb: u32,
    file_system: &str,
) -> Result<MountedVolume, ResultCode> {
    let mut parameters = unsafe { std::mem::zeroed::<create_virtual_disk::Parameters>() };
    parameters.version = create_virtual_disk::Version::Version2;
    unsafe {
        parameters.version_details.version2.maximum_size = disk_size_gb * 1024 * 1024 * 1024;
        parameters.version_details.version2.block_size_in_bytes = block_size_mb * 1024 * 1024;
    }

    let default_storage_type = VirtualStorageType {
        device_id: 0,
        vendor_id: GUID_NULL,
    };

    let virtual_disk = VirtualDisk::create(
        default_storage_type,
        filename,
        VirtualDiskAccessMask::None,
        None,
        create_virtual_disk::Flag::None as u32,
        0,
        &parameters,
        None,
    )?;

    mount_vhd_temporarily_for_setup(&virtual_disk)?;

    let disk = open_vhd_backed_disk(&virtual_disk)?;
    let partition_info = disk.format(file_system)?;

    Ok(MountedVolume {
        vhd: virtual_disk,
        disk: disk,
        partition: partition_info,
    })
}

/// Creates a new diff VHD specified by filename based on the given parent disk.
pub fn create_diff_vhd(
    filename: &str,
    parent_name: &str,
    block_size_mb: u32,
) -> Result<(), ResultCode> {
    assert!(block_size_mb <= 256);
    let mut block_size_in_bytes = block_size_mb * 1024 * 1024;

    if block_size_in_bytes == 0 {
        let mut parameters = unsafe { std::mem::zeroed::<open_virtual_disk::Parameters>() };
        parameters.version = open_virtual_disk::Version::Version2;

        let default_storage_type = VirtualStorageType {
            device_id: 0,
            vendor_id: GUID_NULL,
        };

        let parent = VirtualDisk::open(
            default_storage_type,
            parent_name,
            VirtualDiskAccessMask::None,
            open_virtual_disk::Flag::NoParents as u32,
            Some(&parameters),
        )?;

        let vhd_info_wrapper = parent.get_information(get_virtual_disk::InfoVersion::Size)?;
        block_size_in_bytes = unsafe { vhd_info_wrapper.info().version_details.size.block_size };
    }

    let parent_name_wstr = widestring::WideCString::from_str(parent_name).unwrap();
    let mut parameters = unsafe { std::mem::zeroed::<create_virtual_disk::Parameters>() };
    parameters.version = create_virtual_disk::Version::Version2;
    unsafe {
        parameters.version_details.version2.parent_path = parent_name_wstr.as_ptr();
        parameters.version_details.version2.block_size_in_bytes = block_size_in_bytes;
        parameters.version_details.version2.open_flags = open_virtual_disk::Flag::CachedIo as u32;
    }

    let default_storage_type = VirtualStorageType {
        device_id: 0,
        vendor_id: GUID_NULL,
    };

    let _virtual_disk = VirtualDisk::create(
        default_storage_type,
        filename,
        VirtualDiskAccessMask::None,
        None,
        create_virtual_disk::Flag::None as u32,
        0,
        &parameters,
        None,
    )?;

    Ok(())
}

/// Creates a VHD from the contents of another VHD. This is used to defragment VHDs
/// after they are fully constructed.
pub fn create_vhd_from_source(
    filename: &str,
    source_filename: &str,
    block_size_mb: u32,
) -> Result<(), ResultCode> {
    let source_path_wstr = widestring::WideCString::from_str(source_filename).unwrap();
    let mut parameters = unsafe { std::mem::zeroed::<create_virtual_disk::Parameters>() };
    parameters.version = create_virtual_disk::Version::Version2;
    unsafe {
        parameters.version_details.version2.source_path = source_path_wstr.as_ptr();
        parameters.version_details.version2.block_size_in_bytes = block_size_mb * 1024 * 1024;
        parameters.version_details.version2.open_flags = open_virtual_disk::Flag::CachedIo as u32;
    }

    let default_storage_type = VirtualStorageType {
        device_id: 0,
        vendor_id: GUID_NULL,
    };

    let _virtual_disk = VirtualDisk::create(
        default_storage_type,
        filename,
        VirtualDiskAccessMask::None,
        None,
        create_virtual_disk::Flag::None as u32,
        0,
        &parameters,
        None,
    )?;

    Ok(())
}

/// Finds the given mounted VHD and returns the resulting volume path.
pub fn get_vhd_volume_path(virtual_disk: &VirtualDisk) -> Result<String, ResultCode> {
    let disk = open_vhd_backed_disk(&virtual_disk)?;
    disk.volume_path()
}

/// Determines the VHD path of the VHD hosting a volume or file within the volume.
pub fn get_vhd_from_filename(filename: &str) -> Result<String, ResultCode> {
    use winapi::um::{fileapi, winnt};

    let file = create_file(
        filename,
        0,
        winnt::FILE_SHARE_READ | winnt::FILE_SHARE_WRITE,
        None,
        fileapi::OPEN_EXISTING,
        winnt::FILE_ATTRIBUTE_NORMAL,
        None,
    )?;

    let virtual_disk = VirtualDisk::wrap_handle(file)?;
    let dependency_info_wrapper = match virtual_disk.get_storage_dependency_information(
        storage_dependency::GetFlag::HostVolumes as u32,
        storage_dependency::InfoVersion::Version2,
    ) {
        Err(ResultCode::WindowsErrorCode(error))
            if error == winapi::shared::winerror::ERROR_VIRTDISK_NOT_VIRTUAL_DISK as u32 =>
        {
            return Ok(String::from(""));
        }
        Err(error) => {
            return Err(error);
        }
        Ok(wrapper) => wrapper,
    };

    const MAX_PATH: usize = 256;
    let mut filename: [WChar; MAX_PATH] = [0; MAX_PATH];

    unsafe {
        match PathCchCombine(
            filename.as_mut_ptr(),
            MAX_PATH,
            dependency_info_wrapper.info().version_details.version2[0].host_volume_name,
            dependency_info_wrapper.info().version_details.version2[0]
                .dependent_volume_relative_path,
        ) {
            result if result == 0 => {
                Ok(widestring::WideCString::from_ptr_str(filename.as_ptr()).to_string_lossy())
            }
            _ => Err(ResultCode::GenFailure),
        }
    }
}

/// Sets the caching mode on a mounted VHD.
pub fn set_vhd_caching_mode(virtual_disk: &VirtualDisk, cache_mode: u16) -> Result<(), ResultCode> {
    #[repr(C)]
    struct CachePolicyRequest {
        request_level: u32,
        cache_mode: u16,
    }

    let mut request = CachePolicyRequest {
        request_level: 1,
        cache_mode: cache_mode,
    };

    let mut bytes: DWord = 0;

    unsafe {
        match winapi::um::ioapiset::DeviceIoControl(
            virtual_disk.get_handle(),
            2955792, // IOCTL_STORAGE_SET_SURFACE_CACHE_POLICY
            &mut request as *mut _ as PVoid,
            std::mem::size_of::<CachePolicyRequest>() as u32,
            std::ptr::null_mut(),
            0,
            &mut bytes,
            std::ptr::null_mut(),
        ) {
            result if result != 0 => Ok(()),
            result => Err(error_code_to_result_code(result as u32)),
        }
    }
}

/// Returns the size of the VHD on the physical disk.
pub fn get_physical_vhd_size_in_kb(virtual_disk: &VirtualDisk) -> Result<u64, ResultCode> {
    let info_wrapper = virtual_disk.get_information(get_virtual_disk::InfoVersion::Size)?;
    unsafe { Ok(info_wrapper.info().version_details.size.physical_size / 1024) }
}

/// Opens the disk backed by the secified VHD.
pub fn open_vhd_backed_disk(virtual_disk: &VirtualDisk) -> Result<Disk, ResultCode> {
    let disk_path = virtual_disk.get_physical_path()?;
    Disk::open(
        &disk_path,
        None,
        Some(
            winapi::um::winnt::FILE_ATTRIBUTE_NORMAL | winapi::um::winbase::FILE_FLAG_NO_BUFFERING,
        ),
    )
}

/// Expands the virtual size of a VHD to the requested size, if the current size is smaller
/// than the requested size.
/// Returns true if the VHD was expanded, false if the current size of the VHD is already greater
/// than or equal to the specified new size.
pub fn expand_vhd(virtual_disk: &VirtualDisk, new_size: u64) -> Result<bool, ResultCode> {
    let info_wrapper = virtual_disk.get_information(get_virtual_disk::InfoVersion::Size)?;

    if unsafe { info_wrapper.info().version_details.size.virtual_size } < new_size {
        #[repr(C)]
        struct VhdResizeRequest {
            new_virtual_size: u64,
            expand_only: Boolean,
            allow_unsafe_virtual_size: Boolean,
            shrink_to_minimum_safe_size: Boolean,
        }

        let mut request = VhdResizeRequest {
            new_virtual_size: new_size,
            expand_only: 1,
            allow_unsafe_virtual_size: 0,
            shrink_to_minimum_safe_size: 0,
        };

        let mut bytes: DWord = 0;

        unsafe {
            match winapi::um::ioapiset::DeviceIoControl(
                virtual_disk.get_handle(),
                2955600, // IOCTL_STORAGE_RESIZE_VIRTUAL_DISK
                &mut request as *mut _ as PVoid,
                std::mem::size_of::<VhdResizeRequest>() as u32,
                std::ptr::null_mut(),
                0,
                &mut bytes,
                std::ptr::null_mut(),
            ) {
                result if result != 0 => Ok(true),
                result => Err(error_code_to_result_code(result as u32)),
            }
        }
    } else {
        Ok(false)
    }
}

/// Merges a differencing disk into its immediate parent. This function should be called with caution,
/// there might be destructive side effects if the parent disk has other child disks.
pub fn merge_diff_vhd(virtual_disk: &VirtualDisk) -> Result<(), ResultCode> {
    let event = WinEvent::create(false, false, None, None)?;
    let mut overlapped = unsafe { std::mem::zeroed::<Overlapped>() };
    overlapped.hEvent = event.get_handle();

    let mut parameters = unsafe { std::mem::zeroed::<merge_virtual_disk::Parameters>() };
    parameters.version = merge_virtual_disk::Version::Version2;
    unsafe {
        parameters.version_details.version2.merge_source_depth = 1;
        parameters.version_details.version2.merge_target_depth = 2;
    }

    match virtual_disk.merge(
        merge_virtual_disk::Flag::None as u32,
        &parameters,
        Some(&overlapped),
    ) {
        Err(ResultCode::IoPending) => wait_for_vhd_operation(&virtual_disk, &overlapped),
        Err(ResultCode::Success) => {
            panic!("Success case on a merge call with overlapped struct is unexpected!")
        }
        Err(error) => Err(error),
        Ok(()) => panic!("Success case on a merge call with overlapped struct is unexpected!"),
    }
}

/// Waits for the given operation.
pub fn wait_for_vhd_operation(
    virtual_disk: &VirtualDisk,
    overlapped: &Overlapped,
) -> Result<(), ResultCode> {
    loop {
        let progress = virtual_disk.get_operation_progress(overlapped)?;

        match progress.operation_status {
            winapi::shared::winerror::ERROR_IO_PENDING => {
                // Job is in progress
            }
            winapi::shared::winerror::ERROR_SUCCESS => {
                // Operation completed successfully
                return Ok(());
            }
            winapi::shared::winerror::ERROR_OPERATION_ABORTED => {
                // Job was canceled
                return Err(ResultCode::OperationAborted);
            }
            error => {
                // Job failed
                return Err(error_code_to_result_code(error));
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

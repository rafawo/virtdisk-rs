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

    let disk_path = virtual_disk.get_physical_path()?;
    let disk = Disk::open(
        &disk_path,
        None,
        Some(
            winapi::um::winnt::FILE_ATTRIBUTE_NORMAL | winapi::um::winbase::FILE_FLAG_NO_BUFFERING,
        ),
    )?;

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

    let disk_path = virtual_disk.get_physical_path()?;
    let disk = Disk::open(
        &disk_path,
        None,
        Some(
            winapi::um::winnt::FILE_ATTRIBUTE_NORMAL | winapi::um::winbase::FILE_FLAG_NO_BUFFERING,
        ),
    )?;

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

        let mut vhd_info = unsafe { std::mem::zeroed::<get_virtual_disk::Info>() };
        vhd_info.version = get_virtual_disk::InfoVersion::Size;
        let vhd_info_size = std::mem::size_of::<get_virtual_disk::InfoSize>() as u32;
        assert!(parent.get_information(vhd_info_size, &mut vhd_info)? >= vhd_info_size);

        block_size_in_bytes = unsafe { vhd_info.version_details.size.block_size };
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
    let disk_path = virtual_disk.get_physical_path()?;
    let disk = Disk::open(
        &disk_path,
        None,
        Some(
            winapi::um::winnt::FILE_ATTRIBUTE_NORMAL | winapi::um::winbase::FILE_FLAG_NO_BUFFERING,
        ),
    )?;

    disk.volume_path()
}

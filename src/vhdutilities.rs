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

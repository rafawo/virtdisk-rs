//! Wrappers around basic disk functions used to setup container storage.

use crate::windefs::*;
use crate::{error_code_to_result_code, ResultCode};

pub struct PartitionInfo {
    volume_path: String,
    disk_id: Guid,
    partition_id: Guid,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SetDiskAttributes {
    /// Specifies the size of the structure for versioning.
    pub version: DWord,

    /// Indicates whether to remember these settings across reboots or not.
    pub persist: Boolean,

    /// Reserved. Must set to zero.
    pub reserved1: [Byte; 3],

    /// Specifies the new attributes.
    pub attributes: DWordLong,

    /// Specifies the attributes that are being modified.
    pub attributes_mask: DWordLong,

    /// Reserved. Must set to zero.
    pub reserved2: [DWord; 4],
}

pub const DISK_ATTRIBUTE_OFFLINE: u64 = 0x0000000000000001;
pub const DISK_ATTRIBUTE_READ_ONLY: u64 = 0x0000000000000002;

/// Safe abstraction to a disk handle.
pub struct Disk {
    handle: Handle,
}

impl std::ops::Drop for Disk {
    fn drop(&mut self) {
        if self.handle == std::ptr::null_mut() {
            return;
        }

        #[allow(unused_assignments)]
        let mut result: Bool = 0;

        unsafe {
            result = winapi::um::handleapi::CloseHandle(self.handle);
        }

        match result {
            result if result == 0 => {
                panic!("Closing handle failed with error code {}", unsafe {
                    winapi::um::errhandlingapi::GetLastError()
                });
            }
            _ => {}
        }
    }
}

impl Disk {
    /// Wraps the supplied disk handle, providing a safe drop implementation that will close the handle
    /// on the end of its lifetime.
    pub fn wrap_handle(handle: Handle) -> Result<Disk, ResultCode> {
        match handle {
            handle if handle == std::ptr::null_mut() => Err(ResultCode::InvalidParameter),
            handle => Ok(Disk { handle }),
        }
    }

    /// Releases the wrapped handle to ensure that at the end of the lifetime of this Disk instance
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

    /// Returns a cloned value of the internally stored handle to the disk.
    /// This is useful so that the disk handle can be used on other Windows APIs.
    /// Be careful and do not close the handle returned here because the code will panic at the
    /// end of the lifetime of this Disk instance if CloseHandle fails.
    pub fn get_handle(&self) -> Handle {
        self.handle.clone()
    }

    /// Opens a disk by path. Path can be
    /// a volume path (e.g. \\?\Volume{4c1b02c1-d990-11dc-99ae-806e6f6e6963}\)
    /// or a device path (\\?\scsi#disk&ven_mtfddak1&prod_28mam-1j1#4.....)
    pub fn open(disk_path: &str, access_mask: Option<DWord>) -> Result<Disk, ResultCode> {
        use winapi::um::{fileapi, winbase, winnt};

        let access_mask_flags = match access_mask {
            Some(flags) => flags,
            None => winnt::GENERIC_READ | winnt::GENERIC_WRITE,
        };

        let mut normalized_disk_path = disk_path.to_string();

        if normalized_disk_path.chars().last().unwrap() == '\\' {
            normalized_disk_path.pop();
        }

        unsafe {
            let handle = fileapi::CreateFileW(
                widestring::WideCString::from_str(normalized_disk_path.as_str())
                    .unwrap()
                    .as_ptr(),
                access_mask_flags,
                winnt::FILE_SHARE_READ | winnt::FILE_SHARE_WRITE,
                std::ptr::null_mut(),
                fileapi::OPEN_EXISTING,
                winnt::FILE_ATTRIBUTE_NORMAL | winbase::FILE_FLAG_NO_BUFFERING,
                std::ptr::null_mut(),
            );

            match handle {
                handle if handle != std::ptr::null_mut() => Ok(Disk { handle }),
                _handle => Err(ResultCode::FileNotFound),
            }
        }
    }

    /// Force a volume to be brought online (ie: mounted by a filesystem).
    /// This is needed when automount has been disabled (mountvol /N).
    pub fn force_online(&self) -> Result<(), ResultCode> {
        const SET_DISK_ATTRIBUTES_SIZE: DWord = std::mem::size_of::<SetDiskAttributes>() as DWord;

        let mut params = SetDiskAttributes {
            version: SET_DISK_ATTRIBUTES_SIZE,
            persist: 0,
            reserved1: [0; 3],
            attributes: 0,
            attributes_mask: DISK_ATTRIBUTE_OFFLINE | DISK_ATTRIBUTE_READ_ONLY,
            reserved2: [0; 4],
        };

        unsafe {
            match winapi::um::ioapiset::DeviceIoControl(
                self.handle,
                winapi::um::winioctl::IOCTL_DISK_SET_DISK_ATTRIBUTES,
                &mut params as *mut _ as LPVoid,
                SET_DISK_ATTRIBUTES_SIZE,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) {
                0 => Ok(()),
                _ => Err(error_code_to_result_code(
                    winapi::um::errhandlingapi::GetLastError(),
                )),
            }
        }
    }

    pub fn volume_path(&self) -> String {
        String::new()
    }

    pub fn format(&self, file_system: &str) -> PartitionInfo {
        PartitionInfo {
            volume_path: String::new(),
            disk_id: GUID_NULL,
            partition_id: GUID_NULL,
        }
    }

    pub fn expand_volume(&self) -> bool {
        true
    }
}

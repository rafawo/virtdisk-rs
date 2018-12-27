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
        crate::win_wrappers::close_handle(&mut self.handle);
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
        use winapi::um::{fileapi, winnt};

        let access_mask_flags = match access_mask {
            Some(flags) => flags,
            None => winnt::GENERIC_READ | winnt::GENERIC_WRITE,
        };

        let mut normalized_disk_path = disk_path.to_string();

        if normalized_disk_path.chars().last().unwrap() == '\\' {
            normalized_disk_path.pop();
        }

        match crate::win_wrappers::create_file(
            normalized_disk_path.as_str(),
            access_mask_flags,
            winnt::FILE_SHARE_READ | winnt::FILE_SHARE_WRITE,
            None,
            fileapi::OPEN_EXISTING,
            winnt::FILE_ATTRIBUTE_NORMAL,
            None,
        ) {
            Ok(handle) => Ok(Disk { handle }),
            Err(error) => Err(error),
        }
    }

    /// Force the disk to be brought online and surface its volumes.
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

    /// Retrieves the path to the first volume on a disk, waiting for the volumes to arrive
    /// if the have not yet.
    pub fn volume_path(&self) -> Result<String, ResultCode> {
        use rsevents::Awaitable;
        use winapi::um::{cfgmgr32, winioctl};

        let mut filter = unsafe { std::mem::zeroed::<cfgmgr32::CM_NOTIFY_FILTER>() };
        filter.cbSize = std::mem::size_of::<cfgmgr32::CM_NOTIFY_FILTER>() as DWord;
        filter.FilterType = cfgmgr32::CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE;
        unsafe {
            filter.u.DeviceInterface_mut().ClassGuid = winioctl::GUID_DEVINTERFACE_VOLUME;
        }

        let mut context = VolumeArrivalCallbackContext {
            event: rsevents::AutoResetEvent::new(rsevents::State::Unset),
            path_result: Ok(String::new()),
            disk_handle: self.handle,
        };

        let cm_notification = CmNotification::register(
            &mut filter,
            &mut context as *mut _ as PVoid,
            Some(volume_arrival_callback),
        );

        if let Err(error) = cm_notification {
            return Err(error);
        }

        let mut volume_path = try_get_disk_volume_path(self.handle)?;

        if volume_path.is_empty() {
            pub const VOLUME_ARRIVAL_DEFAULT_FORCE_ONLINE_INTERVAL_MS: u64 = 10000; // 10 seconds
            pub const VOLUME_ARRIVAL_DEFAULT_TIMEOUT_MS: u64 = 60000; // 1 minute
            let force_online_interval = VOLUME_ARRIVAL_DEFAULT_FORCE_ONLINE_INTERVAL_MS;
            let volume_arrival_timeout = VOLUME_ARRIVAL_DEFAULT_TIMEOUT_MS;
            let mut time_waited: u64 = 0;

            //
            // wait for a volume to arrive
            //
            // Periodically attempt to online the disk to work around a race condition:
            //
            // The disk device may have come online and notified partmgr
            // to that process. If the disk had a conflicting disk or partition
            // signature, then partmgr may have kept the disk offline.
            // MountVhd (which is what kicks off the mount) attempts to force online
            // the disk, but that can actually race with partmgr handling the
            // disk arrival notification.
            //
            // So, if the user asks for the volume path, then:
            // 1. Always attempt to bring the disk online. If the disk is already online
            //      then this is a noop. If the disk is currently offline, then this could
            //      be racing with the online process described above.
            // 2. Wait for a small period of time.
            // 3. If the disk still isn't online, attempt to online it again, in case the bit
            //      above raced.
            // 4. Keep doing this until the volume comes online, or until we reach the timeout.
            //
            loop {
                let _result = force_online_disk(self.handle);

                if context
                    .event
                    .wait_for(std::time::Duration::from_millis(force_online_interval))
                {
                    volume_path = match context.path_result {
                        Ok(path) => path,
                        Err(error) => return Err(error),
                    };

                    if volume_path.is_empty() {
                        return Err(ResultCode::FileNotFound);
                    }

                    break;
                }

                time_waited += volume_arrival_timeout;

                if time_waited >= volume_arrival_timeout {
                    break;
                }
            }
        }

        match force_online_volume(&volume_path) {
            Ok(()) => Ok(volume_path),
            Err(error) => Err(error),
        }
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

/// Forces the disk to be brought online and surface its volumes.
pub fn force_online_disk(handle: Handle) -> Result<(), ResultCode> {
    let mut disk = Disk { handle };
    let result = disk.force_online();
    unsafe {
        disk.release_handle();
    }
    result
}

struct Volume {
    handle: Handle,
}

impl std::ops::Drop for Volume {
    fn drop(&mut self) {
        crate::win_wrappers::close_handle(&mut self.handle);
    }
}

impl Volume {
    pub fn open(path: &str, access_mask: Option<DWord>) -> Result<Volume, ResultCode> {
        use winapi::um::{fileapi, winnt};

        let access_mask_flags = match access_mask {
            Some(flags) => flags,
            None => winnt::GENERIC_READ | winnt::GENERIC_WRITE,
        };

        match crate::win_wrappers::create_file(
            path,
            access_mask_flags,
            winnt::FILE_SHARE_READ | winnt::FILE_SHARE_WRITE,
            None,
            fileapi::OPEN_EXISTING,
            winnt::FILE_ATTRIBUTE_NORMAL,
            None,
        ) {
            Ok(handle) => Ok(Volume { handle }),
            Err(error) => Err(error),
        }
    }
}

/// Force a volume to be brought online (ie: mounted by a filesystem).
/// This is needed when automount has been disabled (mountvol /N).
pub fn force_online_volume(volume_name: &str) -> Result<(), ResultCode> {
    use winapi::um::{ioapiset, winioctl};

    match Volume::open(volume_name, None) {
        Ok(volume) => {
            let mut bytes: DWord = 0;

            unsafe {
                if ioapiset::DeviceIoControl(
                    volume.handle,
                    winioctl::IOCTL_VOLUME_OFFLINE,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null_mut(),
                    0,
                    &mut bytes,
                    std::ptr::null_mut(),
                ) == 0
                {
                    return Err(error_code_to_result_code(
                        winapi::um::errhandlingapi::GetLastError(),
                    ));
                }

                if ioapiset::DeviceIoControl(
                    volume.handle,
                    winioctl::IOCTL_VOLUME_ONLINE,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null_mut(),
                    0,
                    &mut bytes,
                    std::ptr::null_mut(),
                ) == 0
                {
                    return Err(error_code_to_result_code(
                        winapi::um::errhandlingapi::GetLastError(),
                    ));
                }

                Ok(())
            }
        }
        Err(error) => Err(error),
    }
}

#[repr(C)]
#[allow(dead_code)]
struct StorageDeviceNumber {
    device_type: u32,
    device_number: DWord,
    partition_number: DWord,
}

#[allow(dead_code)]
struct SafeFindVolumeHandle {
    handle: Handle,
}

impl std::ops::Drop for SafeFindVolumeHandle {
    fn drop(&mut self) {
        if self.handle == std::ptr::null_mut() {
            return;
        }

        #[allow(unused_assignments)]
        let mut result: Bool = 0;

        unsafe {
            result = winapi::um::fileapi::FindVolumeClose(self.handle);
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

/// Tries to get the volume path of the volume in a disk.
/// Returns an empty string if the volume is not found.
fn try_get_disk_volume_path(handle: Handle) -> Result<String, ResultCode> {
    use winapi::um::{fileapi, ioapiset, winioctl};

    let mut dev_number = StorageDeviceNumber {
        device_type: 0,
        device_number: 0,
        partition_number: 0,
    };

    let mut bytes: DWord = 0;

    unsafe {
        if ioapiset::DeviceIoControl(
            handle,
            winapi::um::winioctl::IOCTL_STORAGE_GET_DEVICE_NUMBER,
            std::ptr::null_mut(),
            0,
            &mut dev_number as *mut _ as LPVoid,
            std::mem::size_of::<StorageDeviceNumber>() as DWord,
            &mut bytes,
            std::ptr::null_mut(),
        ) == 0
        {
            return Err(error_code_to_result_code(
                winapi::um::errhandlingapi::GetLastError(),
            ));
        }

        const MAX_PATH: usize = 256;
        let mut volume_name_buffer: [WChar; MAX_PATH] = [0; MAX_PATH];
        let find_volume_handle =
            fileapi::FindFirstVolumeW(volume_name_buffer.as_mut_ptr(), MAX_PATH as DWord);

        if find_volume_handle == std::ptr::null_mut() {
            return Err(error_code_to_result_code(
                winapi::um::errhandlingapi::GetLastError(),
            ));
        }

        let find_volume = SafeFindVolumeHandle {
            handle: find_volume_handle,
        };

        loop {
            let volume_name_wstr =
                widestring::WideCString::from_ptr_str(volume_name_buffer.as_ptr());
            let mut volume_name = volume_name_wstr.to_string_lossy();

            if volume_name.chars().last().unwrap() == '\\' {
                volume_name.pop();
                volume_name.push('\0');
            }

            if let Ok(volume) = Volume::open(&volume_name, Some(0)) {
                let mut extents = std::mem::zeroed::<winioctl::VOLUME_DISK_EXTENTS>();

                if ioapiset::DeviceIoControl(
                    volume.handle,
                    winioctl::IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
                    std::ptr::null_mut(),
                    0,
                    &mut extents as *mut _ as LPVoid,
                    std::mem::size_of::<winioctl::VOLUME_DISK_EXTENTS>() as DWord,
                    &mut bytes,
                    std::ptr::null_mut(),
                ) == 0
                {
                    if extents.Extents[0].DiskNumber == dev_number.device_number {
                        return Ok(volume_name);
                    }
                }
            }

            if fileapi::FindNextVolumeW(
                find_volume.handle,
                volume_name_buffer.as_mut_ptr(),
                MAX_PATH as DWord,
            ) == 0
            {
                break;
            }
        }
    }

    Ok(String::new())
}

/// Context structure used for asynchronous volume arrival.
struct VolumeArrivalCallbackContext {
    event: rsevents::AutoResetEvent,
    path_result: Result<String, ResultCode>,
    disk_handle: Handle,
}

/// The callback called when a new volume arrives in the system. Checks to see if the volume
/// we are looking for has arrived yet (i.e. if this is the correct one) and signals the waiter if so.
#[no_mangle]
unsafe extern "system" fn volume_arrival_callback(
    _: winapi::um::cfgmgr32::HCMNOTIFICATION,
    context: PVoid,
    action: winapi::um::cfgmgr32::CM_NOTIFY_ACTION,
    _: winapi::um::cfgmgr32::PCM_NOTIFY_EVENT_DATA,
    _: DWord,
) -> DWord {
    if action == winapi::um::cfgmgr32::CM_NOTIFY_ACTION_DEVICEINTERFACEARRIVAL {
        let mut callback_context: VolumeArrivalCallbackContext = std::ptr::read(context as *mut _);
        callback_context.path_result = try_get_disk_volume_path(callback_context.disk_handle);

        match callback_context.path_result {
            Ok(ref path) if !path.is_empty() => callback_context.event.set(),
            Err(_) => callback_context.event.set(),
            _ => {}
        }
    }

    winapi::shared::winerror::ERROR_SUCCESS
}

#[link(name = "cfgmgr32")]
extern "C" {
    pub fn CM_Register_Notification(
        pFilter: winapi::um::cfgmgr32::PCM_NOTIFY_FILTER,
        pContext: PVoid,
        pCallback: winapi::um::cfgmgr32::PCM_NOTIFY_CALLBACK,
        pNotifyContex: winapi::um::cfgmgr32::PHCMNOTIFICATION,
    ) -> winapi::um::cfgmgr32::CONFIGRET;

    pub fn CM_Unregister_Notification(
        NotifyContext: winapi::um::cfgmgr32::HCMNOTIFICATION,
    ) -> winapi::um::cfgmgr32::CONFIGRET;

    pub fn CM_MapCrToWin32Err(
        CmReturnCode: winapi::um::cfgmgr32::CONFIGRET,
        DefaultErr: DWord,
    ) -> DWord;
}

struct CmNotification {
    handle: winapi::um::cfgmgr32::HCMNOTIFICATION,
}

impl std::ops::Drop for CmNotification {
    fn drop(&mut self) {
        unsafe {
            match CM_Unregister_Notification(self.handle) {
                error if error != winapi::um::cfgmgr32::CR_SUCCESS => {
                    let error_code =
                        CM_MapCrToWin32Err(error, winapi::shared::winerror::ERROR_GEN_FAILURE);
                    panic!(
                        "Failed to unregister CM Notification with error code {}",
                        error_code
                    );
                }
                _ => {}
            }
        }
    }
}

impl CmNotification {
    pub fn register(
        filter: winapi::um::cfgmgr32::PCM_NOTIFY_FILTER,
        context: PVoid,
        callback: winapi::um::cfgmgr32::PCM_NOTIFY_CALLBACK,
    ) -> Result<CmNotification, ResultCode> {
        unsafe {
            let mut handle = std::mem::zeroed::<winapi::um::cfgmgr32::HCMNOTIFICATION>();

            match CM_Register_Notification(filter, context, callback, &mut handle) {
                error if error != winapi::um::cfgmgr32::CR_SUCCESS => {
                    Err(error_code_to_result_code(CM_MapCrToWin32Err(
                        error,
                        winapi::shared::winerror::ERROR_GEN_FAILURE,
                    )))
                }
                _ => Ok(CmNotification { handle }),
            }
        }
    }
}

// Copyright (c) 2019 Rafael Alcaraz Mercado. All rights reserved.
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
// THE SOURCE CODE IS AVAILABLE UNDER THE ABOVE CHOSEN LICENSE "AS IS", WITH NO WARRANTIES.

//! Wrappers around basic disk functions used to setup container storage.

use winutils_rs::diskformat::*;
use winutils_rs::errorcodes::{error_code_to_winresult_code, WinResult, WinResultCode};
use winutils_rs::utilities::*;
use winutils_rs::windefs::*;

#[allow(dead_code)]
pub struct PartitionInfo {
    volume_path: String,
    disk_id: Guid,
    partition_id: Guid,
}

const PARTITION_MSFT_RESERVED_GUID: Guid = Guid {
    Data1: 0xE3C9E316,
    Data2: 0x0B5C,
    Data3: 0x4DB8,
    Data4: [0x81, 0x7D, 0xF9, 0x2D, 0xF0, 0x02, 0x15, 0xAE],
};

const PARTITION_BASIC_DATA_GUID: Guid = Guid {
    Data1: 0xEBD0A0A2,
    Data2: 0xB9E5,
    Data3: 0x4433,
    Data4: [0x87, 0xC0, 0x68, 0xB6, 0xB7, 0x26, 0x99, 0xC7],
};

const GPT_BASIC_DATA_ATTRIBUTE_NO_DRIVE_LETTER: u64 = 0x8000000000000000;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct SetDiskAttributes {
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

const DISK_ATTRIBUTE_OFFLINE: u64 = 0x0000000000000001;
const DISK_ATTRIBUTE_READ_ONLY: u64 = 0x0000000000000002;

/// Safe abstraction to a disk handle.
pub struct Disk {
    handle: Handle,
}

impl std::ops::Drop for Disk {
    fn drop(&mut self) {
        close_handle(&mut self.handle);
    }
}

impl Disk {
    /// Wraps the supplied disk handle, providing a safe drop implementation that will close the handle
    /// on the end of its lifetime.
    pub fn wrap_handle(handle: Handle) -> WinResult<Disk> {
        match handle {
            handle if handle == std::ptr::null_mut() => Err(WinResultCode::ErrorInvalidArgument),
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
    pub fn open(
        disk_path: &str,
        access_mask: Option<DWord>,
        flags: Option<DWord>,
    ) -> WinResult<Disk> {
        use winapi::um::{fileapi, winnt};

        let access_mask_flags = match access_mask {
            Some(flags) => flags,
            None => winnt::GENERIC_READ | winnt::GENERIC_WRITE,
        };

        let file_flags = match flags {
            Some(flags) => flags,
            None => winnt::FILE_ATTRIBUTE_NORMAL,
        };

        let mut normalized_disk_path = disk_path.to_string();

        if normalized_disk_path.chars().last().unwrap() == '\\' {
            normalized_disk_path.pop();
        }

        normalized_disk_path.shrink_to_fit();

        match create_file(
            normalized_disk_path.as_str(),
            access_mask_flags,
            winnt::FILE_SHARE_READ | winnt::FILE_SHARE_WRITE,
            None,
            fileapi::OPEN_EXISTING,
            file_flags,
            None,
        ) {
            Ok(handle) => Ok(Disk { handle }),
            Err(error) => Err(error),
        }
    }

    /// Force the disk to be brought online and surface its volumes.
    pub fn force_online(&self) -> WinResult<()> {
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
                0 => Err(error_code_to_winresult_code(
                    winapi::um::errhandlingapi::GetLastError(),
                )),
                _ => Ok(()),
            }
        }
    }

    /// Retrieves the path to the first volume on a disk, waiting for the volumes to arrive
    /// if the have not yet.
    pub fn volume_path(&self) -> WinResult<String> {
        use winapi::um::{cfgmgr32, winioctl};

        let mut filter = unsafe { std::mem::zeroed::<cfgmgr32::CM_NOTIFY_FILTER>() };
        filter.cbSize = std::mem::size_of::<cfgmgr32::CM_NOTIFY_FILTER>() as DWord;
        filter.FilterType = cfgmgr32::CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE;
        unsafe {
            filter.u.DeviceInterface_mut().ClassGuid = winioctl::GUID_DEVINTERFACE_VOLUME;
        }

        let mut event = WinEvent::create(false, false, None, None).unwrap();
        let mut path_result = Ok(String::new());

        let mut context = VolumeArrivalCallbackContext {
            event: &mut event,
            path_result: &mut path_result,
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
            pub const VOLUME_ARRIVAL_DEFAULT_FORCE_ONLINE_INTERVAL_MS: DWord = 10000; // 10 seconds
            pub const VOLUME_ARRIVAL_DEFAULT_TIMEOUT_MS: DWord = 60000; // 1 minute
            let force_online_interval = VOLUME_ARRIVAL_DEFAULT_FORCE_ONLINE_INTERVAL_MS;
            let volume_arrival_timeout = VOLUME_ARRIVAL_DEFAULT_TIMEOUT_MS;
            let mut time_waited: DWord = 0;

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
                force_online_disk(self.handle)?;

                if context.event.wait(force_online_interval) == WinEventResult::WaitObject0 {
                    volume_path = match *context.path_result {
                        Ok(ref path) => String::from(path.as_str()),
                        Err(error) => return Err(error),
                    };

                    if volume_path.is_empty() {
                        return Ok(String::new());
                    }

                    break;
                }

                time_waited += volume_arrival_timeout;

                if time_waited >= volume_arrival_timeout {
                    break;
                }
            }

            if volume_path.is_empty() {
                return Ok(String::new());
            }
        }

        force_online_volume(&volume_path)?;
        Ok(volume_path)
    }

    /// Initializes, partitions, and formats the given disk into a single volume.
    pub fn format(&self, file_system: &str) -> WinResult<PartitionInfo> {
        use winapi::um::{ioapiset, winioctl};

        let format_module = WinLibrary::load(
            "fmifs.dll",
            winapi::um::libloaderapi::LOAD_LIBRARY_SEARCH_SYSTEM32,
        )?;
        let format_ex2_farproc = format_module.proc_address("FormatEx2")?;
        let format_ex2: FormatEx2Routine = unsafe { std::mem::transmute(format_ex2_farproc) };

        // Partition the disk
        unsafe {
            let mut create_disk = std::mem::zeroed::<winioctl::CREATE_DISK>();
            create_disk.PartitionStyle = winioctl::PARTITION_STYLE_GPT;
            let mut bytes: DWord = 0;

            if ioapiset::DeviceIoControl(
                self.handle,
                winioctl::IOCTL_DISK_CREATE_DISK,
                &mut create_disk as *mut _ as PVoid,
                std::mem::size_of::<winioctl::CREATE_DISK>() as DWord,
                std::ptr::null_mut(),
                0,
                &mut bytes,
                std::ptr::null_mut(),
            ) == 0
            {
                return Err(error_code_to_winresult_code(
                    winapi::um::errhandlingapi::GetLastError(),
                ));
            }

            #[repr(C)]
            struct Layout {
                info: winioctl::DRIVE_LAYOUT_INFORMATION_EX,
                partitions: [winioctl::PARTITION_INFORMATION_EX; 1],
            }

            const LAYOUT_BUFFER_SIZE: usize = std::mem::size_of::<Layout>()
                + std::mem::size_of::<winioctl::PARTITION_INFORMATION_EX>();
            let mut layout_buffer: [Byte; LAYOUT_BUFFER_SIZE] = [0; LAYOUT_BUFFER_SIZE];
            let layout: &mut Layout = std::mem::transmute(layout_buffer.as_mut_ptr());

            if ioapiset::DeviceIoControl(
                self.handle,
                winioctl::IOCTL_DISK_GET_DRIVE_LAYOUT_EX,
                std::ptr::null_mut(),
                0,
                layout_buffer.as_mut_ptr() as PVoid,
                std::mem::size_of::<Layout>() as DWord,
                &mut bytes,
                std::ptr::null_mut(),
            ) == 0
            {
                return Err(error_code_to_winresult_code(
                    winapi::um::errhandlingapi::GetLastError(),
                ));
            }

            assert!(LAYOUT_BUFFER_SIZE >= bytes as usize);

            let mut layout_mut_ref: &mut Layout = std::mem::transmute(layout_buffer.as_mut_ptr());
            let mut partition_info = PartitionInfo {
                volume_path: String::new(),
                disk_id: layout_mut_ref.info.u.Gpt().DiskId,
                partition_id: create_guid()?,
            };

            layout_mut_ref.info.PartitionCount = 2;

            let partition_entries = {
                let mut partition_1 = std::mem::zeroed::<winioctl::PARTITION_INFORMATION_EX>();
                partition_1.PartitionStyle = winioctl::PARTITION_STYLE_GPT;
                *partition_1.StartingOffset.QuadPart_mut() = 1024 * 1024; //MB
                *partition_1.PartitionLength.QuadPart_mut() = 128 * 1024 * 1024; // 128 MB
                partition_1.PartitionNumber = 0;
                partition_1.RewritePartition = 1;
                partition_1.u.Gpt_mut().PartitionType = PARTITION_MSFT_RESERVED_GUID;
                let start: i64 =
                    partition_1.StartingOffset.QuadPart() + partition_1.PartitionLength.QuadPart();

                let mut partition_2 = std::mem::zeroed::<winioctl::PARTITION_INFORMATION_EX>();
                partition_2.PartitionStyle = winioctl::PARTITION_STYLE_GPT;
                *partition_2.StartingOffset.QuadPart_mut() = start;
                *partition_2.PartitionLength.QuadPart_mut() =
                    (layout.info.u.Gpt().StartingUsableOffset.QuadPart()
                        + layout.info.u.Gpt().UsableLength.QuadPart())
                        - start;
                partition_2.PartitionNumber = 1;
                partition_2.RewritePartition = 1;
                partition_2.u.Gpt_mut().PartitionType = PARTITION_BASIC_DATA_GUID;
                partition_2.u.Gpt_mut().PartitionId = partition_info.partition_id;
                partition_2.u.Gpt_mut().Attributes = GPT_BASIC_DATA_ATTRIBUTE_NO_DRIVE_LETTER;

                (partition_1, partition_2)
            };

            layout_mut_ref.info.PartitionEntry[0] = partition_entries.0;
            let part_info = (&mut layout_mut_ref.info.PartitionEntry[0]
                as *mut winioctl::PARTITION_INFORMATION_EX)
                .offset(1);
            *part_info = partition_entries.1;

            if ioapiset::DeviceIoControl(
                self.handle,
                winioctl::IOCTL_DISK_SET_DRIVE_LAYOUT_EX,
                layout_buffer.as_mut_ptr() as *mut _ as PVoid,
                LAYOUT_BUFFER_SIZE as u32,
                std::ptr::null_mut(),
                0,
                &mut bytes,
                std::ptr::null_mut(),
            ) == 0
            {
                return Err(error_code_to_winresult_code(
                    winapi::um::errhandlingapi::GetLastError(),
                ));
            }

            // Get the mounted volume path
            partition_info.volume_path = volume_path_disk(self.handle)?;

            // Store a string that lives longer than the loop below.
            let label_string = widestring::WideCString::from_str("").unwrap();
            let label_string_ptr = label_string.into_raw();

            // This uses a static initialized context since FormatEx2 does not provide a context
            // pointer in its callback routine.
            let _lock = FORMAT_CONTEXT_LOCK
                .get_or_insert(std::sync::Mutex::new(0))
                .lock()
                .unwrap();

            FORMAT_CONTEXT = Some(FormatContext {
                event: WinEvent::create(true, false, None, None).unwrap(),
                result: WinResultCode::ErrorSuccess,
            });

            // Unfortunately, FormatEx2 can fail if another thread is accessing the volume, perhaps
            // because it is responding to the arrival notification. We will retry the format
            // three times before finally giving up.
            for _retry in 0..3 {
                // Format the volume without TxF or short name support.
                let mut format_param = std::mem::zeroed::<FmIfsFormatEx2Param>();
                format_param.major = 2;
                format_param.label_string = label_string_ptr;
                format_param.flags = FMIFS_FORMAT_QUICK
                    | FMIFS_FORMAT_TXF_DISABLE
                    | FMIFS_FORMAT_SHORT_NAMES_DISABLE
                    | FMIFS_FORMAT_FORCE;

                let mut volume_path_wstr =
                    widestring::WideString::from_str(&partition_info.volume_path).into_vec();
                volume_path_wstr.push(0);
                let mut file_system_wstr = widestring::WideString::from_str(file_system).into_vec();
                file_system_wstr.push(0);

                format_ex2(
                    volume_path_wstr.as_mut_ptr(),
                    FmIfsMediaType::FmMediaFixed,
                    file_system_wstr.as_mut_ptr(),
                    &mut format_param,
                    format_ex2_callback,
                );

                if let Some(ref context) = FORMAT_CONTEXT {
                    context.event.wait(winapi::um::winbase::INFINITE);
                    match context.result {
                        WinResultCode::ErrorSuccess => {
                            return Ok(partition_info);
                        }
                        _ => {
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                        }
                    };
                }
            }

            Err(WinResultCode::ErrorGenFailure)
        }
    }

    /// Expands the last basic partition and its file system to occupy any available space left on disk.
    /// Returns true if the file system was expanded, false if there is no more space left for further expansion.
    pub fn expand_volume(&self) -> WinResult<bool> {
        #[allow(unused_assignments, dead_code)]
        unsafe {
            use winapi::um::{errhandlingapi, ioapiset, winioctl};
            let mut result: bool = false;

            // Query the current partition layout
            let mut bytes_returned: DWord = 0;
            let mut drive_layout: winioctl::PDRIVE_LAYOUT_INFORMATION_EX = std::ptr::null_mut();
            let mut buffer: Vec<Byte> = Vec::new();

            #[repr(C)]
            struct ExpectedLayout {
                info: winioctl::DRIVE_LAYOUT_INFORMATION_EX,
                partitions: [winioctl::PARTITION_INFORMATION_EX; 1],
            }

            let mut expected_layout = std::mem::zeroed::<ExpectedLayout>();

            if ioapiset::DeviceIoControl(
                self.handle,
                winioctl::IOCTL_DISK_GET_DRIVE_LAYOUT_EX,
                std::ptr::null_mut(),
                0,
                &mut expected_layout as *mut _ as PVoid,
                std::mem::size_of::<ExpectedLayout>() as DWord,
                &mut bytes_returned,
                std::ptr::null_mut(),
            ) == 0
            {
                let error = errhandlingapi::GetLastError();

                if winapi::shared::winerror::ERROR_INSUFFICIENT_BUFFER != error {
                    return Err(error_code_to_winresult_code(error));
                }

                buffer.reserve(4096);

                if ioapiset::DeviceIoControl(
                    self.handle,
                    winioctl::IOCTL_DISK_GET_DRIVE_LAYOUT_EX,
                    std::ptr::null_mut(),
                    0,
                    buffer.as_mut_ptr() as PVoid,
                    buffer.len() as DWord,
                    &mut bytes_returned,
                    std::ptr::null_mut(),
                ) == 0
                {
                    return Err(error_code_to_winresult_code(
                        winapi::um::errhandlingapi::GetLastError(),
                    ));
                }

                drive_layout = std::mem::transmute(buffer.as_ptr());
            } else {
                drive_layout = &mut expected_layout.info;
            }

            // Find the last basic partition
            if (*drive_layout).PartitionStyle != winioctl::PARTITION_STYLE_GPT {
                return Err(WinResultCode::ErrorInvalidArgument);
            }

            let mut partition_info: winioctl::PPARTITION_INFORMATION_EX = std::ptr::null_mut();
            let mut drive_layout_ptr =
                &mut (*drive_layout).PartitionEntry[0] as winioctl::PPARTITION_INFORMATION_EX;

            for _i in 0..(*drive_layout).PartitionCount {
                if guid_are_equal(
                    &(*drive_layout_ptr).u.Gpt().PartitionType,
                    &PARTITION_BASIC_DATA_GUID,
                ) {
                    partition_info = drive_layout_ptr;
                    break;
                }
                drive_layout_ptr = drive_layout_ptr.offset(1);
            }

            if partition_info == std::ptr::null_mut() {
                return Err(WinResultCode::ErrorInvalidArgument);
            }

            // Determine the new partition size and extend the partition
            let current_partition_end: LongLong = (*partition_info).StartingOffset.QuadPart()
                + (*partition_info).PartitionLength.QuadPart();
            let new_partition_end: LongLong =
                (*drive_layout).u.Gpt().StartingUsableOffset.QuadPart()
                    + (*drive_layout).u.Gpt().UsableLength.QuadPart();

            assert!(current_partition_end <= new_partition_end);
            let mut new_partition_size: LongLong = *(*partition_info).PartitionLength.QuadPart();

            if current_partition_end < new_partition_end {
                #[repr(C)]
                struct DiskGrowPartition {
                    partition_number: DWord,
                    bytes_to_grow: winapi::shared::ntdef::LARGE_INTEGER,
                }

                let mut grow_partition = std::mem::zeroed::<DiskGrowPartition>();
                grow_partition.partition_number = (*partition_info).PartitionNumber;
                *grow_partition.bytes_to_grow.QuadPart_mut() =
                    new_partition_end - current_partition_end;

                new_partition_size += *grow_partition.bytes_to_grow.QuadPart();

                if ioapiset::DeviceIoControl(
                    self.handle,
                    winioctl::IOCTL_DISK_GROW_PARTITION,
                    &mut grow_partition as *mut _ as PVoid,
                    std::mem::size_of::<DiskGrowPartition>() as DWord,
                    std::ptr::null_mut(),
                    0,
                    &mut bytes_returned,
                    std::ptr::null_mut(),
                ) == 0
                {
                    return Err(error_code_to_winresult_code(
                        winapi::um::errhandlingapi::GetLastError(),
                    ));
                }
            }

            // Query the current file system size.
            let volume_path = volume_path_disk(self.handle)?;
            let ntfsinfo = get_ntfsinfo(&volume_path).unwrap();

            // Compute the new number of clusters (rounding down) and extend the file system.
            let new_number_of_clusters =
                new_partition_size / ntfsinfo.bytes_per_cluster as LongLong;
            let old_number_of_clusters = ntfsinfo.total_clusters - ntfsinfo.free_clusters;

            // NTFS may extend the volume by one sector less than requested (NtfsChangeVolumeSize),
            // so increase the current size by one to check if there's any space left.
            if old_number_of_clusters + 1 < new_number_of_clusters as u64 {
                let sectors_in_cluster = ntfsinfo.bytes_per_cluster / ntfsinfo.bytes_per_sector;
                let mut new_number_of_sectors =
                    new_number_of_clusters * sectors_in_cluster as LongLong;
                let volume = Volume::open(&volume_path, None)?;

                if ioapiset::DeviceIoControl(
                    volume.handle,
                    winioctl::FSCTL_EXTEND_VOLUME,
                    &mut new_number_of_sectors as *mut _ as PVoid,
                    std::mem::size_of::<LongLong>() as DWord,
                    std::ptr::null_mut(),
                    0,
                    &mut bytes_returned,
                    std::ptr::null_mut(),
                ) == 0
                {
                    return Err(error_code_to_winresult_code(
                        winapi::um::errhandlingapi::GetLastError(),
                    ));
                }

                result = true;
            }

            Ok(result)
        }
    }
}

/// Forces the disk to be brought online and surface its volumes.
pub fn force_online_disk(handle: Handle) -> WinResult<()> {
    let mut disk = Disk { handle };
    let result = disk.force_online();
    unsafe {
        disk.release_handle();
    }
    result
}

/// Retrieves the volume disk path.
pub fn volume_path_disk(handle: Handle) -> WinResult<String> {
    let mut disk = Disk { handle };
    let result = disk.volume_path();
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
        close_handle(&mut self.handle);
    }
}

impl Volume {
    pub fn open(path: &str, access_mask: Option<DWord>) -> WinResult<Volume> {
        use winapi::um::{fileapi, winnt};

        let access_mask_flags = match access_mask {
            Some(flags) => flags,
            None => winnt::GENERIC_READ | winnt::GENERIC_WRITE,
        };

        match create_file(
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
pub fn force_online_volume(volume_name: &str) -> WinResult<()> {
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
                    return Err(error_code_to_winresult_code(
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
                    return Err(error_code_to_winresult_code(
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
            0 => {
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
fn try_get_disk_volume_path(handle: Handle) -> WinResult<String> {
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
            return Err(error_code_to_winresult_code(
                winapi::um::errhandlingapi::GetLastError(),
            ));
        }

        const MAX_PATH: usize = 256;
        let mut volume_name_buffer: [WChar; MAX_PATH] = [0; MAX_PATH];
        let find_volume_handle =
            fileapi::FindFirstVolumeW(volume_name_buffer.as_mut_ptr(), MAX_PATH as DWord);

        if find_volume_handle == std::ptr::null_mut() {
            return Err(error_code_to_winresult_code(
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
            volume_name.shrink_to_fit();

            if volume_name.chars().last().unwrap() == '\\' {
                volume_name.pop();
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
                ) != 0
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
struct VolumeArrivalCallbackContext<'event, 'result> {
    event: &'event mut WinEvent,
    path_result: &'result mut WinResult<String>,
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
        let callback_context: VolumeArrivalCallbackContext = std::ptr::read(context as *mut _);
        *callback_context.path_result = try_get_disk_volume_path(callback_context.disk_handle);

        #[allow(unused_must_use)]
        {
            match callback_context.path_result {
                Ok(ref path) => {
                    if !path.is_empty() {
                        callback_context.event.set();
                    }
                }
                Err(_) => {
                    callback_context.event.set();
                }
            };
        }
    }

    winapi::shared::winerror::ERROR_SUCCESS
}

#[derive(Clone)]
pub struct NtFileSystemInfo {
    pub ntfs_volume_serial_number: u64,
    pub ntfs_version: String,
    pub lfs_version: String,
    pub total_sectors: u64,
    pub total_clusters: u64,
    pub free_clusters: u64,
    pub total_reserved_clusters: u64,
    pub reserved_for_storage_reserve: u64,
    pub bytes_per_sector: u32,
    pub bytes_per_physical_sector: u32,
    pub bytes_per_cluster: u32,
    pub bytes_per_file_record_segment: u32,
    pub clusters_per_file_record_segment: u32,
    pub mft_valid_data_length: u64,
    pub mft_start_lcn: u64,
    pub mft2_start_lcn: u64,
    pub mft_zone_start: u64,
    pub mft_zone_end: u64,
    pub mft_zone_size: u64,
    pub max_device_trim_extent_count: u32,
    pub max_device_trim_byte_count: u32,
    pub max_volume_trim_extent_count: u32,
    pub max_volume_trim_byte_count: u32,
    pub resource_manager_identifier: Guid,
}

pub fn get_ntfsinfo(volume_path: &str) -> WinResult<NtFileSystemInfo> {
    let command = format!("fsutil fsinfo ntfsinfo {}", volume_path);
    let output = std::process::Command::new("cmd")
        .args(&["/C", &command])
        .output();

    if output.is_err() {
        return Err(WinResultCode::ErrorGenFailure);
    }

    let output = output.unwrap();
    let output_string = String::from_utf8_lossy(&output.stdout);

    let mut ntfsinfo = NtFileSystemInfo {
        ntfs_volume_serial_number: 0,
        ntfs_version: String::new(),
        lfs_version: String::new(),
        total_sectors: 0,
        total_clusters: 0,
        free_clusters: 0,
        total_reserved_clusters: 0,
        reserved_for_storage_reserve: 0,
        bytes_per_sector: 0,
        bytes_per_physical_sector: 0,
        bytes_per_cluster: 0,
        bytes_per_file_record_segment: 0,
        clusters_per_file_record_segment: 0,
        mft_valid_data_length: 0,
        mft_start_lcn: 0,
        mft2_start_lcn: 0,
        mft_zone_start: 0,
        mft_zone_end: 0,
        mft_zone_size: 0,
        max_device_trim_extent_count: 0,
        max_device_trim_byte_count: 0,
        max_volume_trim_extent_count: 0,
        max_volume_trim_byte_count: 0,
        resource_manager_identifier: GUID_NULL,
    };

    let u64_from_byte_string_with_parenthesis = |byte_string: &str| -> u64 {
        let sections: Vec<_> = byte_string.trim().split("(").collect();
        if sections.len() != 2 {
            panic!("Expected 2 sections for {:?}", sections);
        }
        let value = sections[0].trim().replace(",", "");
        u64::from_str_radix(&value, 10).expect(&format!("Failed to parse u64 {}", value))
    };

    let u64_from_byte_string = |byte_string: &str| -> u64 {
        let byte_string = byte_string.trim();
        bytefmt::parse(byte_string).expect(&format!("Failed to parse byte string {}", byte_string))
    };

    for line in output_string.lines() {
        let splitted: Vec<_> = line.split(":").collect();
        if splitted.len() == 2 {
            match splitted[0].trim() {
                "NTFS Volume Serial Number" => {
                    let hex_value = splitted[1].trim().trim_start_matches("0x");
                    ntfsinfo.ntfs_volume_serial_number = u64::from_str_radix(hex_value, 16)
                        .expect(&format!("Failed to parse hex {} ", hex_value));
                }
                "NTFS Version" => {
                    ntfsinfo.ntfs_version = String::from(splitted[1].trim());
                }
                "LFS Version" => {
                    ntfsinfo.lfs_version = String::from(splitted[1].trim());
                }
                "Total Sectors" => {
                    ntfsinfo.total_sectors = u64_from_byte_string_with_parenthesis(splitted[1]);
                }
                "Total Clusters" => {
                    ntfsinfo.total_clusters = u64_from_byte_string_with_parenthesis(splitted[1]);
                }
                "Free Clusters" => {
                    ntfsinfo.free_clusters = u64_from_byte_string_with_parenthesis(splitted[1]);
                }
                "Total Reserved Clusters" => {
                    ntfsinfo.total_reserved_clusters =
                        u64_from_byte_string_with_parenthesis(splitted[1]);
                }
                "Reserved For Storage Reserve" => {
                    ntfsinfo.reserved_for_storage_reserve =
                        u64_from_byte_string_with_parenthesis(splitted[1]);
                }
                "Bytes Per Sector" => {
                    ntfsinfo.bytes_per_sector = splitted[1].trim().parse::<u32>().unwrap();
                }
                "Bytes Per Physical Sector" => {
                    ntfsinfo.bytes_per_physical_sector = splitted[1].trim().parse::<u32>().unwrap();
                }
                "Bytes Per Cluster" => {
                    ntfsinfo.bytes_per_cluster = splitted[1].trim().parse::<u32>().unwrap();
                }
                "Bytes Per FileRecord Segment" => {
                    ntfsinfo.bytes_per_file_record_segment =
                        splitted[1].trim().parse::<u32>().unwrap();
                }
                "Clusters Per FileRecord Segment" => {
                    ntfsinfo.clusters_per_file_record_segment =
                        splitted[1].trim().parse::<u32>().unwrap();
                }
                "Mft Valid Data Length" => {
                    ntfsinfo.mft_valid_data_length = u64_from_byte_string(splitted[1]);
                }
                "Mft Start Lcn" => {
                    let hex_value = splitted[1].trim().trim_start_matches("0x");
                    ntfsinfo.mft_start_lcn = u64::from_str_radix(hex_value, 16)
                        .expect(&format!("Failed to parse hex {} ", hex_value));
                }
                "Mft2 Start Lcn" => {
                    let hex_value = splitted[1].trim().trim_start_matches("0x");
                    ntfsinfo.mft2_start_lcn = u64::from_str_radix(hex_value, 16)
                        .expect(&format!("Failed to parse hex {} ", hex_value));
                }
                "Mft Zone Start" => {
                    let hex_value = splitted[1].trim().trim_start_matches("0x");
                    ntfsinfo.mft_zone_start = u64::from_str_radix(hex_value, 16)
                        .expect(&format!("Failed to parse hex {} ", hex_value));
                }
                "Mft Zone End" => {
                    let hex_value = splitted[1].trim().trim_start_matches("0x");
                    ntfsinfo.mft_zone_end = u64::from_str_radix(hex_value, 16)
                        .expect(&format!("Failed to parse hex {} ", hex_value));
                }
                "Mft Zone Size" => {
                    ntfsinfo.mft_zone_size = u64_from_byte_string(splitted[1]);
                }
                "Max Device Trim Extent Count" => {
                    ntfsinfo.max_device_trim_extent_count =
                        splitted[1].trim().parse::<u32>().unwrap();
                }
                "Max Device Trim Byte Count" => {
                    let hex_value = splitted[1].trim().trim_start_matches("0x");
                    ntfsinfo.max_device_trim_byte_count = u32::from_str_radix(hex_value, 16)
                        .expect(&format!("Failed to parse hex {} ", hex_value));
                }
                "Max Volume Trim Extent Count" => {
                    ntfsinfo.max_volume_trim_extent_count =
                        splitted[1].trim().parse::<u32>().unwrap();
                }
                "Max Volume Trim Byte Count" => {
                    let hex_value = splitted[1].trim().trim_start_matches("0x");
                    ntfsinfo.max_volume_trim_byte_count = u32::from_str_radix(hex_value, 16)
                        .expect(&format!("Failed to parse hex {} ", hex_value));
                }
                "Resource Manager Identifier" => {
                    ntfsinfo.resource_manager_identifier = parse_guid(splitted[1].trim()).unwrap();
                }
                _ => {}
            };
        }
    }

    Ok(ntfsinfo)
}

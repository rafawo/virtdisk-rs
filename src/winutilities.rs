use crate::windefs::*;
use crate::{error_code_to_result_code, ResultCode};

pub fn close_handle(handle: &mut Handle) {
    if *handle == std::ptr::null_mut() {
        return;
    }

    #[allow(unused_assignments)]
    let mut result: Bool = 0;

    unsafe {
        result = winapi::um::handleapi::CloseHandle(*handle);
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

pub fn create_file(
    path: &str,
    access_mask: DWord,
    share_mode: DWord,
    security_descriptor: Option<&mut winapi::um::minwinbase::SECURITY_ATTRIBUTES>,
    creation_disposition: DWord,
    flags_and_attributes: DWord,
    template_file: Option<Handle>,
) -> Result<Handle, crate::ResultCode> {
    let security_descriptor_ptr = match security_descriptor {
        Some(security_descriptor) => security_descriptor,
        None => std::ptr::null_mut(),
    };

    let template_file_handle = match template_file {
        Some(template_file) => template_file,
        None => std::ptr::null_mut(),
    };

    unsafe {
        let handle = winapi::um::fileapi::CreateFileW(
            widestring::WideCString::from_str(path).unwrap().as_ptr(),
            access_mask,
            share_mode,
            security_descriptor_ptr,
            creation_disposition,
            flags_and_attributes,
            template_file_handle,
        );

        match handle {
            handle if handle != std::ptr::null_mut() => Ok(handle),
            _handle => Err(crate::ResultCode::FileNotFound),
        }
    }
}

pub fn guid_are_equal(left: &Guid, right: &Guid) -> bool {
    left.Data1 == right.Data1
        && left.Data2 == right.Data2
        && left.Data3 == right.Data3
        && left.Data4 == right.Data4
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

pub struct CmNotification {
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

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WinEventResult {
    WaitObject0,
    WaitTimeout,
    WaitFailed(ResultCode),
}

pub struct WinEvent {
    handle: Handle,
}

impl std::ops::Drop for WinEvent {
    fn drop(&mut self) {
        close_handle(&mut self.handle);
    }
}

impl WinEvent {
    pub fn create(
        manual_reset: bool,
        initial_state: bool,
        name: Option<&str>,
        event_attributes: Option<winapi::um::minwinbase::SECURITY_ATTRIBUTES>,
    ) -> Result<WinEvent, ResultCode> {
        let event_attributes_ptr = match event_attributes {
            Some(mut event_attributes) => &mut event_attributes,
            None => std::ptr::null_mut(),
        };

        let name_wstr = match name {
            Some(name) => widestring::WideCString::from_str(name).unwrap(),
            None => widestring::WideCString::from_str("").unwrap(),
        };

        let name_ptr = match name {
            Some(_) => name_wstr.as_ptr(),
            None => std::ptr::null(),
        };

        let manual_reset: Bool = match manual_reset {
            true => 1,
            false => 0,
        };

        let initial_state: Bool = match initial_state {
            true => 1,
            false => 0,
        };

        unsafe {
            match winapi::um::synchapi::CreateEventW(
                event_attributes_ptr,
                manual_reset,
                initial_state,
                name_ptr,
            ) {
                handle if handle != std::ptr::null_mut() => Ok(WinEvent { handle }),
                _ => {
                    return Err(error_code_to_result_code(
                        winapi::um::errhandlingapi::GetLastError(),
                    ))
                }
            }
        }
    }

    pub fn open(
        name: &str,
        desired_access: DWord,
        inherit_handle: bool,
    ) -> Result<WinEvent, ResultCode> {
        let inherit_handle: Bool = match inherit_handle {
            true => 1,
            false => 0,
        };

        unsafe {
            match winapi::um::synchapi::OpenEventW(
                desired_access,
                inherit_handle,
                widestring::WideCString::from_str(name).unwrap().as_ptr(),
            ) {
                handle if handle != std::ptr::null_mut() => Ok(WinEvent { handle }),
                _ => {
                    return Err(error_code_to_result_code(
                        winapi::um::errhandlingapi::GetLastError(),
                    ))
                }
            }
        }
    }

    pub fn set(&self) -> Result<(), ResultCode> {
        unsafe {
            match winapi::um::synchapi::SetEvent(self.handle) {
                result if result != 0 => {
                    return Err(error_code_to_result_code(
                        winapi::um::errhandlingapi::GetLastError(),
                    ))
                }
                _ => Ok(()),
            }
        }
    }

    pub fn reset(&self) -> Result<(), ResultCode> {
        unsafe {
            match winapi::um::synchapi::ResetEvent(self.handle) {
                result if result != 0 => {
                    return Err(error_code_to_result_code(
                        winapi::um::errhandlingapi::GetLastError(),
                    ))
                }
                _ => Ok(()),
            }
        }
    }

    pub fn pulse(&self) -> Result<(), ResultCode> {
        unsafe {
            match winapi::um::winbase::PulseEvent(self.handle) {
                result if result != 0 => {
                    return Err(error_code_to_result_code(
                        winapi::um::errhandlingapi::GetLastError(),
                    ))
                }
                _ => Ok(()),
            }
        }
    }

    pub fn wait(&self, milliseconds: DWord) -> WinEventResult {
        unsafe {
            match winapi::um::synchapi::WaitForSingleObject(self.handle, milliseconds) {
                winapi::um::winbase::WAIT_OBJECT_0 => WinEventResult::WaitObject0,
                winapi::shared::winerror::WAIT_TIMEOUT => WinEventResult::WaitTimeout,
                winapi::um::winbase::WAIT_FAILED => WinEventResult::WaitFailed(
                    error_code_to_result_code(winapi::um::errhandlingapi::GetLastError()),
                ),
                _ => WinEventResult::WaitFailed(error_code_to_result_code(
                    winapi::um::errhandlingapi::GetLastError(),
                )),
            }
        }
    }
}

pub struct WinLibrary {
    handle: winapi::shared::minwindef::HMODULE,
}

impl std::ops::Drop for WinLibrary {
    fn drop(&mut self) {
        unsafe {
            match winapi::um::libloaderapi::FreeLibrary(self.handle) {
                result if result != 0 => {
                    panic!(
                        "Failed to free library with error code {}",
                        winapi::um::errhandlingapi::GetLastError(),
                    );
                }
                _ => {}
            }
        }
    }
}

impl WinLibrary {
    pub fn load(lib_file_name: &str, flags: DWord) -> Result<WinLibrary, ResultCode> {
        unsafe {
            match winapi::um::libloaderapi::LoadLibraryExW(
                widestring::WideCString::from_str(lib_file_name)
                    .unwrap()
                    .as_ptr(),
                std::ptr::null_mut(),
                flags,
            ) {
                handle if handle != std::ptr::null_mut() => Ok(WinLibrary { handle }),
                _ => Err(error_code_to_result_code(
                    winapi::um::errhandlingapi::GetLastError(),
                )),
            }
        }
    }

    pub fn proc_address(
        &self,
        proc_name: &str,
    ) -> Result<winapi::shared::minwindef::FARPROC, ResultCode> {
        unsafe {
            match winapi::um::libloaderapi::GetProcAddress(
                self.handle,
                std::ffi::CString::new(proc_name).unwrap().as_ptr(),
            ) {
                farproc if farproc != std::ptr::null_mut() => Ok(farproc),
                _ => Err(error_code_to_result_code(
                    winapi::um::errhandlingapi::GetLastError(),
                )),
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(non_snake_case, non_camel_case_types)]
pub enum FmIfsMediaType {
    FmMediaUnknown,
    FmMediaF5_160_512,   // 5.25", 160KB,  512 bytes/sector
    FmMediaF5_180_512,   // 5.25", 180KB,  512 bytes/sector
    FmMediaF5_320_512,   // 5.25", 320KB,  512 bytes/sector
    FmMediaF5_320_1024,  // 5.25", 320KB,  1024 bytes/sector
    FmMediaF5_360_512,   // 5.25", 360KB,  512 bytes/sector
    FmMediaF3_720_512,   // 3.5",  720KB,  512 bytes/sector
    FmMediaF5_1Pt2_512,  // 5.25", 1.2MB,  512 bytes/sector
    FmMediaF3_1Pt44_512, // 3.5",  1.44MB, 512 bytes/sector
    FmMediaF3_2Pt88_512, // 3.5",  2.88MB, 512 bytes/sector
    FmMediaF3_20Pt8_512, // 3.5",  20.8MB, 512 bytes/sector
    FmMediaRemovable,    // Removable media other than floppy
    FmMediaFixed,
    FmMediaF3_120M_512, // 3.5", 120M Floppy
    // FMR Sep.8.1994 SFT YAM
    // FMR Jul.14.1994 SFT KMR
    FmMediaF3_640_512, // 3.5" ,  640KB,  512 bytes/sector
    FmMediaF5_640_512, // 5.25",  640KB,  512 bytes/sector
    FmMediaF5_720_512, // 5.25",  720KB,  512 bytes/sector
    // FMR Sep.8.1994 SFT YAM
    // FMR Jul.14.1994 SFT KMR
    FmMediaF3_1Pt2_512, // 3.5" , 1.2Mb,   512 bytes/sector
    // FMR Sep.8.1994 SFT YAM
    // FMR Jul.14.1994 SFT KMR
    FmMediaF3_1Pt23_1024, // 3.5" , 1.23Mb, 1024 bytes/sector
    FmMediaF5_1Pt23_1024, // 5.25", 1.23MB, 1024 bytes/sector
    FmMediaF3_128Mb_512,  // 3.5" , 128MB,  512 bytes/sector  3.5"MO
    FmMediaF3_230Mb_512,  // 3.5" , 230MB,  512 bytes/sector  3.5"MO
    FmMediaF3_200Mb_512,  // 3.5" , 200MB,  512 bytes/sector  HiFD (200MB Floppy)
    FmMediaF3_240Mb_512,  // 3.5" , 240MB,  512 bytes/sector  HiFD (240MB Floppy)
    FmMediaEndOfData,     // Total data count.
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FmIfsPacketType {
    FmIfsPercentCompleted = 0,
    FmIfsFormatReport = 1,
    FmIfsInsertDisk = 2,
    FmIfsIncompatibleFileSystem = 3,
    FmIfsFormattingDestination = 4,
    FmIfsIncompatibleMedia = 5,
    FmIfsAccessDenied = 6,
    FmIfsMediaWriteProtected = 7,
    FmIfsCantLock = 8,
    FmIfsCantQuickFormat = 9,
    FmIfsIoError = 10,
    FmIfsFinished = 11,
    FmIfsBadLabel = 12,
    FmIfsCheckOnReboot = 13,
    FmIfsTextMessage = 14,
    FmIfsHiddenStatus = 15,
    FmIfsClusterSizeTooSmall = 16,
    FmIfsClusterSizeTooBig = 17,
    FmIfsVolumeTooSmall = 18,
    FmIfsVolumeTooBig = 19,
    FmIfsNoMediaInDevice = 20,
    FmIfsClustersCountBeyond32bits = 21,
    FmIfsCantChkMultiVolumeOfSameFS = 22,
    FmIfsFormatFatUsing64KCluster = 23,
    FmIfsDeviceOffLine = 24,
    FmIfsChkdskProgress = 25,
    FmIfsBadSectorInfo = 26,
    FmIfsBadUdfRevision = 27,
    FmIfsCantInvalidateFve = 28,
    FmIfsFveInvalidated = 29,
    FmIfsLowLevelLongTimeFormat = 30,
    FmIfsFormatHardwareFailure = 31,
    FmIfsCantContinueInReadOnly = 32,
    FmIfsCheckOnDismount = 33,
    FmIfsScanAlreadyRunning = 34,
    FmIfsClusterSizeIllegal = 35,
    FmIfsClusterSizeSectorSizeMismatch = 36,
    FmIfsPartitionNotClusterAligned = 37,
}

/// The structure below defines information to be passed into FormatEx2.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FmIfsFormatEx2Param {
    // These are fields supported in version 1.0
    pub major: UChar, // initial version is 1.0
    pub minor: UChar,
    pub flags: ULong,
    pub label_string: PWStr, // supplies the volume's label
    pub cluster_size: ULong, // supplies the cluster size for the volume

    // These are fields added in version 2.0
    pub version: UShort,   // supplies the UDF version
    pub context: ULongPtr, // context supplied on call-backs
    pub passes: UInt,      // number of passes of random data to make during format

    // There are fields added in version 2.1
    pub log_file_size: ULong, // supplies the initial size for $LogFile in bytes
}

pub type FmIfsCallback = extern "C" fn(
    packet_type: FmIfsPacketType,
    packet_length: ULong,
    packet_data: PVoid,
) -> Boolean;

pub type FormatEx2Routine = extern "C" fn(
    drive_name: PWStr,
    media_type: FmIfsMediaType,
    file_system_name: PWStr,
    param: *mut FmIfsFormatEx2Param,
    callback: FmIfsCallback,
);

pub const FMIFS_FORMAT_QUICK: u32 = 0x00000001;
pub const FMIFS_FORMAT_TXF_DISABLE: u32 = 0x00002000;
pub const FMIFS_FORMAT_SHORT_NAMES_DISABLE: u32 = 0x00000040;
pub const FMIFS_FORMAT_FORCE: u32 = 0x00000004;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FmIfsFinishedInformation {
    pub success: Boolean,
    pub final_result: ULong,
}

pub struct FormatContext {
    pub event: WinEvent,
    pub result: ResultCode,
}

pub static mut FORMAT_CONTEXT: Option<FormatContext> = None;

pub extern "C" fn format_ex2_callback(
    packet_type: FmIfsPacketType,
    _packet_length: ULong,
    packet_data: PVoid,
) -> Boolean {
    match packet_type {
        FmIfsPacketType::FmIfsFinished => {
            let info: FmIfsFinishedInformation = unsafe { std::mem::transmute(packet_data) };

            unsafe {
                if let Some(ref mut context) = FORMAT_CONTEXT {
                    context.result = match info.success {
                        result if result != 0 => ResultCode::Success,
                        _ => error_code_to_result_code(info.final_result),
                    };

                    if info.success != 0 && info.final_result == 0 {
                        // Format can fail without populating the FinalResult parameter, just assume general failure
                        context.result = ResultCode::GenFailure;
                    }

                    match context.event.set() {
                        Err(_) => panic!("Failed to signal event for format context"),
                        Ok(_) => {}
                    }
                }
            }
        }
        _ => {}
    }

    1
}

#[link(name = "Rpcrt4")]
extern "C" {
    pub fn UuidCreate(guid: *mut Guid) -> winapi::shared::rpc::RPC_STATUS;
}

pub fn create_guid() -> Result<Guid, ResultCode> {
    let mut guid: Guid = GUID_NULL;
    unsafe {
        match UuidCreate(&mut guid) {
            0 => Ok(guid),
            error_code => Err(ResultCode::WindowsErrorCode(error_code as u32)),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub union IoStatusBlockDetails {
    pub Status: winapi::shared::ntdef::NTSTATUS,
    pub Pointer: PVoid,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct IoStatusBlock {
    pub u: IoStatusBlockDetails,
    pub Information: ULongPtr,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct FileFsFullSizeInformation {
    pub TotalAllocationUnits: LargeInteger,
    pub CallerAvailableAllocationUnits: LargeInteger,
    pub ActualAvailableAllocationUnits: LargeInteger,
    pub SectorsPerAllocationUnit: ULong,
    pub BytesPerSector: ULong,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FsInfoClass {
    FileFsVolumeInformation          = 1,
    FileFsLabelInformation,         // 2
    FileFsSizeInformation,          // 3
    FileFsDeviceInformation,        // 4
    FileFsAttributeInformation,     // 5
    FileFsControlInformation,       // 6
    FileFsFullSizeInformation,      // 7
    FileFsObjectIdInformation,      // 8
    FileFsDriverPathInformation,    // 9
    FileFsVolumeFlagsInformation,   // 10
    FileFsSectorSizeInformation,    // 11
    FileFsDataCopyInformation,      // 12
    FileFsMetadataSizeInformation,  // 13
    FileFsFullSizeInformationEx,    // 14
    FileFsMaximumInformation,
}

#[link(name = "NotosKrnl")]
extern "C" {
    pub fn NtQueryVolumeInformationFile(
        FileHandle: Handle,
        IoStatusBlock: *mut IoStatusBlock,
        FsInformation: PVoid,
        Length: ULong,
        FsInformationClass: FsInfoClass,
    ) -> winapi::shared::ntdef::NTSTATUS;
}

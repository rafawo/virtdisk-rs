use crate::windefs::*;
use crate::{error_code_to_result_code, ResultCode};

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
        crate::win_wrappers::close_handle(&mut self.handle);
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

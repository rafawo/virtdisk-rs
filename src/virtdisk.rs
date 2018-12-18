//! This module provides Rust idiomatic abstractions to the C bindings of VirtDisk.
//! Both the FFI bindings and Rust wrappers are public to this crate, to give flexibility
//! to consumer code to use the bindings directly as they see fit.
//!

use crate::virtdisk_bindings::*;
use crate::virtdiskdefs::*;
use crate::windefs::*;

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
                panic!("Closing handle failed with error code {}", unsafe { kernel32::GetLastError() });
            },
            _ => {},
        }
    }
}

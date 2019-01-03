// Copyright © rafawo (rafawo1@hotmail.com). All rights reserved.
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
// THE SOURCE CODE IS AVAILABLE UNDER THE ABOVE CHOSEN LICENSE "AS IS", WITH NO WARRANTIES.

//! These tests verify basic workflows of the vhdutilities module, and not the entire crate.

use std::sync::atomic::{AtomicUsize, Ordering};
use virtdisk_rs::vhdutilities::*;

struct DeleteDiskScopeExit<'a> {
    filepath: &'a str,
}

impl<'a> std::ops::Drop for DeleteDiskScopeExit<'a> {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_file(self.filepath) {
            panic!("Failed to delete file {}: {}", self.filepath, error);
        };
    }
}

static FILE_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn can_create_base_vhd() {
    let disk_path = String::from(format!(
        "base_{}.vhdx",
        FILE_ID.fetch_add(1, Ordering::SeqCst)
    ));
    let _delete_file_scope_exit = DeleteDiskScopeExit {
        filepath: &disk_path,
    };

    let _mounted_volume = create_base_vhd(&disk_path, 1, 1, "NTFS").unwrap();
}

#[test]
fn can_open_vhd() {
    let disk_path = String::from(format!(
        "base_{}.vhdx",
        FILE_ID.fetch_add(1, Ordering::SeqCst)
    ));
    let _delete_file_scope_exit = DeleteDiskScopeExit {
        filepath: &disk_path,
    };

    let mounted_volume = create_base_vhd(&disk_path, 1, 1, "NTFS").unwrap();
    drop(mounted_volume);

    let _vhd = open_vhd(&disk_path, true).unwrap();
}

#[test]
fn can_expand_vhd() {
    let disk_path = String::from(format!(
        "base_{}.vhdx",
        FILE_ID.fetch_add(1, Ordering::SeqCst)
    ));
    let _delete_file_scope_exit = DeleteDiskScopeExit {
        filepath: &disk_path,
    };

    let mounted_volume = create_base_vhd(&disk_path, 20, 32, "NTFS").unwrap();
    drop(mounted_volume);

    let vhd = open_vhd(&disk_path, false).unwrap();
    assert!(expand_vhd(&vhd, 50 * 1024 * 1024 * 1024).unwrap());
}

#[test]
fn can_expand_volume() {
    let disk_path = String::from(format!(
        "base_{}.vhdx",
        FILE_ID.fetch_add(1, Ordering::SeqCst)
    ));
    let _delete_file_scope_exit = DeleteDiskScopeExit {
        filepath: &disk_path,
    };

    let mounted_volume = create_base_vhd(&disk_path, 20, 32, "NTFS").unwrap();
    drop(mounted_volume);

    let vhd = open_vhd(&disk_path, false).unwrap();
    assert!(expand_vhd(&vhd, 50 * 1024 * 1024 * 1024).unwrap());
    assert_eq!((), mount_vhd_temporarily_for_setup(&vhd).unwrap());

    let disk = open_vhd_backed_disk(&vhd).unwrap();
    assert!(disk.expand_volume().unwrap());
}

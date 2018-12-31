// Copyright © rafawo (rafawo1@hotmail.com). All rights reserved.
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
// THE SOURCE CODE IS AVAILABLE UNDER THE ABOVE CHOSEN LICENSE "AS IS", WITH NO WARRANTIES.

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

#[test]
fn dummy_test() {
    let disk_path = String::from("base.vhdx");
    let _delete_file_scope_exit = DeleteDiskScopeExit {
        filepath: &disk_path,
    };

    let _mounted_volume = create_base_vhd(&disk_path, 1, 1, "NTFS");
}

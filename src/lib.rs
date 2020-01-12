// Copyright (c) 2019 Rafael Alcaraz Mercado. All rights reserved.
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
// THE SOURCE CODE IS AVAILABLE UNDER THE ABOVE CHOSEN LICENSE "AS IS", WITH NO WARRANTIES.

//! Rust wrapper of VirtDisk APIs
//!
//! # Overview
//!
//! This project is a collection of Rust libraries that wrap functionality exposed by [VirtDisk](https://docs.microsoft.com/en-us/windows/desktop/api/virtdisk/).
//!
//! VirtDisk APIs are part of the [Windows 10 SDK](https://developer.microsoft.com/en-us/windows/downloads/windows-10-sdk).
//!
//! # Requirements
//!
//! For this wrapper to build properly, the following requirements need to be met by the building machine:
//!
//! - Windows 10 SDK version **10.0.17763.132**.
//! - **amd64** architecture.
//!   - This Rust wrapper, for now, expects to build only in amd64.
//!
//! # Wrapped Windows 10 SDK APIs
//!
//! **_Note: This section includes the paths in the Windows SDK for the header and lib files based on the default installation path `c:\Program Files (x86)\Windows Kits\10`._**
//!
//! The relevant Windows 10 SDK files that this project is wrapping are:
//! - C:\Program Files (x86)\Windows Kits\10\Include\10.0.18362.0\um\virtdisk.h
//! - C:\Program Files (x86)\Windows Kits\10\Lib\10.0.18362.0\um\x64\virtdisk.lib
//! - C:\Windows\System32\virtdisk.dll
//!

pub mod diskutilities;
pub mod vhdutilities;
pub mod virtdisk;
pub mod virtdiskdefs;

pub(crate) mod virtdisk_bindings;

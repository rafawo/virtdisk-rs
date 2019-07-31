# virtdisk-rs
Rust wrapper of VirtDisk APIs

## Overview

This project is a collection of Rust libraries that wrap functionality exposed by [VirtDisk](https://docs.microsoft.com/en-us/windows/desktop/api/virtdisk/).

VirtDisk APIs are part of the [Windows 10 SDK](https://developer.microsoft.com/en-us/windows/downloads/windows-10-sdk).

**NOTE:This crate is untested and simply provides safe Rust abstractions to the virtdisk C bindings. Fixes might come at later updates to the crate. There is no plan for now to create a fully suited integration test for the APIs.**

## Requirements

For this wrapper to build properly, the following requirements need to be met by the building machine:

- Windows 10 SDK version **10.0.18362.0**.
- **amd64** architecture.
  - This Rust wrapper, for now, expects to build only in amd64.

## Wrapped Windows 10 SDK APIs

**_Note: This section includes the paths in the Windows SDK for the header and lib files based on the default installation path `c:\Program Files (x86)\Windows Kits\10`._**

The relevant Windows 10 SDK files that this project is wrapping are:
- C:\Program Files (x86)\Windows Kits\10\Include\10.0.18362.0\um\virtdisk.h
- C:\Program Files (x86)\Windows Kits\10\Lib\10.0.18362.0\um\x64\virtdisk.lib
- C:\Windows\System32\virtdisk.dll

## How to use locally

Clone the repo to a folder:

```
git clone https://github.com/rafawo/virtdisk-rs.git
```

Make sure the machine where you are building has Windows 10 SDK version **10.0.17763.132** installed. Then run:

```
cd virtdisk-rs
cargo build
```

Finally, open documentation by running:
```
cargo doc --open
```

## Crates.io version notes

This section briefly describes all published crates.io [versions](https://crates.io/crates/virtdisk-rs/versions) of this project, ordered from latest to oldest.

- [**2.0.0 Jul 31, 2019**](https://crates.io/crates/virtdisk-rs/2.0.0)
  - Updated hardcoded dependency to Windows 10 SDK version 10.0.18362.0
  - Subtle dependencies to Windows RS5
- [**1.5.0 Jan 4, 2019**](https://crates.io/crates/virtdisk-rs/1.5.0)
  - Oldest stable version
  - Containers VHD and Disk utilities to aid container storage setup
  - API is tentatively finalized for this crate
  - Hardcoded dependency to Windows 10 SDK version 10.0.17763.0
  - Implementation has subtle dependencies to Windows RS4
- [**1.4.0 Jan 3, 2019**](https://crates.io/crates/virtdisk-rs/1.4.0)
  - **YANKED, DO NOT USE**
- [**1.3.0 Jan 3, 2019**](https://crates.io/crates/virtdisk-rs/1.3.0)
  - **YANKED, DO NOT USE**
- [**1.2.0 Jan 3, 2019**](https://crates.io/crates/virtdisk-rs/1.2.0)
  - **YANKED, DO NOT USE**
- [**1.1.1 Jan 2, 2019**](https://crates.io/crates/virtdisk-rs/1.1.1)
  - **YANKED, DO NOT USE**
- [**1.1.0 Jan 2, 2019**](https://crates.io/crates/virtdisk-rs/1.1.0)
  - **YANKED, DO NOT USE**
- [**1.0.1 Dec 31, 2018**](https://crates.io/crates/virtdisk-rs/1.0.1)
  - **YANKED, DO NOT USE**
- [**1.0.0 Dec 28, 2018**](https://crates.io/crates/virtdisk-rs/1.0.0)
  - **YANKED, DO NOT USE**
- [**0.1.2 Dec 20, 2018**](https://crates.io/crates/virtdisk-rs/0.1.2)
  - **YANKED, DO NOT USE**
- [**0.1.1 Dec 20, 2018**](https://crates.io/crates/virtdisk-rs/0.1.1)
  - **YANKED, DO NOT USE**
- [**0.1.0 Dec 19, 2018**](https://crates.io/crates/virtdisk-rs/0.1.0)
  - **YANKED, DO NOT USE**

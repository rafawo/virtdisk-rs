use virtdisk_rs::virtdiskdefs::*;

fn main() {
    println!("size of u8: {}", std::mem::size_of::<u8>());
    println!("size of u32: {}", std::mem::size_of::<u32>());
    println!("size of u64: {}", std::mem::size_of::<u64>());
    println!("size of VirtualDiskAccessMask: {}", std::mem::size_of::<VirtualDiskAccessMask>());
    println!("size of OpenVirtualDiskFlag: {}", std::mem::size_of::<OpenVirtualDiskFlag>());
    println!("size of CreateVirtualDiskVersion: {}", std::mem::size_of::<CreateVirtualDiskVersion>());
    println!("size of CreateVirtualDiskFlag: {}", std::mem::size_of::<CreateVirtualDiskFlag>());
    println!("size of AttachVirtualDiskVersion: {}", std::mem::size_of::<AttachVirtualDiskVersion>());
    println!("size of DetachVirtualDiskFlag: {}", std::mem::size_of::<DetachVirtualDiskFlag>());
    println!("size of DependentDiskFlag: {}", std::mem::size_of::<DependentDiskFlag>());
    println!("size of StorageDependencyInfoVersion: {}", std::mem::size_of::<StorageDependencyInfoVersion>());
    println!("size of GetStorageDependencyFlag: {}", std::mem::size_of::<GetStorageDependencyFlag>());
}
#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;

use log::info;
use uefi::mem::memory_map::MemoryMap;
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileHandle, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;

fn open_root_dir() -> uefi::Result<Directory> {
    let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle())?;
    let device_handle = loaded_image.device().expect("Device handle should exist.");
    let mut fs = boot::open_protocol_exclusive::<SimpleFileSystem>(device_handle)?;
    fs.open_volume()
}

fn save_memory_map(file: FileHandle) -> uefi::Result {
    let mut file = file.into_regular_file().unwrap();

    // Print header
    let header = "Index, Type, Type(name), PhysicalStart, NumberOfPages, Attribute\n";
    file.write(header.as_bytes()).unwrap();

    let memory_map = boot::memory_map(boot::MemoryType::LOADER_DATA)?;
    for (i, desc) in memory_map.entries().enumerate() {
        file.write(
            format!(
                "{}, {:#x}, {:?}, {:#08x}, {}, {:#x}\n",
                i,
                desc.ty.0,
                desc.ty,
                desc.phys_start,
                desc.page_count,
                desc.att.bits() & 0xfffff,
            )
            .as_bytes(),
        )
        .unwrap();
    }
    Ok(())
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Hello, mikanos-rs!");

    let mut root_dir = open_root_dir().expect("Failed to open root directory.");
    let memmap_file = root_dir
        .open(
            cstr16!("\\memmap"),
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .expect("Failed to open memmap file.");
    save_memory_map(memmap_file).expect("Failed to save memory map.");

    info!("All done.");
    boot::stall(10_000_000);
    loop {}
    Status::SUCCESS
}

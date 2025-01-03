#![no_std]
#![no_main]

extern crate alloc;
use alloc::{format, vec, vec::Vec};

use core::slice;

use goblin::elf;
use log::info;
use uefi::mem::memory_map::MemoryMap;
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{
    Directory, File, FileAttribute, FileHandle, FileInfo, FileMode, RegularFile,
};
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

fn read_file(file: &mut RegularFile) -> uefi::Result<Vec<u8>> {
    let info = file.get_boxed_info::<FileInfo>()?;
    let size = info.file_size() as usize;
    let mut buf = vec![0; size];
    file.read(&mut buf)?;
    Ok(buf)
}

fn load_elf(elf_data: &[u8]) -> elf::Elf {
    let prog = elf::Elf::parse(elf_data).unwrap();

    // Calculate address range
    let mut addr_start = usize::MAX;
    let mut addr_end = 0;
    for phdr in prog.program_headers.iter() {
        if phdr.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        addr_start = usize::min(addr_start, phdr.p_vaddr as usize);
        addr_end = usize::max(addr_end, (phdr.p_vaddr + phdr.p_memsz) as usize);
    }

    // Allocate memory for kernel image
    let memsz = addr_end - addr_start;
    let page_size = 0x1000;
    let page_cnt = (memsz + page_size - 1) / page_size;
    boot::allocate_pages(
        boot::AllocateType::Address(addr_start as u64),
        boot::MemoryType::LOADER_DATA,
        page_cnt,
    )
    .unwrap();

    // Copy loadable segments
    for phdr in prog.program_headers.iter() {
        if phdr.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        let dest =
            unsafe { slice::from_raw_parts_mut(phdr.p_vaddr as *mut u8, phdr.p_memsz as usize) };
        dest[..phdr.p_filesz as usize].copy_from_slice(
            &elf_data[phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize],
        );
        dest[phdr.p_filesz as usize..].fill(0);
    }

    prog
}

type EntryPoint = extern "sysv64" fn();
fn load_kernel(kernel_file: &mut RegularFile) -> uefi::Result<EntryPoint> {
    let buf = read_file(kernel_file)?;
    info!("Read kernel file: size={}", buf.len());
    let prog = load_elf(&buf);
    let entry: EntryPoint = unsafe { core::mem::transmute(prog.entry) };
    Ok(entry)
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

    let mut kernel_file = root_dir
        .open(
            cstr16!("\\kernel.elf"),
            FileMode::Read,
            FileAttribute::empty(),
        )
        .expect("Failed to open kernel file.")
        .into_regular_file()
        .unwrap();
    let entry = load_kernel(&mut kernel_file).expect("Failed to load kernel");
    info!("Successfully loaded kernel!");

    info!("Exiting boot services...");
    // Is it correct to use LOADER_DATA type here?
    let _ = unsafe { boot::exit_boot_services(boot::MemoryType::LOADER_DATA) };

    entry();

    info!("All done.");
    boot::stall(10_000_000);
    Status::SUCCESS
}

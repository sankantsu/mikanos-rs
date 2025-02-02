#![no_std]
#![no_main]

extern crate alloc;
use alloc::{vec, vec::Vec};

use core::slice;

use goblin::elf;
use log::info;
use uefi::mem::memory_map::MemoryMapOwned;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode, RegularFile};
use uefi::proto::media::fs::SimpleFileSystem;

use mikanos_rs_frame_buffer::FrameBuffer;

fn open_root_dir() -> uefi::Result<Directory> {
    let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle())?;
    let device_handle = loaded_image.device().expect("Device handle should exist.");
    let mut fs = boot::open_protocol_exclusive::<SimpleFileSystem>(device_handle)?;
    fs.open_volume()
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

type EntryPoint = extern "sysv64" fn(&FrameBuffer, &MemoryMapOwned);
fn load_kernel(kernel_file: &mut RegularFile) -> uefi::Result<EntryPoint> {
    let buf = read_file(kernel_file)?;
    info!("Read kernel file: size={}", buf.len());
    let prog = load_elf(&buf);
    let entry: EntryPoint = unsafe { core::mem::transmute(prog.entry) };
    Ok(entry)
}

fn open_gop() -> uefi::Result<boot::ScopedProtocol<GraphicsOutput>> {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;

    let gop = unsafe {
        boot::open_protocol::<GraphicsOutput>(
            boot::OpenProtocolParams {
                handle: gop_handle,
                agent: boot::image_handle(),
                controller: None,
            },
            // Don't open in exclusive mode.
            // That would break the connection between stdout and the video console.
            // ref: https://github.com/rust-osdev/uefi-rs/issues/524
            boot::OpenProtocolAttributes::GetProtocol,
        )?
    };
    Ok(gop)
}

fn log_gop_info(gop: &mut GraphicsOutput) {
    let (horizontal, vertical) = gop.current_mode_info().resolution();
    let pixel_format = gop.current_mode_info().pixel_format();
    let pixels_per_scanline = gop.current_mode_info().stride();
    info!(
        "Resolution: {}x{}, Pixel Format: {:?}, {} pixels/line",
        horizontal, vertical, pixel_format, pixels_per_scanline
    );

    let frame_buffer_base = gop.frame_buffer().as_mut_ptr() as usize;
    let frame_buffer_size = gop.frame_buffer().size();
    info!(
        "Frame Buffer: {:#x} - {:#x}, Size: {} bytes",
        frame_buffer_base,
        frame_buffer_base + frame_buffer_size,
        frame_buffer_size
    );
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Hello, mikanos-rs!");

    let mut root_dir = open_root_dir().expect("Failed to open root directory.");

    let mut gop = open_gop().expect("Failed to open gop.");
    let frame_buffer = FrameBuffer::new(&mut gop);
    log_gop_info(&mut gop);

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
    let memory_map = unsafe { boot::exit_boot_services(boot::MemoryType::LOADER_DATA) };
    entry(&frame_buffer, &memory_map);

    info!("All done.");
    boot::stall(10_000_000);
    Status::SUCCESS
}

#![no_std]
#![feature(lang_items, start)]
#![no_main]

#[macro_use]
extern crate log;

extern crate uefi;
extern crate uefi_services;

use core::mem::MaybeUninit;
use core::ptr;
use uefi::prelude::{Boot, Handle, Status, SystemTable};
use uefi::proto::console::gop;
use uefi::proto::console::gop::PixelFormat;
use uefi::proto::loaded_image;
use uefi::proto::media::file;
use uefi::proto::media::fs;
use uefi::table::boot::MemoryType;
use uefi::table::boot::SearchType;
use uefi::ResultExt;

fn reset_console(system_table: &SystemTable<Boot>) -> () {
    system_table
        .stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");
}

fn initialize_uefi_utilities(system_table: &SystemTable<Boot>) -> () {
    uefi_services::init(&system_table).expect_success("Failed to initialize_uefi_utilities");
}

fn get_buf_len_for_locate_handler(
    system_table: &SystemTable<Boot>,
    search_type: SearchType,
) -> usize {
    // To get the length of buffer, this function should be called with None.
    // See: https://docs.rs/uefi/0.4.7/uefi/table/boot/struct.BootServices.html#method.locate_handle
    system_table
        .boot_services()
        .locate_handle(search_type, None)
        .expect_success("Failed to get buffer length for locate_handler.")
}

fn malloc<T: Sized>(system_table: &SystemTable<Boot>, num: usize) -> uefi::Result<*mut T> {
    let buffer = system_table
        .boot_services()
        .allocate_pool(MemoryType::LOADER_DATA, num * core::mem::size_of::<T>());

    match buffer {
        Err(e) => Err(e),
        Ok(buf) => Ok(buf.map(|x| x as *mut T)),
    }
}

fn get_gop<'a>(system_table: &'a SystemTable<Boot>) -> &'a mut gop::GraphicsOutput<'a> {
    let gop = system_table
        .boot_services()
        .locate_protocol::<gop::GraphicsOutput>()
        .expect_success("Your computer does not support Graphics Output Protocol!");

    unsafe { &mut *gop.get() }
}

fn open_root_dir(image: &Handle, system_table: &SystemTable<Boot>) -> file::Directory {
    let loaded_image = system_table
        .boot_services()
        .handle_protocol::<loaded_image::LoadedImage>(*image)
        .expect_success("Failed to load image");

    let loaded_image = unsafe { &*loaded_image.get() };

    let simple_file_system = system_table
        .boot_services()
        .handle_protocol::<fs::SimpleFileSystem>(loaded_image.device())
        .expect_success("Failed to prepare simple file system.");

    let simple_file_system = unsafe { &mut *simple_file_system.get() };

    simple_file_system
        .open_volume()
        .expect_success("Failed to open volume.")
}

fn initialize(system_table: &SystemTable<Boot>) -> () {
    initialize_uefi_utilities(&system_table);
    reset_console(&system_table);
    info!("Hello World!");
}

fn is_usable_gop_mode(mode: &gop::ModeInfo) -> bool {
    if mode.pixel_format() != PixelFormat::BGR {
        return false;
    }

    // According to UEFI Specification 2.8 Errata A, P.479,
    // . : Pixel
    // P : Padding
    // ..........................................PPPPPPPPPP
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^|^^^^^^^^^^
    //             HorizontalResolution         | Paddings
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //                    PixelsPerScanLine
    //
    // This OS doesn't deal with pixel paddings, so return an error if pixel paddings exist.
    let (width, _) = mode.resolution();
    if width != mode.stride() {
        return false;
    }

    true
}

fn set_resolution(gop: &mut gop::GraphicsOutput) -> () {
    let mut max_height = 0;
    let mut max_width = 0;
    let mut preferred_mode = MaybeUninit::<gop::Mode>::uninit();

    for mode in gop.modes() {
        let mode = mode.expect("Failed to get gop mode.");

        let (width, height) = mode.info().resolution();
        if height > max_height && width > max_width && is_usable_gop_mode(&mode.info()) {
            max_height = height;
            max_width = width;
            unsafe { preferred_mode.as_mut_ptr().write(mode) }
        }
    }

    gop.set_mode(unsafe { &preferred_mode.assume_init() })
        .expect_success("Failed to set resolution.");

    info!("width: {} height: {}", max_width, max_height);
}

fn init_gop(image: &Handle, system_table: &SystemTable<Boot>) -> () {
    set_resolution(get_gop(system_table));
}

#[start]
#[no_mangle]
pub fn efi_main(image: Handle, system_table: SystemTable<Boot>) -> Status {
    initialize(&system_table);
    open_root_dir(&image, &system_table);
    info!("Opened volume");
    init_gop(&image, &system_table);
    info!("GOP set.");
    loop {}
}

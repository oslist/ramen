// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::str;
use core::ptr;
use x86_64::VirtAddr;

use super::ustar::Meta;

pub fn list_files(addr: VirtAddr) {
    let mut p = addr;
    while unsafe {
        ptr::read_unaligned((p + 257_u64).as_ptr() as *const [u8; 5]) == *"ustar".as_bytes()
    } {
        let meta: Meta = unsafe { ptr::read_unaligned(p.as_ptr()) };
        info!("{}", meta.name());
        p += (((meta.filesize_as_dec() + 511) / 512) + 1) * 512;
    }
}

pub fn find_file(addr: VirtAddr, key: &str) {
    let mut p = addr;
    while unsafe {
        ptr::read_unaligned((p + 257_u64).as_ptr() as *const [u8; 5]) == *"ustar".as_bytes()
    } {
        let meta: Meta = unsafe { ptr::read_unaligned(p.as_ptr()) };
        if meta.name() == key {
            info!("{:?}", meta);
        }
        p += (((meta.filesize_as_dec() + 511) / 512) + 1) * 512;
    }
}

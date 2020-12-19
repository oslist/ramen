// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::str;
use common::constant::INITRD_ADDR;
use core::ptr;
use x86_64::VirtAddr;

use super::ustar::Meta;

pub fn list_files(addr: VirtAddr) {
    for m in iter() {
        info!("{}", m.name());
    }
}

pub fn find_file(addr: VirtAddr, key: &str) {
    for m in iter() {
        if m.name() == key {
            info!("{:?}", m);
        }
    }
}

fn iter() -> impl Iterator<Item = Meta> {
    Iter::new()
}

struct Iter(VirtAddr);
impl Iter {
    fn new() -> Self {
        Self(INITRD_ADDR)
    }
}
impl Iterator for Iter {
    type Item = Meta;

    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { ustar_item(self.0) } {
            let m: Meta = unsafe { ptr::read_unaligned(self.0.as_ptr()) };
            self.0 = next_entry(self.0, &m);
            Some(m)
        } else {
            None
        }
    }
}

/// Safety: `p` must contain a valid address to the start of USTAR data.
unsafe fn ustar_item(p: VirtAddr) -> bool {
    ptr::read_unaligned((p + 257_u64).as_ptr() as *const [u8; 5]) == *"ustar".as_bytes()
}

fn next_entry(p: VirtAddr, m: &Meta) -> VirtAddr {
    p + (((m.filesize_as_dec() + 511) / 512) + 1) * 512
}

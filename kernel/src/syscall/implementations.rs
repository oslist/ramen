// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{mem::allocator, process};
use os_units::{Bytes, NumOfPages};
use x86_64::{
    instructions::{
        self, interrupts,
        port::{PortReadOnly, PortWriteOnly},
    },
    structures::paging::Size4KiB,
    PhysAddr, VirtAddr,
};

/// SAFETY: This function is unsafe because reading from I/O port may have side effects which
/// violate memory safety.
pub(super) unsafe fn sys_inb(port: u16) -> u8 {
    let mut p = PortReadOnly::new(port);
    p.read()
}

/// SAFETY: This function is unsafe because writing to I/O port may have side effects which violate
/// memory safety.
pub(super) unsafe fn sys_outb(port: u16, v: u8) -> u64 {
    let mut p = PortWriteOnly::new(port);
    p.write(v);
    0
}

/// SAFETY: This function is unsafe because reading from I/O port may have side effects which
/// violate memory safety.
pub(super) unsafe fn sys_inl(port: u16) -> u32 {
    let mut p = PortReadOnly::new(port);
    p.read()
}

/// SAFETY: This function is unsafe because writing to I/O port may have side effects which violate
/// memory safety.
pub(super) unsafe fn sys_outl(port: u16, v: u32) -> u64 {
    let mut p = PortWriteOnly::new(port);
    p.write(v);
    0
}

pub(super) fn sys_halt() -> u64 {
    instructions::hlt();
    0
}

pub(super) fn sys_disable_interrupt() -> u64 {
    interrupts::disable();
    0
}

pub(super) fn sys_enable_interrupt() -> u64 {
    interrupts::enable();
    0
}

pub(super) fn sys_enable_interrupt_and_halt() -> u64 {
    interrupts::enable_and_hlt();
    0
}

pub(super) fn sys_allocate_pages(num_of_pages: NumOfPages<Size4KiB>) -> VirtAddr {
    allocator::allocate_pages(num_of_pages).unwrap_or_else(VirtAddr::zero)
}

pub(super) fn sys_deallocate_pages(virt: VirtAddr, pages: NumOfPages<Size4KiB>) -> u64 {
    allocator::deallocate_pages(virt, pages);
    0
}

pub(super) fn sys_map_pages(start: PhysAddr, bytes: Bytes) -> VirtAddr {
    crate::mem::map_pages(start, bytes)
}

pub(super) fn sys_unmap_pages(start: VirtAddr, bytes: Bytes) -> u64 {
    crate::mem::unmap_pages(start, bytes);
    0
}

pub(super) fn sys_getpid() -> i32 {
    process::manager::getpid()
}

pub(super) fn sys_exit() -> ! {
    process::manager::exit();
}

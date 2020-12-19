// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(asm)]
#![deny(clippy::pedantic)]

use core::convert::TryInto;

use num_derive::FromPrimitive;
use os_units::{Bytes, NumOfPages};
use x86_64::{structures::paging::Size4KiB, PhysAddr, VirtAddr};

/// Safety: This function is unsafe because writing a value to I/O port may have side effects which
/// violate memory safety.
pub unsafe fn outb(port: u16, value: u8) {
    general_syscall(Syscalls::Outb, port.into(), value.into());
}

/// Safety: This function is unsafe because reading a value from I/O port may have side effects which violate memory safety.
pub unsafe fn inl(port: u16) -> u32 {
    general_syscall(Syscalls::Inl, port.into(), 0)
        .try_into()
        .unwrap()
}

/// Safety: This function is unsafe because writing a value via I/O port may have side effects
/// which violate memory safety.
pub unsafe fn outl(port: u16, value: u32) {
    general_syscall(Syscalls::Outl, port.into(), value.into());
}

pub fn halt() {
    // Safety: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Syscalls::Halt, 0, 0) };
}

pub fn disable_interrupt() {
    // Safety: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Syscalls::DisableInterrupt, 0, 0) };
}

pub fn enable_interrupt() {
    // Safety: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Syscalls::EnableInterrupt, 0, 0) };
}

pub fn enable_interrupt_and_halt() {
    // Safety: This operation is safe as it does not touch any unsafe things.
    unsafe { general_syscall(Syscalls::EnableInterruptAndHalt, 0, 0) };
}

pub fn allocate_pages(pages: NumOfPages<Size4KiB>) -> VirtAddr {
    // Safety: This operation is safe as the arguments are propertly passed.
    VirtAddr::new(unsafe {
        general_syscall(
            Syscalls::AllocatePages,
            pages.as_usize().try_into().unwrap(),
            0,
        )
    })
}

pub fn deallocate_pages(virt: VirtAddr, pages: NumOfPages<Size4KiB>) {
    // Safety: This operation is safe as the all arguments are propertly passed.
    unsafe {
        general_syscall(
            Syscalls::DeallocatePages,
            virt.as_u64(),
            pages.as_usize().try_into().unwrap(),
        )
    };
}

pub fn map_pages(start: PhysAddr, bytes: Bytes) -> VirtAddr {
    // Safety: This operation is safe as the all arguments are propertly passed.
    VirtAddr::new(unsafe {
        general_syscall(
            Syscalls::MapPages,
            start.as_u64(),
            bytes.as_usize().try_into().unwrap(),
        )
    })
}

pub fn unmap_pages(start: VirtAddr, bytes: Bytes) {
    unsafe {
        general_syscall(
            Syscalls::UnmapPages,
            start.as_u64(),
            bytes.as_usize().try_into().unwrap(),
        );
    }
}

/// Safety: This function is unsafe if arguments are invalid.
unsafe fn general_syscall(ty: Syscalls, a1: u64, a2: u64) -> u64 {
    let ty = ty as u64;
    let r: u64;
    asm!("syscall",
        inout("rax") ty => r, inout("rdi") a1 => _, inout("rsi") a2 => _, out("rdx") _,
        out("rcx") _, out("r8") _, out("r9") _, out("r10") _, out("r11") _,);
    r
}

#[derive(FromPrimitive)]
pub enum Syscalls {
    Inb,
    Outb,
    Inl,
    Outl,
    Halt,
    DisableInterrupt,
    EnableInterrupt,
    EnableInterruptAndHalt,
    AllocatePages,
    DeallocatePages,
    MapPages,
    UnmapPages,
}

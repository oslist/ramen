// SPDX-License-Identifier: GPL-3.0-or-later

use os_units::{Bytes, NumOfPages};
use x86_64::{
    instructions::port::{PortReadOnly, PortWriteOnly},
    structures::paging::Size4KiB,
    PhysAddr, VirtAddr,
};

pub enum SystemTask {
    Inb(PortReadOnly<u8>),
    Outb(PortWriteOnly<u8>, u8),
    Inl(PortReadOnly<u32>),
    Outl(PortWriteOnly<u32>, u32),
    Halt,
    DisableInterrupt,
    EnableInterrupt,
    EnableInterruptAndHalt,
    AllocatePages(NumOfPages<Size4KiB>),
    DeallocatePages(VirtAddr, NumOfPages<Size4KiB>),
    MapPages(PhysAddr, Bytes),
    UnmapPages(VirtAddr, Bytes),
}

// SPDX-License-Identifier: GPL-3.0-or-later

mod manager;

use core::{convert::TryInto, mem, ptr};

use crate::mem::{allocator::page_box::PageBox, paging::pml4::PML4};
use os_units::Bytes;
use x86_64::{
    structures::paging::{PageSize, PageTable, PageTableFlags, Size4KiB},
    VirtAddr,
};

pub struct Process {
    pml4: PageBox<PageTable>,
    stack: PageBox<[u8]>,
    rsp: VirtAddr,
    rip: VirtAddr,
    f: fn(),
}
impl Process {
    fn new(f: fn()) -> Self {
        let stack_size = Bytes::new(Size4KiB::SIZE.try_into().unwrap());
        let stack = PageBox::new_slice(0, stack_size.as_usize());
        let rsp = stack.virt_addr() + stack_size.as_usize() - mem::size_of::<InitialStack>();

        Self {
            pml4: PageBox::new(PageTable::new()),
            stack,
            rsp,
            rip: VirtAddr::new((exec as usize).try_into().unwrap()),
            f,
        }
    }

    fn init(&mut self) {
        self.enable_recursive_mapping();
        self.map_kernel_entry();
        self.push_initial_register_values();
    }

    fn enable_recursive_mapping(&mut self) {
        let a = self.pml4.phys_addr();
        let f =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        self.pml4[511].set_addr(a, f)
    }

    fn map_kernel_entry(&mut self) {
        let a = PML4.lock().level_4_table()[510].addr();
        let f = PML4.lock().level_4_table()[510].flags();
        self.pml4[510].set_addr(a, f);
    }

    fn push_initial_register_values(&mut self) {
        let s = self.initial_stack();

        // Safety: This operation is safe as the pointer is valid and is aligned.
        unsafe { ptr::write(self.rsp.as_mut_ptr(), s) }
    }

    fn initial_stack(&self) -> InitialStack {
        let self_addr = VirtAddr::new(self as *const Self as u64);
        InitialStack::new(self_addr, self.rip)
    }
}

/// Safety: `proc` must be a valid pointer.
unsafe fn exec(p: *const Process) {
    let p = &*p;
    (p.f)();
}

#[repr(C)]
struct InitialStack {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rbp: u64,
    rsi: u64,
    rdi: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rip: VirtAddr,
}
impl InitialStack {
    fn new(process: VirtAddr, rip: VirtAddr) -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rbp: 0,
            rdi: process.as_u64(),
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip,
        }
    }
}

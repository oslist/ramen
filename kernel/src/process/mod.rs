// SPDX-License-Identifier: GPL-3.0-or-later

mod context;
mod manager;

use core::{
    convert::TryInto,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    gdt::GDT,
    mem::{allocator::page_box::PageBox, paging::pml4::PML4},
};
use alloc::vec::Vec;
use manager::Manager;
use spinning_top::Spinlock;
use x86_64::{
    instructions::interrupts,
    registers::rflags,
    structures::{
        idt::InterruptStackFrameValue,
        paging::{page_table::PageTableEntry, PageSize, PageTable, PageTableFlags, Size4KiB},
    },
    PhysAddr, VirtAddr,
};

static QUEUE: Spinlock<Vec<Process>> = Spinlock::new(Vec::new());
static CURRENT: AtomicUsize = AtomicUsize::new(0);

fn task_a() {
    info!("Task A");
    loop {}
}

fn task_b() {
    info!("Task B");
    loop {}
}

/// Safety: `current_rsp` must be a valid pointer to RSP value.
unsafe fn switch_context(current_rsp: *mut u64, old_rsp: u64) {
    asm!("
    # Save general registers
        push rbp
        push r15
        push r14
        push r13
        push r12
        push r11
        push r10
        push r9
        push r8
        push rdi
        push rsi
        push rdx
        push rcx
        push rax

    # Save the current rsp
        mov [{}], rsp
    # switch rsp
        mov rsp, {}

    # Restore general registers
        pop rax
        pop rcx
        pop rdx
        pop rsi
        pop rdi
        pop r8
        pop r9
        pop r10
        pop r11
        pop r12
        pop r13
        pop r14
        pop r15
        pop rbp", out(reg) *current_rsp, in(reg) old_rsp);
}

pub struct Process {
    pml4: PageBox<PageTable>,
    rip: VirtAddr,
    rsp: VirtAddr,
    stack: PageBox<[u8]>,
    stack_frame: PageBox<[u8]>,
    running: bool,
}
impl Process {
    fn new(f: fn()) -> Self {
        let stack = PageBox::new_slice(0, Size4KiB::SIZE.try_into().unwrap());
        Self {
            pml4: Pml4Creator::new().create(),
            rip: VirtAddr::new((f as usize).try_into().unwrap()),
            rsp: stack.virt_addr() + stack.bytes().as_usize(),
            stack,
            stack_frame: PageBox::new_slice(0, Size4KiB::SIZE.try_into().unwrap()),
            running: true,
        }
    }

    fn initial_stack_frame(&self) -> InterruptStackFrameValue {
        InterruptStackFrameValue {
            instruction_pointer: self.rip,
            code_segment: GDT.user_code.0.into(),
            cpu_flags: rflags::read().bits(),
            stack_pointer: self.stack_bottom_addr(),
            stack_segment: GDT.user_data.0.into(),
        }
    }

    fn stack_bottom_addr(&self) -> VirtAddr {
        self.stack.virt_addr() + self.stack.bytes().as_usize()
    }
}

struct Pml4Creator {
    pml4: PageBox<PageTable>,
}
impl Pml4Creator {
    fn new() -> Self {
        Self {
            pml4: PageBox::new(PageTable::new()),
        }
    }

    fn create(mut self) -> PageBox<PageTable> {
        self.enable_recursive_mapping();
        self.map_kernel_regions();
        self.pml4
    }

    fn enable_recursive_mapping(&mut self) {
        let a = self.pml4.phys_addr();
        self.pml4[511].set_addr(a, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
    }

    fn map_kernel_regions(&mut self) {
        // Kernel region starts from `0xffff_ffff_8000_0000`.
        let p3 = PML4.lock().level_4_table()[510].addr();
        self.pml4[510].set_addr(p3, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
    }
}

pub fn switch(rsp: VirtAddr) -> VirtAddr {
    Manager::switch_process(rsp)
}

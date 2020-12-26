// SPDX-License-Identifier: GPL-3.0-or-later

mod manager;
mod message;
mod stack_frame;

use crate::{mem::allocator::page_box::PageBox, tss::TSS};
use alloc::{boxed::Box, vec::Vec};
use common::constant::INTERRUPT_STACK;
use core::convert::TryInto;
use message::Message;
use stack_frame::StackFrame;
use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    VirtAddr,
};

pub fn init() {
    TSS.lock().interrupt_stack_table[0] = INTERRUPT_STACK;
}

pub fn add(p: Process) {
    manager::add_process(p);
}

pub fn switch() -> VirtAddr {
    manager::switch_process()
}

pub struct Process {
    _stack: PageBox<[u8]>,
    messages: Vec<Message>,
    stack_frame: PageBox<StackFrame>,
}
impl Process {
    pub fn new_kernel(f: fn() -> !) -> Self {
        Self::new(f, Ty::Kernel)
    }

    pub fn new_user(f: fn() -> !) -> Self {
        Self::new(f, Ty::User)
    }

    fn new(f: fn() -> !, t: Ty) -> Self {
        let stack = PageBox::new_slice(0, Size4KiB::SIZE.try_into().unwrap());
        let stack_bottom_addr = stack.virt_addr() + stack.bytes().as_usize();
        let rip = VirtAddr::new((f as usize).try_into().unwrap());
        let stack_frame = PageBox::new(match t {
            Ty::Kernel => StackFrame::new_kernel(rip, stack_bottom_addr),
            Ty::User => StackFrame::new_user(rip, stack_bottom_addr),
        });

        Self {
            _stack: stack,
            messages: Vec::new(),
            stack_frame,
        }
    }

    fn stack_frame_top_addr(&self) -> VirtAddr {
        self.stack_frame.virt_addr()
    }

    fn stack_frame_bottom_addr(&self) -> VirtAddr {
        self.stack_frame_top_addr() + self.stack_frame.bytes().as_usize()
    }
}

#[derive(Copy, Clone)]
enum Ty {
    Kernel,
    User,
}

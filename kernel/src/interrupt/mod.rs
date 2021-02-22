// SPDX-License-Identifier: GPL-3.0-or-later

use crate::tss::TSS;
use common::constant::INTERRUPT_STACK;

pub mod apic;
pub mod handler;
mod idt;
pub mod timer;

pub fn init() {
    idt::init();
    set_initial_interrupt_stack_frame();
}

fn set_initial_interrupt_stack_frame() {
    TSS.lock().privilege_stack_table[0] = INTERRUPT_STACK;
}

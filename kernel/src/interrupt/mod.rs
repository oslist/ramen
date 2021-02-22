// SPDX-License-Identifier: GPL-3.0-or-later

pub mod apic;
pub mod handler;
mod idt;
pub mod timer;

pub fn init() {
    idt::init();
}

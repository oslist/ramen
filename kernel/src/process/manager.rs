// SPDX-License-Identifier: GPL-3.0-or-later

use super::Process;
use alloc::vec::Vec;
use spinning_top::Spinlock;

static MANAGER: Spinlock<Manager> = Spinlock::new(Manager::new());

struct Manager {
    process: Vec<Process>,
    current: usize,
}
impl Manager {
    const fn new() -> Self {
        Self {
            process: Vec::new(),
            current: 0,
        }
    }
}

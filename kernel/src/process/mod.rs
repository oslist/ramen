// SPDX-License-Identifier: GPL-3.0-or-later

mod manager;

use crate::mem::{allocator::page_box::PageBox, paging::pml4::PML4};
use x86_64::structures::paging::{PageTable, PageTableFlags};

pub struct Process {
    pml4: PageBox<PageTable>,
}
impl Process {
    fn new() -> Self {
        Self {
            pml4: PageBox::new(PageTable::new()),
        }
    }

    fn init(&mut self) {
        self.enable_recursive_mapping();
        self.map_kernel_entry();
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
}

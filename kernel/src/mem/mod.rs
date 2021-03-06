// SPDX-License-Identifier: GPL-3.0-or-later

use allocator::{phys::FRAME_MANAGER, virt};
use core::convert::TryFrom;
use os_units::Bytes;
use paging::pml4::PML4;
use x86_64::{
    structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

pub mod accessor;
pub mod allocator;
pub mod paging;

pub fn map_pages(start: PhysAddr, object_size: Bytes) -> VirtAddr {
    let start_frame_addr = start.align_down(Size4KiB::SIZE);
    let end_frame_addr = (start + object_size.as_usize()).align_down(Size4KiB::SIZE);

    let num_pages = Bytes::new(usize::try_from(end_frame_addr - start_frame_addr).unwrap() + 1)
        .as_num_of_pages::<Size4KiB>();

    let virt = virt::search_free_addr(num_pages)
        .expect("OOM during creating a new accessor to a register.");

    for i in 0..num_pages.as_usize() {
        let page = Page::<Size4KiB>::containing_address(virt + Size4KiB::SIZE * i as u64);
        let frame = PhysFrame::containing_address(start_frame_addr + Size4KiB::SIZE * i as u64);
        let flag =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        unsafe {
            PML4.lock()
                .map_to(page, frame, flag, &mut *FRAME_MANAGER.lock())
                .unwrap()
                .flush()
        }
    }

    let page_offset = start.as_u64() % Size4KiB::SIZE;

    virt + page_offset
}

pub fn unmap_pages(start: VirtAddr, object_size: Bytes) {
    let start_frame_addr = start.align_down(Size4KiB::SIZE);
    let end_frame_addr = (start + object_size.as_usize()).align_down(Size4KiB::SIZE);

    let num_pages = Bytes::new(usize::try_from(end_frame_addr - start_frame_addr).unwrap())
        .as_num_of_pages::<Size4KiB>();

    for i in 0..num_pages.as_usize() {
        let page =
            Page::<Size4KiB>::containing_address(start_frame_addr + Size4KiB::SIZE * i as u64);

        let (_, flush) = PML4.lock().unmap(page).unwrap();
        flush.flush();
    }
}

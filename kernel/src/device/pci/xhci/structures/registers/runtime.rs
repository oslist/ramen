// SPDX-License-Identifier: GPL-3.0-or-later

use crate::mem::accessor::Accessor;
use os_units::Bytes;
use x86_64::PhysAddr;
use xhci::registers::runtime::{
    EventRingDequeuePointerRegister, EventRingSegmentTableBaseAddressRegister,
    EventRingSegmentTableSizeRegister,
};

pub struct Runtime {
    pub erst_sz: Accessor<EventRingSegmentTableSizeRegister>,
    pub erst_ba: Accessor<EventRingSegmentTableBaseAddressRegister>,
    pub erd_p: Accessor<EventRingDequeuePointerRegister>,
}
impl<'a> Runtime {
    /// SAFETY: This method is unsafe because if `mmio_base` is not a valid address, or
    /// `runtime_register_space_offset` is not a valid value, it can violate memory safety.
    pub unsafe fn new(mmio_base: PhysAddr, runtime_register_space_offset: usize) -> Self {
        let runtime_base = mmio_base + runtime_register_space_offset;
        let erst_sz = Accessor::user(runtime_base, Bytes::new(0x28));
        let erst_ba = Accessor::user(runtime_base, Bytes::new(0x30));
        let erd_p = Accessor::user(runtime_base, Bytes::new(0x38));

        Self {
            erst_sz,
            erst_ba,
            erd_p,
        }
    }
}

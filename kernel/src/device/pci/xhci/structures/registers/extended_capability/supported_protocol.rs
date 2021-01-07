// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;

use crate::mem::accessor::Accessor;
use bit_field::BitField;
use os_units::Bytes;
use x86_64::PhysAddr;

pub struct SupportedProtocol {
    header: Accessor<[u32; 4]>,
    psis: Accessor<[u32]>,
}
impl SupportedProtocol {
    /// # SAFETY
    ///
    /// `head` must be a valid address to the head of the xHCI Supported Protocol Capability.
    pub(super) unsafe fn new(head: PhysAddr) -> Self {
        let header = Accessor::<[u32; 4]>::user(head, Bytes::zero());
        let psic = header.read()[2].get_bits(28..=31);
        let psis = Accessor::user_slice(head, Bytes::new(0x10), psic.try_into().unwrap());

        Self { header, psis }
    }
}

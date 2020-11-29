// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::{CycleBit, Link};
use crate::add_trb;
use bit_field::BitField;
use core::convert::TryInto;
use os_units::Bytes;
use x86_64::PhysAddr;

pub enum Trb {
    Link(Link),
    EnableSlot(EnableSlot),
    AddressDevice(AddressDevice),
    ConfigureEndpoint(ConfigureEndpoint),
}
impl Trb {
    pub const SIZE: Bytes = Bytes::new(16);

    pub fn new_enable_slot() -> Self {
        Self::EnableSlot(EnableSlot::new())
    }

    pub fn new_link(a: PhysAddr) -> Self {
        Self::Link(Link::new(a))
    }

    pub fn new_address_device(input_context_addr: PhysAddr, slot_id: u8) -> Self {
        Self::AddressDevice(AddressDevice::new(input_context_addr, slot_id))
    }

    pub fn new_configure_endpoint(context_addr: PhysAddr, slot_id: u8) -> Self {
        Self::ConfigureEndpoint(ConfigureEndpoint::new(context_addr, slot_id))
    }

    pub fn set_c(&mut self, c: CycleBit) {
        match self {
            Self::Link(l) => l.set_cycle_bit(c),
            Self::EnableSlot(e) => e.set_cycle_bit(c),
            Self::AddressDevice(a) => a.set_cycle_bit(c),
            Self::ConfigureEndpoint(e) => e.set_cycle_bit(c),
        }
    }
}
impl From<Trb> for [u32; 4] {
    fn from(t: Trb) -> Self {
        match t {
            Trb::Link(l) => l.0,
            Trb::EnableSlot(e) => e.0,
            Trb::AddressDevice(a) => a.0,
            Trb::ConfigureEndpoint(c) => c.0,
        }
    }
}

add_trb!(EnableSlot);
impl EnableSlot {
    const ID: u8 = 9;
    pub fn new() -> Self {
        let mut enable_slot = Self([0; 4]);
        enable_slot.set_trb_type(Self::ID);
        enable_slot
    }
}
impl Default for EnableSlot {
    fn default() -> Self {
        Self::new()
    }
}

add_trb!(AddressDevice);
impl AddressDevice {
    const ID: u8 = 11;
    pub fn new(addr_to_input_context: PhysAddr, slot_id: u8) -> Self {
        let mut trb = Self([0; 4]);

        assert!(addr_to_input_context.is_aligned(16_u64));
        trb.set_input_context_ptr_as_u64(addr_to_input_context.as_u64());
        trb.set_trb_type(Self::ID);
        trb.set_slot_id(slot_id);
        trb
    }

    fn set_input_context_ptr_as_u64(&mut self, p: u64) {
        let l = p & 0xffff_ffff;
        let u = p >> 32;
        self.0[0] = l.try_into().unwrap();
        self.0[1] = u.try_into().unwrap();
    }

    fn set_slot_id(&mut self, id: u8) {
        self.0[3].set_bits(24..=31, id.into());
    }
}

add_trb!(ConfigureEndpoint);
impl ConfigureEndpoint {
    const ID: u8 = 12;
    pub fn new(context_addr: PhysAddr, slot_id: u8) -> Self {
        let mut t = Self([0; 4]);
        t.set_context_addr(context_addr);
        t.set_slot_id(slot_id);
        t.set_trb_type(Self::ID);
        t
    }

    fn set_context_addr(&mut self, a: PhysAddr) {
        assert!(a.is_aligned(16_u64));

        let l = a.as_u64() & 0xffff_ffff;
        let u = a.as_u64() >> 32;

        self.0[0] = l.try_into().unwrap();
        self.0[1] = u.try_into().unwrap();
    }

    fn set_slot_id(&mut self, id: u8) {
        self.0[3].set_bits(24..=31, id.into());
    }
}

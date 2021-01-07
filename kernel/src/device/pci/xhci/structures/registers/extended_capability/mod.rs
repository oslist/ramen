// SPDX-License-Identifier: GPL-3.0-or-later

mod supported_protocol;
pub mod usb_legacy_support;

use super::capability::Capability;
use crate::mem::accessor::Accessor;
use core::convert::TryInto;
use os_units::Bytes;
use supported_protocol::SupportedProtocol;
use usb_legacy_support::UsbLegacySupport;
use x86_64::PhysAddr;

pub struct List {
    head: PhysAddr,
}
impl List {
    /// SAFETY: Caller must ensure that `addr` points to the head of a xHCI extended capability.
    pub unsafe fn new(mmio_base: PhysAddr, capability: &Capability) -> Option<Self> {
        let p1 = capability.hc_cp_params_1.read();
        let xecp = p1.xhci_extended_capabilities_pointer();

        if xecp > 0 {
            let head = mmio_base + u64::from(xecp << 2);

            Some(Self { head })
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = ExtendedCapability> {
        Iter {
            addr: Some(self.head),
        }
    }
}

struct Iter {
    addr: Option<PhysAddr>,
}
impl Iter {
    fn next_addr(&self) -> Option<PhysAddr> {
        let offset = (self.header()? >> 8) & 0xff;
        if offset == 0 {
            None
        } else {
            Some(self.addr? + u64::from(offset << 2))
        }
    }

    fn id(&self) -> Option<u8> {
        Some((self.header()? & 0xff).try_into().unwrap())
    }

    fn header(&self) -> Option<u32> {
        // SAFETY: This is safe because `self.addr` points to the head of an extended capability.
        let a: Accessor<u32> = unsafe { Accessor::user(self.addr?, Bytes::zero()) };
        Some(a.read())
    }
}
impl Iterator for Iter {
    type Item = ExtendedCapability;

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.addr?;
        let next_addr = self.next_addr();

        let item = if let Some(x) = self.id() {
            // SAFETY: `a` is the valid address to the head of a capability.
            unsafe {
                match x {
                    1 => {
                        let a = Accessor::user(a, Bytes::zero());
                        ExtendedCapability::UsbLegacySupport(a)
                    }
                    2 => ExtendedCapability::SupportedProtocol(SupportedProtocol::new(a)),
                    _ => ExtendedCapability::UnImplemented,
                }
            }
        } else {
            ExtendedCapability::UnImplemented
        };

        self.addr = next_addr;

        Some(item)
    }
}

pub enum ExtendedCapability {
    UsbLegacySupport(Accessor<UsbLegacySupport>),
    SupportedProtocol(SupportedProtocol),
    UnImplemented,
}

// SPDX-License-Identifier: GPL-3.0-or-later

mod ahc;
mod port;
mod registers;

use {
    crate::device::pci::{self, config::bar},
    ahc::Ahc,
    alloc::rc::Rc,
    core::cell::RefCell,
    registers::Registers,
    x86_64::PhysAddr,
};

pub async fn task() {
    let (mut ahc, mut ports) = match init() {
        Some(x) => x,
        None => return,
    };

    ahc.init();
    ports.init();
    ports.start();
}

fn init() -> Option<(Ahc, port::Collection)> {
    let registers = Rc::new(RefCell::new(fetch_registers()?));
    let ahc = Ahc::new(registers.clone());
    let port_collection = port::Collection::new(&registers);

    Some((ahc, port_collection))
}

fn fetch_registers() -> Option<Registers> {
    let abar = AchiBaseAddr::new()?;
    Some(Registers::new(abar))
}

#[derive(Copy, Clone)]
pub struct AchiBaseAddr(PhysAddr);
impl AchiBaseAddr {
    fn new() -> Option<Self> {
        for device in pci::iter_devices() {
            if device.is_ahci() {
                let addr = device.base_address(bar::Index::new(5));
                return Some(Self(addr));
            }
        }

        None
    }
}
impl From<AchiBaseAddr> for PhysAddr {
    fn from(abar: AchiBaseAddr) -> Self {
        abar.0
    }
}

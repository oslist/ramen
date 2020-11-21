// SPDX-License-Identifier: GPL-3.0-or-later

use super::{super::Registers, raw, CycleBit};
use alloc::rc::Rc;
use core::cell::RefCell;
use trb::Trb;
use x86_64::PhysAddr;

mod trb;

// 4KB / 16 = 256
const SIZE_OF_RING: usize = 256;

pub struct Ring {
    raw: raw::Ring,
    enqueue_ptr: usize,
    cycle_bit: CycleBit,
    registers: Rc<RefCell<Registers>>,
}
impl<'a> Ring {
    pub fn new(registers: Rc<RefCell<Registers>>) -> Self {
        Self {
            raw: raw::Ring::new(SIZE_OF_RING),
            enqueue_ptr: 0,
            cycle_bit: CycleBit::new(true),
            registers,
        }
    }

    pub fn init(&mut self) {
        self.register_address_to_xhci_register();
        self.set_initial_command_ring_cycle_state();
    }

    pub fn send_enable_slot(&mut self) -> Result<PhysAddr, Error> {
        let enable_slot = Trb::new_enable_slot(self.cycle_bit);
        let phys_addr_to_trb = self.try_enqueue(enable_slot)?;
        self.notify_command_is_sent();
        Ok(phys_addr_to_trb)
    }

    pub fn send_address_device(
        &mut self,
        addr_to_input_context: PhysAddr,
        slot_id: u8,
    ) -> Result<PhysAddr, Error> {
        let address_device =
            Trb::new_address_device(self.cycle_bit, addr_to_input_context, slot_id);
        let phys_addr_to_trb = self.try_enqueue(address_device)?;
        self.notify_command_is_sent();
        Ok(phys_addr_to_trb)
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.raw.phys_addr()
    }

    fn notify_command_is_sent(&mut self) {
        let doorbell_array = &mut self.registers.borrow_mut().doorbell_array;
        doorbell_array.update(0, |reg| *reg = 0)
    }

    fn register_address_to_xhci_register(&mut self) {
        let crcr = &mut self.registers.borrow_mut().hc_operational.crcr;
        crcr.update(|crcr| crcr.set_ptr(self.phys_addr()));
    }

    fn set_initial_command_ring_cycle_state(&mut self) {
        let crcr = &mut self.registers.borrow_mut().hc_operational.crcr;
        crcr.update(|crcr| crcr.set_ring_cycle_state(true));
    }

    fn try_enqueue(&mut self, trb: Trb) -> Result<PhysAddr, Error> {
        if self.full() {
            Err(Error::QueueIsFull)
        } else {
            Ok(self.enqueue(trb))
        }
    }

    fn full(&self) -> bool {
        let raw = self.raw[self.enqueue_ptr];
        raw.cycle_bit() == self.cycle_bit
    }

    fn enqueue(&mut self, trb: Trb) -> PhysAddr {
        self.write_trb_on_memory(trb);
        let addr_to_trb = self.addr_to_enqueue_ptr();
        self.increment_enqueue_ptr();
        addr_to_trb
    }

    fn write_trb_on_memory(&mut self, trb: Trb) {
        self.raw[self.enqueue_ptr] = trb.into();
    }

    fn addr_to_enqueue_ptr(&self) -> PhysAddr {
        self.phys_addr() + Trb::SIZE.as_usize() * self.enqueue_ptr
    }

    fn increment_enqueue_ptr(&mut self) {
        self.enqueue_ptr += 1;
        if self.enqueue_ptr < self.len() - 1 {
            return;
        }

        self.append_link_trb();
        self.move_enqueue_ptr_to_the_beginning();
    }

    fn len(&self) -> usize {
        self.raw.len()
    }

    fn append_link_trb(&mut self) {
        self.raw[self.enqueue_ptr] = Trb::new_link(self.phys_addr(), self.cycle_bit).into();
    }

    fn move_enqueue_ptr_to_the_beginning(&mut self) {
        self.enqueue_ptr = 0;
        self.cycle_bit.toggle();
    }
}

#[derive(Debug)]
pub enum Error {
    QueueIsFull,
}
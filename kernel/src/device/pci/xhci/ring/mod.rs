// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::mem::allocator::page_box::PageBox,
    core::ops::{Index, IndexMut},
    trb::Trb,
    x86_64::PhysAddr,
};

pub mod command;
pub mod event;
mod trb;

struct Raw {
    arr: PageBox<[trb::Raw]>,
    enqueue_ptr: usize,
    dequeue_ptr: usize,
    cycle_bit: CycleBit,
}
impl Raw {
    fn new(num_trb: usize) -> Self {
        Self {
            arr: PageBox::new_slice(num_trb),
            enqueue_ptr: 0,
            dequeue_ptr: 0,
            cycle_bit: CycleBit::new(true),
        }
    }

    fn len(&self) -> usize {
        self.arr.len()
    }

    fn phys_addr(&self) -> PhysAddr {
        self.arr.phys_addr()
    }

    fn enqueue(&mut self, trb: Trb) {
        if !self.enqueueable() {
            return;
        }
        self.arr[self.enqueue_ptr] = trb.into();

        self.enqueue_ptr += 1;
        if self.enqueue_ptr < self.len() {
            return;
        }

        self.enqueue_ptr %= self.len();
        self.cycle_bit.toggle();
    }

    fn enqueueable(&self) -> bool {
        let raw = self.arr[self.enqueue_ptr];
        raw.cycle_bit() != self.cycle_bit
    }
}
impl Index<usize> for Raw {
    type Output = trb::Raw;
    fn index(&self, index: usize) -> &Self::Output {
        &self.arr[index]
    }
}
impl IndexMut<usize> for Raw {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.arr[index]
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub struct CycleBit(bool);
impl CycleBit {
    fn new(val: bool) -> Self {
        Self(val)
    }

    fn toggle(&mut self) {
        self.0 = !self.0;
    }
}
impl From<CycleBit> for bool {
    fn from(cycle_bit: CycleBit) -> Self {
        cycle_bit.0
    }
}

// SPDX-License-Identifier: GPL-3.0-or-later

use super::ring::{transfer, trb::TransferEvent};
use alloc::{collections::BTreeMap, rc::Rc};
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::task::AtomicWaker;
use x86_64::PhysAddr;

pub struct Transferor {
    ring: Rc<RefCell<transfer::Ring>>,
    receiver: Rc<RefCell<Receiver>>,
    waker: Rc<RefCell<AtomicWaker>>,
}
impl Transferor {
    pub fn new(ring: Rc<RefCell<transfer::Ring>>, receiver: Rc<RefCell<Receiver>>) -> Self {
        Self {
            ring,
            receiver,
            waker: Rc::new(RefCell::new(AtomicWaker::new())),
        }
    }

    fn try_register_with_receiver(&mut self, addr_to_trb: PhysAddr) -> Result<(), Error> {
        self.receiver
            .borrow_mut()
            .try_insert(addr_to_trb, self.waker.clone())
    }

    async fn get_trb(&mut self, addr_to_trb: PhysAddr) -> TransferEvent {
        ReceiveFuture::new(addr_to_trb, self.receiver.clone(), self.waker.clone()).await
    }
}

pub struct Receiver {
    trbs: BTreeMap<PhysAddr, Option<TransferEvent>>,
    wakers: BTreeMap<PhysAddr, Rc<RefCell<AtomicWaker>>>,
}
impl Receiver {
    pub fn new() -> Self {
        Self {
            trbs: BTreeMap::new(),
            wakers: BTreeMap::new(),
        }
    }

    pub fn try_insert(
        &mut self,
        addr_to_trb: PhysAddr,
        waker: Rc<RefCell<AtomicWaker>>,
    ) -> Result<(), Error> {
        if self.trbs.insert(addr_to_trb, None).is_some() {
            return Err(Error::AlreadyRegistered);
        }

        if self.wakers.insert(addr_to_trb, waker).is_some() {
            return Err(Error::AlreadyRegistered);
        }

        Ok(())
    }

    pub fn try_receive(&mut self, trb: TransferEvent) -> Result<(), Error> {
        let addr_to_trb = PhysAddr::new(trb.trb_ptr());
        self.store_trb(trb)?;
        self.wake_transferor(addr_to_trb);
        Ok(())
    }

    fn store_trb(&mut self, trb: TransferEvent) -> Result<(), Error> {
        let addr_to_transfer_trb = PhysAddr::new(trb.trb_ptr());
        *self
            .trbs
            .get_mut(&addr_to_transfer_trb)
            .ok_or(Error::NotRegistered)? = Some(trb);
        Ok(())
    }

    fn wake_transferor(&mut self, addr_to_trb: PhysAddr) -> Result<(), Error> {
        self.wakers
            .remove(&addr_to_trb)
            .ok_or(Error::NotRegistered)?
            .borrow_mut()
            .wake();
        Ok(())
    }

    fn trb_arrives(&self, addr: PhysAddr) -> Result<bool, Error> {
        match self.trbs.get(&addr) {
            Some(trb) => Ok(trb.is_some()),
            None => Err(Error::NotRegistered),
        }
    }

    fn remove_entry(&mut self, addr: PhysAddr) -> Result<Option<TransferEvent>, Error> {
        match self.trbs.remove(&addr) {
            Some(trb) => Ok(trb),
            None => Err(Error::NotRegistered),
        }
    }
}

struct ReceiveFuture {
    addr_to_trb: PhysAddr,
    receiver: Rc<RefCell<Receiver>>,
    waker: Rc<RefCell<AtomicWaker>>,
}
impl ReceiveFuture {
    fn new(
        addr_to_trb: PhysAddr,
        receiver: Rc<RefCell<Receiver>>,
        waker: Rc<RefCell<AtomicWaker>>,
    ) -> Self {
        Self {
            addr_to_trb,
            receiver,
            waker,
        }
    }
}
impl Future for ReceiveFuture {
    type Output = TransferEvent;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = self.waker.clone();
        let addr = self.addr_to_trb;
        let receiver = &mut Pin::into_inner(self).receiver;

        waker.borrow_mut().register(cx.waker());
        let trb_arrives = receiver.borrow().trb_arrives(addr).unwrap();
        if trb_arrives {
            waker.borrow_mut().take();
            let trb = receiver.borrow_mut().remove_entry(addr).unwrap().unwrap();
            Poll::Ready(trb)
        } else {
            Poll::Pending
        }
    }
}

#[derive(Debug)]
pub enum Error {
    AlreadyRegistered,
    NotRegistered,
}

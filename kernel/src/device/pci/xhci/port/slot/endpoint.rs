// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::{
    exchanger::{command, receiver::Receiver, transfer},
    structures::{
        context::{self, Context},
        descriptor,
        registers::Registers,
        ring::{transfer::Ring as TransferRing, CycleBit},
    },
};
use alloc::{rc::Rc, vec::Vec};
use bit_field::BitField;
use core::cell::RefCell;
use futures_intrusive::sync::LocalMutex;
use num_traits::FromPrimitive;
use transfer::DoorbellWriter;

use super::Slot;

pub struct Collection {
    eps: Vec<Endpoint>,
    cx: Rc<RefCell<Context>>,
    cmd: Rc<LocalMutex<command::Sender>>,
    slot_id: u8,
}
impl Collection {
    pub async fn new(mut slot: Slot, cmd: Rc<LocalMutex<command::Sender>>) -> Self {
        let eps = slot.endpoints().await;
        Self {
            eps,
            cx: slot.context,
            cmd,
            slot_id: slot.id,
        }
    }

    pub async fn init(&mut self) {
        self.enable_eps();
        self.issue_configure_eps().await;
        info!("Endpoints initialized");
    }

    fn enable_eps(&mut self) {
        for ep in &mut self.eps {
            ep.init_context();
        }
    }

    async fn issue_configure_eps(&mut self) {
        let mut cmd = self.cmd.lock().await;
        let cx_addr = self.cx.borrow().input.phys_addr();
        cmd.configure_endpoint(cx_addr, self.slot_id).await;
    }
}

pub struct Endpoint {
    desc: descriptor::Endpoint,
    cx: Rc<RefCell<Context>>,
}
impl Endpoint {
    pub fn new(desc: descriptor::Endpoint, cx: Rc<RefCell<Context>>) -> Self {
        Self { desc, cx }
    }

    pub fn init_context(&mut self) {
        ContextInitializer::new(&self.desc, &mut self.cx.borrow_mut()).init();
    }
}

pub struct Default {
    sender: transfer::Sender,
    cx: Rc<RefCell<Context>>,
}
impl Default {
    fn new(rcv: Rc<RefCell<Receiver>>, reg: Rc<RefCell<Registers>>, slot: &Slot) -> Self {
        Self {
            sender: transfer::Sender::new(
                TransferRing::new(),
                rcv,
                DoorbellWriter::new(reg, slot.id),
            ),
            cx: slot.context.clone(),
        }
    }
}

struct ContextInitializer<'a> {
    ep: &'a descriptor::Endpoint,
    context: &'a mut Context,
}
impl<'a> ContextInitializer<'a> {
    fn new(ep: &'a descriptor::Endpoint, context: &'a mut Context) -> Self {
        Self { ep, context }
    }

    fn init(&mut self) {
        self.set_aflag();
        self.init_ep_context();
    }

    fn set_aflag(&mut self) {
        let dci: usize = self.calculate_dci().into();

        self.context.input.control_mut().clear_aflag(1); // See xHCI dev manual 4.6.6.
        self.context.input.control_mut().set_aflag(dci);
    }

    fn calculate_dci(&self) -> u8 {
        let a = self.ep.endpoint_address;
        2 * a.get_bits(0..=3) + a.get_bit(7) as u8
    }

    fn init_ep_context(&mut self) {
        let ep_ty = self.ep_ty();
        let max_packet_size = self.ep.max_packet_size;
        let interval = self.ep.interval;

        let c = self.ep_context();
        c.set_endpoint_type(ep_ty);
        c.set_max_packet_size(max_packet_size);
        c.set_max_burst_size(0);
        c.set_dequeue_cycle_state(CycleBit::new(true));
        c.set_max_primary_streams(0);
        c.set_mult(0);
        c.set_error_count(3);
        c.set_interval(interval);
    }

    fn ep_context(&mut self) -> &mut context::Endpoint {
        let ep_idx: usize = self.ep.endpoint_address.get_bits(0..=3).into();
        let out_input = self.ep.endpoint_address.get_bit(7);
        let context_inout = &mut self.context.output_device.ep_inout[ep_idx];
        if out_input {
            &mut context_inout.input
        } else {
            &mut context_inout.out
        }
    }

    fn ep_ty(&self) -> context::EndpointType {
        context::EndpointType::from_u8(if self.ep.attributes == 0 {
            0
        } else {
            self.ep.attributes + if self.ep.endpoint_address == 0 { 0 } else { 4 }
        })
        .unwrap()
    }
}

// SPDX-License-Identifier: GPL-3.0-or-later

use crate::device::pci::xhci::structures::{
    context::{self, Context},
    descriptor,
    ring::CycleBit,
};
use alloc::rc::Rc;
use bit_field::BitField;
use core::cell::RefCell;
use num_traits::FromPrimitive;

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

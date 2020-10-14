// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{CapabilitySpec, MessageAddress, MessageData, RegisterIndex, Registers, TypeSpec},
    bitfield::bitfield,
    core::convert::TryFrom,
};

#[derive(Debug)]
pub struct Msi<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}

impl<'a> Msi<'a> {
    pub fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }

    fn edit_message_address<T>(&self, f: T)
    where
        T: Fn(&mut MessageAddress),
    {
        let mut message_address = self.get_message_address();
        f(&mut message_address);
        self.set_message_address(message_address);
    }

    fn get_message_address(&self) -> MessageAddress {
        MessageAddress::from(self.registers.get(self.message_address_index()))
    }

    fn set_message_address(&self, message_address: MessageAddress) {
        self.registers
            .set(self.message_address_index(), message_address.into())
    }

    fn message_address_index(&self) -> RegisterIndex {
        self.base + 1
    }

    fn edit_message_data<T>(&self, f: T)
    where
        T: Fn(&mut MessageData),
    {
        let mut message_data = self.get_message_data();
        f(&mut message_data);
        self.set_message_data(message_data);
    }

    fn get_message_data(&self) -> MessageData {
        MessageData::from(self.registers.get(self.message_data_index()))
    }

    fn set_message_data(&self, message_data: MessageData) {
        self.registers
            .set(self.message_data_index(), message_data.into())
    }

    fn message_data_index(&self) -> RegisterIndex {
        self.base + 3
    }

    fn edit_message_control<T>(&self, f: T)
    where
        T: Fn(&mut MessageControl),
    {
        let mut message_control = self.get_message_control();
        f(&mut message_control);
        self.set_message_control(message_control);
    }

    fn get_message_control(&self) -> MessageControl {
        MessageControl::from(u16::try_from(self.registers.get(self.base) >> 16).unwrap())
    }

    fn set_message_control(&self, message_control: MessageControl) {
        let mut register = self.registers.get(self.base);
        register &= 0xffff;
        register |= u32::from(u16::from(message_control)) << 16;
        self.registers.set(self.base, register)
    }
}

impl<'a> CapabilitySpec for Msi<'a> {
    fn init_for_xhci(&self, _config_spec: &TypeSpec) {
        info!("Initializing MSI...");

        self.edit_message_address(MessageAddress::init_for_xhci);
        info!("Edited Message Address.");

        self.edit_message_data(MessageData::init_for_xhci);
        info!("Edited Message Data.");

        self.edit_message_control(MessageControl::init_for_xhci);
        info!("Edited Message Control.");
    }
}

bitfield! {
    struct MessageControl(u16);

    interrupt_status, set_interrupt_status: 0;
    num_of_enabled_interrupt_vectors, set_num_of_enabled_interrupt_vectors: 6, 4;
}
impl MessageControl {
    fn init_for_xhci(&mut self) {
        self.set_interrupt_status(true);
        self.set_num_of_enabled_interrupt_vectors(0);
    }
}
impl From<u16> for MessageControl {
    fn from(control: u16) -> Self {
        Self(control)
    }
}
impl From<MessageControl> for u16 {
    fn from(control: MessageControl) -> Self {
        control.0
    }
}

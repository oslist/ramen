// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{CapabilitySpec, MessageAddress, MessageData, RegisterIndex, Registers},
    crate::{
        accessor::slice,
        device::pci::config::{bar, type_spec::TypeSpec},
    },
    bitfield::bitfield,
    core::convert::{From, TryFrom},
    os_units::{Bytes, Size},
    x86_64::PhysAddr,
};

#[derive(Debug)]
pub struct MsiX<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}

impl<'a> CapabilitySpec for MsiX<'a> {
    fn init_for_xhci(&self, config_type_spec: &TypeSpec) {
        let base_address = config_type_spec.base_address(self.bir());
        let mut table = self.table(base_address);

        let pending_base = config_type_spec.base_address(self.pending_bir());
        self.pending_bit_table(pending_base)[0] = 1;
        table[0].init_for_xhci();

        self.enable_interrupt();
    }
}

impl<'a> MsiX<'a> {
    pub fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }

    fn bir(&self) -> bar::Index {
        bar::Index::from(Bir::new(self.registers, self.base))
    }

    fn table(&self, base_address: PhysAddr) -> slice::Accessor<Element> {
        slice::Accessor::new(
            base_address,
            self.table_offset(),
            usize::from(self.num_of_table_elements()),
        )
    }

    fn pending_bit_table(&self, base_address: PhysAddr) -> slice::Accessor<u64> {
        slice::Accessor::new(
            base_address,
            self.pending_table_offset(),
            (usize::from(self.num_of_table_elements()) + 64 - 1) / 64,
        )
    }

    fn enable_interrupt(&self) {
        let val = self.registers.get(self.base) | 0xf000_0000;
        self.registers.set(self.base, val);
    }

    fn table_offset(&self) -> Size<Bytes> {
        Size::from(TableOffset::new(self.registers, self.base))
    }

    fn num_of_table_elements(&self) -> TableSize {
        TableSize::new(self.registers, self.base)
    }

    fn pending_bir(&self) -> bar::Index {
        bar::Index::from(PendingBitBir::new(self.registers, self.base))
    }

    fn pending_table_offset(&self) -> Size<Bytes> {
        Size::from(PendingBitTableOffset::new(self.registers, self.base))
    }
}

pub struct Bir(bar::Index);
impl Bir {
    fn new(registers: &Registers, base: RegisterIndex) -> Self {
        Self(bar::Index::new(registers.get(base + 1) & 0b111))
    }
}
impl From<Bir> for bar::Index {
    fn from(bir: Bir) -> Self {
        bir.0
    }
}

struct TableOffset(Size<Bytes>);
impl TableOffset {
    fn new(registers: &Registers, base: RegisterIndex) -> Self {
        Self(Size::new((registers.get(base + 1) & !0b111) as usize))
    }
}
impl From<TableOffset> for Size<Bytes> {
    fn from(offset: TableOffset) -> Self {
        offset.0
    }
}

#[derive(Debug)]
struct TableSize<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}
impl<'a> TableSize<'a> {
    fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }
}
impl<'a> From<TableSize<'a>> for usize {
    fn from(size: TableSize) -> Self {
        // Table size is N - 1 encoded.
        // See: https://wiki.osdev.org/PCI#Enabling_MSI-X
        usize::try_from(((size.registers.get(size.base) >> 16) & 0x7ff) + 1).unwrap()
    }
}

bitfield! {
    #[derive(Debug)]
    #[repr(transparent)]
    struct Element(u128);

    u32, from into MessageAddress, message_address,set_message_address: 31, 0;
    u32, from into MessageData, message_data, set_message_data: 95, 64;
    masked, set_mask: 96;
}
impl Element {
    fn init_for_xhci(&mut self) {
        self.message_address().init_for_xhci();
        self.message_data().init_for_xhci();
        self.set_mask(false);
    }
}

#[derive(Debug)]
struct PendingBitBir<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}
impl<'a> PendingBitBir<'a> {
    fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }
}
impl<'a> From<PendingBitBir<'a>> for bar::Index {
    fn from(bir: PendingBitBir) -> Self {
        bar::Index::new(bir.registers.get(bir.base + 2) & 0b111)
    }
}

#[derive(Debug)]
struct PendingBitTableOffset<'a> {
    registers: &'a Registers,
    base: RegisterIndex,
}
impl<'a> PendingBitTableOffset<'a> {
    fn new(registers: &'a Registers, base: RegisterIndex) -> Self {
        Self { registers, base }
    }
}
impl<'a> From<PendingBitTableOffset<'a>> for Size<Bytes> {
    fn from(pending: PendingBitTableOffset) -> Self {
        Size::new(usize::try_from(pending.registers.get(pending.base + 2) & !0b111).unwrap())
    }
}

// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{fis::RegFixH2D, Registers},
    crate::mem::allocator::page_box::PageBox,
    alloc::vec::Vec,
    bitfield::bitfield,
    core::convert::TryInto,
    x86_64::PhysAddr,
};

const NUM_OF_PRDT: usize = 8;

pub struct CommandList {
    pub headers: PageBox<[Header]>,
    pub tables: Vec<PageBox<Table>>,
}
impl CommandList {
    pub fn new(registers: &Registers) -> Self {
        let headers = PageBox::new_slice(
            Header::null(),
            Self::num_of_command_slots_supported(registers),
        );
        let tables = Self::new_tables(registers);
        let mut list = Self { headers, tables };
        list.set_ptrs_of_headers();
        list
    }

    pub fn phys_addr_to_headers(&self) -> PhysAddr {
        self.headers.phys_addr()
    }

    fn new_tables(registers: &Registers) -> Vec<PageBox<Table>> {
        let mut tables = Vec::new();
        for _ in 0..Self::num_of_command_slots_supported(registers) {
            tables.push(PageBox::new(Table::null()));
        }
        tables
    }

    fn set_ptrs_of_headers(&mut self) {
        for (i, header) in self.headers.iter_mut().enumerate() {
            header.set_command_table_base_addr(self.tables[i].phys_addr());
        }
    }

    fn num_of_command_slots_supported(registers: &Registers) -> usize {
        registers
            .generic
            .cap
            .read()
            .num_of_command_slots()
            .try_into()
            .unwrap()
    }
}

pub type Header = CommandHeaderStructure<[u32; 8]>;
bitfield! {
    #[repr(transparent)]
    #[derive(Copy,Clone)]
    pub struct CommandHeaderStructure([u32]);
    impl Debug;
    u64, _, set_ctba: 64+31, 64;
    pub u8, _, set_cfl: 4, 0;
    pub u16, prdtl, set_prdtl: 31, 16;
}
impl CommandHeaderStructure<[u32; 8]> {
    fn null() -> Self {
        Self([0; 8])
    }

    fn set_command_table_base_addr(&mut self, addr: PhysAddr) {
        assert!(addr.is_aligned(128_u64));
        self.set_ctba(addr.as_u64());
    }
}

pub struct Table {
    pub fis: RegFixH2D,
    rsvd: [u8; 0x80 - 20],
    pub prdt: [PhysicalRegionDescriptorTable; NUM_OF_PRDT],
}
impl Table {
    fn null() -> Self {
        Self {
            fis: RegFixH2D::null(),
            rsvd: [0; 0x80 - 20],
            prdt: [PhysicalRegionDescriptorTable::null(); NUM_OF_PRDT],
        }
    }
}

pub type PhysicalRegionDescriptorTable = PhysicalRegionDescriptorTableStructure<[u32; 8]>;
bitfield! {
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct PhysicalRegionDescriptorTableStructure([u32]);
    impl Debug;
    u64, _, set_dba_as_u64: 63, 0;
    u32, _, set_dbc_as_u32: 96+21, 96;
}
impl PhysicalRegionDescriptorTable {
    pub fn set_dba(&mut self, addr: PhysAddr) {
        assert!(addr.is_aligned(2_u64));
        self.set_dba_as_u64(addr.as_u64());
    }

    pub fn set_dbc(&mut self, dbc: u32) {
        assert_eq!(dbc & 1, 1);
        self.set_dbc_as_u32(dbc);
    }

    fn null() -> Self {
        Self([0; 8])
    }
}

// SPDX-License-Identifier: GPL-3.0-or-later

use {bitfield::bitfield, core::convert::TryInto};

pub type RegFixH2D = RegFixH2DStructure<[u32; 5]>;
bitfield! {
    #[repr(transparent)]
    pub struct RegFixH2DStructure([u32]);
    impl Debug;
    u8, _, set_fis_type: 7, 0;
    pub _, set_c: 15;
    pub u8, _, set_command: 23, 16;
    u32, _, set_lba_as_u32: 32+23, 32;
    pub u8, _, set_device: 32+31, 32+24;
    pub u32, _, set_upper_lba_as_u32: 64+23, 64;
    pub u16, _, set_count: 96+15, 96;
}
impl RegFixH2D {
    const ID: u8 = 0x27;
    pub fn null() -> Self {
        let mut fis = Self([0; 5]);
        fis.set_fis_type(Self::ID);
        fis
    }

    pub fn set_lba(&mut self, lba: u64) {
        self.set_lba_as_u32((lba & 0xfff).try_into().unwrap());
        self.set_upper_lba_as_u32((lba >> 24).try_into().unwrap());
    }
}

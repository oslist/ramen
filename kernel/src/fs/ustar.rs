// SPDX-License-Identifier: GPL-3.0-or-later

use alloc::str;

#[repr(C, packed)]
#[derive(Debug)]
pub struct Meta {
    name: [u8; 100],
    mode: [u8; 8],
    owner: [u8; 8],
    group: [u8; 8],
    size: [u8; 12],
    modified_time: [u8; 12],
    checksum: [u8; 8],
    ty: [u8; 1],
    linked_file_name: [u8; 100],
    magic: [u8; 6],
    version: [u8; 2],
    owner_name: [u8; 32],
    group_name: [u8; 32],
    device_major_number: [u8; 8],
    device_minor_number: [u8; 8],
    filename_prefix: [u8; 155],
    _rsvd: [u8; 12],
}
impl Meta {
    pub fn filesize_as_dec(&self) -> usize {
        let mut sz: usize = 0;

        // The last byte of `size` field is always 0 (u8), not 0 (char).
        for d in 0..self.size.len() - 1 {
            sz *= 8;
            sz += usize::from(self.size[d] - b'0');
        }
        sz
    }

    pub fn name(&self) -> &str {
        let name = &self.name;
        str::from_utf8(name).unwrap().trim_matches(char::from(0))
    }
}

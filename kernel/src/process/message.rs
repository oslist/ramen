// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug)]
pub(super) struct Message {
    header: Header,
    body: Body,
}
impl Message {
    fn new(header: Header, body: Body) -> Self {
        Self { header, body }
    }
}

#[derive(Debug)]
struct Header {
    sender: super::Id,
}
impl Header {
    fn new(sender: super::Id) -> Self {
        Self { sender }
    }
}

#[derive(Debug)]
struct Body {
    m1: u64,
    m2: u64,
    m3: u64,
}
impl Body {
    fn new(m1: u64, m2: u64, m3: u64) -> Self {
        Self { m1, m2, m3 }
    }
}

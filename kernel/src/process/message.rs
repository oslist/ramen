// SPDX-License-Identifier: GPL-3.0-or-later

struct Message {
    header: Header,
    body: Body,
}
impl Message {
    fn new(header: Header, body: Body) -> Self {
        Self { header, body }
    }
}

struct Header {
    sender: super::Id,
}
impl Header {
    fn new(sender: super::Id) -> Self {
        Self { sender }
    }
}

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

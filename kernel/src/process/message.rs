// SPDX-License-Identifier: GPL-3.0-or-later

struct Message {
    header: Header,
    body: Body,
}

struct Header {
    sender: super::Id,
}

struct Body {
    m1: u64,
    m2: u64,
    m3: u64,
}

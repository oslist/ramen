// SPDX-License-Identifier: GPL-3.0-or-later

use super::collections;

fn send(to: super::Id, m: Message) {
    collections::process::handle_mut(to, |p| p.inbox.push_back(m))
}

fn receive() -> Option<Message> {
    collections::process::handle_running_mut(|p| p.outbox.pop_front())
}

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

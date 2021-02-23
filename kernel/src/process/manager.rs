// SPDX-License-Identifier: GPL-3.0-or-later

use super::{collections, collections::woken_pid, switch, Privilege, Process};
use crate::{device::pci::xhci, multitask, tests, tss::TSS};
use common::constant::INTERRUPT_STACK;
use conquer_once::spin::Lazy;
use crossbeam_queue::ArrayQueue;
use multitask::{executor::Executor, task::Task};

pub use super::exit::exit;
pub use switch::switch;

const MAX_MESSAGE: usize = 128;
static MESSAGE: Lazy<ArrayQueue<Message>> = Lazy::new(|| ArrayQueue::new(MAX_MESSAGE));

pub fn main() {
    add_processes();

    loop {
        while let Some(m) = MESSAGE.pop() {
            match m {
                Message::Add(f, p) => match p {
                    Privilege::Kernel => push_process_to_queue(Process::kernel(f)),
                    Privilege::User => push_process_to_queue(Process::user(f)),
                },
                Message::Exit(id) => collections::process::remove(id),
            }
        }
    }
}

pub fn init() {
    set_temporary_stack_frame();
    push_process_to_queue(Process::user(main));
}

pub fn add(f: fn(), p: Privilege) {
    send_message(Message::Add(f, p));
}

pub fn getpid() -> i32 {
    collections::process::handle_running(|p| p.id.as_i32())
}

pub fn notify(pid: i32) {
    let _ = collections::process::handle(super::Id::from(pid), |p| {
        p.inbox.push(super::message::Message)
    });
}

pub fn notify_exists() -> bool {
    collections::process::handle_running(|p| p.inbox.pop()).is_some()
}

pub(super) fn send_message(m: Message) {
    MESSAGE.push(m).expect("`MESSAGE` is full.");
}

pub(super) fn set_temporary_stack_frame() {
    TSS.lock().interrupt_stack_table[0] = INTERRUPT_STACK;
}

fn add_processes() {
    add(run_tasks, Privilege::User);
    add(ps2_keyboard::main, Privilege::User);
    add(ps2_mouse::main, Privilege::User);
    add(tsukemen::main, Privilege::User);

    if cfg!(feature = "qemu_test") {
        add(tests::main, Privilege::User);
        add(tests::process::kernel_privilege_test, Privilege::Kernel);
        add(tests::process::exit_test, Privilege::User);

        for _ in 0..100 {
            add(tests::process::do_nothing, Privilege::User);
        }
    }
}

fn run_tasks() {
    multitask::add(Task::new(xhci::task()));

    let mut executor = Executor::new();
    executor.run();
}

fn push_process_to_queue(p: Process) {
    add_pid(p.id());
    add_process(p);
}

fn add_pid(id: super::Id) {
    woken_pid::add(id);
}

fn add_process(p: Process) {
    collections::process::add(p);
}

pub(super) fn loader(f: fn()) -> ! {
    f();
    syscalls::exit();
}

#[derive(Debug)]
pub(super) enum Message {
    Add(fn(), Privilege),
    Exit(super::Id),
}

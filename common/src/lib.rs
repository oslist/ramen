// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
#![feature(const_fn, unsafe_block_in_unsafe_fn)]
#![deny(clippy::pedantic)]
#![deny(clippy::all)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod constant;
pub mod debug;
pub mod kernelboot;
pub mod mem;
pub mod vram;

extern crate x86_64;

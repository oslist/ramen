// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    interrupt,
    mem::{allocator, paging::pml4::PML4},
    process,
};
use core::{convert::TryInto, ffi::c_void, slice};
use num_traits::FromPrimitive;
use os_units::{Bytes, NumOfPages};
use x86_64::{
    instructions::{
        self, interrupts,
        port::{PortReadOnly, PortWriteOnly},
    },
    registers::model_specific::{Efer, EferFlags, LStar},
    structures::paging::{Size4KiB, Translate},
    PhysAddr, VirtAddr,
};

pub fn init() {
    enable();
    register();
}

fn enable() {
    // SAFETY: This operation is safe as this does not touch any unsafe things.
    unsafe { Efer::update(|e| *e |= EferFlags::SYSTEM_CALL_EXTENSIONS) }
}

fn register() {
    let addr = save_rip_and_rflags as usize;

    LStar::write(VirtAddr::new(addr.try_into().unwrap()));
}

/// `syscall` instruction calls this function.
///
/// RAX: system call index
/// RDI: 1st argument
/// RSI: 2nd argument
/// RDX: 3rd argument
#[naked]
extern "C" fn save_rip_and_rflags() -> u64 {
    unsafe {
        asm!(
            "
        cli
        push rcx    # Save rip
        push r11    # Save rflags

        call prepare_arguments

        pop r11     # Restore rflags
        pop rcx     # Restore rip
        sti
        sysretq
        ",
            options(noreturn)
        );
    }
}

/// SAFETY: This function is unsafe because invalid values in registers may break memory safety.
#[no_mangle]
unsafe fn prepare_arguments() {
    let syscall_index: u64;
    let a1: u64;
    let a2: u64;
    let a3: u64;

    asm!("", out("rax") syscall_index, out("rdi") a1, out("rsi") a2,out("rdx") a3);
    asm!("", in("rax") select_proper_syscall(syscall_index, a1, a2,a3))
}

/// SAFETY: This function is unsafe because invalid arguments may break memory safety.
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
unsafe fn select_proper_syscall(idx: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    match FromPrimitive::from_u64(idx) {
        Some(s) => match s {
            syscalls::Ty::Inb => sys_inb(a1.try_into().unwrap()).into(),
            syscalls::Ty::Outb => sys_outb(a1.try_into().unwrap(), a2.try_into().unwrap()),
            syscalls::Ty::Inl => sys_inl(a1.try_into().unwrap()).into(),
            syscalls::Ty::Outl => sys_outl(a1.try_into().unwrap(), a2.try_into().unwrap()),
            syscalls::Ty::Halt => sys_halt(),
            syscalls::Ty::DisableInterrupt => sys_disable_interrupt(),
            syscalls::Ty::EnableInterrupt => sys_enable_interrupt(),
            syscalls::Ty::EnableInterruptAndHalt => sys_enable_interrupt_and_halt(),
            syscalls::Ty::AllocatePages => {
                sys_allocate_pages(NumOfPages::new(a1.try_into().unwrap())).as_u64()
            }
            syscalls::Ty::DeallocatePages => {
                sys_deallocate_pages(VirtAddr::new(a1), NumOfPages::new(a2.try_into().unwrap()))
            }
            syscalls::Ty::MapPages => {
                sys_map_pages(PhysAddr::new(a1), Bytes::new(a2.try_into().unwrap())).as_u64()
            }
            syscalls::Ty::UnmapPages => {
                sys_unmap_pages(VirtAddr::new(a1), Bytes::new(a2.try_into().unwrap()))
            }
            syscalls::Ty::GetPid => sys_getpid().try_into().unwrap(),
            syscalls::Ty::Exit => sys_exit(),
            syscalls::Ty::TranslateAddress => sys_translate_address(VirtAddr::new(a1)).as_u64(),
            syscalls::Ty::Write => sys_write(
                a1.try_into().unwrap(),
                a2 as *const _,
                a3.try_into().unwrap(),
            )
            .try_into()
            .unwrap(),
            syscalls::Ty::NotifyExists => sys_notify_exists() as _,
            syscalls::Ty::NotifyOnInterrupt => {
                sys_notify_on_interrupt(a1.try_into().unwrap(), a2.try_into().unwrap());
                0
            }
        },
        None => panic!("Unsupported syscall index: {}", idx),
    }
}

/// SAFETY: This function is unsafe because reading from I/O port may have side effects which
/// violate memory safety.
unsafe fn sys_inb(port: u16) -> u8 {
    let mut p = PortReadOnly::new(port);
    p.read()
}

/// SAFETY: This function is unsafe because writing to I/O port may have side effects which violate
/// memory safety.
unsafe fn sys_outb(port: u16, v: u8) -> u64 {
    let mut p = PortWriteOnly::new(port);
    p.write(v);
    0
}

/// SAFETY: This function is unsafe because reading from I/O port may have side effects which
/// violate memory safety.
unsafe fn sys_inl(port: u16) -> u32 {
    let mut p = PortReadOnly::new(port);
    p.read()
}

/// SAFETY: This function is unsafe because writing to I/O port may have side effects which violate
/// memory safety.
unsafe fn sys_outl(port: u16, v: u32) -> u64 {
    let mut p = PortWriteOnly::new(port);
    p.write(v);
    0
}

fn sys_halt() -> u64 {
    instructions::hlt();
    0
}

fn sys_disable_interrupt() -> u64 {
    interrupts::disable();
    0
}

fn sys_enable_interrupt() -> u64 {
    interrupts::enable();
    0
}

fn sys_enable_interrupt_and_halt() -> u64 {
    interrupts::enable_and_hlt();
    0
}

fn sys_allocate_pages(num_of_pages: NumOfPages<Size4KiB>) -> VirtAddr {
    allocator::allocate_pages(num_of_pages).unwrap_or_else(VirtAddr::zero)
}

fn sys_deallocate_pages(virt: VirtAddr, pages: NumOfPages<Size4KiB>) -> u64 {
    allocator::deallocate_pages(virt, pages);
    0
}

fn sys_map_pages(start: PhysAddr, bytes: Bytes) -> VirtAddr {
    crate::mem::map_pages(start, bytes)
}

fn sys_unmap_pages(start: VirtAddr, bytes: Bytes) -> u64 {
    crate::mem::unmap_pages(start, bytes);
    0
}

fn sys_getpid() -> i32 {
    process::manager::getpid()
}

fn sys_exit() -> ! {
    process::manager::exit();
}

fn sys_translate_address(v: VirtAddr) -> PhysAddr {
    PML4.lock().translate_addr(v).unwrap_or_else(PhysAddr::zero)
}

/// # Safety
///
/// `buf` must be valid.
unsafe fn sys_write(fildes: i32, buf: *const c_void, nbyte: u32) -> i32 {
    if fildes == 1 {
        let buf: *const u8 = buf.cast();

        // SAFETY: The caller ensures that `buf` is valid.
        let s = slice::from_raw_parts(buf, nbyte.try_into().unwrap());
        let s = core::str::from_utf8(s);

        match s {
            Ok(s) => {
                // TODO: rewrite with `write` macro.
                info!("{}", s);

                nbyte.try_into().unwrap()
            }
            Err(_) => 0,
        }
    } else {
        unimplemented!("Not stdout");
    }
}

fn sys_notify_exists() -> bool {
    process::manager::notify_exists()
}

fn sys_notify_on_interrupt(vec: usize, pid: i32) {
    interrupt::handler::notify_on_interrupt(vec, pid);
}

// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use num_traits::FromPrimitive;
use os_units::{Bytes, NumOfPages};
use x86_64::{
    self,
    registers::model_specific::{Efer, EferFlags, LStar},
    PhysAddr, VirtAddr,
};

mod implementations;

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

    asm!("", out("rax") syscall_index, out("rdi") a1, out("rsi") a2);
    asm!("", in("rax") select_proper_syscall(syscall_index, a1, a2))
}

/// SAFETY: This function is unsafe because invalid arguments may break memory safety.
#[allow(clippy::too_many_lines)]
unsafe fn select_proper_syscall(idx: u64, a1: u64, a2: u64) -> u64 {
    match FromPrimitive::from_u64(idx) {
        Some(s) => match s {
            syscalls::Ty::Inb => implementations::sys_inb(a1.try_into().unwrap()).into(),
            syscalls::Ty::Outb => {
                implementations::sys_outb(a1.try_into().unwrap(), a2.try_into().unwrap())
            }
            syscalls::Ty::Inl => implementations::sys_inl(a1.try_into().unwrap()).into(),
            syscalls::Ty::Outl => {
                implementations::sys_outl(a1.try_into().unwrap(), a2.try_into().unwrap())
            }
            syscalls::Ty::Halt => implementations::sys_halt(),
            syscalls::Ty::DisableInterrupt => implementations::sys_disable_interrupt(),
            syscalls::Ty::EnableInterrupt => implementations::sys_enable_interrupt(),
            syscalls::Ty::EnableInterruptAndHalt => {
                implementations::sys_enable_interrupt_and_halt()
            }
            syscalls::Ty::AllocatePages => {
                implementations::sys_allocate_pages(NumOfPages::new(a1.try_into().unwrap()))
                    .as_u64()
            }
            syscalls::Ty::DeallocatePages => implementations::sys_deallocate_pages(
                VirtAddr::new(a1),
                NumOfPages::new(a2.try_into().unwrap()),
            ),
            syscalls::Ty::MapPages => implementations::sys_map_pages(
                PhysAddr::new(a1),
                Bytes::new(a2.try_into().unwrap()),
            )
            .as_u64(),
            syscalls::Ty::UnmapPages => implementations::sys_unmap_pages(
                VirtAddr::new(a1),
                Bytes::new(a2.try_into().unwrap()),
            ),
            syscalls::Ty::GetPid => implementations::sys_getpid().try_into().unwrap(),
            syscalls::Ty::Exit => implementations::sys_exit(),
        },
        None => panic!("Unsupported syscall index: {}", idx),
    }
}

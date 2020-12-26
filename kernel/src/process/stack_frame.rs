// SPDX-License-Identifier: GPL-3.0-or-later

use x86_64::{
    registers::rflags,
    structures::{gdt::SegmentSelector, idt::InterruptStackFrameValue},
    VirtAddr,
};

use crate::gdt::GDT;

#[repr(C)]
pub struct StackFrame {
    regs: GeneralRegisters,
    interrupt: InterruptStackFrameValue,
}
impl StackFrame {
    pub fn new_kernel(instruction_pointer: VirtAddr, stack_pointer: VirtAddr) -> Self {
        Creator::new(instruction_pointer, stack_pointer).create_kernel()
    }

    pub fn new_user(instruction_pointer: VirtAddr, stack_pointer: VirtAddr) -> Self {
        Creator::new(instruction_pointer, stack_pointer).create_user()
    }

    fn from_interrupt_stack_frame(interrupt: InterruptStackFrameValue) -> StackFrame {
        StackFrame {
            regs: GeneralRegisters::default(),
            interrupt,
        }
    }
}

struct Creator {
    ip: VirtAddr,
    sp: VirtAddr,
}
impl Creator {
    fn new(ip: VirtAddr, sp: VirtAddr) -> Self {
        Self { ip, sp }
    }

    fn create_kernel(self) -> StackFrame {
        let ist = self.kernel_ist();
        StackFrame::from_interrupt_stack_frame(ist)
    }

    fn create_user(self) -> StackFrame {
        let ist = self.user_ist();
        StackFrame::from_interrupt_stack_frame(ist)
    }

    fn kernel_ist(&self) -> InterruptStackFrameValue {
        self.interrupt_stack_frame(GDT.kernel_code, GDT.kernel_data)
    }

    fn user_ist(&self) -> InterruptStackFrameValue {
        self.interrupt_stack_frame(GDT.user_code, GDT.user_data)
    }

    fn interrupt_stack_frame(
        &self,
        cs: SegmentSelector,
        ss: SegmentSelector,
    ) -> InterruptStackFrameValue {
        InterruptStackFrameValue {
            instruction_pointer: self.ip,
            code_segment: cs.0.into(),
            cpu_flags: rflags::read_raw(),
            stack_pointer: self.sp,
            stack_segment: ss.0.into(),
        }
    }
}

#[repr(C)]
#[derive(Default)]
struct GeneralRegisters {
    _rbp: u64,
    _rax: u64,
    _rbx: u64,
    _rcx: u64,
    _rdx: u64,
    _rsi: u64,
    _rdi: u64,
    _r8: u64,
    _r9: u64,
    _r10: u64,
    _r11: u64,
    _r12: u64,
    _r13: u64,
    _r14: u64,
    _r15: u64,
}

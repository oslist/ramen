// SPDX-License-Identifier: GPL-3.0-or-later

pub fn system_task_main() -> ! {
    info!("I am system task.");
    loop {
        x86_64::instructions::interrupts::enable_and_hlt();
    }
}

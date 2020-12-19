#![no_std]
#![feature(start, lang_items)]

#[start]
#[no_mangle]
pub fn main() {
    syscalls::debug();
}

#[panic_handler]
fn p(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
fn e() {}

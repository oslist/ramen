#![no_std]
#![feature(start)]

#[start]
#[no_mangle]
pub fn main() {}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    panic!()
}

// SPDX-License-Identifier: GPL-3.0-or-later

use crate::graphics::screen::Screen;
use crate::graphics::Vram;
use crate::graphics::RGB;
use crate::print_with_pos;
use core::convert::TryFrom;
use vek::Vec2;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    Screen::draw_rectangle(
        RGB::new(0x00ff_0000),
        &Vec2::new(0, 0),
        &Vec2::new(
            isize::try_from(Vram::resolution().x - 1).unwrap(),
            isize::try_from(Vram::resolution().y - 1).unwrap(),
        ),
    );

    if let Some(location) = info.location() {
        print_with_pos!(
            Vec2::new(0, 0),
            RGB::new(0x00ff_ffff),
            "Panic in {} at ({}, {}):{}",
            location.file(),
            location.line(),
            location.column(),
            info.message().unwrap_or(&format_args!(""))
        );
    }

    loop {
        x86_64::instructions::hlt();
    }
}

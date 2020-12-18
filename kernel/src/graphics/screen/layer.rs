// SPDX-License-Identifier: GPL-3.0-or-later

use super::Vram;
use conquer_once::spin::OnceCell;
use core::convert::TryInto;
use screen_layer::{Layer, Vec2};
use spinning_top::Spinlock;

pub static CONTROLLER: OnceCell<Spinlock<screen_layer::Controller>> = OnceCell::uninit();

pub fn init() {
    CONTROLLER
        .try_init_once(|| {
            Spinlock::new(unsafe {
                screen_layer::Controller::new(
                    Vram::resolution().as_(),
                    Vram::bpp().try_into().unwrap(),
                    Vram::ptr().as_u64().try_into().unwrap(),
                )
            })
        })
        .expect("Layer controller is already initialized.")
}

pub fn add(l: Layer) -> screen_layer::Id {
    get_controller().lock().add_layer(l)
}

pub fn edit<T>(id: screen_layer::Id, f: T) -> Result<(), screen_layer::Error>
where
    T: Fn(&mut Layer),
{
    get_controller().lock().edit_layer(id, f)
}

pub fn slide(id: screen_layer::Id, new_top_left: Vec2<isize>) -> Result<(), screen_layer::Error> {
    get_controller().lock().slide_layer(id, new_top_left)
}

fn get_controller() -> &'static Spinlock<screen_layer::Controller> {
    CONTROLLER
        .try_get()
        .expect("Layer controller is not initialized.")
}

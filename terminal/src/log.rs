// SPDX-License-Identifier: GPL-3.0-or-later

use super::writer::Writer;
use conquer_once::spin::Lazy;
use core::fmt::Write;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use rgb::RGB8;
use spinning_top::Spinlock;
use uart_16550::SerialPort;

static LOGGER: Logger = Logger;

static LOG_WRITER: Lazy<Spinlock<Writer>> =
    Lazy::new(|| Spinlock::new(Writer::new(RGB8::new(0xff, 0xff, 0xff))));

static QEMU_PORT: Lazy<Spinlock<SerialPort>> = Lazy::new(|| {
    // SAFETY: The port number is correct.
    let mut p = unsafe { SerialPort::new(0x3f8) };
    p.init();
    Spinlock::new(p)
});

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        writeln!(*LOG_WRITER.lock(), "{} - {}", record.level(), record.args()).unwrap();

        if !cfg!(feature = "qemu_test") {
            return;
        }

        writeln!(*QEMU_PORT.lock(), "{} - {}", record.level(), record.args())
            .expect("Failed to send a log to the QEMU port.")
    }

    fn flush(&self) {}
}

/// # Errors
///
/// This function may return an error from `log::set_logger` function.
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

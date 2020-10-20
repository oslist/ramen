// SPDX-License-Identifier: GPL-3.0-or-later

use super::pci;

pub async fn task() {
    iter_devices();
}

fn iter_devices() {
    for device in pci::iter_devices() {
        if device.is_pci() {
            info!("AHCI device found.");
        }
    }
}

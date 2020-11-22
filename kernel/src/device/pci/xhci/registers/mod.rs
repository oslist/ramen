// SPDX-License-Identifier: GPL-3.0-or-later

pub mod doorbell;
pub mod hc_capability;
pub mod hc_operational;
pub mod runtime;
pub mod usb_legacy_support_capability;

use hc_capability::HCCapabilityRegisters;
use hc_operational::HCOperational;
use runtime::Runtime;
use usb_legacy_support_capability::UsbLegacySupportCapability;
use x86_64::PhysAddr;

pub struct Registers {
    pub usb_legacy_support_capability: Option<UsbLegacySupportCapability>,
    pub hc_capability: HCCapabilityRegisters,
    pub hc_operational: HCOperational,
    pub runtime: Runtime,
    pub doorbell_array: doorbell::Array,
}
impl Registers {
    /// Safety: This method is unsafe because if `mmio_base` is not the valid MMIO base address,
    /// it can violate memory safety.
    pub unsafe fn new(mmio_base: PhysAddr) -> Self {
        let hc_capability_registers = HCCapabilityRegisters::new(mmio_base);
        let usb_legacy_support_capability =
            UsbLegacySupportCapability::new(mmio_base, &hc_capability_registers);
        let hc_operational = HCOperational::new(mmio_base, &hc_capability_registers);
        let runtime_base_registers = Runtime::new(
            mmio_base,
            hc_capability_registers.rts_off.read().get() as usize,
        );
        let doorbell_array =
            doorbell::Array::new(mmio_base, hc_capability_registers.db_off.read().get());

        Self {
            usb_legacy_support_capability,
            hc_capability: hc_capability_registers,
            hc_operational,
            runtime: runtime_base_registers,
            doorbell_array,
        }
    }
}

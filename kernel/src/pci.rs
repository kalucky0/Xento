use x86_64::instructions::port::Port;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

/// PCI Class codes
pub const CLASS_MASS_STORAGE: u8 = 0x01;

/// PCI Subclass codes for Mass Storage
pub const SUBCLASS_SATA: u8 = 0x06;

/// PCI Programming Interface for SATA
pub const PROG_IF_AHCI: u8 = 0x01;

/// Represents a PCI device location
#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
}

impl PciDevice {
    /// Read a 32-bit configuration register
    pub fn read_config(&self, offset: u8) -> u32 {
        let address = 0x80000000u32
            | ((self.bus as u32) << 16)
            | ((self.device as u32) << 11)
            | ((self.function as u32) << 8)
            | ((offset as u32) & 0xFC);

        unsafe {
            let mut addr_port: Port<u32> = Port::new(CONFIG_ADDRESS);
            let mut data_port: Port<u32> = Port::new(CONFIG_DATA);

            addr_port.write(address);
            data_port.read()
        }
    }

    /// Write a 32-bit configuration register
    pub fn write_config(&self, offset: u8, value: u32) {
        let address = 0x80000000u32
            | ((self.bus as u32) << 16)
            | ((self.device as u32) << 11)
            | ((self.function as u32) << 8)
            | ((offset as u32) & 0xFC);

        unsafe {
            let mut addr_port: Port<u32> = Port::new(CONFIG_ADDRESS);
            let mut data_port: Port<u32> = Port::new(CONFIG_DATA);

            addr_port.write(address);
            data_port.write(value);
        }
    }

    /// Read Base Address Register (BAR)
    pub fn read_bar(&self, bar_num: u8) -> u32 {
        assert!(bar_num < 6);
        self.read_config(0x10 + (bar_num * 4))
    }

    /// Enable bus mastering for DMA
    pub fn enable_bus_mastering(&self) {
        let mut command = self.read_config(0x04) as u16;
        command |= 0x04; // Bus Master Enable bit
        self.write_config(0x04, command as u32);
    }

    /// Enable memory space
    pub fn enable_memory_space(&self) {
        let mut command = self.read_config(0x04) as u16;
        command |= 0x02; // Memory Space Enable bit
        self.write_config(0x04, command as u32);
    }
}

/// Scan the PCI bus for devices
/// NOTE: This performs a full bus scan (255 buses × 32 devices × 8 functions).
/// In a production system, this could be optimized by:
/// - Checking if a bus exists before scanning all devices
/// - Using ACPI MCFG to determine valid bus ranges
/// - Implementing early termination for empty buses
pub fn scan_bus() -> alloc::vec::Vec<PciDevice> {
    use alloc::vec::Vec;
    
    let mut devices = Vec::new();

    for bus in 0..=255u8 {
        for device in 0..32u8 {
            for function in 0..8u8 {
                if let Some(pci_device) = probe_device(bus, device, function) {
                    devices.push(pci_device);
                    
                    // If this is not a multi-function device, skip remaining functions
                    if function == 0 {
                        let header_type = ((pci_device.read_config(0x0C) >> 16) & 0xFF) as u8;
                        if (header_type & 0x80) == 0 {
                            break;
                        }
                    }
                }
            }
        }
    }

    devices
}

/// Probe a specific PCI device location
fn probe_device(bus: u8, device: u8, function: u8) -> Option<PciDevice> {
    let address = 0x80000000u32
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8);

    unsafe {
        let mut addr_port: Port<u32> = Port::new(CONFIG_ADDRESS);
        let mut data_port: Port<u32> = Port::new(CONFIG_DATA);

        addr_port.write(address);
        let vendor_device = data_port.read();

        let vendor_id = (vendor_device & 0xFFFF) as u16;
        let device_id = ((vendor_device >> 16) & 0xFFFF) as u16;

        // 0xFFFF means no device present
        if vendor_id == 0xFFFF {
            return None;
        }

        // Read class code information
        addr_port.write(address | 0x08);
        let class_info = data_port.read();

        let class = ((class_info >> 24) & 0xFF) as u8;
        let subclass = ((class_info >> 16) & 0xFF) as u8;
        let prog_if = ((class_info >> 8) & 0xFF) as u8;

        Some(PciDevice {
            bus,
            device,
            function,
            vendor_id,
            device_id,
            class,
            subclass,
            prog_if,
        })
    }
}

/// Find the first AHCI controller
pub fn find_ahci_controller() -> Option<PciDevice> {
    let devices = scan_bus();
    
    devices.into_iter().find(|dev| {
        dev.class == CLASS_MASS_STORAGE 
            && dev.subclass == SUBCLASS_SATA 
            && dev.prog_if == PROG_IF_AHCI
    })
}

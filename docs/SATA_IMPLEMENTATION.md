# SATA Drive Support Implementation

## Overview

This implementation adds support for reading data from SATA drives using the Advanced Host Controller Interface (AHCI) specification.

## Components

### 1. PCI Device Enumeration (`kernel/src/pci.rs`)

The PCI module provides functionality to:
- Scan the PCI bus for devices
- Read and write PCI configuration space
- Find AHCI controllers (Class 0x01, Subclass 0x06, Programming Interface 0x01)
- Enable bus mastering and memory space for DMA operations

Key functions:
- `scan_bus()` - Scans all PCI buses and returns a list of devices
- `find_ahci_controller()` - Finds the first AHCI controller in the system

### 2. AHCI Driver (`kernel/src/ahci.rs`)

The AHCI driver implements:
- AHCI HBA (Host Bus Adapter) memory-mapped register access
- Port initialization and configuration
- Command submission using Command List and FIS (Frame Information Structure)
- DMA-based sector reading from SATA disks

Key structures:
- `HbaMemory` - AHCI HBA memory registers
- `HbaPort` - Per-port AHCI registers
- `FisRegH2D` - Host to Device Register FIS for ATA commands
- `HbaCmdHeader` - Command header structure
- `HbaCmdTbl` - Command table with PRDT (Physical Region Descriptor Table)

Key functions:
- `AhciController::new()` - Initializes the AHCI controller
- `AhciController::read()` - Reads sectors from a SATA disk
- `read_sectors()` - Public API to read sectors from the first available disk

## Usage

The AHCI driver is initialized automatically during kernel boot in the `init()` function.

To read sectors from a SATA disk:

```rust
use kernel::ahci;

// Read one sector (512 bytes) starting at LBA 0
let mut buffer = vec![0u8; 512];
match ahci::read_sectors(0, 1, &mut buffer) {
    Ok(()) => {
        // Data is now in buffer
        println!("Read successful!");
    }
    Err(e) => {
        println!("Read failed: {}", e);
    }
}
```

## Technical Details

### AHCI Initialization Process

1. Find AHCI controller on PCI bus
2. Enable bus mastering and memory space
3. Get ABAR (AHCI Base Address Register)
4. Enable AHCI mode
5. Detect active SATA ports
6. Allocate command lists and FIS receive areas
7. Start command engines

### Reading Data

1. Find a free command slot
2. Build Command FIS (H2D Register FIS with ATA READ DMA EX command)
3. Setup PRDT entries with buffer physical addresses
4. Issue command by writing to Port Command Issue register
5. Wait for command completion
6. Check for errors

### Memory Layout

For each port:
- Command List: 1KB (32 command headers, 32 bytes each)
- FIS Receive Area: 256 bytes
- Command Tables: 384 bytes each (32 tables)

## Limitations

Current implementation:
- Supports reading only (no write support yet)
- Uses polling for command completion (no interrupt handling)
- Single command at a time per port
- Assumes physical memory addresses match virtual addresses for DMA buffers

## Future Enhancements

- Add write support
- Implement interrupt-based completion
- Support for multiple concurrent commands
- Port multiplier support
- NCQ (Native Command Queuing)
- TRIM/UNMAP support
- S.M.A.R.T. monitoring

## References

- [OSDev Wiki - AHCI](https://wiki.osdev.org/AHCI)
- [Intel AHCI Specification](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/serial-ata-ahci-spec-rev1_3.pdf)
- [SATA Specification](https://sata-io.org/)

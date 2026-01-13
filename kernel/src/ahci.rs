use crate::pci::PciDevice;
use crate::serial_println;
use core::ptr::{read_volatile, write_volatile};
use spin::Mutex;

/// AHCI HBA Memory Registers
#[repr(C)]
pub struct HbaMemory {
    // Generic Host Control
    cap: u32,      // 0x00, Host capability
    ghc: u32,      // 0x04, Global host control
    is: u32,       // 0x08, Interrupt status
    pi: u32,       // 0x0C, Port implemented
    vs: u32,       // 0x10, Version
    ccc_ctl: u32,  // 0x14, Command completion coalescing control
    ccc_pts: u32,  // 0x18, Command completion coalescing ports
    em_loc: u32,   // 0x1C, Enclosure management location
    em_ctl: u32,   // 0x20, Enclosure management control
    cap2: u32,     // 0x24, Host capabilities extended
    bohc: u32,     // 0x28, BIOS/OS handoff control and status
}

/// AHCI Port Registers
#[repr(C)]
pub struct HbaPort {
    clb: u64,       // 0x00, Command list base address
    fb: u64,        // 0x08, FIS base address
    is: u32,        // 0x10, Interrupt status
    ie: u32,        // 0x14, Interrupt enable
    cmd: u32,       // 0x18, Command and status
    _rsv0: u32,     // 0x1C, Reserved
    tfd: u32,       // 0x20, Task file data
    sig: u32,       // 0x24, Signature
    ssts: u32,      // 0x28, SATA status (SCR0:SStatus)
    sctl: u32,      // 0x2C, SATA control (SCR2:SControl)
    serr: u32,      // 0x30, SATA error (SCR1:SError)
    sact: u32,      // 0x34, SATA active (SCR3:SActive)
    ci: u32,        // 0x38, Command issue
    sntf: u32,      // 0x3C, SATA notification (SCR4:SNotification)
    fbs: u32,       // 0x40, FIS-based switch control
    _rsv1: [u32; 11], // 0x44 ~ 0x6F, Reserved
    vendor: [u32; 4], // 0x70 ~ 0x7F, Vendor specific
}

/// Host to Device FIS
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FisRegH2D {
    // DWORD 0
    fis_type: u8,   // FIS_TYPE_REG_H2D
    flags: u8,      // Port multiplier, C bit
    command: u8,    // Command register
    featurel: u8,   // Feature register, 7:0

    // DWORD 1
    lba0: u8,       // LBA low register, 7:0
    lba1: u8,       // LBA mid register, 15:8
    lba2: u8,       // LBA high register, 23:16
    device: u8,     // Device register

    // DWORD 2
    lba3: u8,       // LBA register, 31:24
    lba4: u8,       // LBA register, 39:32
    lba5: u8,       // LBA register, 47:40
    featureh: u8,   // Feature register, 15:8

    // DWORD 3
    countl: u8,     // Count register, 7:0
    counth: u8,     // Count register, 15:8
    icc: u8,        // Isochronous command completion
    control: u8,    // Control register

    // DWORD 4
    rsv1: [u8; 4],  // Reserved
}

/// Command header
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HbaCmdHeader {
    flags: u16,     // Command FIS length in DWORDS, 2 ~ 16, atapi, write, prefetchable, reset, bist, clear busy, port multiplier
    prdtl: u16,     // Physical region descriptor table length in entries
    prdbc: u32,     // Physical region descriptor byte count transferred
    ctba: u64,      // Command table descriptor base address
    rsv1: [u32; 4], // Reserved
}

/// Physical Region Descriptor Table Entry
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HbaPrdtEntry {
    dba: u64,       // Data base address
    rsv0: u32,      // Reserved
    flags: u32,     // Interrupt on completion, Byte count
}

/// Command Table
#[repr(C, align(128))]
pub struct HbaCmdTbl {
    cfis: [u8; 64],                     // Command FIS
    acmd: [u8; 16],                     // ATAPI command, 12 or 16 bytes
    rsv: [u8; 48],                      // Reserved
    prdt_entry: [HbaPrdtEntry; 1],      // Physical region descriptor table entries
}

// FIS Types
const FIS_TYPE_REG_H2D: u8 = 0x27; // Register FIS - host to device

// ATA Commands
const ATA_CMD_READ_DMA_EX: u8 = 0x25;

// Port command register bits
const HBA_PX_CMD_ST: u32 = 0x0001;
const HBA_PX_CMD_FRE: u32 = 0x0010;
const HBA_PX_CMD_FR: u32 = 0x4000;
const HBA_PX_CMD_CR: u32 = 0x8000;

// Port SATA Status
const HBA_PORT_IPM_ACTIVE: u32 = 1;
const HBA_PORT_DET_PRESENT: u32 = 3;

pub struct AhciController {
    abar: usize,
    ports: [Option<usize>; 32],
}

impl AhciController {
    pub fn new(pci_device: &PciDevice) -> Option<Self> {
        // Enable bus mastering and memory space
        pci_device.enable_bus_mastering();
        pci_device.enable_memory_space();

        // Get ABAR (AHCI Base Address Register) - BAR5
        let abar = pci_device.read_bar(5) as usize & !0xFFF;
        
        if abar == 0 {
            serial_println!("AHCI: Invalid ABAR");
            return None;
        }

        serial_println!("AHCI: ABAR at 0x{:x}", abar);

        let mut controller = AhciController {
            abar,
            ports: [None; 32],
        };

        controller.init();
        Some(controller)
    }

    fn init(&mut self) {
        unsafe {
            let hba = self.abar as *mut HbaMemory;
            
            // Read capabilities
            let cap = read_volatile(&(*hba).cap);
            let pi = read_volatile(&(*hba).pi);
            
            serial_println!("AHCI: Capabilities: 0x{:x}", cap);
            serial_println!("AHCI: Ports implemented: 0x{:x}", pi);

            // Enable AHCI mode
            let mut ghc = read_volatile(&(*hba).ghc);
            ghc |= 0x80000000; // Set AE bit
            write_volatile(&mut (*hba).ghc, ghc);

            // Probe ports
            for i in 0..32 {
                if (pi & (1 << i)) != 0 {
                    self.probe_port(i);
                }
            }
        }
    }

    fn probe_port(&mut self, port_num: u32) {
        unsafe {
            let port_offset = 0x100 + (port_num as usize * 0x80);
            let port = (self.abar + port_offset) as *mut HbaPort;

            let ssts = read_volatile(&(*port).ssts);
            let det = ssts & 0xF;
            let ipm = (ssts >> 8) & 0xF;

            if det != HBA_PORT_DET_PRESENT || ipm != HBA_PORT_IPM_ACTIVE {
                return;
            }

            let sig = read_volatile(&(*port).sig);
            serial_println!("AHCI: Port {} is active, signature: 0x{:x}", port_num, sig);

            self.ports[port_num as usize] = Some(port as usize);
            self.port_rebase(port_num);
        }
    }

    fn port_rebase(&mut self, port_num: u32) {
        if self.ports[port_num as usize].is_none() {
            return;
        }

        unsafe {
            let port = self.ports[port_num as usize].unwrap() as *mut HbaPort;
            
            // Stop command engine
            self.stop_cmd(port);

            // Allocate memory for command list (1K per port)
            let cmdlist_base = alloc::alloc::alloc(
                alloc::alloc::Layout::from_size_align(1024, 1024).unwrap()
            ) as u64;
            
            // Allocate memory for FIS (256 bytes per port)
            let fis_base = alloc::alloc::alloc(
                alloc::alloc::Layout::from_size_align(256, 256).unwrap()
            ) as u64;

            write_volatile(&mut (*port).clb, cmdlist_base);
            write_volatile(&mut (*port).fb, fis_base);

            // Clear any pending interrupts
            write_volatile(&mut (*port).is, read_volatile(&(*port).is));
            write_volatile(&mut (*port).ie, 0);

            // Allocate command tables
            let cmdheader = cmdlist_base as *mut HbaCmdHeader;
            for i in 0..32 {
                let header = cmdheader.add(i);
                (*header).prdtl = 8; // 8 PRDT entries per command
                
                // Allocate command table (256 bytes + 8*16 PRDT)
                let cmdtbl = alloc::alloc::alloc(
                    alloc::alloc::Layout::from_size_align(384, 128).unwrap()
                ) as u64;
                
                write_volatile(&mut (*header).ctba, cmdtbl);
            }

            // Start command engine
            self.start_cmd(port);
        }
    }

    unsafe fn stop_cmd(&self, port: *mut HbaPort) {
        // Clear ST (bit 0)
        let mut cmd = read_volatile(&(*port).cmd);
        cmd &= !HBA_PX_CMD_ST;
        write_volatile(&mut (*port).cmd, cmd);

        // Wait until FR (bit 14), CR (bit 15) are cleared
        loop {
            cmd = read_volatile(&(*port).cmd);
            if (cmd & (HBA_PX_CMD_FR | HBA_PX_CMD_CR)) == 0 {
                break;
            }
        }

        // Clear FRE (bit 4)
        cmd = read_volatile(&(*port).cmd);
        cmd &= !HBA_PX_CMD_FRE;
        write_volatile(&mut (*port).cmd, cmd);
    }

    unsafe fn start_cmd(&self, port: *mut HbaPort) {
        // Wait until CR (bit 15) is cleared
        loop {
            let cmd = read_volatile(&(*port).cmd);
            if (cmd & HBA_PX_CMD_CR) == 0 {
                break;
            }
        }

        // Set FRE (bit 4) and ST (bit 0)
        let mut cmd = read_volatile(&(*port).cmd);
        cmd |= HBA_PX_CMD_FRE;
        cmd |= HBA_PX_CMD_ST;
        write_volatile(&mut (*port).cmd, cmd);
    }

    fn find_cmdslot(&self, port: *mut HbaPort) -> Option<u32> {
        unsafe {
            let slots = read_volatile(&(*port).sact) | read_volatile(&(*port).ci);
            
            for i in 0..32 {
                if (slots & (1 << i)) == 0 {
                    return Some(i);
                }
            }
            None
        }
    }

    pub fn read(&self, port_num: u32, start_lba: u64, count: u16, buf: &mut [u8]) -> Result<(), &'static str> {
        if self.ports[port_num as usize].is_none() {
            return Err("Port not available");
        }

        unsafe {
            let port = self.ports[port_num as usize].unwrap() as *mut HbaPort;
            let slot = self.find_cmdslot(port).ok_or("No free command slot")?;

            let cmdheader = read_volatile(&(*port).clb) as *mut HbaCmdHeader;
            let header = cmdheader.add(slot as usize);
            
            (*header).flags = (core::mem::size_of::<FisRegH2D>() / 4) as u16; // Command FIS size
            (*header).flags &= !0x0040; // Read from device
            (*header).prdtl = 1; // One PRDT entry

            let cmdtbl = read_volatile(&(*header).ctba) as *mut HbaCmdTbl;
            
            // Setup PRDT
            let buf_phys = buf.as_ptr() as u64;
            (*cmdtbl).prdt_entry[0].dba = buf_phys;
            (*cmdtbl).prdt_entry[0].flags = ((count as u32 * 512) - 1) & 0x3FFFFF; // Byte count
            
            // Setup command FIS
            let cmdfis = &mut (*cmdtbl).cfis as *mut _ as *mut FisRegH2D;
            core::ptr::write_bytes(cmdfis, 0, 1);
            
            (*cmdfis).fis_type = FIS_TYPE_REG_H2D;
            (*cmdfis).flags = 0x80; // Command bit
            (*cmdfis).command = ATA_CMD_READ_DMA_EX;
            
            (*cmdfis).lba0 = (start_lba & 0xFF) as u8;
            (*cmdfis).lba1 = ((start_lba >> 8) & 0xFF) as u8;
            (*cmdfis).lba2 = ((start_lba >> 16) & 0xFF) as u8;
            (*cmdfis).lba3 = ((start_lba >> 24) & 0xFF) as u8;
            (*cmdfis).lba4 = ((start_lba >> 32) & 0xFF) as u8;
            (*cmdfis).lba5 = ((start_lba >> 40) & 0xFF) as u8;
            
            (*cmdfis).device = 0x40; // LBA mode
            (*cmdfis).countl = (count & 0xFF) as u8;
            (*cmdfis).counth = ((count >> 8) & 0xFF) as u8;

            // Wait for port to be ready
            let mut spin = 0;
            loop {
                let tfd = read_volatile(&(*port).tfd);
                if (tfd & 0x88) == 0 {
                    break;
                }
                if spin > 1000000 {
                    return Err("Port hung");
                }
                spin += 1;
            }

            // Issue command
            write_volatile(&mut (*port).ci, 1 << slot);

            // Wait for completion
            loop {
                let ci = read_volatile(&(*port).ci);
                if (ci & (1 << slot)) == 0 {
                    break;
                }
                
                let is = read_volatile(&(*port).is);
                if (is & (1 << 30)) != 0 { // Task file error
                    return Err("Disk read error");
                }
            }

            Ok(())
        }
    }
}

pub static AHCI: Mutex<Option<AhciController>> = Mutex::new(None);

pub fn init() {
    use crate::pci;
    
    serial_println!("Initializing AHCI...");
    
    if let Some(ahci_dev) = pci::find_ahci_controller() {
        serial_println!(
            "Found AHCI controller: {:04x}:{:04x} at bus {}, dev {}, func {}",
            ahci_dev.vendor_id, ahci_dev.device_id,
            ahci_dev.bus, ahci_dev.device, ahci_dev.function
        );
        
        if let Some(controller) = AhciController::new(&ahci_dev) {
            *AHCI.lock() = Some(controller);
            serial_println!("AHCI initialized successfully");
        } else {
            serial_println!("Failed to initialize AHCI controller");
        }
    } else {
        serial_println!("No AHCI controller found");
    }
}

/// Read sectors from the first available SATA disk
pub fn read_sectors(start_lba: u64, count: u16, buf: &mut [u8]) -> Result<(), &'static str> {
    let ahci = AHCI.lock();
    let controller = ahci.as_ref().ok_or("AHCI not initialized")?;
    
    // Find first active port
    for i in 0..32 {
        if controller.ports[i].is_some() {
            return controller.read(i as u32, start_lba, count, buf);
        }
    }
    
    Err("No active SATA ports")
}

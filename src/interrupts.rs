use crate::hlt_loop;
use crate::pic;
use crate::serial_println;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

const PIC1: u16 = 0x21;
const PIC2: u16 = 0xA1;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;
pub const GENERAL_PROTECTION_FAULT_IST_INDEX: u16 = 2;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Cascade,
    COM2,
    COM1,
    LPT2,
    FloppyDisk,
    LPT1,
    RTC,
    Peripherals1,
    Peripherals2,
    Peripherals3,
    PS2,
    FPU,
    PrimaryATA,
    SecondaryATA,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

fn default_irq_handler() {}

lazy_static! {
    pub static ref IRQ_HANDLERS: Mutex<[fn(); 16]> = Mutex::new([default_irq_handler; 16]);
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
            idt.page_fault
                .set_handler_fn(page_fault_handler)
                .set_stack_index(PAGE_FAULT_IST_INDEX);
            idt.general_protection_fault
                .set_handler_fn(general_protection_fault_handler)
                .set_stack_index(GENERAL_PROTECTION_FAULT_IST_INDEX);
        }
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(irq0_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Cascade.as_usize()].set_handler_fn(irq2_handler);
        idt[InterruptIndex::COM2.as_usize()].set_handler_fn(irq3_handler);
        idt[InterruptIndex::COM1.as_usize()].set_handler_fn(irq4_handler);
        idt[InterruptIndex::LPT2.as_usize()].set_handler_fn(irq5_handler);
        idt[InterruptIndex::FloppyDisk.as_usize()].set_handler_fn(irq6_handler);
        idt[InterruptIndex::LPT1.as_usize()].set_handler_fn(irq7_handler);
        idt[InterruptIndex::RTC.as_usize()].set_handler_fn(irq8_handler);
        idt[InterruptIndex::Peripherals1.as_usize()].set_handler_fn(irq9_handler);
        idt[InterruptIndex::Peripherals2.as_usize()].set_handler_fn(irq10_handler);
        idt[InterruptIndex::Peripherals3.as_usize()].set_handler_fn(irq11_handler);
        idt[InterruptIndex::PS2.as_usize()].set_handler_fn(irq12_handler);
        idt[InterruptIndex::FPU.as_usize()].set_handler_fn(irq13_handler);
        idt[InterruptIndex::PrimaryATA.as_usize()].set_handler_fn(irq14_handler);
        idt[InterruptIndex::SecondaryATA.as_usize()].set_handler_fn(irq15_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("EXCEPTION: STACK SEGMENT FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    serial_println!("EXCEPTION: PAGE FAULT");
    serial_println!("Accessed Address: {:?}", Cr2::read());
    serial_println!("Error Code: {:?}", error_code);
    serial_println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        pic::PICS
            .lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

macro_rules! irq_handler {
    ($handler:ident, $irq:expr) => {
        pub extern "x86-interrupt" fn $handler(_stack_frame: InterruptStackFrame) {
            let handlers = IRQ_HANDLERS.lock();
            handlers[$irq as usize - PIC_1_OFFSET as usize]();
            unsafe {
                pic::PICS.lock().notify_end_of_interrupt($irq);
            }
        }
    };
}

irq_handler!(irq0_handler, InterruptIndex::Timer.as_u8());
irq_handler!(irq2_handler, InterruptIndex::Cascade.as_u8());
irq_handler!(irq3_handler, InterruptIndex::COM2.as_u8());
irq_handler!(irq4_handler, InterruptIndex::COM1.as_u8());
irq_handler!(irq5_handler, InterruptIndex::LPT2.as_u8());
irq_handler!(irq6_handler, InterruptIndex::FloppyDisk.as_u8());
irq_handler!(irq7_handler, InterruptIndex::LPT1.as_u8());
irq_handler!(irq8_handler, InterruptIndex::RTC.as_u8());
irq_handler!(irq9_handler, InterruptIndex::Peripherals1.as_u8());
irq_handler!(irq10_handler, InterruptIndex::Peripherals2.as_u8());
irq_handler!(irq11_handler, InterruptIndex::Peripherals3.as_u8());
irq_handler!(irq12_handler, InterruptIndex::PS2.as_u8());
irq_handler!(irq13_handler, InterruptIndex::FPU.as_u8());
irq_handler!(irq14_handler, InterruptIndex::PrimaryATA.as_u8());
irq_handler!(irq15_handler, InterruptIndex::SecondaryATA.as_u8());

pub fn set_irq_handler(irq: u8, handler: fn()) {
    instructions::interrupts::without_interrupts(|| {
        let mut handlers = IRQ_HANDLERS.lock();
        handlers[irq as usize] = handler;

        clear_irq_mask(irq);
    });
}

pub fn set_irq_mask(irq: u8) {
    let mut port: Port<u8> = Port::new(if irq < 8 { PIC1 } else { PIC2 });
    unsafe {
        let value = port.read() | (1 << (if irq < 8 { irq } else { irq - 8 }));
        port.write(value);
    }
}

pub fn clear_irq_mask(irq: u8) {
    let mut port: Port<u8> = Port::new(if irq < 8 { PIC1 } else { PIC2 });
    unsafe {
        let value = port.read() & !(1 << if irq < 8 { irq } else { irq - 8 });
        port.write(value);
    }
}

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(exclusive_range_pattern)]

extern crate alloc;
extern crate log;

pub mod allocator;
pub mod clock;
pub mod cmos;
pub mod interrupts;
pub mod memory;
pub mod pic;
pub mod serial;
pub mod task;
pub mod time;

use crate::{
    memory::BootInfoFrameAllocator,
    task::{executor::Executor, keyboard, Task},
};
use alloc::string::String;
use bootloader::{boot_info::FrameBufferInfo, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

pub fn main(boot_info: &'static mut BootInfo) -> ! {
    interrupts::init_idt();

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info: FrameBufferInfo = framebuffer.info();

        if let Some(physical_memory_offset) = boot_info.physical_memory_offset.as_mut() {
            let phys_mem_offset = VirtAddr::new(*physical_memory_offset);
            let mut mapper = unsafe { memory::init(phys_mem_offset) };
            let mut frame_allocator =
                unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

            allocator::init_heap(&mut mapper, &mut frame_allocator)
                .expect("heap initialization failed");
        } else {
            panic!("Could not find physical memory offset");
        }

        let renderer = userland::init_renderer(
            framebuffer.buffer_mut(),
            info.horizontal_resolution,
            info.vertical_resolution,
        );

        userland::show_splash(renderer);

        init();

        let mut executor = Executor::new();
        executor.spawn(Task::new(keyboard::print_keypresses()));
        executor.run();
    }

    loop {}
}

fn init() {
    task::mouse::init();

    pic::init();
    time::init();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn binary_to_text(binary: &[u8]) -> String {
    let mut text = String::new();
    for byte in binary {
        if *byte == 0 {
            break;
        }
        text.push(*byte as char);
    }
    text
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_print!("{}", info);
    hlt_loop();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

// #[no_mangle]
// fn fminf(a: f32, b: f32) -> f32 {
//     if a < b {
//         a
//     } else {
//         b
//     }
// }

// #[no_mangle]
// fn fmaxf(a: f32, b: f32) -> f32 {
//     if a > b {
//         a
//     } else {
//         b
//     }
// }

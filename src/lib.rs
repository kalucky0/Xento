#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(exclusive_range_pattern)]

extern crate alloc;
use alloc::string::String;
use bootloader::boot_info::FrameBufferInfo;
use core::panic::PanicInfo;

pub mod allocator;
pub mod clock;
pub mod cmos;
pub mod interrupts;
pub mod memory;
pub mod pic;
pub mod renderer;
pub mod serial;
pub mod task;
pub mod time;
pub mod gui;

pub fn init() {
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

pub fn init_renderer(
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
) -> &renderer::LockedRenderer {
    let renderer =
        renderer::RENDERER.get_or_init(move || renderer::LockedRenderer::new(framebuffer, info));
    log::set_logger(renderer).expect("renderer already set");
    log::set_max_level(log::LevelFilter::Trace);
    renderer
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

#[no_mangle]
fn fminf(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

#[no_mangle]
fn fmaxf(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}
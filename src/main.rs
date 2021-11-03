#![no_std]
#![no_main]

extern crate alloc;
extern crate font8x8;
extern crate log;

use tk_os::task::{executor::Executor, keyboard, Task};
use bootloader::boot_info::{FrameBufferInfo};
use bootloader::{entry_point, BootInfo};
use tk_os::init_logger;
use tk_os::logger::{LockedLogger};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use tk_os::allocator;
    use tk_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    tk_os::init();

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info: FrameBufferInfo = framebuffer.info();
        let logger = init_logger(framebuffer.buffer_mut(), info);

        intro(logger);

        if let Some(physical_memory_offset) = boot_info.physical_memory_offset.as_mut() {
            let phys_mem_offset = VirtAddr::new(*physical_memory_offset);
            let mut mapper = unsafe { memory::init(phys_mem_offset) };
            let mut frame_allocator =
                unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

            allocator::init_heap(&mut mapper, &mut frame_allocator)
                .expect("heap initialization failed");

        } else {
            log::error!("Could not find physical memory offset");
        }

        let mut executor = Executor::new();
        executor.spawn(Task::new(keyboard::print_keypresses()));
        executor.run();
    }

    loop {}
}

fn intro(l: &LockedLogger) {
    let mut logger = l.lock();
    logger.write_char('\n');
    logger.write_centered_string("KAL INDUSTRIES TERAKRAFT OPERATING SYSTEM\n");
    logger.write_centered_string("COPYRIGHT 2020-2022 KAL INDUSTRIES\n");
    logger.write_centered_string("-Server 6-");
    logger.write_char('\n');
    logger.write_char('\n');
    logger.write_string("\n > ");
}
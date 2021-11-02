#![no_std]
#![no_main]

extern crate alloc;
extern crate font8x8;
extern crate log;

use tk_os::logger::Logger;
use tk_os::task::{executor::Executor, keyboard, Task};
use bootloader::boot_info::{FrameBufferInfo, FrameBuffer};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use tk_os::{serial_println, serial_print};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use tk_os::allocator;
    use tk_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info: FrameBufferInfo = framebuffer.info();
        let mut logger = Logger::new(framebuffer.buffer_mut(), info);

        logger.write_string("\n > ");

        // serial_println!("A");

        tk_os::init();

        if let Some(physical_memory_offset) = boot_info.physical_memory_offset.as_mut() {
            let phys_mem_offset = VirtAddr::new(*physical_memory_offset);
            let mut mapper = unsafe { memory::init(phys_mem_offset) };
            let mut frame_allocator =
                unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

            allocator::init_heap(&mut mapper, &mut frame_allocator)
                .expect("heap initialization failed");

        } else {
            logger.write_string("Could not find physical memory offset");
        }

        serial_println!("A");
        
        // executor.spawn(Task::new(example_task()));
        let mut executor = Executor::new();
        // executor.spawn(Task::new(keyboard::print_keypresses()));
        executor.run();
    } else {
        let mut executor = Executor::new();
        executor.run();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_print!("{}", info);
    tk_os::hlt_loop();
}

/*async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}*/

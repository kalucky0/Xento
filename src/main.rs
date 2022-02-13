#![no_std]
#![no_main]

extern crate alloc;
extern crate font8x8;
extern crate log;

use bootloader::boot_info::FrameBufferInfo;
use bootloader::{entry_point, BootInfo};
use tk_os::renderer::LockedRenderer;
use tk_os::task::{executor::Executor, keyboard, Task};
use tk_os::{gui, init_renderer};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use tk_os::allocator;
    use tk_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    tk_os::init();

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
            log::error!("Could not find physical memory offset");
        }

        let renderer = init_renderer(framebuffer.buffer_mut(), info);

        gui::splash::show(renderer);

        tk_os::sec_init();

        // tk_os::time::sleep(4.0);

        let mut desktop = gui::Desktop::new(renderer);
        desktop.start();

        // let _terminal = init_terminal(renderer);

        let mut executor = Executor::new();
        executor.spawn(Task::new(keyboard::print_keypresses()));
        executor.run();
    }

    loop {}
}
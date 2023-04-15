use bootloader::boot_info::FrameBufferInfo;
use conquer_once::spin::OnceCell;
use spinning_top::{RawSpinlock, Spinlock};
use tiny_skia::{Color, Pixmap};

pub static RENDERER: OnceCell<LockedRenderer> = OnceCell::uninit();
pub struct LockedRenderer(Spinlock<Renderer>);

impl LockedRenderer {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        LockedRenderer(Spinlock::new(Renderer::new(framebuffer, info)))
    }

    pub fn lock(&self) -> spinning_top::lock_api::MutexGuard<'_, RawSpinlock, Renderer> {
        self.0.lock()
    }

    pub unsafe fn force_unlock(&self) {
        self.0.force_unlock();
    }
}

impl log::Log for LockedRenderer {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        // let mut renderer = self.0.lock();
        // if record.level() == log::Level::Info {
        // writeln!(renderer, "{}", record.args()).unwrap();
        // } else {
        // writeln!(renderer, "{}: {}", record.level(), record.args()).unwrap();
        // }
    }

    fn flush(&self) {}
}

pub struct Renderer {
    framebuffer: &'static mut [u8],
    pixmap: Pixmap,
    info: FrameBufferInfo,
}

impl Renderer {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let width: u32 = match info.horizontal_resolution.try_into() {
            Ok(width) => width,
            Err(_) => panic!("width too large"),
        };

        let height: u32 = match info.vertical_resolution.try_into() {
            Ok(height) => height,
            Err(_) => panic!("height too large"),
        };

        let pixmap = match Pixmap::new(width, height) {
            Some(pixmap) => pixmap,
            None => panic!("failed to create pixmap"),
        };

        let mut renderer = Self {
            framebuffer,
            pixmap,
            info,
        };
        renderer.clear();
        renderer
    }

    pub fn get(&mut self) -> &mut Self {
        self
    }

    pub fn info(&self) -> &FrameBufferInfo {
        &self.info
    }

    pub fn clear(&mut self) {
        self.pixmap.fill(Color::from_rgba8(0, 0, 0, 0));
    }

    pub fn fill(&mut self, color: Color) {
        self.pixmap.fill(color);
    }

    pub fn update(&mut self) {
        let num_pixels = self.framebuffer.len() / 4;
        let data = self.pixmap.data();

        for i in 0..num_pixels {
            let offset = i * 4;
            self.framebuffer[offset] = data[offset + 2];
            self.framebuffer[offset + 1] = data[offset + 1];
            self.framebuffer[offset + 2] = data[offset];
        }
    }

    pub fn width(&self) -> usize {
        self.info.horizontal_resolution
    }

    pub fn height(&self) -> usize {
        self.info.vertical_resolution
    }

    pub fn pixmap(&mut self) -> &mut Pixmap {
        &mut self.pixmap
    }
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

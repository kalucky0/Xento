#![no_std]

extern crate alloc;

pub mod gui;
pub mod renderer;
pub mod resources;

use renderer::{LockedRenderer, RENDERER};

pub fn init_renderer(framebuffer: &'static mut [u8], width: usize, height: usize) -> &LockedRenderer {
    let renderer = RENDERER.get_or_init(move || LockedRenderer::new(framebuffer, width, height));
    renderer
}

pub fn show_splash(renderer: &'static LockedRenderer) {
    gui::splash::render(&renderer);
}
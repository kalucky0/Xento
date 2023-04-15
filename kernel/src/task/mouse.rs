use ps2_mouse::{Mouse, MouseState};
use spinning_top::Spinlock;
use conquer_once::spin::Lazy;
use crate::serial_print;

pub static MOUSE: Lazy<Spinlock<Mouse>> = Lazy::new(|| Spinlock::new(Mouse::new()));

pub fn init() {
    MOUSE.lock().init().unwrap();
    MOUSE.lock().set_on_complete(on_complete);
}

fn on_complete(mouse_state: MouseState) {
    serial_print!("{:?} ", mouse_state);
}

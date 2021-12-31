use super::terminal::parse_command;
use crate::renderer::{LockedRenderer, RENDERER};
use crate::serial_println;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the keyboard interrupt handler
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            serial_println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        serial_println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // fast path
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

fn print_char(r: &LockedRenderer, s: char) {
    let mut renderer = r.lock();
    renderer.write_char(s);
}

fn print_string(r: &LockedRenderer, s: &str) {
    let mut renderer = r.lock();
    renderer.write_string(s);
}

fn remove_last(r: &LockedRenderer) {
    let mut renderer = r.lock();
    renderer.remove_last(1);
}

fn clear_row(r: &LockedRenderer, cmd: String) {
    let mut renderer = r.lock();
    renderer.remove_last(cmd.len());
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    let mut current_command = String::from("");
    let mut prev_commands = Vec::from([String::from("")]);
    let mut selected_command = 0;

    if let Some(renderer) = RENDERER.get() {
        while let Some(scancode) = scancodes.next().await {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => {
                            if character as u8 == 8 {
                                if current_command.len() > 0 {
                                    remove_last(renderer);
                                    current_command.pop();
                                }
                            } else if character as u8 == 10 {
                                let cmds = current_command.split(';');
                                print_char(renderer, '\n');
                                for cmd in cmds {
                                    let result: &str = &parse_command(cmd.to_string(), renderer);
                                    print_string(renderer, result);
                                }
                                print_string(renderer, "\n > ");
                                prev_commands.insert(0, current_command.clone());
                                current_command = String::from("");
                            } else {
                                current_command += &character.to_string();
                                print_char(renderer, character);
                            }
                        }
                        DecodedKey::RawKey(key) => {
                            if key == KeyCode::ArrowUp && selected_command < prev_commands.len() - 1
                            {
                                if current_command != "" {
                                    selected_command += 1;
                                }
                                clear_row(renderer, current_command);
                                current_command = prev_commands.get(selected_command).unwrap().clone();
                                print_string(renderer, current_command.as_str());
                            } else if key == KeyCode::ArrowDown && selected_command >= 1 {
                                selected_command -= 1;
                                clear_row(renderer, current_command);
                                current_command = prev_commands.get(selected_command).unwrap().clone();
                                print_string(renderer, current_command.as_str());
                            } else {
                            }
                        }
                    }
                }
            }
        }
    }
}

use super::calculator;
use crate::renderer::Color;
use crate::renderer::LockedRenderer;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::{Rgb888, RgbColor},
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use font8x8::UnicodeFonts;
use pc_keyboard::{DecodedKey, KeyCode};
use spinning_top::{RawSpinlock, Spinlock};

struct Variable {
    name: String,
    value: String,
}

pub static TERMINAL: OnceCell<LockedTerminal> = OnceCell::uninit();
pub struct LockedTerminal(Spinlock<Terminal>);

impl LockedTerminal {
    pub fn new(renderer: &'static LockedRenderer) -> Self {
        LockedTerminal(Spinlock::new(Terminal::new(&renderer)))
    }

    pub fn init_events(&'static self) {
        unsafe {
            super::keyboard::LISTENERS.push(Box::new(|key: DecodedKey| {
                let mut terminal = self.0.lock();
                terminal.on_keypress(key);
            }));
        }
    }

    pub fn lock(&self) -> spinning_top::lock_api::MutexGuard<'_, RawSpinlock, Terminal> {
        self.0.lock()
    }

    pub unsafe fn force_unlock(&self) {
        self.0.force_unlock();
    }
}

pub struct Terminal {
    variables: Vec<Variable>,
    buffer: String,
    current_command: String,
    history: Vec<String>,
    history_index: usize,
    renderer: &'static LockedRenderer,
    cursor: usize,
}

impl Terminal {
    pub fn new(r: &'static LockedRenderer) -> Self {
        Self {
            variables: Vec::new(),
            buffer: String::from(" > "),
            current_command: String::new(),
            history: Vec::from([String::from("")]),
            history_index: 0,
            renderer: r,
            cursor: 0,
        }
    }

    fn on_keypress(&mut self, key: DecodedKey) {
        let mut renderer = self.renderer.lock();
        renderer.clear();
        unsafe { self.renderer.force_unlock(); }

        match key {
            DecodedKey::Unicode(character) => {
                if character as u8 == 8 {
                    if self.current_command.len() > 0 {
                        self.current_command.pop();
                    }
                } else if character as u8 == 10 {
                    self.buffer.push_str(&self.current_command);
                    self.buffer.push('\n');
                    let command = self.current_command.clone();
                    let cmds = command.split(';');
                    for cmd in cmds {
                        let result = self.parse_command(String::from(cmd));
                        self.buffer.push_str(result.as_str());
                    }
                    self.buffer.push_str("\n > ");
                    self.history.insert(0, self.current_command.clone());
                    self.current_command = String::from("");
                } else {
                    self.current_command += &character.to_string();
                }
            }
            DecodedKey::RawKey(key) => {
                if key == KeyCode::ArrowUp && self.history_index < self.history.len() - 1 {
                    if self.current_command != "" {
                        self.history_index += 1;
                    }
                    self.current_command = self.history.get(self.history_index).unwrap().clone();
                } else if key == KeyCode::ArrowDown && self.history_index >= 1 {
                    self.history_index -= 1;
                    self.current_command = self.history.get(self.history_index).unwrap().clone();
                } else {
                }
            }
        }

        renderer = self.renderer.lock();
        let style = MonoTextStyle::new(&FONT_8X13, Rgb888::WHITE);
        let output = format!("{}{}", self.buffer, self.current_command);

        Text::new(&output, Point::new(0, 0), style)
            .draw(renderer.get())
            .unwrap();
        renderer.update();
    }

    fn parse_command(&mut self, cmd: String) -> String {
        let cmd_ = self.replace_vars(cmd.trim().to_string());
        let parts: Vec<&str> = cmd_.split(" ").collect();

        if cmd.starts_with("$") {
            if cmd.contains("=") {
                return self.set_var(cmd);
            } else if !self.is_command(&cmd_) || cmd_.contains("\"") {
                return self.get_var(cmd);
            }
        }

        match parts[0] {
            "echo" => self.offset(&self.echo(&parts)),
            "chars" => self.offset(&self.get_chars()),
            "ver" => self.offset(&self.get_version()),
            "eval" => self.offset(&self.evaluate(&parts)),
            "vars" => self.get_vars(),
            "clear" => self.clear_screen(),
            "color" => self.change_color(&parts),
            "help" => self.help(),
            "title" => self.title(&parts),
            "uptime" => self.uptime(),
            "fill" => self.fill(&parts),
            "rect" => self.rect(&parts),
            _ => self.offset(&self.error("command not found")),
        }
    }

    fn uptime(&self) -> String {
        let uptime = crate::clock::uptime();
        format!("{}", uptime)
    }

    fn title(&self, cmd: &Vec<&str>) -> String {
        let mut r = self.renderer.lock();

        if cmd.len() > 1 {
            let title = cmd[1..].join(" ");

            let style = PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::GREEN)
                .build();

            for i in 0..title.len() {
                let rendered = crate::font::FONTS
                    .get(title.chars().nth(i).unwrap())
                    .expect("character not found in basic font");

                for (_y, byte) in rendered.iter().enumerate() {
                    for (_x, bit) in (0..8).enumerate() {
                        if *byte & (1 << bit) != 0 {
                            Rectangle::new(
                                Point::new((_x * 8 + 8 * 8 * i) as i32, (_y * 8 + 24) as i32),
                                Size::new(8, 8),
                            )
                            .into_styled(style)
                            .draw(r.get())
                            .unwrap();
                        }
                    }
                }
            }
        }

        "\n\n\n\n\n".to_string()
    }

    fn change_color(&self, cmd: &Vec<&str>) -> String {
        if cmd.len() > 3 {
            // l.set_text_color(Color {
            //     r: parts[1].parse::<u8>().unwrap(),
            //     g: parts[2].parse::<u8>().unwrap(),
            //     b: parts[3].parse::<u8>().unwrap(),
            // });
        }
        "".to_string()
    }

    fn fill(&self, cmd: &Vec<&str>) -> String {
        let mut r = self.renderer.lock();

        if cmd.len() > 3 {
            r.fill(Color {
                r: cmd[1].parse::<u8>().unwrap(),
                g: cmd[2].parse::<u8>().unwrap(),
                b: cmd[3].parse::<u8>().unwrap(),
            });
        }
        "".to_string()
    }

    fn rect(&self, cmd: &Vec<&str>) -> String {
        let mut r = self.renderer.lock();

        if cmd.len() > 6 {
            let style = PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::new(
                    cmd[5].parse::<u8>().unwrap(),
                    cmd[6].parse::<u8>().unwrap(),
                    cmd[7].parse::<u8>().unwrap(),
                ))
                .build();

            Rectangle::new(
                Point::new(
                    cmd[1].parse::<i32>().unwrap(),
                    cmd[2].parse::<i32>().unwrap(),
                ),
                Size::new(
                    cmd[3].parse::<u32>().unwrap(),
                    cmd[4].parse::<u32>().unwrap(),
                ),
            )
            .into_styled(style)
            .draw(r.get())
            .unwrap();
        }

        "".to_string()
    }

    fn clear_screen(&mut self) -> String {
        let mut r = self.renderer.lock();
        self.buffer = String::new();
        r.clear();
        "".to_string()
    }

    fn offset(&self, s: &str) -> String {
        let mut _s = String::new();
        _s.push_str("   ");
        _s.push_str(s);
        _s
    }

    fn help(&self) -> String {
        let mut help = String::new();
        help.push_str("   CHARS   displays a list of all available characters\n");
        help.push_str("   CLEAR   clears the screen\n");
        help.push_str("   COLOR   change text color\n");
        help.push_str("   ECHO    displays a message\n");
        help.push_str("   EVAL    evaluates a mathematical expression\n");
        help.push_str("   HELP    displays this message\n");
        help.push_str("   TITLE   displays a message using bigger font\n");
        help.push_str("   VARS    displays currently declared variables\n");
        help.push_str("   VER     prints the version of the os\n");
        help
    }

    fn is_command(&self, cmd: &String) -> bool {
        let commands = Vec::from_iter([
            "echo", "chars", "ver", "eval", "title", "vars", "clear", "help",
            "color",
        ]);
        for c in commands {
            if cmd.contains(c) {
                return true;
            }
        }
        false
    }

    fn replace_vars(&self, cmd: String) -> String {
        let mut new_cmd = cmd.clone();

        for var in self.variables.iter() {
            new_cmd = new_cmd.replace(&var.name, &var.value);
        }
        new_cmd
    }

    pub fn get_vars(&self) -> String {
        let mut vars_str = String::new();
        for var in self.variables.iter() {
            vars_str.push_str("   ");
            vars_str.push_str(&var.name);
            vars_str.push_str("=");
            vars_str.push_str(&var.value);
            vars_str.push_str("\n");
        }
        vars_str
    }

    pub fn set_var(&mut self, cmd: String) -> String {
        let mut parts = cmd.split("=");
        let name = parts.next().unwrap();
        let value = parts.next().unwrap();

        for i in 0..self.variables.len() {
            if self.variables[i].name == name {
                self.variables[i].value = value.to_string();
                return "".to_string();
            }
        }

        self.variables.push(Variable {
            name: name.to_string(),
            value: value.to_string(),
        });

        return "".to_string();
    }

    pub fn get_var(&self, name: String) -> String {
        let mut found = false;
        let mut value = "".to_string();

        for var in self.variables.iter() {
            if var.name == name.to_string() {
                found = true;
                value = "   ".to_string();
                value.push_str(&var.value.clone().replace("\"", ""));
            }
        }

        if !found {
            return self.error("variable not found");
        }

        return value;
    }

    pub fn echo(&self, cmd: &Vec<&str>) -> String {
        if cmd.len() > 1 {
            return cmd[1..].join(" ") + "\n";
        }
        self.error("echo takes at least 1 argument")
    }

    pub fn get_chars(&self) -> String {
        let mut chars: String = String::new();
        for i in 28..256 {
            if i == 10 {
                chars.push(' ');
            } else {
                chars.push(i as u8 as char);
            }
            chars.push(' ');
            if i % 32 == 0 {
                chars.push_str("\n   ");
            }
        }
        chars.push('\n');
        chars
    }

    pub fn evaluate(&self, cmd: &Vec<&str>) -> String {
        if cmd.len() >= 2 {
            let expression: String = cmd[1..].join("");
            return calculator::eval(expression.as_str()).to_string();
        }

        self.error("eval takes at least 1 argument")
    }

    pub fn get_version(&self) -> String {
        "TERAKRAFT 2022.0.1\n".to_string()
    }

    fn error(&self, msg: &str) -> String {
        let mut s = String::new();
        s.push_str("error: ");
        s.push_str(msg);
        s.push('\n');
        s
    }
}

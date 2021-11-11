use super::calculator;
use crate::logger::LockedLogger;
use crate::logger::Color;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use font8x8::UnicodeFonts;

struct Variable {
    name: String,
    value: String,
}

static mut VARS: Vec<Variable> = Vec::new();

pub fn parse_command(cmd: String, logger: &LockedLogger) -> String {
    let cmd_ = replace_vars(cmd.trim().to_string());
    if cmd.starts_with("$") {
        if cmd.contains("=") {
            return set_var(cmd);
        } else if !is_command(&cmd_) || cmd_.contains("\"") {
            return get_var(cmd);
        }
    }
    if cmd_.starts_with("echo") {
        return offset(&echo(&cmd_));
    } else if cmd_.starts_with("chars") {
        return offset(&get_chars());
    } else if cmd_.starts_with("ver") {
        return offset(&get_version());
    } else if cmd_.starts_with("eval") {
        return offset(&evaluate(&cmd_));
    } else if cmd_.starts_with("vars") {
        return get_vars();
    } else if cmd_.starts_with("clear") {
        return clear_screen(logger, &cmd_);
    } else if cmd_.starts_with("color") {
        return change_color(logger, &cmd_);
    } else if cmd_.starts_with("help") {
        return help();
    } else if cmd_.starts_with("title") {
        return title(logger, &cmd_);
    } else if cmd_.starts_with("uptime") {
        return uptime();
    }
    offset(&error("command not found"))
}

fn uptime() -> String {
    let uptime = crate::clock::uptime();
    format!("{}", uptime)
}

fn title(logger: &LockedLogger, cmd: &String) -> String {
    let mut l = logger.lock();
    let parts: Vec<&str> = cmd.split(" ").collect();

    if parts.len() > 1 {
        let title = parts[1..].join(" ");

        for i in 0..title.len() {
            let rendered = crate::font::FONTS
                .get(title.chars().nth(i).unwrap())
                .expect("character not found in basic font");
                l.set_hspace(8*8*i);

            for (_y, byte) in rendered.iter().enumerate() {
                for (_x, bit) in (0..8).enumerate() {
                    if *byte & (1 << bit) == 0 {
                        l.write_char(' ');
                    } else {
                        l.write_char('█');
                    };
                }
                l.add_vspace(8);
                l.set_hspace(8*8*i);
            }
            l.sub_vspace(8*8);
        }
        l.add_vspace(8*8);
    }

    offset("")
}

fn change_color(logger: &LockedLogger, cmd: &str) -> String {
    let mut l = logger.lock();
    let parts: Vec<&str> = cmd.split(" ").collect();

    if parts.len() > 3 {
        l.set_text_color(Color {
            r: parts[1].parse::<u8>().unwrap(),
            g: parts[2].parse::<u8>().unwrap(),
            b: parts[3].parse::<u8>().unwrap(),
        });
    }
    "".to_string()
}

fn clear_screen(logger: &LockedLogger, cmd: &str) -> String {
    let mut l = logger.lock();
    let parts: Vec<&str> = cmd.split(" ").collect();

    if parts.len() > 1 {
        if parts[1] == "true" {
            l.should_clear(true);
        } else {
            l.should_clear(false);
        }
        return "".to_string()
    }
    l.clear();
    "".to_string()
}

fn offset(s: &str) -> String {
    let mut _s = String::new();
    _s.push_str("   ");
    _s.push_str(s);
    _s
}

fn help() -> String {
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

fn is_command(cmd: &String) -> bool {
    let commands = Vec::from_iter([
        "echo", "chars", "ver", "eval", "title", "vars", "clear", "help", "color",
    ]);
    for c in commands {
        if cmd.contains(c) {
            return true;
        }
    }
    false
}

fn replace_vars(cmd: String) -> String {
    let mut new_cmd = cmd.clone();
    let vars_ = unsafe { &VARS };
    for var in vars_.iter() {
        new_cmd = new_cmd.replace(&var.name, &var.value);
    }
    new_cmd
}

pub fn get_vars() -> String {
    let mut vars_str = String::new();
    let vars_ = unsafe { &VARS };
    for var in vars_.iter() {
        vars_str.push_str("   ");
        vars_str.push_str(&var.name);
        vars_str.push_str("=");
        vars_str.push_str(&var.value);
        vars_str.push_str("\n");
    }
    vars_str
}

pub fn set_var(cmd: String) -> String {
    let mut parts = cmd.split("=");
    let name = parts.next().unwrap();
    let value = parts.next().unwrap();

    let vars_ = unsafe { &mut VARS };

    for i in 0..vars_.len() {
        if vars_[i].name == name {
            vars_[i].value = value.to_string();
            return "".to_string();
        }
    }

    vars_.push(Variable {
        name: name.to_string(),
        value: value.to_string(),
    });

    return "".to_string();
}

pub fn get_var(name: String) -> String {
    let mut found = false;
    let mut value = "".to_string();

    let vars_ = unsafe { &mut VARS };

    for var in vars_.iter() {
        if var.name == name.to_string() {
            found = true;
            value = "   ".to_string();
            value.push_str(&var.value.clone().replace("\"", ""));
        }
    }

    if !found {
        return error("variable not found");
    }

    return value;
}

pub fn echo(cmd: &String) -> String {
    let parts: Vec<&str> = cmd.split(" ").collect();

    if parts.len() > 1 {
        return parts[1..].join(" ") + "\n";
    }
    error("echo takes at least 1 argument")
}

pub fn get_chars() -> String {
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

pub fn evaluate(cmd: &String) -> String {
    let parts: Vec<&str> = cmd.split(" ").collect();

    if parts.len() >= 2 {
        let expression: String = parts[1..].join("");
        return calculator::eval(expression.as_str()).to_string();
    }

    error("eval takes at least 1 argument")
}

pub fn get_version() -> String {
    "TERAKRAFT 2021.1\n".to_string()
}

pub fn error(msg: &str) -> String {
    let mut s = String::new();
    s.push_str("error: ");
    s.push_str(msg);
    s.push('\n');
    s
}
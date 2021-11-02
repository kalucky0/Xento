extern crate bfcore;

use super::calculator;
// use crate::{print, println};
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use bfcore::{Input, Interpreter, Output};

struct Variable {
    name: String,
    value: String,
}

static mut VARS: Vec<Variable> = Vec::new();

pub fn parse_command(cmd: String) -> String {
    let cmd_ = replace_vars(cmd.trim().to_string());
    if cmd.starts_with("$") {
        if cmd.contains("=") {
            return set_var(cmd);
        } else if !is_command(&cmd_) || cmd_.contains("\"") {
            return get_var(cmd);
        }
    }
    
    if cmd_.starts_with("echo") {
        return echo(&cmd_);
    } else if cmd_.starts_with("chars") {
        return get_chars();
    } else if cmd_.starts_with("ver") {
        return get_version();
    } else if cmd_.starts_with("eval") {
        return evaluate(&cmd_);
    } else if cmd_.starts_with("bf") {
        return brainfuck(&cmd_);
    }  else if cmd_.starts_with("vars") {
        return get_vars();
    }
    error("command not found")
}

fn is_command(cmd: &String) -> bool {
    let commands = Vec::from(["echo", "chars", "ver", "eval", "bf", "vars"]);
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

    unsafe {
        VARS.push(Variable {
            name: name.to_string(),
            value: value.to_string(),
        });
    }

    return "".to_string();
}

pub fn get_var(name: String) -> String {
    let mut found = false;
    let mut value = "".to_string();

    let vars_ = unsafe { &mut VARS };

    for var in vars_.iter() {
        if var.name == name.to_string() {
            found = true;
            value = var.value.clone().replace("\"", "");
        }
    }

    if !found {
        return error("variable not found");
    }

    return value;
}

pub fn brainfuck(cmd: &String) -> String {
    let parts: Vec<&str> = cmd.split(" ").collect();

    if parts.len() >= 2 {
        let code = parts[1..].join(" ");
        // println!("");
        Interpreter::new(&code.to_string(), &mut In::default(), &mut Out::default()).run();
        return "".to_string();
    }
    // println!("{} args", parts.len());
    error("bf takes at least 1 argument")
}

pub fn echo(cmd: &String) -> String {
    let parts: Vec<&str> = cmd.split(" ").collect();

    if parts.len() >= 2 {
        return parts[1..].join(" ") + "\n";
    }
    error("echo takes at least 1 argument")
}

pub fn get_chars() -> String {
    let mut chars: String = String::new();
    for i in 0..192 {
        if i == 10 {
            chars.push(' ');
        } else {
            chars.push(i as u8 as char);
        }
        chars.push(' ');
        if i % 32 == 0 {
            chars.push('\n');
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
    " TERAKRAFT 2021.1\n".to_string()
}

pub fn error(msg: &str) -> String {
    let mut s = String::new();
    s.push_str("\nerror: ");
    s.push_str(msg);
    s
}

#[derive(Default)]
struct In;
impl Input for In {
    fn input(&mut self) -> char {
        '\0'
    }
}

#[derive(Default)]
struct Out;
impl Output for Out {
    fn output(&mut self, ch: char) {
        // print!("{}", ch);
    }
}

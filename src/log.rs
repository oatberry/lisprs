use std::fmt::Display;

pub const RED: &str = "\x1B[1;31m";
pub const GRN: &str = "\x1B[1;32m";
pub const YEL: &str = "\x1B[1;33m";
pub const GRY: &str = "\x1B[1;30m";
pub const RESET: &str = "\x1B[0m";

#[allow(dead_code)]
pub fn error<S: Display>(msg: S) {
    eprintln!("[lisprs] {}error:{} {}", RED, RESET, msg);
}

#[allow(dead_code)]
pub fn warn<S: Display>(msg: S) {
    eprintln!("[lisprs] {}warning:{} {}", YEL, RESET, msg);
}

#[allow(dead_code)]
pub fn info<S: Display>(msg: S) {
    eprintln!("[lisprs] {}info:{} {}", GRN, RESET, msg);
}

#[allow(dead_code)]
pub fn debug<S: Display>(msg: S) {
    eprintln!("[lisprs] {}DEBUG:{} {}", GRY, RESET, msg);
}

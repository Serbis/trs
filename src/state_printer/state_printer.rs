//! Generic trait for all state printers

use crate::tsafe::TSafe;
use termion::{color, style};
use std::any::Any;
use std::sync::{Mutex, Arc};
use std::io::Write;
use std::io::stdout;

pub const LINE_SIZE: u16 = 100;

pub trait StatePrinter {
    fn add_one_line(&mut self, title: &str, body: &str);
    fn add_progress(&mut self, title: &str, body: &str, bar_title: &str);
    fn set_progress(&mut self, count: f32);
    fn update_bar_title(&mut self, bar_title: &str);
    fn complete_current(&self);
    fn error_current(&self, text: &str);
    fn print_to_current(&self, text: &str);
    fn print_read_request(&self, prompt: &str);
}


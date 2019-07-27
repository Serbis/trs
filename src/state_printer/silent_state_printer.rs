//! State printer used in the silent mode. Only prints messages and errors without any formatting.
//! See default_state_printer for more info about what each method in this struct is doing.

use crate::tsafe::TSafe;
use super::state_printer::StatePrinter;
use std::io::stdout;
use std::io::Write;


pub struct SilentStatePrinter {}

impl SilentStatePrinter {
    /// Create new printer with empty state
    pub fn new() -> SilentStatePrinter {
        SilentStatePrinter {}
    }
}

impl StatePrinter for SilentStatePrinter {
    fn add_one_line(&mut self, title: &str, body: &str) {}
    fn add_progress(&mut self, title: &str, body: &str, bar_title: &str) {}
    fn set_progress(&mut self, count: f32) {}
    fn update_bar_title(&mut self, bar_title: &str) {}
    fn complete_current(&self) {}

    fn error_current(&self, text: &str) {
        println!("ERROR: {}", text);
        stdout().flush();
    }

    fn print_to_current(&self, text: &str) {
        println!("{}", text);
    }

    fn print_read_request(&self, prompt: &str) {
        println!("{}", prompt);
    }
}

//! Simple state with one text label and body. Example of this state type is the EXEC operation.

use crate::tsafe::TSafe;
use super::state_printer::LINE_SIZE;
use termion::{color, style};
use std::any::Any;
use std::sync::{Mutex, Arc};
use std::io::Write;
use std::io::stdout;

pub struct OneLineState {

    /// Text of the title
    title: String,

    /// Text of the body
    body: String,

    /// Count of service symbols in the line
    offset: i32,

    /// Empty space after main block of text
    filler: String
}

impl OneLineState {

    /// Create new state with data where the title arg is the title of state an the body arg is the
    /// body of state
    pub fn new(title: &str, body: &str) -> OneLineState {
        let title_len = title.len() as i32;
        let body_len = body.len() as i32;
        let ad_len = 7;
        let (line_len, _) = (LINE_SIZE, 0);//termion::terminal_size().unwrap();
        let line_len = line_len as i32;
        let filler_len = (line_len - (title_len + body_len + ad_len)) as i32;
        let filler = (0..filler_len).map(|_| " ").collect::<String>();

        OneLineState {
            title: String::from(title),
            body: String::from(body),
            offset: line_len,
            filler
        }
    }

    /// Completes state with error with specified text
    pub fn error(&self, text: &str) {
        println!("  | {}ERROR: {}{}", color::Fg(color::Red), text, style::Reset);
        println!(" ");
        stdout().flush();
    }

    /// Prints message from the script
    pub fn print(&self, text: &str) {
        let text: Vec<char> = text.chars().collect();
        let mut arr = Vec::new();
        let mut counter = 0;

        for i in 0..text.len() {
            arr.push(text[i]);

            if counter >= LINE_SIZE {
                counter = 0;

                let line: String = arr.iter().collect();
                print!("  | -> {}\n", line);
                arr.clear();
            } else {
                counter = counter + 1;
            }
        }

        if arr.len() > 0 {
            let line: String = arr.iter().collect();
            print!("  | -> {}\n", line);
            // print!("  |\n");
        }
        stdout().flush();
    }

    /// Render current state to the STDOUT
    pub fn render(&self) {
        println!("{}⊙ | {} : {}{}", style::Bold, &self.title, self.body, self.filler);
        stdout().flush();
    }
}
//! State with title, body and progress bar. Example of this state type is the SEND FILE operation

use crate::tsafe::TSafe;
use super::state_printer::LINE_SIZE;
use termion::{color, style};
use std::any::Any;
use std::sync::{Mutex, Arc};
use std::io::Write;
use std::io::stdout;

pub struct ProgressState {

    /// Text of the title
    title: String,

    /// Text of the body
    body: String,

    /// Count of service symbols in the first line
    offset: i32,

    /// Empty space after main block of text in the first line
    filler: String,

    /// Text behind progress bar
    bar_title: String,

    /// Current progress in percentage
    progress_size: f32
}

impl ProgressState {

    /// Create new state with data where the title arg is the title of state, the body arg is the
    /// body of stat and bar title is the text behind progress the progress bar
    pub fn new(title: &str, body: &str, bar_title: &str) -> ProgressState {
        let title_len = title.len() as i32;
        let body_len = body.len() as i32;
        let ad_len = 7;
        let (line_len, _) = (LINE_SIZE, 0);//termion::terminal_size().unwrap();
        let line_len = line_len as i32;
        let filler_len = (line_len - (title_len + body_len + ad_len)) as i32;
        let filler = (0..filler_len).map(|_| " ").collect::<String>();

        ProgressState {
            title: String::from(title),
            body: String::from(body),
            offset: line_len,
            filler,
            bar_title: String::from(bar_title),
            progress_size: 0.0
        }
    }

    /// Set value of the progress in percentage
    pub fn set_progress(&mut self, count: f32) {
        let ticks = count / 2.0;

        if self.progress_size != ticks {
            let offset = 52 + self.bar_title.len() as u16;
            print!("{}", termion::cursor::Left(offset));

            for i in 0..50 {
                print!(" ");
            }

            print!("{}", termion::cursor::Left(50));

            for i in 0..ticks as u32 {
                print!("#");
            }

            print!("{}", termion::cursor::Right(offset - ticks as u16));

            self.progress_size = self.progress_size + ticks;
            stdout().flush();
        }
    }

    /// Update text behind the progress bar
    pub fn update_bar_title(&mut self, bar_title: &str) {
        let old_len = self.bar_title.len() as u16;
        print!("{}", termion::cursor::Left(old_len));
        for i in 0..old_len {
            print!(" ");
        }
        print!("{}", termion::cursor::Left(old_len));
        print!("{}", bar_title);

        self.bar_title = String::from(bar_title);
        stdout().flush();
    }

    /// Completes state with error with specified text
    pub fn error(&self, text: &str) {
        println!("  | {}ERROR: {}{}", color::Fg(color::Red), text, style::Reset);
        println!(" ");
        stdout().flush();
    }

    /// Completes state in normal mode (prints new line character)
    pub fn complete(&self) {
        print!("\n");
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
            print!("  |\n");
        }
        stdout().flush();
    }

    /// Render current state to the STDOUT
    pub fn render(&self) {
        println!("{}⊙ | {} : {}{}", style::Bold, &self.title, self.body, self.filler);
        print!("{}  | [                                                  ] {}", style::Bold, &self.bar_title);
        stdout().flush();
    }
}
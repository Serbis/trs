//! Default state printer. Format out as feed of styled sequence of states

use crate::tsafe::TSafe;
use super::state_printer::StatePrinter;
use super::one_line_state::OneLineState;
use super::progress_state::ProgressState;
use termion::{color, style};
use std::any::Any;
use std::sync::{Mutex, Arc};
use std::io::Write;
use std::io::stdout;

pub struct DefaultStatePrinter {
    /// Current state
    current: Option<TSafe<Any + Send>>
}

impl DefaultStatePrinter {

    /// Create new printer with empty state
    pub fn new() -> DefaultStatePrinter {
        DefaultStatePrinter {
            current: Some(tsafe!(OneLineState::new("-", "-")))
        }
    }
}

impl StatePrinter for DefaultStatePrinter {

    /// Replaces the current component with OneLineState
    fn add_one_line(&mut self, title: &str, body: &str) {
        let state = OneLineState::new(title, body);
        state.render();
        self.current = Some(tsafe!(state));
    }

    /// Replaces the current component with ProgressComponent
    fn add_progress(&mut self, title: &str, body: &str, bar_title: &str) {
        let state = ProgressState::new(title, body, bar_title);
        state.render();
        self.current = Some(tsafe!(state));
    }

    /// Changes value of progress bar of the current component. Value is specifies
    /// in percentage. If the current component is not ProgressState, panic will be caused.
    fn set_progress(&mut self, count: f32) {
        let current= self.current.as_ref().unwrap().clone();
        let mut current = current.lock().unwrap();

        match_downcast_mut!(current, {
            s: ProgressState => {
                s.set_progress(count);
            },
            _ => panic!("Current state is not 'progress'")
        });
    }

    /// Changes bar title of the current component if it is ProgressComponent. If the current
    /// component is not ProgressState, panic will be caused.
    fn update_bar_title(&mut self, bar_title: &str) {
        let current= self.current.as_ref().unwrap().clone();
        let mut current = current.lock().unwrap();

        match_downcast_mut!(current, {
            s: ProgressState => {
                s.update_bar_title(bar_title);
            },
            _ => panic!("Current state is not 'progress'")
        });
    }

    /// Completes the current component. What will do this action, depends on type of the current
    /// component
    fn complete_current(&self) {
        let current= self.current.as_ref().unwrap().clone();
        let mut current = current.lock().unwrap();
        match_downcast_ref!(current, {
            s: ProgressState => {
                s.complete()
            },
            _ => panic!("Unsupported print state")
        });
    }

    /// Fail the current component with error. What will do this action, depends on type of the
    /// current component
    fn error_current(&self, text: &str) {
        let current= self.current.as_ref().unwrap().clone();
        let mut current = current.lock().unwrap();
        match_downcast_ref!(current, {
            s: OneLineState => {
                s.error(text)
            },
            s: ProgressState => {
                s.error(text)
            },
            _ => panic!("Unsupported print state")
        });
    }

    /// Prints text  to the current state
    fn print_to_current(&self, text: &str) {
        let current= self.current.as_ref().unwrap().clone();
        let mut current = current.lock().unwrap();
        match_downcast_ref!(current, {
            s: OneLineState => {
                s.print(text)
            },
            s: ProgressState => {
                s.error(text)
            },
            _ => panic!("Unsupported print state")
        });
    }

    /// Prints user input request
    fn print_read_request(&self, prompt: &str) {
        print!("  | <- {}", prompt);
        stdout().flush();
    }
}
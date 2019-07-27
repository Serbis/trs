//! Module for formatted out information of script processing state to STDOUT. He work in the
//! following way. In each time of the work, on screen exist only one state. This state may be
//! modified or replaced by another state. Each state may be outed to an error mode or print
//! messages mode.

pub mod default_state_printer;
pub mod one_line_state;
pub mod progress_state;
pub mod silent_state_printer;
pub mod state_printer;
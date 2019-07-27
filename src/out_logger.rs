//! This module is used for transitive logging of commands and them output to file

use crate::tsafe::TSafe;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct OutLogger {
    fp: TSafe<File>
}

impl OutLogger {

    /// Creates new out logger which will print log to the specified file
    pub fn new(fp: File) -> OutLogger {
        OutLogger {
            fp: tsafe!(fp)
        }
    }

    /// Marks script start
    pub fn start_script(&self, name: &str) {
        let mut fp = self.fp.lock().unwrap();
        write!(&mut fp, "========== START SCRIPT - {} ========== \n", name);
    }

    /// Marks script end
    pub fn end_script(&self, name: &str) {
        let mut fp = self.fp.lock().unwrap();
        write!(&mut fp, "========== END SCRIPT - {} ========== \n", name);
    }

    /// Separates start of a command block
    pub fn start_block(&self) {

    }

    /// Separates end of a command block
    pub fn end_block(&self) {
        let mut fp = self.fp.lock().unwrap();
        write!(&mut fp, "--------------------------------------------------\n");
    }

    /// Logs command execution
    pub fn log_command(&self, cmd: &str) {
        let mut fp = self.fp.lock().unwrap();
        write!(&mut fp, "<< {}\n", cmd);
    }

    /// Logs command output
    pub fn log_out(&self, out: &str) {
        let mut fp = self.fp.lock().unwrap();
        write!(&mut fp, ">> {}\n", out);
    }
}

//! Top level functions all dynamic script data at runtime

use crate::tsafe::TSafe;
use crate::out_logger::OutLogger;
use crate::state_printer::state_printer::StatePrinter;
use super::connection::{Connection, KeyAuthentication, SimpleAuthentication};
use regex::Regex;
use std::marker::PhantomData;
use std::io::prelude::*;
use std::io;
use std::sync::{Arc, Mutex};

/// Size of block of data used with file operations
pub const BLOCK_SIZE: usize = 32700;

pub struct ScriptRuntime {

    /// State printer instance
    state_printer: TSafe<StatePrinter + Send>,

    /// Out logger instance
    out_logger: OutLogger,

    /// Default shell prompt
    default_prompt: Regex,

    /// Connections list
    connections: Vec<TSafe<Connection>>
}

impl  ScriptRuntime {
    pub fn new(state_printer: TSafe<StatePrinter + Send>, out_logger: OutLogger) -> ScriptRuntime {
        ScriptRuntime {
            state_printer,
            out_logger,
            default_prompt: Regex::new("\\$ ").unwrap(),
            connections: Vec::new()
        }
    }

    /// Creates new connection through ssh bridge use user/password authentication method
    pub fn connect_ssh_simple(&mut self, addr: String, user: String, password: String, prompt: Option<String>) -> TSafe<Connection> {
        let prompt = if prompt.is_some() {
            let p = Regex::new(&prompt.unwrap());
            if p.is_ok() {
                Some(p.unwrap())
            } else {
                None
            }
        } else {
            Some(self.default_prompt.clone())
        };

        let atk = Box::new(SimpleAuthentication { user, password });
        let conn = Connection::new(addr, atk, self.state_printer.clone(), self.out_logger.clone(), prompt);
        let conn = tsafe!(conn);
        self.connections.push(conn.clone());

        conn
    }

    /// Creates new connection through ssh bridge use key authentication method
    pub fn connect_ssh_key(&mut self, addr: String, user: String, private_key: String, prompt: Option<String>, passphrase: Option<String>, public_key: Option<String>) -> TSafe<Connection> {
        let prompt = if prompt.is_some() {
            let p = Regex::new(&prompt.unwrap());
            if p.is_ok() {
                Some(p.unwrap())
            } else {
                None
            }
        } else {
            Some(self.default_prompt.clone())
        };

        let atk = Box::new(KeyAuthentication {
            user,
            private_key,
            public_key,
            passphrase
        });
        let conn = Connection::new(addr, atk, self.state_printer.clone(), self.out_logger.clone(), prompt);
        let conn = tsafe!(conn);
        self.connections.push(conn.clone());

        conn
    }

    /// Prints text line to the current state
    pub fn print(&self, text: &str) {
        self.state_printer.lock().unwrap().print_to_current(text);
    }

    /// Requests input from user
    pub fn read(&self, prompt: &str) -> String {
        self.state_printer.lock().unwrap().print_read_request(prompt);
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_line(&mut buffer).unwrap();
        String::from(&buffer[..buffer.len() - 1])
    }

    /// Requests input from user
    pub fn read_pass(&self, prompt: &str) -> String {
        self.state_printer.lock().unwrap().print_read_request(prompt);
        let buffer = rpassword::read_password().unwrap();
        String::from(&buffer[..buffer.len()])
    }

    /// Closes all opened connection. This is the app level destructor of the runtime
    pub fn close_connections(&mut self) {
        while self.connections.len() > 0 {
            self.connections.pop().unwrap().lock().unwrap().close();
        }
    }
}

unsafe impl  Send for ScriptRuntime {}
unsafe impl  Sync for ScriptRuntime {}
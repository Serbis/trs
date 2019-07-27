//! Runtime ssh connection representation

use super::ssh_thread_safe::{ThreadSafeSession, ThreadSafeChannel};
use crate::state_printer::state_printer::StatePrinter;
use crate::tsafe::TSafe;
use crate::out_logger::OutLogger;
use crate::runtime::script_runtime::BLOCK_SIZE;
use ssh2::{Session, Channel};
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::net::TcpStream;
use std::io::prelude::*;
use std::marker::PhantomData;
use std::time::Duration;
use std::net::{AddrParseError, SocketAddr};
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::io::stdout;
use std::fs::File;
use std::path::Path;

enum CoOps {
    Exec1(String, Option<String>, Option<bool>),
    SendFile(String, String),
    SetPrompt(String),
    Close
}

enum CoData {
    Out(String),
    BoolResult(bool),
    Error(String)
}


/// Ssh connection object
pub struct Connection {
    sender: Sender<CoOps>,
    r_receiver: Receiver<CoData>,
    error: TSafe<Option<String>>
}

impl Connection {

    /// Opens connection to to the specified host and authenticates them with user/password method.
    pub fn new(addr: String, user: String, password: String, state_printer: TSafe<StatePrinter + Send>, out_logger: OutLogger, prompt: Option<Regex>)-> Connection {
        let (sender, receiver) = mpsc::channel();
        let (r_sender, r_receiver) = mpsc::channel();
        let mut failed = false;
        let error_o = tsafe!(None);

        let prompt = if prompt.is_some() {
            prompt.unwrap()
        } else {
            *error_o.lock().unwrap() = Some(String::from("Bad optional prompt"));
            failed = true;
            Regex::new("\\$ ").unwrap()
        };


        let error = error_o.clone();

        thread::spawn(move || {
            state_printer.lock().unwrap().add_one_line("CONNECT SSH SIMPLE", &format!("{} {} ******", addr, user));
            let socket_addr: Result<SocketAddr, AddrParseError> = addr.parse();

            if socket_addr.is_err() {
                let err_t = format!("Invalid ip address: {}", socket_addr.err().unwrap());
                state_printer.lock().unwrap().error_current(&err_t);
                Self::err_conn(&state_printer, error, err_t);
                r_sender.send(CoData::BoolResult(false));
                return;
            }
            let socket_addr = socket_addr.unwrap();

            let tcp = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(10));
            if tcp.is_err() {
                let err_t = format!("Tcp connection error: {}", tcp.err().unwrap());
                state_printer.lock().unwrap().error_current(&err_t);
                Self::err_conn(&state_printer,error, err_t);
                r_sender.send(CoData::BoolResult(false));
                return;
            }
            let tcp = tcp.unwrap();

            let session = Session::new();
            if session.is_none() {
                let err_t = format!("Unable to initialize ssh session");
                state_printer.lock().unwrap().error_current(&err_t);
                Self::err_conn(&state_printer,error, err_t);
                r_sender.send(CoData::BoolResult(false));
                return;
            }
            let mut  session = session.unwrap();
            //session.set_timeout(5000);

            let handshake_result = session.handshake(&tcp);
            if handshake_result.is_err() {
                let err_t = format!("Handshake error: {}", handshake_result.err().unwrap());
                state_printer.lock().unwrap().error_current(&err_t);
                Self::err_conn(&state_printer,error, err_t);
                r_sender.send(CoData::BoolResult(false));
                return;
            }

            //session.userauth_pubkey_file()
            let auth_result = session.userauth_password(&user, &password);
            if auth_result.is_err() {
                let err_t = format!("Authentication error: {}", auth_result.err().unwrap());
                state_printer.lock().unwrap().error_current(&err_t);
                Self::err_conn(&state_printer,error, err_t);
                r_sender.send(CoData::BoolResult(false));
                return;
            }

            // Default prompt
            let mut prompt = prompt;

            // Create shell
            let mut shell: Channel = session.channel_session().unwrap();
            shell.request_pty("ansi", None, None);
            shell.request_pty_size(100, 100, None, None);
            shell.shell().unwrap();

            // Read  out to the first prompt
            let out = Self::read_out(&mut shell, &prompt, true, 0);

            // Setup new default prompt
             write!(shell, "PS1=qwerty\n");
             prompt = Regex::new("qwerty").unwrap();
             let out = Self::read_out(&mut shell, &prompt, false, 0);
             let out = Self::read_out(&mut shell, &prompt, false, 0);
             //println!("out = {}", out);

            r_sender.send(CoData::BoolResult(true));

            //println!("out = {}", out);
            stdout().flush();
            loop {

                let action = receiver.recv();
                if action.is_err() {
                    println!("{}", &action.err().unwrap());
                    return
                }

                let action = action.unwrap();
                match action {
                    CoOps::Exec1(cmd, custom_prompt, with_prompt) => {
                        let mut state_printer = state_printer.lock().unwrap();

                        let mut pstr = String::from(&cmd[..]);
                        if custom_prompt.is_some() {
                            pstr.push_str(&format!(" / {}", custom_prompt.as_ref().unwrap()));
                        }
                        if custom_prompt.is_some() {
                            pstr.push_str(" / Hold");
                        }
                        state_printer.add_one_line("EXEC",&pstr);

                        write!(shell, "{}\n", cmd);

                        let with_prompt = {
                            if with_prompt.is_none() {
                                false
                            }  else {
                                with_prompt.unwrap()
                            }
                        };

                        let out = if custom_prompt.is_some() {
                            let cp = custom_prompt.unwrap();
                            let custom_prompt = Regex::new(&cp);
                            if custom_prompt.is_err() {
                                r_sender.send(CoData::Error(format!("Incorrect prompt regexp '{}'", &cp)));
                                state_printer.error_current(&format!("Incorrect prompt regexp '{}'", &cp));
                                continue;
                            }
                            let custom_prompt = custom_prompt.unwrap();

                            Self::read_out(&mut shell, &custom_prompt, with_prompt, cmd.len() + 2)
                        } else {
                            Self::read_out(&mut shell, &prompt, with_prompt, cmd.len() + 2)
                        };

                        r_sender.send(CoData::Out(out));
                    },
                    CoOps::SetPrompt(pattern) => {
                        let mut state_printer = state_printer.lock().unwrap();
                        state_printer.add_one_line("SET PROMPT",&format!("{}", &pattern));
                        let np = Regex::new(&pattern);
                        if np.is_ok() {
                            prompt = np.unwrap();
                            r_sender.send(CoData::BoolResult(true));
                        } else {
                            state_printer.error_current(&format!("Incorrect prompt regexp '{}'", &pattern));
                            r_sender.send(CoData::BoolResult(false));
                        }
                    },
                    CoOps::SendFile(source, dest) => {
                        let mut state_printer = state_printer.lock().unwrap();

                        let file = File::open(&source);
                        if file.is_err() {
                            let error = file.err().unwrap();
                            state_printer.add_one_line("SEND FILE",&format!("{} -> {}", &source, &dest));
                            let err_text = format!("Unable to open source file: {}", error);
                            state_printer.error_current(&err_text);
                            r_sender.send(CoData::Error(err_text));
                            continue;
                        }
                        let mut file = file.unwrap();

                        let f_meta = file.metadata().unwrap();
                        let f_size = f_meta.len();

                        let mut remote_file = session.scp_send(Path::new(&dest),
                                                               0o644, f_size, None);
                        if remote_file.is_err() {
                            let error = remote_file.err().unwrap();
                            state_printer.add_one_line("SEND FILE",&format!("{} -> {}", &source, &dest));
                            let err_text = format!("Unable to open dest file: {}", &error);
                            state_printer.error_current(&err_text);
                            r_sender.send(CoData::Error(err_text));
                            continue;
                        }
                        let mut remote_file = remote_file.unwrap();



                        let p_size = (f_size as f64 / BLOCK_SIZE as f64).ceil();

                        let mut buf = [0; BLOCK_SIZE];

                        state_printer.add_progress("SEND FILE",&format!("{} -> {}", &source, &dest), &format!("{}/0", f_size));

                        for i in 0..(p_size as u64) {
                            let read = file.read(&mut buf);
                            if read.is_err() {
                                let error = read.err().unwrap();
                                let err_text = format!("Unable to read source file: {}", &error);
                                state_printer.error_current(&err_text);
                                state_printer.complete_current();
                                r_sender.send(CoData::Error(err_text));
                                continue;
                            }
                            let read = read.unwrap();
                            let writed = remote_file.write(&buf[0..read]);
                            if writed.is_err() {
                                let error = writed.err().unwrap();
                                let err_text = format!("Unable to write dest file: {}", &error);
                                state_printer.error_current(&err_text);
                                state_printer.complete_current();
                                r_sender.send(CoData::Error(err_text));
                            }

                            remote_file.flush();
                            state_printer.update_bar_title(&format!("{}/{}", f_size, BLOCK_SIZE * i as usize + read));
                            let percent = (i as f32 / (p_size as f32 / 100.0));
                            state_printer.set_progress(percent);
                        }

                        state_printer.set_progress(100.0);
                        state_printer.complete_current();
                        r_sender.send(CoData::BoolResult(true));
                    },
                    CoOps::Close => return
                }
            }
        });

        r_receiver.recv();

        Connection {
            sender,
            r_receiver,
            error: error_o
        }
    }

    /// Internal API - sets error of the connection
    fn err_conn(state_printer: &TSafe<StatePrinter + Send>, err: TSafe<Option<String>>, err_t: String) {
        state_printer.lock().unwrap().error_current(&err_t);
        let mut  err = err.lock().unwrap();
        *err = Some(err_t);
    }

    /// Internal API - reads out from the shell
    fn read_out(shell: &mut Channel, prompt: &Regex, with_prompt: bool, fs: usize) -> String {
        // Read output to the first prompt
        let mut out = String::new();
        let mut inter_buf = Vec::new();
        loop {
            let mut buf = vec![0; 1];
            let read_result = shell.read_exact(&mut buf).unwrap();
            inter_buf.push(buf[0]);
            let utf8 = std::str::from_utf8(&buf);

            if utf8.is_ok() {
                out.push_str(&utf8.unwrap());
                let pf = prompt.find(&out);
                if pf.is_some() {
                   // println!("out_int = {}", &out);
                    if !with_prompt {
                        let pf = pf.unwrap();
                        let len = pf.end() - pf.start();
                        out = String::from(&out[fs..out.len() - len])
                    } else {
                        out = String::from(&out[fs..out.len()])
                    }
                    break;
                }

                inter_buf.clear();
            }
        }

        out
    }

    /// Execute shell command on the remote server. Returns error flag and output of the executed
    /// command.
    pub fn exec(&mut self, cmd: String, prompt: Option<String>, with_prompt: Option<bool>) -> (bool, String) {
        let sr = self.sender.send(CoOps::Exec1(cmd, prompt, with_prompt));

        if sr.is_err() {
            {
                let err = self.error.lock().unwrap();
                if err.is_some() {
                    return (true, err.as_ref().unwrap().clone())
                } else {
                    return (true, String::from("Unknown error"))
                }
            }
        }

        let r = self.r_receiver.recv();
        if r.is_ok() {
            match r.unwrap() {
                CoData::Out(out) => return (false, out),
                CoData::Error(err) => return (true, err),
                _ => {
                    panic!()
                }
            }
        } else {
            let err = self.error.lock().unwrap();
            if err.is_some() {
                return (true, err.as_ref().unwrap().clone())
            } else {
                return (true, String::from("Unknown error"))
            }
        }

    }

    /// Sends file to the remote server. Returns error flag and error text of error if it was o
    /// occurs
    pub fn send_file(&mut self, source: String, dest: String) -> (bool, String) {
        let sr = self.sender.send(CoOps::SendFile(source, dest));

        if sr.is_err() {
            {
                let err = self.error.lock().unwrap();
                if err.is_some() {
                    return (true, err.as_ref().unwrap().clone())
                } else {
                    return (true, String::from("Unknown error"))
                }
            }
        }

        let r = self.r_receiver.recv();
        if r.is_ok() {
            match r.unwrap() {
                CoData::BoolResult(r) => return (true, String::new()),
                CoData::Error(err) => return (true, err),
                _ => {
                    panic!()
                }
            }
        } else {
            let err = self.error.lock().unwrap();
            if err.is_some() {
                return (true, err.as_ref().unwrap().clone())
            } else {
                return (true, String::from("Unknown error"))
            }
        }
    }

    /// Setup new prompt for connection. Pattern must be a valid rust regexp. If pattern is
    /// correct, true will be returned, instead false
    pub fn set_prompt(&self, pattern: String) -> bool {
        self.sender.send(CoOps::SetPrompt(pattern));

        match self.r_receiver.recv().unwrap() {
            CoData::BoolResult(r) => return r,
            _ => {
                panic!()
            }
        }
    }

    /// Returns connection error
    pub fn get_error(&self) -> Option<String> {
        self.error.lock().unwrap().clone()
    }

    /// Closes connection
    pub fn close(&mut self) {
        self.sender.send(CoOps::Close);
    }
}
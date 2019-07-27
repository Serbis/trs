#[macro_use] extern crate match_downcast;

#[macro_use] pub mod tsafe;
mod script_executor;
mod state_printer;
mod out_logger;
mod runtime;

use tsafe::TSafe;
use state_printer::silent_state_printer::SilentStatePrinter;
use state_printer::default_state_printer::DefaultStatePrinter;
use state_printer::state_printer::StatePrinter;
use script_executor::ScriptConfig;
use out_logger::OutLogger;
use std::env;
use std::fs;
use std::path::Path;
use clap::{Arg, App, SubCommand};
use std::sync::{Mutex, Arc};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::collections::vec_deque::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::io::prelude;

fn main() {
    let mut args: VecDeque<String> = std::env::args().collect();

    let mut fin_args: VecDeque<String> = VecDeque::new();


    let pts = if args.len() >= 2 {
        if args.get(1).unwrap().starts_with("-x") {
            if args.len() >= 3 {
                let sp = String::from(&args.get(2).unwrap()[..]);
                let args_n = String::from(&args.get(1).unwrap()[..]);
                let args_n: Vec<&str> = args_n.split_whitespace().collect();
                args.pop_front();
                args.pop_front();
                args.pop_front();

                fin_args = args_n.iter().map(|v| String::from(*v)).collect();
                while args.len() > 0 {
                    let v = args.pop_front().unwrap();
                    fin_args.push_back(v);
                }
                Some(sp)
            } else {
                return;
            }
        } else {
            fin_args = args;
            None
        }
    } else {
        None
    };

    //println!("{:?}", env::current_dir().unwrap());
    let mut matches = App::new("TRS - ssh extension for the lua script engine")
        .arg(Arg::with_name("x")
            .short("x")
            .help("Starts script in shebang mode. In this mode, program assumes that it runs by the system loader after you calls ./you_script.lua which contains shebang link to the trs. This argumenet is positional, it must always be placed before any other arguments.")
            .required(false))
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("Input script file. This option is not used when program starts in shebang mode.")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("log_file")
            .short("l")
            .long("log")
            .value_name("FILE")
            .help("Sets path to the shell log file")
            .default_value("./.trs.log")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("libs")
            .long("libs")
            .value_name("PATHS")
            .help("Sets lua libraries paths. Each element must match to the lua semantic. Elements separated by the char ';'  . Example of path string '/tmp/libs/?.lua;/usr/share/lua/libs?.lua' .")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("silent")
            .long("silent")
            .help("Activates silent mode. In this mode default logging system is disabled. No addition info prints to the stdout except the script itself outputs")
            .required(false))
        .arg(Arg::from_usage("[arg0] 'optional script argument'"))
        .arg(Arg::from_usage("[arg1] 'optional script argument'"))
        .arg(Arg::from_usage("[arg2] 'optional script argument'"))
        .arg(Arg::from_usage("[arg3] 'optional script argument'"))
        .arg(Arg::from_usage("[arg4] 'optional script argument'"))
        .arg(Arg::from_usage("[arg5] 'optional script argument'"))
        .arg(Arg::from_usage("[arg6] 'optional script argument'"))
        .arg(Arg::from_usage("[arg7] 'optional script argument'"))
        .arg(Arg::from_usage("[arg8] 'optional script argument'"))
        .arg(Arg::from_usage("[arg9] 'optional script argument'"))
        .arg(Arg::from_usage("[arg10] 'optional script argument'"))
        .arg(Arg::from_usage("[arg11] 'optional script argument'"))
        .arg(Arg::from_usage("[arg12] 'optional script argument'"))
        .arg(Arg::from_usage("[arg13] 'optional script argument'"))
        .arg(Arg::from_usage("[arg14] 'optional script argument'"))
        .arg(Arg::from_usage("[arg15] 'optional script argument'"))
        .arg(Arg::from_usage("[arg16] 'optional script argument'"))
        .arg(Arg::from_usage("[arg17] 'optional script argument'"))
        .arg(Arg::from_usage("[arg18] 'optional script argument'"))
        .arg(Arg::from_usage("[arg19] 'optional script argument'"))
        .get_matches_from(fin_args);


    // Read script args
    let mut script_args = VecDeque::new();

    for i in 0..19 {
        let arg =  matches.value_of(format!("arg{}", i));
        if arg.is_some() {
            script_args.push_back(String::from(arg.unwrap()));
        }
    }

    // Read script code
    let file_path = if pts.is_some() {
        pts.unwrap()
    } else {
        let fp = matches.value_of("file");
        if fp.is_some() {
            String::from(fp.unwrap())
        } else {
            println!("In normal mode must have --file argument");
            return;
        }
    };


    let file = File::open(&file_path);
    let mut script = String::new();
    if file.is_err() {
        println!("Unable to load script file '{}', error '{}'", file_path, file.err().unwrap());
        return;
    }
    let file = file.unwrap();
    let mut script = String::with_capacity(file.metadata().unwrap().len() as usize + 2000);

    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        if !line.starts_with("#!") {
            script.push_str(&line);
            script.push_str("\n");
        }
    }

    // Prepare log file
    let log_file_path = matches.value_of("log_file").unwrap();
    let mut log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(log_file_path);
    if log_file.is_err() {
        println!("Unable to open log file '{}', error '{}'", log_file_path, log_file.err().unwrap());
        return;
    }
    let log_file = log_file.unwrap();

    //Read path to libs
    let libs_paths = matches.value_of("libs");
    let libs = if libs_paths.is_some() {
        Some(String::from(libs_paths.unwrap()))
    } else {
        None
    };


    // Prepare enlivenment and run script
    let out_logger = OutLogger::new(log_file);
    let state_printer: TSafe<StatePrinter + Send> = if matches.index_of("silent").is_none() {
        tsafe!(DefaultStatePrinter::new())
    } else {
        tsafe!(SilentStatePrinter::new())
    };

    let cfg = ScriptConfig {
        script,
        state_printer,
        out_logger: out_logger.clone(),
        args: script_args,
        libs
    };

    out_logger.start_script(&file_path);
    script_executor::execute(cfg);
    out_logger.end_script(&file_path);
}
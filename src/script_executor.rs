//! This module responsible for performs all actions around LUA virtual machine. He is the core
//! of execution of a separate script.

use crate::runtime::script_runtime::ScriptRuntime;
use crate::runtime::connection::Connection;
use crate::state_printer::state_printer::StatePrinter;
use crate::out_logger::OutLogger;
use crate::tsafe::TSafe;
use std::collections::vec_deque::VecDeque;
use rlua::{Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Variadic, Table, Value};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct ScriptConfig {

    /// Script code
    pub script: String,

    /// State printer
    pub state_printer: TSafe<StatePrinter + Send>,

    /// Commands logger
    pub out_logger: OutLogger,

    /// Script arguments
    pub args: VecDeque<String>,

    /// Additional modules path
    pub libs: Option<String>
}

/// Run execution of the specified script text. This function do all actions needed for run script
/// in the LUA vm.
pub fn execute<'script>(cfg: ScriptConfig) {

    // Create lua vm instance
    let lua = Lua::new();

    lua.context(|lua_ctx| {

        // Create script application runtime
        let runtimer = tsafe!(ScriptRuntime::new(cfg.state_printer.clone(), cfg.out_logger.clone()));

        // Extract global context of vm
        let globals = lua_ctx.globals();

        // Bind args
        let args = lua_ctx.create_sequence_from(cfg.args).unwrap();
        globals.set("args", args);

        // Bind current_dir
        let cd = std::env::current_dir().unwrap();
        let cd  = cd.to_str().unwrap();
        let current_dir = lua_ctx.create_string(cd).unwrap();
        globals.set("dir", current_dir);


        // Append module path's
        let package: Table = globals.get("package").unwrap();
        let path: String = package.get("path").unwrap();
        let additions_path = cfg.libs.unwrap_or(String::from(" "));
        package.set("path", format!("{};{};{}", path, "./libs/?.lua", additions_path));


        //-------------------------------------------------------


        // Bind global function 'connect_ssh_simple'
        let mut runtime = runtimer.clone();
        let connect_ssh_simple =
            lua_ctx.create_function(move |_, (addr, user, password, prompt): (String, String, String, Option<String>)| {
                let mut connection = runtime.lock().unwrap().connect_ssh_simple(addr, user, password, prompt);

                Ok(LuaConnection(connection))
            }).unwrap();
        globals.set("connect_ssh_simple", connect_ssh_simple);

        // Bind global function 'connect_ssh_key'
        let mut runtime = runtimer.clone();
        let connect_ssh_key =
            lua_ctx.create_function(move |_, (addr, user, private_key, prompt, passphrase, public_key): (String, String, String, Option<String>, Option<String>, Option<String>)| {
                let mut connection = runtime.lock().unwrap().connect_ssh_key(addr, user, private_key, prompt, passphrase, public_key);

                Ok(LuaConnection(connection))
            }).unwrap();
        globals.set("connect_ssh_key", connect_ssh_key);

        // Bind global function 'print'
        let runtime = runtimer.clone();
        let print =
            lua_ctx.create_function(move |_, (text): (String)| {
                runtime.lock().unwrap().print(&text);

                Ok(())
            }).unwrap();
        globals.set("print", print);

        // Bind global function 'read'
        let runtime = runtimer.clone();
        let read =
            lua_ctx.create_function(move |_, (prompt): (String)| {
                let input = runtime.lock().unwrap().read(&prompt);

                Ok((input))
            }).unwrap();
        globals.set("read", read);

        // Bind global function 'read_pass'
        let runtime = runtimer.clone();
        let read_pass =
            lua_ctx.create_function(move |_, (prompt): (String)| {
                let input = runtimer.lock().unwrap().read_pass(&prompt);

                Ok((input))
            }).unwrap();
        globals.set("read_pass", read_pass);

        // Run script
        let result = lua_ctx.load(&cfg.script)
            .set_name("script").unwrap()
            .exec();

        if result.is_err() {
            cfg.state_printer.lock().unwrap().error_current(&format!("lua execution error -> {}", result.err().unwrap()));
        }

        runtime.lock().unwrap().close_connections();
    });
}

/// Lua representation of the runtime Connection object
struct LuaConnection(TSafe<Connection>);

impl UserData for LuaConnection {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("exec", |lua_ctx, mut s, (cmd, prompt, with_prompt): (String, Option<String>, Option<bool>)| {
            let r = s.0.lock().unwrap().exec(cmd, prompt, with_prompt);

            let t = lua_ctx.create_table().unwrap();

            t.set("error", r.0);
            t.set("out", r.1);

            Ok(t)
        });

        methods.add_method_mut("send_file", |lua_ctx, mut s, (source, dest): (String, String)| {
            let r = s.0.lock().unwrap().send_file(source, dest);

            let t = lua_ctx.create_table().unwrap();
            t.set("error", r.0);
            t.set("out", r.1);

            Ok(t)
        });

        methods.add_method_mut("set_prompt", |lua_ctx, mut s, (pattern): (String)| {
            let r = s.0.lock().unwrap().set_prompt(pattern);

            Ok(r)
        });

        methods.add_method_mut("is_error", |lua_ctx, mut s, (): ()| {
            let err = s.0.lock().unwrap().get_error();

            Ok(err.is_some())
        });

        methods.add_method_mut("get_error", |lua_ctx, mut s, (): ()| {
            let err = s.0.lock().unwrap().get_error();

            if err.is_some() {
                Ok(Some(err.unwrap()))
            } else {
                Ok(None)
            }
        });
    }
}
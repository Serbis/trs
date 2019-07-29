TRS is the program recognized to eliminate the gap between low level shell scripting and high level and heavy configuration systems. You can think about it as such as shell scripting on steroids. I was take the libssh2 library, LUA script engine and tied them through hight level domain api. This program takes lua script, and permits you to interact with any hosts through ssh -  execute commands, read them out, send files and some other thing. From the program you will get humanized api to work with ssh. From the Lua you will get all power of probably the most popular scripting language in the world. That's all that TRS do.

# List of features

* Various types of ssh authentication (password, key)
* Switchable structural system of logging.
* Transitive logging of all interactions with remote shell
* External Lua libraries. You can attach any lua library to your script and use it through 'require' in your code
* Argument passing
* User input (plain and password)
* Shell prompt switching
* Work in shebang mode

# Building

This program is written on the Rust programming language. To build it you need to install the Rust sdk:

```
curl https://sh.rustup.rs -sSf | sh
```

and build the program:

```
git clone https://github.com/Serbis/trs
cd trs
cargo build --release
```
After that, you find executable file of the program in the 'target/release' directory. Copy it to a convenient place for you, for example to '/usr/bin' or some other common place for binary programs.

# Api, examples and etc.

Api functions list placed in the api.md file. Examples of scripts writes on Lua and used with this program may be found in the examples directory. Help about command line options you can get run trs with -h option.
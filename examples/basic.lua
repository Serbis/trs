#!/usr/bin/trs -x --libs ./data/?.lua --silent

-- How to run this this script. Run it as binary brogram and specify localhost ssh address as the
-- first argument - ./basic.lua 127.0.0.1:22

-- In this file demonstrated the basic models of work with trs. At first line you see shebang line.
-- As you can see, trs run with -x option, wich indicates to the proram, that it will be called
-- from the system loader in the shebang mode. In option --libs indicated path to the example
-- module, which will be used in this script. Option --silent idicate to program to disable default
-- logger, for this example it will be disturb to us.

-- At this line we requires the external lua module. Because we previously indicated the path where
-- it may be found, nothing problem is here does not occurs.
mymodule = require "mymodule"

-- Here we get host address for connection from the script arguments
host = args[1]

-- Now we get username for connection from the user
user = read("Username: ")

-- Getting use password, also from user, but now with password masking
password = read_pass("Password: ")


-- Before we goes next, you must understand the main work model of this scripting language. Unlike
-- other similar programs, which works with shell instance, from which them was started, trs
-- works with ssh connections. This features permits you work not only single shell, but with
-- any count of shells of you need. For connect to a remote host, you need to call the
-- connection constructor function. In this example we use the simple authenication method, and
-- pass the previously intercepted user and password values to the constructor function.
c = connect_ssh_simple(host, user, password)

-- If at connection stage something goes wrong, this situation is will be handled. You may intercept
-- error and do with it some actions. Also, connection errors is always printed to stdout, and you
-- always will know what was happen.
if c:is_error() then
    local error = c:get_error()

    -- Do here something useful with error
    return
end

-- Here we execute cd command in the remote shell. Used global dir variable which allow to us go
-- directly to the script execution directory and after that the data directory.
c:exec("cd " .. dir .. "/data")

-- Executes ls command and print its output
r = c:exec("ls")
print(r.out)

-- In the next block of lines script will be work with interactive program. How it work, you may
-- see in the appropriate script file in the data directory. For work with interactive program,
-- such as previously, exec method is used, but now we pass two addition arguments to it. Second
-- argument idicates text which must be intercepted as last peice of text before we can pass next
-- command to shell. Third argumant indicates that intercepted text must be preserved. In the
-- normal mode, this text is a system prompt, and it alway removed from the output. Bur for this
-- scenario we must save him, else we lost piece of output. After each interception we pass next
-- command to the interactive program, while it doesn't be end. Along the way, output of this
-- program is printed.
r = c:exec("./i_test.sh", "First Question: ", true)
print(r.out)
r = c:exec("1", "Second Question: ", true)
print(r.out)
r = c:exec("2")
print(r.out)

-- You can send files between your local machine and remote. This feature works as scp utility,
-- you specify local source file and remote file and pass data between them. Becuase you work
-- with localhost connection, dir variable may be used for prepate path for both - remote and
-- local files.
c:exec("cd ..")
c:exec("mkdir data2")
c:send_file(dir .. "/data/important_data.dat", dir .. "/data2/important_data.dat")

-- Scp protocol has one significant disadvantage - it always transfer all file entirely despite
-- that some parts of file may already exists on the remote machine. Rsync is lacks this
-- disadvantage, and can be used for partial transeffering of files. Before, need understand
-- some things. Current opened connection is local. This is do for example demonstration
-- purposes. In the real world script, for this part of scenario, this connection will be referes
-- to some remote server. Now we open the new local connection and use it for transfer our data
-- through rsync in the interactive mode.
lc = connect_ssh_simple(host, user, password)
c:exec("cd " .. dir)
c:exec("mkdir data3")
c:exec("cd data")
spl = mymodule:split("127.0.0.1:22", ":")
c:exec("rsync --partial --progress -avzhe 'ssh -p " .. spl[2] .. "' important_data.dat " .. user .. "@" .. spl[1] .. ":" .. dir .. "/data3/", "password: ", true)
c:exec(password)


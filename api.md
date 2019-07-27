Global tables:

**args** - script arguments list. All values has the string type.

**dir** - current absolute work path

-------------------------------------------

Global functions:

**connect_ssh_simple(host: string, user: string, password: string, prompt: string) -> Connection** - Establish connections with remote host uses user/password authentication method. Prompt is the optional field. For understand what this field is do, you must anderstand how the programm works with remote shell. Right afte creating shell on a remote host, he send to the stdout some info text (such as os version, last login date and some others). After that text, he respond with default system prompt. In most count of the systems, the system prompt at the end containts the '$ ' characters. But in some rarely case this is may be not true. Last argument of this function exists for solve this problem. If this arg is specified, default system prompt will be replaced to the specified value. Need to pay attention, that this argument accept not plain text but regular expression. As result of this function call the Connection object will be returned.

**print(text: string)** - Prints text to the out

**read(text: string)** - Reads user input. Passed text will be printed before input request prompt

**read_pass(text: string)** - Reads user input in the password mode. Passed text will be printed before input request prompt

-------------------------------------------

Connection object:

**exec(cmd: string, prompt: string, save_prompt: bool) -> Result** - Executes command in the remote shell. In the cmd argument is indicated the executed command. The last two argument is used in when you work with an interactive program. Prompt argument is used for temorary replace the system prompt to the custom. This operation allows to trs intercept input requests from the interactive program. Last argument used for disable the prompt consumption. In the normal mode, this value is always set to false, which indicates, that handled prompt will be removed from result output. If this parameter is set, prompt will be saved. This opportunity is used for save text handled be the custom prompt when you work in interactive mode. Returns the table with two fields - error and out. Error contains a boolean value, indicate that some error occurs. The out field contains text of occurred error.

**send_file(source: string, dest: string)** -> Result - Sends the file from the local fs to the remote fs through ssh (work as scp). In the first argument specifies local file and in the second  remote. Returns the table with two fields - error and out. Error contains a boolean value, indicate that some error occurs. The out field contains text of occurred error.

**set_prompt(prompt) -> bool** - Setups a new system prompt. I don't know when this need may occurs. But let this method to be, just in case. Prompt argument is a regular expression. Method return boolean value which indicates result of the prompt updating.

**is_error() -> bool** - Checks if connection was created with errors

**get_error() -> string** - Returns an error text if some error early had the place to be
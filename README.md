# Task

Tiny tasks CLI in Rust for unix based systems.
Supports ssh2 to read and write on remote file.

## Usage

<pre>
Usage:
tasks use                     uses or creates new context
tasks ls                      shows the list of tasks
tasks lsa                     shows the list of all tasks from all contexts
tasks lsc                     shows the list of contexts
tasks add "{content}"         creates task based on content string
tasks done {id}               marks one or several tasks (separated by a comma) as done 
tasks rm {id}                 deletes one or several tasks (separated by a comma) based on the id 
tasks rmc {name}              deletes context based on the name
tasks clear                   deletes one or several contexts (separated by a comma) based on the name 

OPTIONS:
-h, --help                    shows help
</pre>

## Storing data file locally

By default, the data file is stored under `/home/{USER}/.local/share/tasks/tasks.json`.
You can modify this behavior by setting you own `ssh_local_path` in the config:

```toml
local_file_path = '/opt/tasks'
```

## Storing data file remotely with SSH

You can use a remote file to store your data file in order to use the same one whatever the device your on.
It supports ssh2.

To do that you simply need to  update the config file:

```toml
ssh_ip = '666.66.66.666:22' 
ssh_username = 'root'
ssh_file_path = 'apps/tasks'
```
`ssh_ip` is The IP of the remote server you want to connect to (you'll need the port which is 22 by default).
`ssh_username` is the user name of the server you want to connect with (which is often `root` by default).
`ssh_file_path` is the location where you want to store the `tasks` data file.

The ssh connection uses ssh-agent to get the right ssh key.
You might need to run this command to add your ssh key to ssh-agent:
```bash
eval `ssh-agent -s` && ssh-add # You can specify the name of the key if you are using a different one
```

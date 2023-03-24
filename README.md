# Tasks

Tiny tasks CLI in Rust.

<pre>
Usage:
tasks use                     uses or creates new context
tasks ls                      shows the list of tasks
tasks lsc                     shows the list of contexts
tasks add "{content}"         creates task based on content string
tasks done {id}               marks one or several tasks (separated by a comma) as done 
tasks rm {id}                 deletes one or several tasks (separated by a comma) based on the id 
tasks rmc {name}              deletes context based on the name
tasks clear                   deletes one or several contexts (separated by a comma) based on the name 

OPTIONS:
-h, --help                    shows help
</pre>

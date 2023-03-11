# Tasks

Tiny tasks CLI in Rust.

<pre>
Usage:
tasks use                     uses or creates new context
tasks ls                      shows the list of tasks
tasks lsc                     shows the list of contexts
tasks add "{{content}}"       creates task based on content string
tasks done {{id}}             marks task as done
tasks rm {{id}}               deletes task based on the id
tasks rmc {{name}}            deletes context based on the name
tasks clear                   clear all tasks for active context

OPTIONS:
-h, --help                    shows help
</pre>

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    pub name: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// uses or creates new context
    Use(TaskArgs),
    /// Shows the list of tasks
    Ls,
    /// Shows the list of contexts
    Lsc,
    /// shows the list of all tasks from all contexts
    Lsa,
    /// Created task based on content string
    Add(TaskArgs),
    /// Marks one or several tasks (separated by a comma) as done
    Done(TaskArgs),
    /// Deletes one or several tasks (separated by a comma) based on the id
    Rm(TaskArgs),
    /// deletes one or several contexts (separated by a comma) based on the name    
    Rmc(TaskArgs),
    /// Clear all tasks for the active context
    Clear,
}

#[derive(Args, Debug)]
pub struct TaskArgs {
    pub name: Option<String>,
}

use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(name = "task")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long = "generate", value_enum)]
    pub generator: Option<Shell>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// uses or creates new context
    Use(TaskArgs),
    /// edits task content (takes id of the task then its new content)
    Up(UpdateArgs),
    /// edits context name (takes id of the new name)
    Upc(UpdateArgs),
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
pub struct UpdateArgs {
    pub id: usize,
    pub name: String,
}

#[derive(Args, Debug)]
pub struct TaskArgs {
    pub name: String,
}

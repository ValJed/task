use clap::{Command, CommandFactory};
use clap_complete::{generate, Generator};

use std::io;

mod args;
mod services;
mod structs;
mod utils;

use structs::{Config, UserConfig};

use services::api as api_service;
use services::file as file_service;

use args::{Cli, Commands};
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    let user_config: UserConfig =
        confy::load("tasks", "config").expect("Error when loading the config file");

    let mut cli_cmd = Cli::command();
    if let Some(generator) = cli.generator {
        print_completions(generator, &mut cli_cmd);
        return;
    }

    let config = Config::new(user_config);

    // if cli.command.is_none() {
    //     list_tasks(&config, false);
    //     return;
    // }

    match &cli.command.unwrap() {
        Commands::Use(cmd) => file_service::use_context(&config, cmd.name.clone()),
        Commands::Up(cmd) => file_service::edit_task(&config, cmd.id.clone(), cmd.name.clone()),
        Commands::Upc(cmd) => file_service::edit_context(&config, cmd.id.clone(), cmd.name.clone()),
        Commands::Add(cmd) => file_service::add_task(&config, cmd.name.clone()),
        Commands::Rm(cmd) => file_service::del_task(&config, cmd.name.clone()),
        Commands::Rmc(cmd) => file_service::del_context(&config, cmd.name.clone()),
        Commands::Ls => file_service::list_tasks(&config, false),
        Commands::Lsa => file_service::list_tasks(&config, true),
        Commands::Lsc => file_service::list_contexts(&config),
        Commands::Done(cmd) => file_service::mark_done(&config, cmd.name.clone()),
        Commands::Clear => file_service::clear_tasks(&config),
        Commands::Migrate => api_service::migrate(&config),
        _ => (),
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

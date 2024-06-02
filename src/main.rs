use clap::{Command, CommandFactory};
use clap_complete::{generate, Generator};

use std::io;

mod args;
mod services;
mod structs;
mod utils;

use services::api::migrate;
use services::api::ApiService;
use services::file::FileService;
use structs::{Config, Service, UserConfig};

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

    if config.api_url.is_empty() {
        run_cmd(&config, cli, &FileService);
    } else {
        run_cmd(&config, cli, &ApiService);
    }
}

fn run_cmd(config: &Config, cli: Cli, data_service: &impl Service) {
    if let None = cli.command {
        data_service.list_tasks(&config, false);
        return;
    }

    match &cli.command.unwrap() {
        Commands::Use(cmd) => data_service.use_context(&config, cmd.name.clone()),
        Commands::Up(cmd) => data_service.edit_task(&config, cmd.id.clone(), cmd.name.clone()),
        Commands::Upc(cmd) => data_service.edit_context(&config, cmd.id.clone(), cmd.name.clone()),
        Commands::Add(cmd) => data_service.add_task(&config, cmd.name.clone()),
        Commands::Rm(cmd) => data_service.del_task(&config, cmd.name.clone()),
        Commands::Rmc(cmd) => data_service.del_context(&config, cmd.name.clone()),
        Commands::Ls => data_service.list_tasks(&config, false),
        Commands::Lsa => data_service.list_tasks(&config, true),
        Commands::Lsc => data_service.list_contexts(&config),
        Commands::Done(cmd) => data_service.mark_done(&config, cmd.name.clone()),
        Commands::Clear => data_service.clear_tasks(&config),
        Commands::Migrate => migrate(&config),
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

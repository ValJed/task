use chrono::Local;
use clap::{Command, CommandFactory};
use clap_complete::{generate, Generator};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Table};
use serde::{Deserialize, Serialize};
use ssh2::{Session, Sftp};
use terminal_size::{terminal_size, Height, Width};

use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::net::TcpStream;
use std::path::Path;
use std::{env, io, vec};

mod args;

use args::{Cli, Commands};
use clap::Parser;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Task {
    id: usize,
    name: String,
    done: bool,
    creation_date: String,
    modification_date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Context {
    id: usize,
    name: String,
    tasks: Vec<Task>,
    active: bool,
}

impl Context {
    fn new(name: &String, size: usize) -> Self {
        Self {
            id: size + 1,
            name: name.to_owned(),
            tasks: vec![],
            active: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct UserConfig {
    ssh_ip: String,
    ssh_username: String,
    ssh_file_path: String,
    local_file_path: String,
    max_line_lengh: usize,
    api_url: String,
    api_key: String,
}

#[derive(Debug)]
struct Config {
    ssh_ip: String,
    ssh_username: String,
    ssh_file_path: String,
    local_file_path: String,
    max_line_lengh: usize,
    api_url: String,
    api_key: String,
    folder_path: String,
}

impl Config {
    fn new(config: UserConfig) -> Self {
        let [file_path, folder_path] = get_file_paths(&config);
        Self {
            ssh_ip: config.ssh_ip,
            ssh_username: config.ssh_username,
            ssh_file_path: config.ssh_file_path,
            local_file_path: file_path,
            max_line_lengh: config.max_line_lengh,
            api_url: config.api_url,
            api_key: config.api_key,
            folder_path: folder_path,
        }
    }
}

impl ::std::default::Default for UserConfig {
    fn default() -> Self {
        Self {
            ssh_ip: "".into(),
            ssh_username: "".into(),
            ssh_file_path: "".into(),
            local_file_path: "".into(),
            max_line_lengh: get_terminal_width(),
            api_url: "".into(),
            api_key: "".into(),
        }
    }
}

const DEFAULT_LINE_LENGTH: usize = 50;
const LAYOUT: usize = 15;
const LINE_LEN_FALLBACK: usize = 10;

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

    if cli.command.is_none() {
        list_tasks(&config, false);
        return;
    }

    match &cli.command.unwrap() {
        Commands::Use(cmd) => use_context(&config, cmd.name.clone()),
        Commands::Up(cmd) => edit_task(&config, cmd.id.clone(), cmd.name.clone()),
        Commands::Upc(cmd) => edit_context(&config, cmd.id.clone(), cmd.name.clone()),
        Commands::Add(cmd) => add_task(&config, cmd.name.clone()),
        Commands::Rm(cmd) => del_task(&config, cmd.name.clone()),
        Commands::Rmc(cmd) => del_context(&config, cmd.name.clone()),
        Commands::Ls => list_tasks(&config, false),
        Commands::Lsa => list_tasks(&config, true),
        Commands::Lsc => list_contexts(&config),
        Commands::Done(cmd) => mark_done(&config, cmd.name.clone()),
        Commands::Clear => clear_tasks(&config),
        _ => (),
    }
}

fn get_file_data(config: &Config) -> Result<(Vec<Context>, usize), &str> {
    let data_res = if config.ssh_ip.is_empty() {
        Ok(get_or_create_data_file(
            &config.local_file_path,
            &config.folder_path,
        ))
    } else {
        get_or_create_data_file_ssh(&config)
    };

    if data_res.is_err() {
        return Err("Error when getting data from file");
    }

    let data = data_res.unwrap();
    let active_index = data.iter().position(|context| context.active == true);

    let cli = Cli::parse();
    println!("cli.command: {:?}", cli.command);
    match cli.command {
        Some(Commands::Use(_)) => {
            let ctx_index = active_index.unwrap();
            Ok((data, ctx_index))
        }
        _ => {
            if active_index.is_none() {
                return Err("No current active context, let's create one using tasks use {{name}}");
            }

            let ctx_index = active_index.unwrap_or(0);
            Ok((data, ctx_index))
        }
    }
}

fn edit_context(config: &Config, id: usize, name: String) {
    match get_file_data(&config) {
        Ok((data, _)) => {
            let active_context = data.iter().find(|ctx| ctx.id == id);

            match active_context {
                Some(_) => {
                    let updated_data: Vec<Context> = data
                        .into_iter()
                        .map(|mut context| {
                            if context.id != id {
                                return context;
                            }

                            context.name = name.clone();
                            return context;
                        })
                        .collect();

                    write_to_file(updated_data, &config)
                }
                None => {
                    println!("No context found with this ID: {}", id);
                }
            }
        }
        Err(err) => {
            println!(err);
        }
    }
}

fn edit_task(config: &Config, id: usize, content: String) {
    match get_file_data(&config) {
        Ok((mut data, index)) => {
            let active_tasks = data[index].tasks.clone();

            data[index].tasks = active_tasks
                .into_iter()
                .map(|mut task| {
                    if id == task.id {
                        task.name = content.clone();
                        return task;
                    }

                    task
                })
                .collect();

            write_to_file(data, &config);
        }
        Err(_) => {
            println!("Error when getting data from file")
        }
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn use_context(config: &Config, name: String) {
    match get_file_data(&config) {
        Ok((mut data, _)) => {
            let exists = data.iter().find(|ctx| ctx.name == name.to_owned());

            if exists.is_none() {
                let new_context = Context::new(&name, data.len());
                data.push(new_context);
            }

            let updated_data = data
                .into_iter()
                .map(|mut ctx| {
                    ctx.active = ctx.name == name.to_owned();

                    ctx
                })
                .collect();

            write_to_file(updated_data, config)
        }
        Err(_) => {
            println!("Error when getting data from file");
        }
    }
}

fn normalize_path(path: &String, starts_with_backslash: bool) -> String {
    if starts_with_backslash && !path.starts_with("/") {
        format!("/{path}")
    } else if !starts_with_backslash && path.starts_with("/") {
        let mut chars = path.chars();
        chars.next();
        chars.collect::<String>()
    } else {
        path.to_owned()
    }
}

fn get_file_paths(config: &UserConfig) -> [String; 2] {
    let folder_path = if config.ssh_ip.is_empty() {
        if config.local_file_path.is_empty() {
            let user = env::var("USER").expect("No user set on this machine");
            format!("/home/{user}/.local/share/tasks")
        } else {
            normalize_path(&config.local_file_path, true)
        }
    } else {
        normalize_path(&config.ssh_file_path, false)
    };

    let file_path = format!("{folder_path}/tasks.json");

    [file_path, folder_path]
}

fn get_sftp(config: &Config) -> Result<Sftp, ()> {
    // Connect to the local SSH server
    let tcp = TcpStream::connect(&config.ssh_ip).expect("TCP connection failed");
    let mut sess = Session::new().expect("Error while creating TCP Session");

    sess.set_tcp_stream(tcp);
    sess.handshake().expect("Error with the TCP connection");

    // Try to authenticate with the first identity in the agent.
    sess.userauth_agent(&config.ssh_username)
        .expect("Error when setting user agent, you might need to add ssh key to ssh-agent");
    if !sess.authenticated() {
        println!("Authentication failed");
        return Err(());
    }

    let sftp = sess.sftp().expect("Sftp subsystem initialization failed");

    Ok(sftp)
}

fn get_remote_path(config: &Config) -> String {
    let sep = if config.ssh_file_path.is_empty() {
        ""
    } else {
        "/"
    };
    let file_path = format!("{}{sep}tasks.json", config.ssh_file_path);

    file_path
}

fn get_or_create_data_file_ssh(config: &Config) -> Result<Vec<Context>, ()> {
    let sftp_res = get_sftp(&config);
    if sftp_res.is_err() {
        return Err(());
    };

    let sftp = sftp_res.unwrap();
    let path_str = get_remote_path(&config);
    let path = Path::new(&path_str);
    let file_res = sftp.open(path);

    match file_res {
        Ok(file) => {
            let reader = BufReader::new(file);
            let contexts: Vec<Context> =
                serde_json::from_reader(reader).expect("Error when extracting data from file");

            Ok(contexts)
        }
        Err(_) => {
            let mut file = sftp.create(path).expect("Error when creating file");

            file.write("[]".as_bytes())
                .expect("Error when writing to file");

            file.close().unwrap();

            Ok(vec![])
        }
    }
}

fn get_or_create_data_file(file: &String, folder: &String) -> Vec<Context> {
    let folder_path = Path::new(folder.as_str());
    let file_path = Path::new(file.as_str());

    if !folder_path.exists() {
        create_dir_all(folder_path).expect("Error when creating folder tasks");
    }

    if !file_path.is_file() {
        let mut file = File::create(file_path).expect("Error when creating file tasks.json");
        file.write("[]".as_bytes())
            .expect("Error when writing to file");

        return Vec::new();
    };

    let file = File::open(file_path).expect("Error when opening file");
    let reader = BufReader::new(file);
    let contexts: Vec<Context> =
        serde_json::from_reader(reader).expect("Error when extracting data from file");

    contexts
}

fn add_task(config: &Config, task: String) {
    match get_file_data(&config) {
        Ok((mut data, index)) => {
            let date = Local::now();

            let task: Task = Task {
                id: data[index].tasks.len() + 1,
                name: task,
                done: false,
                creation_date: date.to_string(),
                modification_date: date.to_string(),
            };

            data[index].tasks.push(task);

            write_to_file(data, &config);
        }
        Err(_) => {
            println!("Error when getting data from file");
        }
    }
}

fn write_to_file(data: Vec<Context>, config: &Config) {
    let json = serde_json::to_string(&data).expect("Error when stringifying data");

    if config.ssh_ip.is_empty() {
        let mut file = File::create(&config.local_file_path).expect("Error when creating file");

        file.write(json.as_bytes())
            .expect("Error when writing to file");

        return;
    }

    let sftp_res = get_sftp(&config);
    if sftp_res.is_err() {
        return;
    }

    let sftp = sftp_res.unwrap();
    let path_str = get_remote_path(&config);
    let path = Path::new(&path_str);

    let mut file = sftp
        .create(path)
        .expect("Impossible to write on remote file");

    file.write(json.as_bytes())
        .expect("Error when writing to file");

    file.close().unwrap();
}

fn del_task(config: &Config, args: String) {
    match get_file_data(&config) {
        Ok((mut data, index)) => {
            let ids = parse_ids(parse_args(&args));
            let mut counter = 0;

            let active_tasks = data[index].tasks.clone();

            data[index].tasks = active_tasks
                .into_iter()
                .filter_map(|mut task| {
                    if ids.contains(&task.id) {
                        return None;
                    }

                    counter += 1;
                    task.id = counter;

                    Some(task)
                })
                .collect();

            write_to_file(data, &config);
        }
        Err(_) => {
            println!("Error when getting data from file");
        }
    }
}

fn list_tasks(config: &Config, all: bool) {
    match get_file_data(&config) {
        Ok((data, index)) => {
            if all {
                for ctx in &data {
                    print_table(&ctx, &config);
                }
            } else {
                print_table(&data[index], &config);
            }
        }
        Err(_) => {
            println!("Error when getting data from file");
        }
    }
}

fn print_table(ctx: &Context, config: &Config) {
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);

    table.set_header(vec![
        Cell::new(""),
        Cell::new(""),
        Cell::new(&break_line(ctx.name.to_owned(), &config.max_line_lengh)),
    ]);

    for task in &ctx.tasks {
        let check = if task.done {
            "[X]".to_string()
        } else {
            "[]".to_string()
        };

        let splitted_line = break_line(task.name.to_owned(), &config.max_line_lengh);

        table.add_row(vec![
            Cell::new(task.id.to_owned()),
            Cell::new(check),
            Cell::new(splitted_line),
        ]);
    }

    if ctx.tasks.len() == 0 {
        table.add_row(vec![
            Cell::new(""),
            Cell::new(""),
            Cell::new(break_line(
                "No tasks, are you lazy or too efficient?".into(),
                &config.max_line_lengh,
            )),
        ]);
    }

    println!("{table}");
}

fn break_line(line: String, max_line_length: &usize) -> String {
    if line.len() < *max_line_length {
        return line;
    }
    let mut position = 0;
    let mut formatted = String::new();

    loop {
        let end = position + max_line_length;
        if end >= line.len() {
            let substring = &line[position..line.len()];
            formatted.push_str(substring);
            break;
        }

        let substring = &line[position..end];
        let space_pos = substring.rfind(' ').unwrap_or(substring.len());
        let space = if space_pos != substring.len() { 1 } else { 0 };
        let mut updated = substring.to_string();
        updated.replace_range(space_pos.., "\n");
        formatted.push_str(updated.as_str());
        position += space_pos + space;
    }

    formatted
}

fn mark_done(config: &Config, args: String) {
    match get_file_data(&config) {
        Ok((mut data, index)) => {
            let ids = parse_ids(parse_args(&args));

            data[index].tasks = data[index]
                .tasks
                .iter()
                .map(|task| {
                    let mut cloned = task.clone();
                    if ids.contains(&cloned.id) {
                        cloned.done = true
                    }

                    cloned.to_owned()
                })
                .collect();

            write_to_file(data, &config);
        }
        Err(_) => {
            println!("Error when getting data from file");
        }
    }
}

fn clear_tasks(config: &Config) {
    match get_file_data(&config) {
        Ok((mut data, index)) => {
            data[index].tasks = vec![];
            write_to_file(data, &config)
        }
        Err(_) => {
            println!("Error when getting data from file")
        }
    }
}

fn del_context(config: &Config, args: String) {
    match get_file_data(&config) {
        Ok((data, _)) => {
            let ctx_names_or_ids = parse_args(&args);

            let mut updated_data: Vec<Context> = data
                .into_iter()
                .enumerate()
                .filter(|(index, ctx)| {
                    let id = (index + 1).to_string();
                    if ctx_names_or_ids.contains(&ctx.name.as_str())
                        || ctx_names_or_ids.contains(&id.as_str())
                    {
                        return false;
                    }

                    true
                })
                .map(|(_, ctx)| ctx)
                .collect();

            let active_ctx = updated_data.iter().find(|ctx| ctx.active);

            if active_ctx.is_none() && !updated_data.get(0).is_none() {
                updated_data[0].active = true;
            }

            write_to_file(updated_data, &config);
        }
        Err(_) => {
            println!("Error when getting data from file");
        }
    }
}

fn list_contexts(config: &Config) {
    match get_file_data(&config) {
        Ok((data, _)) => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS);

            for (i, ctx) in data.iter().enumerate() {
                let active = if ctx.active { "active" } else { "" };
                table.add_row(vec![
                    (i + 1).to_string(),
                    ctx.name.to_owned(),
                    format!("{} tasks", ctx.tasks.len()),
                    active.to_string(),
                ]);
            }

            if data.len() == 0 {
                table.add_row(vec!["Add your first context using: tasks use {{context}}"]);
            }

            println!("{table}");
        }
        Err(_) => {
            println!("Error when getting data from file");
        }
    }
}

fn parse_args(args: &String) -> Vec<&str> {
    args.split(",").collect()
}

fn parse_ids(ids: Vec<&str>) -> Vec<usize> {
    ids.iter()
        .filter_map(|id_str| {
            if id_str.len() == 0 {
                return None;
            }

            let parsed = id_str.parse();
            match parsed {
                Err(_) => {
                    println!("You can only use Ids, this is not: {id_str}");
                    None
                }
                Ok(id) => Some(id),
            }
        })
        .collect()
}

fn get_terminal_width() -> usize {
    let size = terminal_size();
    if let Some((Width(w), Height(_))) = size {
        let width = usize::from(w);
        if width < (LAYOUT + LINE_LEN_FALLBACK) {
            return LINE_LEN_FALLBACK;
        }

        width - LAYOUT
    } else {
        DEFAULT_LINE_LENGTH
    }
}

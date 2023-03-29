use chrono::Local;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Table};
use serde::{Deserialize, Serialize};
use ssh2::{Session, Sftp};
use std::net::TcpStream;

use std::fs::{create_dir, File};
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::{env, vec};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Task {
    id: usize,
    name: String,
    done: bool,
    creation_date: String,
    modification_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
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
struct Config {
    ssh_ip: String,
    ssh_username: String,
    ssh_file_path: String,
    local_file_path: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            ssh_ip: "".into(),
            ssh_username: "".into(),
            ssh_file_path: "".into(),
            local_file_path: "".into(),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut config: Config = confy::load("tasks", "config").unwrap();

    if args.len() < 2 {
        print_help();
        return;
    }

    let [file_path, folder_path] = get_file_paths(&config.local_file_path);
    config.local_file_path = file_path;

    let data_res = if config.ssh_ip.is_empty() {
        Ok(get_or_create_data_file(
            &config.local_file_path,
            folder_path,
        ))
    } else {
        get_or_create_data_file_ssh(&config)
    };

    if data_res.is_err() {
        return;
    }

    let data = data_res.unwrap();
    let active_index = data.iter().position(|context| context.active == true);

    if active_index.is_none() && args[1] != "use" {
        println!("No current active context, let's create one using tasks use {{name}}");
        return;
    };

    let ctx_index = active_index.unwrap();

    match args[1].as_str() {
        "use" => use_context(data, &args[2], &config),
        "add" => add_task(data, &args[2], &config, ctx_index),
        "rm" => del_task(data, &args[2], &config, ctx_index),
        "rmc" => del_context(data, &args[2], &config, ctx_index),
        "ls" => list_tasks(data, ctx_index),
        "lsc" => list_contexts(data),
        "done" => mark_done(data, &args[2], &config, ctx_index),
        "clear" => clear_tasks(data, &config, ctx_index),
        _ => print_help(),
    }
}

fn use_context(mut data: Vec<Context>, name: &String, config: &Config) {
    let exists = data.iter().find(|ctx| ctx.name == name.to_owned());

    if exists.is_none() {
        let new_context = Context::new(name, data.len());
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

fn get_file_paths(local_file_path: &String) -> [String; 2] {
    let user = env::var("USER").expect("No user set on this machine");
    let folder_path = if local_file_path.is_empty() {
        format!("/home/{user}/.local/share/tasks")
    } else {
        local_file_path.to_owned()
    };
    let file_path = format!("{folder_path}/tasks.json");

    [file_path, folder_path]
}

fn get_sftp(config: &Config) -> Result<Sftp, ()> {
    // Connect to the local SSH server
    let tcp = TcpStream::connect(&config.ssh_ip).expect("TCP connection failed");
    let mut sess = Session::new().expect("Erro while creating TCP Session");

    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    // Try to authenticate with the first identity in the agent.
    sess.userauth_agent(&config.ssh_username).unwrap();

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
            let mut file = sftp
                .create(path)
                .expect("Impossible to write on remote file");

            file.write("[]".as_bytes())
                .expect("Error when writing to file");

            file.close().unwrap();

            Ok(vec![])
        }
    }
}

fn get_or_create_data_file(file: &String, folder: String) -> Vec<Context> {
    let folder_path = Path::new(folder.as_str());
    let file_path = Path::new(file.as_str());

    if !folder_path.exists() {
        create_dir(folder_path).expect("Error when creating folder tasks");
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

fn add_task(mut data: Vec<Context>, to_add: &String, config: &Config, index: usize) {
    let date = Local::now();

    let task: Task = Task {
        id: data[index].tasks.len() + 1,
        name: to_add.to_owned(),
        done: false,
        creation_date: date.to_string(),
        modification_date: date.to_string(),
    };

    data[index].tasks.push(task);

    write_to_file(data, &config);
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

fn del_task(mut data: Vec<Context>, args: &String, config: &Config, index: usize) {
    let ids = parse_ids(parse_args(args));
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

fn list_tasks(data: Vec<Context>, index: usize) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);

    table.set_header(vec![Cell::new(&data[index].name)]);

    for task in &data[index].tasks {
        let check = if task.done {
            "[X]".to_string()
        } else {
            "[]".to_string()
        };

        table.add_row(vec![
            Cell::new(task.id.to_owned()),
            Cell::new(check),
            Cell::new(task.name.to_owned()),
        ]);
    }

    if data[index].tasks.len() == 0 {
        table.add_row(vec![Cell::new("Let's chill, or get to work maybe?")]);
    }

    println!("{table}")
}

fn mark_done(mut data: Vec<Context>, args: &String, config: &Config, index: usize) {
    let ids = parse_ids(parse_args(args));

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

fn clear_tasks(mut data: Vec<Context>, config: &Config, index: usize) {
    data[index].tasks = vec![];

    write_to_file(data, &config)
}

fn del_context(data: Vec<Context>, args: &String, config: &Config, index: usize) {
    let ctx_names = parse_args(args);
    let active_deleted = ctx_names.contains(&data[index].name.as_str());

    let mut updated_data: Vec<Context> = data
        .into_iter()
        .filter(|ctx| !ctx_names.contains(&ctx.name.as_str()))
        .collect();

    if active_deleted && !updated_data.get(0).is_none() {
        updated_data[0].active = true;
    }

    write_to_file(updated_data, &config);
}

fn list_contexts(data: Vec<Context>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);

    for ctx in data.iter() {
        let active = if ctx.active { "active" } else { "" };
        table.add_row(vec![
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

fn print_help() {
    println!("Tiny tasks CLI in Rust.\n");
    println!("Usage:");
    println!("  tasks use                         uses or creates new context");
    println!("  tasks ls                          shows the list of tasks");
    println!("  tasks lsc                         shows the list of contexts");
    println!("  tasks add \"{{content}}\"             creates task based on content string");
    println!("  tasks done {{id}}                   marks one or several tasks (separated by a comma) as done");
    println!("  tasks rm {{id}}                     deletes one or several tasks (separated by a comma) based on the id");
    println!("  tasks rmc {{name}}                  deletes one or several contexts (separated by a comma) based on the name");
    println!("  tasks clear                       clear all tasks for active context\n");
    println!("OPTIONS:");
    println!("  -h, --help                        shows help");
}

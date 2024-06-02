use crate::args::{Cli, Commands};
use chrono::Local;
use clap::Parser;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::structs::{Config, Context, Service, Task};
use crate::utils::{
    get_or_create_data_file, get_or_create_data_file_ssh, get_remote_path, get_sftp, parse_args,
    parse_ids, print_tasks,
};

#[derive(Debug)]
pub struct FileService;

impl Service for FileService {
    fn edit_context(&self, config: &Config, id: usize, name: String) {
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
                println!("{}", err);
            }
        }
    }

    fn edit_task(&self, config: &Config, id: usize, content: String) {
        match get_file_data(&config) {
            Ok((mut data, index)) => {
                let active_tasks = data[index].tasks.clone();

                data[index].tasks = active_tasks
                    .into_iter()
                    .map(|mut task| {
                        if id == task.id {
                            task.content = content.clone();
                            return task;
                        }

                        task
                    })
                    .collect();

                write_to_file(data, &config);
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    fn use_context(&self, config: &Config, name: String) {
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
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    fn add_task(&self, config: &Config, task: String) {
        match get_file_data(&config) {
            Ok((mut data, index)) => {
                let date = Local::now();

                let task: Task = Task {
                    id: data[index].tasks.len() + 1,
                    content: task,
                    done: false,
                    creation_date: date.to_string(),
                    modification_date: date.to_string(),
                };

                data[index].tasks.push(task);

                write_to_file(data, &config);
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    fn del_task(&self, config: &Config, args: String) {
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
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    fn list_tasks(&self, config: &Config, all: bool) {
        match get_file_data(&config) {
            Ok((data, index)) => {
                if all {
                    for ctx in &data {
                        print_tasks(&config, &ctx);
                    }
                } else {
                    print_tasks(&config, &data[index]);
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    fn mark_done(&self, config: &Config, args: String) {
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
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    fn clear_tasks(&self, config: &Config) {
        match get_file_data(&config) {
            Ok((mut data, index)) => {
                data[index].tasks = vec![];
                write_to_file(data, &config)
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    fn del_context(&self, config: &Config, args: String) {
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
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    fn list_contexts(&self, config: &Config) {
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
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}

fn get_file_data(config: &Config) -> Result<(Vec<Context>, usize), String> {
    let data_res = if config.ssh_ip.is_empty() {
        get_or_create_data_file(&config.local_file_path, &config.folder_path, true)
    } else {
        get_or_create_data_file_ssh(&config, true)
    };

    if data_res.is_err() {
        return Err(data_res.unwrap_err());
    }

    let data = data_res.unwrap();
    let active_index = data.iter().position(|context| context.active == true);

    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Use(_)) => {
            let ctx_index = active_index.unwrap();
            Ok((data, ctx_index))
        }
        _ => {
            if active_index.is_none() {
                return Err(format!(
                    "No current active context, let's create one using task use {{name}}"
                ));
            }

            let ctx_index = active_index.unwrap_or(0);
            Ok((data, ctx_index))
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

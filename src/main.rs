use chrono::Local;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Table};
use serde::{Deserialize, Serialize};

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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let [file_path, folder_path] = get_file_paths();
    let data = get_or_create_data_file(&file_path, folder_path);
    let active_index = data.iter().position(|context| context.active == true);

    if active_index.is_none() && args[1] != "use" {
        println!("No current active context, let's create one using tasks use <name>");
    };

    match args[1].as_str() {
        "use" => use_context(data, &args[2], &file_path),
        "add" => add_task(data, &args[2], &file_path, active_index.unwrap()),
        "del" => del_task(data, &args[2], &file_path, active_index.unwrap()),
        "ls" => list_tasks(data, active_index.unwrap()),
        "done" => mark_done(data, &args[2], &file_path, active_index.unwrap()),
        "clear" => clear_tasks(data, &file_path, active_index.unwrap()),
        _ => print_help(),
    }
}

fn use_context(mut data: Vec<Context>, name: &String, file_path: &String) {
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

    write_to_file(updated_data, file_path)
}

fn get_file_paths() -> [String; 2] {
    let user = env::var("USER").expect("No user set on this machine");
    let folder_path = format!("/home/{user}/.local/share/tasks");
    let file_path = format!("{folder_path}/tasks.json");

    [file_path, folder_path]
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

fn add_task(mut data: Vec<Context>, to_add: &String, file_path: &String, index: usize) {
    let date = Local::now();

    let task: Task = Task {
        id: data[index].tasks.len() + 1,
        name: to_add.to_owned(),
        done: false,
        creation_date: date.to_string(),
        modification_date: date.to_string(),
    };

    data[index].tasks.push(task);

    write_to_file(data, file_path);
}

fn write_to_file(data: Vec<Context>, file_path: &String) {
    let json = serde_json::to_string(&data).expect("Error when stringifying data");
    let mut file = File::create(file_path).expect("Error when creating file");

    file.write(json.as_bytes())
        .expect("Error when writing to file");
}

fn del_task(mut data: Vec<Context>, id_str: &String, file_path: &String, index: usize) {
    let id: usize = id_str.parse().unwrap();
    let mut counter = 0;

    let active_tasks = data[index].tasks.clone();

    data[index].tasks = active_tasks
        .into_iter()
        .filter_map(|mut task| {
            if task.id == id {
                return None;
            }

            counter += 1;
            task.id = counter;

            Some(task)
        })
        .collect();

    write_to_file(data, file_path);
}

fn list_tasks(data: Vec<Context>, index: usize) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);

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

    if data.len() == 0 {
        table.add_row(vec![Cell::new("Let's chill, or get to work maybe?")]);
    }

    println!("{table}")
}

fn mark_done(mut data: Vec<Context>, id_str: &String, file_path: &String, index: usize) {
    let id: usize = id_str.parse().unwrap();

    let task_index = data[index].tasks.iter().position(|t| t.id == id);

    if task_index.is_none() {
        println!("No task with the id {}", id);
        return;
    }

    data[index].tasks[task_index.unwrap()].done = true;

    write_to_file(data, file_path);
}

fn clear_tasks(mut data: Vec<Context>, file_path: &String, index: usize) {
    data[index].tasks = vec![];

    write_to_file(data, file_path)
}

fn print_help() {
    println!("Tiny tasks CLI in Rust.\n");
    println!("Usage:");
    println!("tasks ls                 shows the list of tasks");
    println!("tasks add <content>      creates task based on the passed content string");
    println!("tasks done <id>          marks task as done");
    println!("tasks del <id>           deletes task based on the passed id");
    println!("tasks clear              clear all tasks\n");
    println!("OPTIONS:");
    println!("-h, --help              shows help");
}

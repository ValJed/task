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

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: usize,
    name: String,
    done: bool,
    creation_date: String,
    modification_date: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let [file_path, folder_path] = get_file_paths();
    let data = get_or_create_data_file(&file_path, folder_path);

    match args[1].as_str() {
        "add" => add_task(data, &args[2], &file_path),
        "del" => del_task(data, &args[2], &file_path),
        "ls" => list_tasks(data),
        "done" => mark_done(data, &args[2], &file_path),
        "clear" => write_to_file(vec![], &file_path),
        _ => print_help(),
    }
}

fn get_file_paths() -> [String; 2] {
    let user = env::var("USER").expect("No user set on this machine");
    let folder_path = format!("/home/{user}/.local/share/tasks");
    let file_path = format!("{folder_path}/tasks.json");

    [file_path, folder_path]
}

fn get_or_create_data_file(file: &String, folder: String) -> Vec<Task> {
    let folder_path = Path::new(folder.as_str());
    let file_path = Path::new(file.as_str());

    if !folder_path.exists() {
        create_dir(folder_path).expect("Error when creating folder tasks");
    }

    if !file_path.is_file() {
        let mut file = File::create(file_path).expect("Error when creating file tasks.json");
        file.write("[]".as_bytes())
            .expect("Error when writting to file");

        return Vec::new();
    };

    let file = File::open(file_path).expect("Error when opening file");
    let reader = BufReader::new(file);
    let tasks: Vec<Task> =
        serde_json::from_reader(reader).expect("Error when extracting data from file");

    tasks
}

fn add_task(mut data: Vec<Task>, to_add: &String, file_path: &String) {
    let date = Local::now();

    let task: Task = Task {
        id: data.len() + 1,
        name: to_add.to_owned(),
        done: false,
        creation_date: date.to_string(),
        modification_date: date.to_string(),
    };

    data.push(task);

    write_to_file(data, file_path);
}

fn write_to_file(data: Vec<Task>, file_path: &String) {
    let json = serde_json::to_string(&data).expect("Error when stringifying data");
    let mut file = File::create(file_path).expect("Error when creating file");

    file.write(json.as_bytes())
        .expect("Error when writting to file");
}

fn del_task(data: Vec<Task>, id_str: &String, file_path: &String) {
    let id: usize = id_str.parse().unwrap();
    let mut counter = 0;
    let updated_tasks: Vec<Task> = data
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

    write_to_file(updated_tasks, file_path);
}

fn list_tasks(data: Vec<Task>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);

    for task in &data {
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

fn mark_done(data: Vec<Task>, id_str: &String, file_path: &String) {
    let id: usize = id_str.parse().unwrap();
    let udpated_tasks: Vec<Task> = data
        .into_iter()
        .map(|mut task| {
            if task.id == id {
                task.done = true;
            }

            task
        })
        .collect();

    write_to_file(udpated_tasks, file_path);
}

fn print_help() {
    println!("Usage: [options]\n");
    println!("OPTIONS");
    println!("  add\t\tcreated task based on passed string");
    println!("  done\t\tmarks task as done");
    println!("  del\t\tdeletes task based on passed id");
    println!("  clear\t\tclear all tasks");
}

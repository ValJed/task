use chrono::{DateTime, NaiveDateTime, Local};
use serde::{Serialize, Deserialize};

use std::env;
use std::fs::{create_dir, File};
use std::io::Write;
use std::path::{Path,PathBuf};
use std::io::BufReader;

const COMMANDS: [&str; 4] = ["add", "del", "ls", "done"];
const HELP_SIGNS: [&str; 2] = ["--help", "help"];

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    name: String,
    done: bool,
    creation_date: String,
    modification_date: String
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Explain how to use taks");
    }

    let [file_path, folder_path] = get_file_paths();
    let data = get_or_create_data_file(&file_path, folder_path);

    match args[1].as_str() {
        "add" => add_task(data, &args[2], &file_path),
        _ => {
            println!("no shit");
        }
    }
    //     "del" => del_task(),
    //     "ls" => list_tasks(),
    //     "done" => done(),
    // }

}

fn get_file_paths () -> [String; 2] {
    let user = env::var("USER").expect("No user set on this machine");
    let folder_path = format!("/home/{user}/.local/share/tasks");
    let file_path = format!("{folder_path}/tasks.json");

    [file_path, folder_path]
}

fn get_or_create_data_file (file: &String, folder: String) -> Vec<Task> {
    let folder_path = Path::new(folder.as_str());
    let file_path = Path::new(file.as_str());
    
    if !folder_path.exists() {
        create_dir(folder_path).expect("Error when creating folder tasks");
    }

    if !file_path.is_file() {
        let mut file = File::create(file_path)
            .expect("Error when creating file tasks.json");
        file.write("[]".as_bytes()).expect("Error when writting to file");

        return Vec::new();
    };

    let file = File::open(file_path).expect("Error when opening file");
    let reader = BufReader::new(file);
    let tasks: Vec<Task> = serde_json::from_reader(reader)
        .expect("Error when extracting data from file");

    tasks
}

fn add_task (data: Vec<Task>, to_add: &String, file_path: &String) {
    let date = Local::now();

    let task: Task = Task {
        name: to_add.to_owned(),
        done: false,
        creation_date: date.to_string(),
        modification_date: date.to_string()
    };

    println!("task {:?}", task);
    println!("data {:?}", data);
    println!("file_path {:?}", file_path);
}

// fn del_task () {
//
// }
//
// fn list_tasks () {
//
// }
//
// fn done () {
//
// }
//
// fn clean_task () {}
//
//
// fn help () {
//
// }

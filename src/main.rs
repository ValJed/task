use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};

use std::env;
use std::fs::{create_dir, File};
use std::io::Write;
use std::path::Path;
use std::io::BufReader;

const COMMANDS: [&str; 4] = ["add", "del", "ls", "done"];
const HELP_SIGNS: [&str; 2] = ["--help", "help"];

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    done: bool,
    creation_date: String,
    modification_date: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Explain how to use taks");
    }
    //
    // match args[1] {
    //     "add" => add_task(),
    //     "del" => del_task(),
    //     "ls" => list_tasks(),
    //     "done" => done(),
    // }

    get_or_create_data_file();
}

fn get_or_create_data_file() -> Vec<Task> {
    let user = env::var("USER").expect("No user set on this machine");
    let to_folder = format!("/home/{user}/.local/share/tasks");
    let to_file = format!("{to_folder}/tasks.json");
    let folder_path = Path::new(to_folder.as_str());
    let file_path = Path::new(to_file.as_str());
    
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

// fn add_task () {
//
// }
//
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

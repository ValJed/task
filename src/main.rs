use chrono::{DateTime, NaiveDateTime, Utc};

use std::env;
use std::fs::{create_dir, File};
use std::path::Path;

const COMMANDS: [&str; 4] = ["add", "del", "ls", "done"];

const HELP_SIGNS: [&str; 2] = ["--help", "help"];

struct Task {
    done: bool,
    creation_date: String,
    modification_date: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args.len());

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

    get_or_create_file();
    println!("Hello, world!");
}

fn get_or_create_file() {
    let folder_path = "~/.local/share/tasks";
    let file_path = "~/.local/share/tasks/tasks.json";
    let folder_exists = Path::new(folder_path).exists();
    let file_exists = Path::new(file_path).exists();

    if !folder_exists {
        create_dir(folder_path).expect("Error when creating folder tasks");
    }

    if !file_exists {
        File::create(file_path).expect("Error when creating file tasks.json");
    }

    // File::create(path).expect("Error when creating data file");
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

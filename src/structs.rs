use serde::{Deserialize, Serialize};
use std::env;
use terminal_size::{terminal_size, Height, Width};

const DEFAULT_LINE_LENGTH: usize = 50;
const LAYOUT: usize = 15;
const LINE_LEN_FALLBACK: usize = 10;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub done: bool,
    pub creation_date: String,
    pub modification_date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Context {
    pub id: usize,
    pub name: String,
    pub tasks: Vec<Task>,
    pub active: bool,
}

impl Context {
    pub fn new(name: &String, size: usize) -> Self {
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
pub struct UserConfig {
    pub ssh_ip: String,
    pub ssh_username: String,
    pub ssh_file_path: String,
    pub local_file_path: String,
    pub api_url: String,
    pub api_key: String,
}

#[derive(Debug)]
pub struct Config {
    pub ssh_ip: String,
    pub ssh_username: String,
    pub ssh_file_path: String,
    pub local_file_path: String,
    pub max_line_lengh: usize,
    pub api_url: String,
    pub api_key: String,
    pub folder_path: String,
}

impl Config {
    pub fn new(config: UserConfig) -> Self {
        let [file_path, folder_path] = get_file_paths(&config);
        Self {
            ssh_ip: config.ssh_ip,
            ssh_username: config.ssh_username,
            ssh_file_path: config.ssh_file_path,
            local_file_path: file_path,
            max_line_lengh: get_terminal_width(),
            api_url: config.api_url,
            api_key: config.api_key,
            folder_path,
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
            api_url: "".into(),
            api_key: "".into(),
        }
    }
}

pub fn normalize_path(path: &String, starts_with_backslash: bool) -> String {
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

pub fn get_file_paths(config: &UserConfig) -> [String; 2] {
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

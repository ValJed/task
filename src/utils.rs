use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Table};
use ssh2::{Session, Sftp};
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::net::TcpStream;
use std::path::Path;

use crate::structs::{Config, Context};

pub fn parse_args(args: &String) -> Vec<&str> {
    args.split(",").collect()
}

pub fn parse_ids(ids: Vec<&str>) -> Vec<usize> {
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

pub fn print_table(config: &Config, ctx: &Context) {
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

pub fn get_or_create_data_file<'a>(
    file: &String,
    folder: &String,
    create_file: bool,
) -> Result<Vec<Context>, &'a str> {
    let folder_path = Path::new(folder.as_str());
    let file_path = Path::new(file.as_str());

    if !folder_path.exists() || !file_path.is_file() {
        return Err("No data file found: {{file_path}}");
    }

    if !folder_path.exists() {
        create_dir_all(folder_path).expect("Error when creating folder tasks");
    }

    if !file_path.is_file() {
        let mut file = File::create(file_path).expect("Error when creating file tasks.json");
        file.write("[]".as_bytes())
            .expect("Error when writing to file");

        return Ok(Vec::new());
    };

    let file = File::open(file_path).expect("Error when opening file");
    let reader = BufReader::new(file);
    let contexts: Vec<Context> =
        serde_json::from_reader(reader).expect("Error when extracting data from file");

    Ok(contexts)
}

pub fn get_or_create_data_file_ssh(
    config: &Config,
    create_file: bool,
) -> Result<Vec<Context>, &str> {
    let sftp_res = get_sftp(&config);
    if sftp_res.is_err() {
        return Err("Error when getting SFTP connection");
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
            if !create_file {
                return Err("No file found on remote server: {{config.ssh_ip}} -> {{path_str}}");
            }

            let mut file = sftp.create(path).expect("Error when creating file");
            file.write("[]".as_bytes())
                .expect("Error when writing to file");

            file.close().unwrap();

            Ok(vec![])
        }
    }
}

pub fn get_sftp(config: &Config) -> Result<Sftp, ()> {
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

pub fn get_remote_path(config: &Config) -> String {
    let sep = if config.ssh_file_path.is_empty() {
        ""
    } else {
        "/"
    };
    let file_path = format!("{}{sep}tasks.json", config.ssh_file_path);

    file_path
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

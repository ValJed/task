#[allow(dead_code, unused_variables)]
use crate::structs::{Config, Context, Task};
use crate::utils::{get_or_create_data_file, get_or_create_data_file_ssh};
use reqwest::{Client, Method};

// pub fn migate() {
//    // Migrate from File to API or from API to file
// }

pub fn use_context(config: &Config, name: String) {
    let res = request(config, "context", Method::POST);
}

pub fn add_task(config: &Config, name: String) {}

pub fn edit_context(config: &Config, id: usize, name: String) {}

pub fn edit_task(config: &Config, id: usize, name: String) {}

pub fn del_context(config: &Config, name: String) {}

pub fn del_task(config: &Config, name: String) {}

pub fn list_task(config: &Config, all: bool) {
    let res = request(config, "task", Method::GET);
}

pub fn list_contexts(config: &Config, all: bool) {}

pub fn mark_done(config: &Config, name: String) {}

pub fn clear_tasks(config: &Config, name: String) {}

pub async fn migrate(config: &Config) {
    let data_res = get_file_data(config);

    if data_res.is_err() {
        println!("{}", data_res.unwrap_err());
        return;
    }

    let data = data_res.unwrap();

    let del_ctx = request(config, "context", Method::DELETE);
    del_ctx.send().await;

    for context in data {
        println!("context: {:?}", context);
        // let res = request(config, "context", Method::POST);
    }
}

fn request(config: &Config, slug: &str, method: Method) -> Client {
    let client = Client::new();

    client
        .request(method, &format!("{}/{}", config.api_url, slug))
        .header("Authorization", &config.api_key);

    client
}

fn get_file_data(config: &Config) -> Result<Vec<Context>, String> {
    let data_res = if config.ssh_ip.is_empty() {
        get_or_create_data_file(&config.local_file_path, &config.folder_path, false)
    } else {
        get_or_create_data_file_ssh(&config, false)
    };

    data_res
}

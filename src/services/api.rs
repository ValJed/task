use crate::structs::{
    Config, Context, ContextCountTask, ContextOnly, ContextRequest, Service, Task, TaskRequest,
    TaskRequestFull,
};
use crate::utils::{get_or_create_data_file, get_or_create_data_file_ssh, print_tasks};
#[allow(dead_code, unused_variables)]
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use reqwest::blocking::Client;
use reqwest::{header, Error as ReqwestErr};

#[derive(Debug)]
pub struct ApiService;

impl Service for ApiService {
    fn use_context(&self, config: &Config, name: String) {
        let client = get_client(&config).expect("Error when creating http client");
        let body = ContextRequest {
            name,
            active: false,
            simple_create: false,
        };

        let data: Context = client
            .post(get_url(&config, "context"))
            .json(&body)
            .send()
            .expect("Error when creating context")
            .json()
            .expect("Error when parsing response");

        print_tasks(&config, &data);
    }

    fn add_task(&self, config: &Config, content: String) {
        let client = get_client(&config).expect("Error when creating http client");
        let body = TaskRequest { content };

        let data: Task = client
            .post(get_url(&config, "task"))
            .json(&body)
            .send()
            .expect("Error when creating context")
            .json()
            .expect("Error when parsing response");
        println!("data: {:?}", data);

        // print_tasks(&config, &data);
    }

    fn edit_context(&self, config: &Config, id: usize, name: String) {}

    fn edit_task(&self, config: &Config, id: usize, name: String) {}

    fn del_context(&self, config: &Config, name: String) {}

    fn del_task(&self, config: &Config, name: String) {}

    fn list_tasks(&self, config: &Config, all: bool) {
        let client = get_client(&config).expect("Error when creating http client");

        let slug = if all { "task" } else { "task?active=true" };

        let data: Vec<Context> = client
            .get(get_url(&config, slug))
            .send()
            .expect("Error when fetching contexts")
            .json()
            .expect("Error when parsing response");

        for ctx in &data {
            print_tasks(&config, &ctx);
        }
    }

    fn list_contexts(&self, config: &Config) {
        let client = get_client(&config).expect("Error when creating http client");

        let data: Vec<ContextCountTask> = client
            .get(get_url(&config, "context?count=true"))
            .send()
            .expect("Error when fetching contexts")
            .json()
            .expect("Error when parsing response");

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS);

        for (i, ctx) in data.iter().enumerate() {
            let active = if ctx.active { "active" } else { "" };
            table.add_row(vec![
                (i + 1).to_string(),
                ctx.name.to_owned(),
                format!("{} tasks", ctx.task_count),
                active.to_string(),
            ]);
        }

        if data.len() == 0 {
            table.add_row(vec!["Add your first context using: tasks use {{context}}"]);
        }

        println!("{table}");
    }

    fn mark_done(&self, config: &Config, name: String) {}

    fn clear_tasks(&self, config: &Config) {}
}

pub fn migrate(config: &Config) {
    let data_res = get_file_data(config);

    if data_res.is_err() {
        println!("{}", data_res.unwrap_err());
        return;
    }
    let data = data_res.unwrap();
    let client = get_client(&config).expect("Error when creating http client");

    let deleted_ctx = client.delete(get_url(&config, "context")).send();
    let deleted_tasks = client.delete(get_url(&config, "task")).send();

    if deleted_ctx.is_err() || deleted_tasks.is_err() {
        println!("Error when deleting data from API");
        return;
    }

    for context in data {
        let body_ctx = ContextRequest {
            name: context.name,
            active: context.active,
            simple_create: true,
        };
        let created_ctx = client
            .post(get_url(&config, "context"))
            .json(&body_ctx)
            .send()
            .expect(format!("Error when migrating context {}", body_ctx.name).as_str())
            .json::<ContextOnly>()
            .expect("Error when parsing response");

        let tasks: Vec<TaskRequestFull> = context
            .tasks
            .iter()
            .map(|task| TaskRequestFull {
                content: task.content.clone(),
                context_id: created_ctx.id as i32,
                creation_date: task.creation_date.clone(),
                modification_date: task.modification_date.clone(),
            })
            .collect();

        let _created_tasks = client
            .post(get_url(&config, "task/batch"))
            .json(&tasks)
            .send()
            .expect(format!("Error when migrating tasks for context {}", body_ctx.name).as_str());
    }

    println!("Migration completed");
}

fn get_url(config: &Config, slug: &str) -> String {
    format!("{}/{}", config.api_url, slug)
}

fn get_client(config: &Config) -> Result<Client, ReqwestErr> {
    let mut headers = header::HeaderMap::new();
    let mut api_key = header::HeaderValue::from_str(&config.api_key).unwrap();
    api_key.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, api_key);

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    Ok(client)
}

fn get_file_data(config: &Config) -> Result<Vec<Context>, String> {
    let data_res = if config.ssh_ip.is_empty() {
        get_or_create_data_file(&config.local_file_path, &config.folder_path, false)
    } else {
        get_or_create_data_file_ssh(&config, false)
    };

    data_res
}

use std::{
    collections::HashMap, fs::{self, File}, path, process::{self, Child, Stdio}, sync::{Arc, Mutex}, thread::{self, sleep}, time::Duration
};

mod server;
mod task;

pub struct Task {
    name: String,
    description: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    cwd: String,
    object: Option<Child>,
    retry_on_success: bool,
}

impl Task {
    fn start(self: &mut Task){
        println!("Starting task: {}", self.name);
        println!("Description: {}", self.description);

        // 判断目录是否存在
        if!path::Path::new("logs").exists() {
            println!("Warning: Log dir not exists, creating it: logs");
            if let Err(e) = fs::create_dir("logs") {
                println!("Fatal Error: Failed to create log dir: {}", e);
                return;
            }
            return;
        }

        let filename = format!("logs/{}.log", self.name);
        let stdfile = File::create(
            path::Path::new(filename.as_str())
        ).unwrap();

        if let Ok(path) = fs::canonicalize(&self.cwd){
            let child: Child = process::Command::new("cmd")
                .args(&self.args)
                .envs(&self.env)
                .stdin(Stdio::null())
                .stdout(Stdio::from(stdfile.try_clone().unwrap()))
                .stderr(Stdio::from(stdfile))
                .current_dir(path)
                .spawn().expect("Failed to start task");

            self.object = Some(child);
        }else{
            println!("Fatal Error: Task {} :Working Directory not found: {}", self.name, self.cwd);
        }
    }

    fn stop(self: &mut Task){
        if let Some(mut child) = self.object.take() {
            child.kill().expect("Failed to kill task");
        }
    }
}

fn main() {
    let shared_tasks: Arc<Mutex<Vec<Task>>> = Arc::new(Mutex::new(vec![] as Vec<Task>));
    let shared_tasks_clone = Arc::clone(&shared_tasks);
    
    task::task::parse_all(Arc::clone(&shared_tasks));

    thread::spawn(move || {
        println!("Server is starting...");
        server::server::server(shared_tasks_clone);
    });

    loop {
        task::task::feed(Arc::clone(&shared_tasks));
        sleep(Duration::from_secs(1));
    }
}

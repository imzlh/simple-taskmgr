pub mod task{
    use std::{ collections::HashMap, fs, sync::{Arc, Mutex}};

    use ini::configparser::ini::Ini;

    use crate::Task;

    const TASK_DIR: &str = "./";

    pub fn parse_all(tasks: Arc<Mutex<Vec<Task>>>){
        let files = fs::read_dir(
            fs::canonicalize(TASK_DIR).expect("TASK Dir not found")
        ).expect("Read directory failed");

        for file in files{
            let file_name = file.as_ref().unwrap().file_name();
            if let Ok(_file) = file{
                if file_name.to_str().unwrap().starts_with(".") || 
                    !file_name.to_str().unwrap().ends_with(".ini") {
                    continue;
                }

                let path = _file.path();
                if path.is_file(){
                    println!("Parsing file: {}", path.to_str().unwrap());
                    // 解析ini文件
                    let _config = Ini::new().load(path.to_str().unwrap()).unwrap();
                    let config = _config.get("main").expect("Failed to parse ini file");

                    // 解析任务
                    let mut task = Task{
                        name: config.get("name").cloned()
                            .expect("Failed to get [main.name].Do you define it in the ini file?").unwrap(),
                        description: config.get("description").cloned()
                            .expect("Failed to get [main.description].Do you define it in the ini file?").unwrap(),
                        args: config.get("args").cloned()
                            .expect("Failed to get [main.args].Do you define it in the ini file?").unwrap()
                            .split_whitespace().map(|item| item.to_string()).collect(),
                        cwd: config.get("cwd").cloned()
                            .unwrap_or(Some(".".to_string())).unwrap(),
                        env: match _config.get("env") {
                            Some(env) => env.iter()
                                .map(|(k, v)| (k.clone().to_string(), v.clone().unwrap().to_string())).collect(),
                            None => HashMap::new().try_into().unwrap()
                        },
                        retry_on_success: config.get("retry_on_success").cloned()
                            .unwrap_or(Some("false".to_string())).unwrap() == "true",
                        object: None
                    };

                    // 判断cwd是否存在
                    if!fs::metadata(task.cwd.clone()).is_ok(){
                        println!("WARNING: CWD {} not found, using current directory instead.", task.cwd);
                        task.cwd = ".".to_string();
                    }

                    if config.get("autostart").cloned()
                        .unwrap_or(Some("false".to_string())).unwrap() == "true" {
                        task.start();
                    }

                    tasks.lock().unwrap().push(task);
                }
            }else if let Err(_e) = file{
                println!("Error reading directory file: {:?}", _e);
            }
        }

        if tasks.lock().unwrap().len() == 0 {
            println!("WARNING: No task found.");
        }
    }

    pub fn feed(tasks: Arc<Mutex<Vec<Task>>>){
        for task in tasks.lock().unwrap().iter_mut(){
            if let Some(child) = task.object.as_mut(){
                if let Ok(status) = child.try_wait(){
                    if let Some(code) = status {
                        if code.success(){
                            println!("Task {} exited successfully.", task.name);
                            if task.retry_on_success {
                                task.start()
                            }else{
                                task.object = None;
                            }
                        }else{
                            println!("Task {} exited with code: {}", task.name, code);
                            task.object = None;
                            task.start();
                        }
                    }
                }
            }
        }
    }
}
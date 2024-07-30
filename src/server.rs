pub mod server{
    use std::{io::{BufRead, BufReader, Read, Write}, net::TcpListener, sync::{Arc, Mutex}};
    use crate::Task;

    pub fn get_status(task: &Task) -> String {
        if let Some(obj) = task.object.as_ref(){
            format!("{}\tPID: {}", task.name, obj.id())
        }else {
            format!("{}\tNot running\n", task.name)
        }
    }

    pub fn server(tasks: Arc<Mutex<Vec<Task>>>){
        // create server socket
        let server_socket = TcpListener::bind("0.0.0.0:8080").unwrap();
        println!("Server started on port 8080");

        loop {
            if let Ok((mut stream, addr)) = server_socket.accept(){

                if let Err(_) = stream.write("Welcome to Rust Server\n".as_bytes()){
                    println!("Dead client found: {}", addr);
                    continue;
                }

                println!("Connection from: {}", addr);
                // handle client connection
                
                let mut buf = String::new();
                while let Ok(size) = BufReader::new(&stream).read_line(&mut buf){
                    if size == 0 {
                        break;
                    }

                    let mut generated_task = buf.split_whitespace();
                    if let Some(key) = generated_task.next(){
                        let value = generated_task.next().unwrap_or("");
                        let mut returns: String = String::from("\n");
                        let mut return_buffer: Vec<u8> = vec![];

                        // handle commands
                        match key.to_lowercase().as_str() {
                            "exit" => break,
                            "start" => {
                                let mut tasks_lock = tasks.lock().unwrap();
                                for task in tasks_lock.iter_mut() {
                                    if task.name == value {
                                        if let Some(obj) = task.object.as_ref(){
                                            if obj.id() != 0{
                                                returns = format!("Task {} is already running\n", value);
                                            }else{
                                                task.start();
                                                returns = format!("Task {} started\n", value);
                                            }
                                        }else{
                                            task.start();
                                            returns = format!("Task {} started\n", value);
                                        }
                                        break;
                                    }
                                }
                                if returns.is_empty(){
                                    returns = format!("Task {} not found\n", value);
                                }
                            },
                            "stop" => {
                                let mut tasks_lock = tasks.lock().unwrap();
                                for task in tasks_lock.iter_mut() {
                                    if task.name == value {
                                        if let Some(obj) = task.object.as_ref(){
                                            if obj.id() != 0{
                                                task.stop();
                                                returns = format!("Task {} stopped\n", value);
                                            }else{
                                                returns = format!("Task {} is not running\n", value);
                                            }
                                        }else{
                                            returns = format!("Task {} is not running\n", value);
                                        }
                                        break;
                                    }
                                }
                                if returns.is_empty(){
                                    returns = format!("Task {} not found\n", value);
                                }
                            },
                            "restart" => {
                                let mut tasks_lock = tasks.lock().unwrap();
                                for task in tasks_lock.iter_mut() {
                                    if task.name == value {
                                        if let Some(obj) = task.object.as_ref(){
                                            if obj.id() != 0{
                                                task.stop();
                                            }
                                        }
                                        task.start();
                                        returns = format!("Task {} restarted\n", value);
                                        break;
                                    }
                                }
                                if returns.is_empty(){
                                    returns = format!("Task {} not found\n", value);
                                }
                            },
                            "log" => {
                                let mut tasks_lock = tasks.lock().unwrap();
                                for task in tasks_lock.iter_mut() {
                                    if task.name == value {
                                        let log_file = format!("logs/{}.log", task.name);
                                        // 读取log_file
                                        let mut file = match std::fs::File::open(&log_file){
                                            Ok(file) => file,
                                            Err(_) => {
                                                returns = format!("Log file {} not found\n", log_file);
                                                break;
                                            }
                                        };
                                        if let Err(_) = file.read_to_end(&mut return_buffer){
                                            returns = format!("Error reading log file {}\n", log_file);
                                        }
                                        break;
                                    }
                                }
                                if returns.is_empty(){
                                    returns = format!("Task {} not found\n", value);
                                }
                            },
                            "status" => returns = {
                                let mut result: String = format!("There are {} tasks running:\n", tasks.lock().unwrap().len());
                                // check if task exists
                                    // send specific task
                                    for task in tasks.lock().unwrap().iter() {
                                        if task.name == value || value == "" {
                                            result += get_status(&task).as_str();
                                        }
                                    }
                                result + "\n====== END =======\n"
                            },
                            _ => returns = format!("Invalid command: {}\n", key)
                        }
                        
                        if let Err(_) = stream.write(return_buffer.as_slice()){
                            break;
                        }

                        if let Err(_) = stream.write(returns.as_bytes()){
                            break;
                        }

                        if let Err(_) = stream.flush(){
                            break;
                        }

                        buf = String::new();
                    }else{
                        continue;
                    }
                }

            }
        }
    }
}
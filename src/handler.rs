use std::fs::{OpenOptions, File};
use pathsearch::find_executable_in_path;
use std::path::{PathBuf, Path};
use std::io::{self, Write};
use std::process::Stdio;
use std::env::{set_current_dir, home_dir};
use std::process::{Command,Child};

pub const SHELL_COMMANDS: [&str; 5] = ["echo", "type", "exit", "cd", "pwd"];

pub fn echo_handler(args: &Vec<String>, redirect: bool, redirects: Vec<(String, String)>) {
    if !redirect {
        for arg in args {
            print!("{} ", arg);
        }
        println!();
    } else {
        let (write_location, write_type) = &redirects[redirects.len() - 1];

        let mut options = OpenOptions::new();
        options.read(true).write(true);

        match write_type.trim() {
            "append_output" => {
                options.truncate(false).append(true);
            }
            "replace_output" => {
                options.truncate(true).append(false);
            }
            _ => {
                for arg in args {
                    print!("{} ", arg);
                }
                println!();
                return;
            }
        }

        let mut file: File;
        let file_result = options.open(write_location);
        match file_result {
            Ok(result) => {
                file = result;
            }
            Err(e) => {
                println!("Sorry errored out {}", e);
                return;
            }
        }

        for arg in args {
            let _ = writeln!(file, "{} ", arg);
        }
    }
}

pub fn type_handler(args: &Vec<String>, redirect: bool, redirects: Vec<(String, String)>) {
    if !redirect {
        if args.is_empty() {
            println!("Not a valid command");
            return;
        }
        for arg in args {
            if SHELL_COMMANDS.contains(&arg.as_str()) {
                println!("{} is a shell builtin", arg);
            } else if let Some(path) = find_executable_in_path(&arg) {
                println!("{} is {}", arg, path.display()); // this is third party way
            } else {
                println!("{}: not found", arg);
            }
        }
    } else {
        let (write_location, write_type) = &redirects[redirects.len() - 1];

        let mut options = OpenOptions::new();
        options.read(true).write(true);

        match write_type.trim() {
            "replace_output" | "replace_error" => {
                options.truncate(true).append(false);
            }
            "append_output" | "append_error" => {
                options.truncate(false).append(true);
            }
            _ => {
                println!("Sorry errored out");
                return;
            }
        }
        let mut file: File;
        let file_result = options.open(write_location);
        match file_result {
            Ok(result) => {
                file = result;
            }
            Err(e) => {
                println!("Sorry errored out {}", e);
                return;
            }
        }

        if args.is_empty() {
            if write_type.trim().contains("error") {
                let _ = writeln!(file, "Not a valid command");
            } else {
                println!("Not a valid command");
            }
            return;
        }
        for arg in args {
            let inbuilt = SHELL_COMMANDS.contains(&arg.as_str());
            let executible = find_executable_in_path(&arg).is_some();

            let path_result = find_executable_in_path(&arg);
            let mut path = PathBuf::new();

            match path_result {
                Some(result) => {
                    path = result;
                }
                None => {

                }
            }

            if inbuilt | executible {
                let msg: String = if inbuilt {
                    format!("{} is a shell builtin", arg)
                } else {
                    format!("{} is {}", arg, path.display())
                };

                if write_type.trim().contains("output") {
                    let _ = writeln!(file, "{}", msg);
                } else {
                    println!("{}", msg);
                }
            } else {
                if write_type.trim().contains("error") {
                    let _ = writeln!(file, "{}: not found", arg);
                } else {
                    println!("{}: not found", arg);
                }
            }
        }
    }
}

pub fn pwd_handler(
    args: &Vec<String>,
    command: &str,
    redirect: bool,
    redirects: Vec<(String, String)>,
) {
    if !redirect {
        if !args.is_empty() {
            println!("{}: Invalid arguments provided", command.trim());
        } else {
            let path_result: Result<std::path::PathBuf, io::Error> = std::env::current_dir();
            match path_result {
                Err(e) => println!("Not Found Error: {}", e),
                Ok(path_buf) => println!("{}", path_buf.display()),
            };
        }
    } else {
        let (write_location, write_type) = &redirects[redirects.len() - 1];

        let mut options = OpenOptions::new();
        options.read(true).write(true);

        match write_type.trim() {
            "replace_output" | "replace_error" => {
                options.truncate(true).append(false);
            }
            "append_output" | "append_error" => {
                options.truncate(false).append(true);
            }
            _ => {
                println!("Sorry errored out");
                return;
            }
        }

        let mut file: File;
        let file_result = options.open(write_location);
        match file_result {
            Ok(result) => {
                file = result;
            }
            Err(e) => {
                println!("Sorry errored out {}", e);
                return;
            }
        }

        if !args.is_empty() {
            let msg = format!("{}: Invalid arguments provided", command.trim());

            if write_type.trim().contains("error") {
                let write = writeln!(file, "{}", msg);
                if write.is_err(){
                    println!("Sorry could not write to file");
                }
            } else {
                println!("{}: Invalid arguments provided", command.trim());
            }
        } else {
            let msg = if let Ok(path) = std::env::current_dir() {
                format!("{}", path.display())
            } else {
                format!("Sorry could not find current directory")
            };
            if std::env::current_dir().is_ok() {
                if write_type.trim().contains("output") {
                    let write = writeln!(file, "{}", msg);
                    if write.is_err(){
                        println!("Sorry could not write to file");
                    }
                } else {
                    println!("{}", msg);
                }
            } else {
                if write_type.trim().contains("error") {
                    let write = writeln!(file, "{}", msg);
                    if write.is_err(){
                        println!("Sorry could not write to file");
                    }
                } else {
                    println!("{}", msg);
                }
            }
        }
    }
}

pub fn cd_handler(args: &Vec<String>, command: &str) {
    if args.len() >= 2 {
        println!("{}: Too many arguments", command.trim());
        return;
    } else {
        match args[0].as_str() {
            "" => {
                let root = Path::new("/");
                let _result = set_current_dir(root);
                return;
            }
            "~" => {
                let path = home_dir().expect("sorry cannot find your home dir");
                let _result = set_current_dir(path);
                return;
            }
            _ => {
                let path = Path::new(&args[0]);
                let is_path_correct = path
                    .try_exists()
                    .expect("Can't check existence of provided file");
                if is_path_correct {
                    let _result = set_current_dir(path);
                } else {
                    println!("{}: {}: No such file or directory", command.trim(), args[0]);
                }
                return;
            }
        }
    }
}

pub fn general_handler(
    args: &Vec<String>,
    command: &str,
    redirect: bool,
    redirects: Vec<(String, String)>,
) {
    if !redirect {
        if let Some(_path) = find_executable_in_path(&command.trim()) {
            let mut process = Command::new(command.trim()).args(args).spawn().unwrap();
            let _status = process.wait().unwrap();
        } else {
            println!("{}: command not found", command.trim());
        }
    } else {
        let (write_location, write_type) = &redirects[redirects.len() - 1];

        let mut options = OpenOptions::new();
        options.read(true).write(true);

        match write_type.trim() {
            "replace_output" | "replace_error" => {
                options.truncate(true).append(false);
            }
            "append_output" | "append_error" => {
                options.truncate(false).append(true);
            }
            _ => {
                println!("Sorry errored out");
                return;
            }
        }

        let mut file: File;
        let file_result = options.open(write_location);
        match file_result {
            Ok(result) => {
                file = result;
            }
            Err(e) => {
                println!("Sorry file error {}",e);
                return;
            }
        }

        if let Some(_path) = find_executable_in_path(&command.trim()) {
            if write_type.trim().contains("output") {
                let mut process = Command::new(command.trim())
                    .args(args)
                    .stdout(Stdio::from(file))
                    .spawn()
                    .unwrap();
                let _status = process.wait().unwrap();
            } else if write_type.trim().contains("error") {
                let mut process = Command::new(command.trim())
                    .args(args)
                    .stderr(Stdio::from(file))
                    .spawn()
                    .unwrap();
                let _status = process.wait().unwrap();
            } else {
                let mut process = Command::new(command.trim()).args(args).spawn().unwrap();
                let _status = process.wait().unwrap();
            }
        } else {
            if write_type.trim().contains("error") {
                let write = writeln!(file, "{}: command not found", command.trim());
                if write.is_err(){
                    println!("Sorry could not write to file");
                }
            }
            println!("{}: command not found", command.trim());
        }
    }
    return;
}

pub fn redirect_handler(redirects: Vec<(String, String)>) {

    for redirect in redirects {
        let (redirect_loaction, redirect_type) = redirect;
        let path = Path::new(&redirect_loaction);
        match redirect_type.trim() {
            "replace_output" | "replace_error" => {
                let file_result = File::create(path);
                match file_result {
                    Ok(_) => continue,
                    Err(_) => {println!("Sorry internal file creation issue");}
                }
            }
            "append_output" | "append_error" => {
                let file_result = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(path);
                match file_result {
                    Ok(_) => continue,
                    Err(_) => {println!("Sorry internal file creation issue");}
                }
            }

            _ => {}
        }
    }
}

pub fn execute_pipeline(all_commands: &Vec<Vec<String>>, redirect: bool, redirects: Vec<(String, String)>, last_entry: &mut usize) {
    use std::process::{Command, Stdio};
    use std::io::Write;
    
    if all_commands.is_empty() {
        return;
    }
    
    let mut processes: Vec<Child> = Vec::new();
    let mut prev_stdout: Option<Stdio> = None;
    let mut builtin_output: Option<Vec<u8>> = None;
    
    for (i, cmd_args) in all_commands.iter().enumerate() {
        if cmd_args.is_empty() {
            continue;
        }
        
        let command = &cmd_args[0];
        let args = &cmd_args[1..];
        let is_last = i == all_commands.len() - 1;

        if SHELL_COMMANDS.contains(&command.as_str()) {
            let mut output = Vec::new();

            match command.trim() {
                "echo" => {
                    let mut line = String::new();
                    for arg in args {
                        line.push_str(arg);
                        line.push(' ');
                    }
                    if !line.is_empty() {
                        line.pop();
                    }
                    line.push('\n');
                    output = line.into_bytes();
                }
                "pwd" => {
                    if let Ok(path) = std::env::current_dir() {
                        let line = format!("{}\n", path.display());
                        output = line.into_bytes();
                    }
                }
                "type" => {
                    if !args.is_empty() {
                        let arg = &args[0];
                        let line = if SHELL_COMMANDS.contains(&arg.as_str()) {
                            format!("{} is a shell builtin\n", arg)
                        } else if let Some(path) = find_executable_in_path(&arg) {
                            format!("{} is {}\n", arg, path.display())
                        } else {
                            format!("{}: not found\n", arg)
                        };
                        output = line.into_bytes();
                    }
                }

                // For 'cd' and 'exit', dont know how to handle in pipeline, so just skip for now 

                "cd" | "exit" => {
                    println!("{}: dont know how to handle in pipeline sorry", command);
                    continue;
                }
                _ => {
                    println!("{}: builtin command not supported in pipeline", command);
                    continue;
                }
            }
            
            if is_last {

                if redirect {
                    if let Some((write_location, write_type)) = redirects.last() {
                        if write_type.trim().contains("output") {
                            let mut file = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(!write_type.trim().contains("append"))
                                .append(write_type.trim().contains("append"))
                                .open(write_location)
                                .unwrap();
                            file.write_all(&output).unwrap();
                        } else {
                            print!("{}", String::from_utf8_lossy(&output));
                        }
                    }
                } else {
                    print!("{}", String::from_utf8_lossy(&output));
                }
            } else {

                builtin_output = Some(output);
            }
            continue;
        }

        if let Some(_path) = find_executable_in_path(&command.trim()) {
            let mut cmd = Command::new(command.trim());
            cmd.args(args);

            if let Some(prev_out) = prev_stdout.take() {
                cmd.stdin(prev_out);
            } else if let Some(builtin_out) = builtin_output.take() {
                let mut child = cmd.stdin(Stdio::piped()).spawn().unwrap();
                if let Some(stdin) = child.stdin.take() {
                    let mut stdin = stdin;
                    stdin.write_all(&builtin_out).unwrap();
                    drop(stdin);
                }
                processes.push(child);
                continue;
            }

            if !is_last {
                cmd.stdout(Stdio::piped());
            } else if redirect {
                if let Some((write_location, write_type)) = redirects.last() {
                    if write_type.trim().contains("output") {
                        let file = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(!write_type.trim().contains("append"))
                            .append(write_type.trim().contains("append"))
                            .open(write_location);
                        if let Ok(f) = file {
                            cmd.stdout(f);
                        }
                    }
                }
            }
            
            match cmd.spawn() {
                Ok(mut child) => {
                    if !is_last {
                        prev_stdout = child.stdout.take().map(Stdio::from);
                    }
                    processes.push(child);
                }
                Err(e) => {
                    println!("Failed to execute {}: {}", command, e);
                    return;
                }
            }
        } else {
            println!("{}: command not found", command.trim());
            return;
        }
    }

    for mut process in processes {
        let _ = process.wait();
    }
}
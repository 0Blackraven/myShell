#[allow(unused_imports)]
use pathsearch::find_executable_in_path;
use std::io::{self, Write};
use std::path::Path;
use std::env::{current_dir, set_current_dir, home_dir};
use std::process::Command;

const SHELL_COMMANDS: [&str; 5] = ["echo", "type", "exit", "cd", "pwd"];
fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // let mut command_buf: String = String::new();
        let mut command: String = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let index_result: Option<usize> = command.trim().find(" ");
        let index: usize;
        match index_result {
            Option::None => index = command.len(),
            Option::Some(num) => {
                index = num;
            }
        }
        let mut args: Vec<String> = Vec::new();
        
        match command.trim() {
            "" => print!("\n"),
            "exit" => break,
            "echo" => echo_handler(&args),
            "type" => type_handler(&args),
            "pwd" => pwd_handler(&args, &command),
            "cd" => cd_handler(&args, &command),
            _ => general_handler(&args, &command),
        }
    }
}

fn echo_handler(args: &Vec<String>) {
    for arg in args {
        print!("{} ",arg);
    }
    println!();
    return;
}

fn type_handler(args: &Vec<String>) {
    if args.is_empty() {
        println!("Not a valid command");
    } else {
        for arg in args {
            if SHELL_COMMANDS.contains(&arg.as_str()) {
                println!("{} is a shell builtin", arg);
            } else if let Some(path) = find_executable_in_path(&arg) {
                println!("{} is {}", arg, path.display()); // this is third party way
            } else {
                println!("{}: not found", arg);
            }
        }
    }
    return;
}

fn pwd_handler(args: &Vec<String>, command: &str) {
    if !args.is_empty() {
        println!("{}: Invalid arguments provided", command.trim());
    } else {
        let path_result: Result<std::path::PathBuf, io::Error> = std::env::current_dir();
        match path_result {
            Err(e) => println!("Not Found Error: {}", e),
            Ok(path_buf) => println!("{}", path_buf.display()),
        };
    }
    return;
}

fn cd_handler(args: &Vec<String>, command: &str) {
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

fn general_handler(args: &Vec<String>, command: &str) {
    if let Some(_path) = find_executable_in_path(&command.trim()) {
        let mut process = Command::new(command.trim()).args(args).spawn().unwrap();
        let _status = process.wait().unwrap();
    } else {
        println!("{}: command not found", command.trim());
    }
    return;
}
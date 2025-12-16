#[allow(unused_imports)]
use pathsearch::find_executable_in_path;
use std::io::{self, Write};
use std::path::Path;
use std::env::{current_dir, set_current_dir, home_dir};
use std::process::Command;

const SHELL_COMMANDS: [&str; 5] = ["echo", "type", "exit", "cd", "pwd"];
fn main() {
    let mut is_complete:bool = true;
    let mut input: String = String::new();
    loop {
        let mut results: Vec<String> = Vec::new();
        let mut args: Vec<String> = Vec::new();
        let mut command: String = String::new();

        if is_complete {
            input.clear();
            print!("$ ");
        }else{
            print!("> ");
        }
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();

        if input.is_empty() || input == "\r\n" {
            continue;
        }else{
            (is_complete, results) = input_parser(&input);    
            command = results[0].clone();
            args.extend(results[1..].to_vec());

            if is_complete {
                match command.trim() {
                    "" => print!(""),
                    "exit" => break,
                    "echo" => echo_handler(&args),
                    "type" => type_handler(&args),
                    "pwd" => pwd_handler(&args, &command),
                    "cd" => cd_handler(&args, &command),
                     _ => general_handler(&args, &command),
                }
            }
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

fn input_parser(input: &str) ->(bool, Vec<String>) {
    let mut in_double_quotes: bool = false; //true
    let mut in_single_quotes: bool = false;
    let mut is_complete: bool = true;
    let mut is_escaped: bool = false;

    let mut current_argument: String = String::new();
    let mut arguments:Vec<String> = Vec::new();

    let mut current_character = input.chars().peekable();

    let push_current_char = |current_argument: &mut String, arguments: &mut Vec<String>| {
        if !current_argument.is_empty(){
            arguments.push(current_argument.clone());
        }
        current_argument.clear();

    };
    
    while let Some(character) = current_character.next() {

        if is_escaped {
            current_argument.push(character);
            is_escaped = false;
            continue;
        }
        match character {
            '\\' =>{
                if in_single_quotes {
                    current_argument.push(character);
                    continue;
                }else if in_double_quotes{
                    if let Some(&next_character) = current_character.peek() {
                        if matches!(next_character,  '\\' | '"' | '$' | '`') {
                            current_character.next();
                            if next_character != '\n' {
                                current_argument.push(next_character);
                            }
                        }else {
                            current_argument.push(character);   
                        }
                    }else{
                        current_argument.push(character);
                        continue;
                    }
                } else{
                    is_escaped =true;
                    continue;
                }
                
            },
            '\'' =>{
                if in_double_quotes{
                    current_argument.push(character);
                    continue;
                } else {
                    in_single_quotes = !in_single_quotes;
                    is_complete = !is_complete;
                    continue;
                } 
            },

            '\"' =>{
                if in_single_quotes{
                    current_argument.push(character);
                    continue;
                } else {
                    in_double_quotes = !in_double_quotes;
                    is_complete = !is_complete;
                    continue;
                }

            },

            c if c.is_whitespace() =>{
                if in_double_quotes || in_single_quotes {
                    current_argument.push(character);
                    continue;
                } else {
                    if !current_argument.is_empty(){
                        push_current_char(&mut current_argument, &mut arguments);
                        continue;
                    }
                }
            },

            _ =>{
                current_argument.push(character);
            }
        }
    }

    // final check
    if !current_argument.is_empty() {
        push_current_char(&mut current_argument, &mut arguments);
    }

    return (is_complete, arguments);
}
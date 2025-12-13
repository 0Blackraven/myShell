#[allow(unused_imports)]
use pathsearch::find_executable_in_path;
use std::io::{self, Write};
use std::path::Path;
use std::env::{current_dir, set_current_dir, home_dir};
use std::process::Command;

fn main() {
    const SHELL_COMMANDS: [&str; 5] = ["echo", "type", "exit", "cd", "pwd"];
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
        let args: &str = &command.split_off(index);
        match command.trim() {
            "" => {
                print!("\n");  
            },
            "exit" => {
                print!("{}\n", command.trim());
                return;
            },
            "echo" => {
                print!("{}\n", args.trim());
            },
            "type" => {
                if args.trim() == ""{ 
                    print!("Not a valid command\n");
                } else if SHELL_COMMANDS.contains(&args.trim()) {
                    print!("{} is a shell builtin\n", args.trim());
                } else if let Some(path) = find_executable_in_path(&args.trim()) {
                    print!("{} is {}\n", args.trim(), path.display()); // this is third party way
                } else {
                    print!("{}: not found \n", args.trim());
                }
            },
            "pwd" =>{
                if args != "" {
                    print!("{} {}: Not a valid command\n", command.trim(), args.trim());
                }else{
                    let path_result: Result<std::path::PathBuf, io::Error> = std::env::current_dir();
                    match path_result{
                        Err(e) => print!("Not Found Error: {}\n",e),
                        Ok(path_buf)=> print!("{}\n",path_buf.display())
                    };
                }
            },
            "cd" =>{
                match args.trim() {
                 ".." =>{
                    let path = current_dir().unwrap();
                    let path_parent = path.parent().unwrap(); // handle the case when the current dir is the root
                    let _result = set_current_dir(path_parent);
                 },
                 "" =>{
                    let root = Path::new("/");
                    let _result = set_current_dir(root);
                 },
                 "~" =>{
                    let path = home_dir().expect("sorry cannot find your home dir");
                    let _result = set_current_dir(path);
                 },
                 _ => {
                    let path = Path::new(args.trim());
                    let is_path_correct = path.try_exists().expect("Can't check existence of provided file");
                    if is_path_correct {
                        let _result = set_current_dir(path);                        
                    } else {
                        println!("{}: {}: No such file or directory",command.trim(), args.trim());
                    }
                 }   
                }
            }
            _ =>{
                if let Some(_path) = find_executable_in_path(&command.trim()) {
                    // print!("Executable file detected");
                    let arguments: &Vec<&str> = &args.trim().split_whitespace().collect();
                    let mut process = Command::new(command.trim())
                        .args(arguments)
                        .spawn()
                        .unwrap();

                    let _status = process.wait().unwrap();
                } else {
                    print!("{}: command not found \n", command.trim());
                }
            }
        }
    }
}
#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    const SHELL_COMMANDS: [&str; 3] = ["echo", "type", "exit"];
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command: String = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let index_result: Option<usize> = command.find(" ");
        let index: usize;
        match index_result {
            Option::None => index = command.len(),
            Option::Some(num) => {
                index = num;
            }
        }
        let args: &str = &command.split_off(index);
        if command.trim() == "exit" {
            print!("{}\n", command.trim());
            return;
        }
        if command.trim() == "echo" {
            print!("{}\n", args.trim());
            // return;
        }
        if command.trim() == "type" {
            if SHELL_COMMANDS.contains(&args.trim()){
                print!("{} is a shell builtin\n", args.trim());
            }else{
                print!("{}: not found \n", args.trim());
            }
        }
        if !SHELL_COMMANDS.contains(&command.trim()){
            print!("{}: command not found \n", command.trim());
        }
    }
}
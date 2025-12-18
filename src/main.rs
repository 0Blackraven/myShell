#[allow(unused_imports)]
use std::io::{self, Write};

mod input_parser;
use input_parser::input_parser;

mod handler;
use handler::{redirect_handler, echo_handler, type_handler, cd_handler, general_handler, pwd_handler};

fn main() {
    let mut is_complete: bool = true;
    let mut input: String = String::new();
    loop {
        let mut args: Vec<String> = Vec::new();
        

        if is_complete {
            input.clear();
            print!("$ ");
        } else {
            print!("> ");
        }
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();

        if input.is_empty() || input == "\r\n" {
            continue;
        } else {
            let mut redirect: bool = false;
            let mut redirects: Vec<(String, String)> = Vec::new();
            let mut results: Vec<String> = Vec::new();

            (is_complete, results, redirect, redirects) = input_parser(&input);
            let command = results[0].clone();
            args.extend(results[1..].to_vec());

            if is_complete {
                if redirect {
                    redirect_handler(redirects.clone());
                }
                match command.trim() {
                    "" => print!(""),
                    "exit" => break,
                    "echo" => echo_handler(&args, redirect, redirects),
                    "type" => type_handler(&args, redirect, redirects),
                    "pwd" => pwd_handler(&args, &command, redirect, redirects),
                    "cd" => cd_handler(&args, &command),
                    _ => general_handler(&args, &command, redirect, redirects),
                }
            }
        }
    }
}


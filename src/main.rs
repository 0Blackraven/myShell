#[allow(unused_imports)]
use std::io::{self, Write};
use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::{Editor, completion::Completer};

mod input_parser;
use input_parser::input_parser;

mod handler;
use handler::{redirect_handler, echo_handler, type_handler, cd_handler, general_handler, pwd_handler, SHELL_COMMANDS};

struct MyHelper;

impl rustyline::Helper for MyHelper {}
impl Completer for MyHelper {
    type Candidate = String;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        for command in SHELL_COMMANDS {
            if command.starts_with(&line[..pos]) {
                return Ok((0, vec![format!("{} ", command)]));
            }
        }
        Ok((0, vec![]))
    }
}
impl rustyline::hint::Hinter for MyHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        // if !line.is_empty() {
        //     if let Ok((_, c)) = self.complete(line, pos, ctx) {
        //         if let Some(e) = c.get(0) {
        //             return Some(e[pos..].to_string());
        //         }
        //     }
        // }
        None
    }
}
impl rustyline::highlight::Highlighter for MyHelper {}
impl rustyline::validate::Validator for MyHelper {}

fn main() {
    let mut complete: bool = true;
    let mut readline: Editor<MyHelper, _>;
    let readline_result: Result<Editor<MyHelper, rustyline::history::FileHistory>, ReadlineError> =
        Editor::new();
    match readline_result {
        Ok(result) => {
            readline = result;
        }
        Err(_) => {
            println!("Sorry editor broke");
            return;
        }
    }
    readline.set_helper(Some(MyHelper));
    if readline.load_history("history.txt").is_err() {
        println!("No history present");
    }
    loop {
        let mut args: Vec<String> = Vec::new();
        let input = readline.readline(if complete { "$ " } else { "> " });
        match input {
            Ok(line) => {
                let history = readline.add_history_entry(line.as_str());
                match history {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error while keeping history: {}", e);
                    }
                }
                if line.trim().is_empty() {
                    println!("line is empty");
                    continue;
                } else {
                    let (is_complete, results, redirect, redirects) = input_parser(&line);
                    complete = is_complete;

                    let command = results[0].clone();
                    args.extend(results[1..].to_vec());

                    if complete {
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
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    let history = readline.save_history("history.txt");
    match history {
        Ok(_) => {}
        Err(e) => {
            println!("Error in making history file : {}", e);
        }
    }
}
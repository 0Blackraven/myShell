use std::fs;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;
use anyhow::Result;
use is_executable::IsExecutable;
use rustyline::Config;
use rustyline::completion::Pair;
use rustyline::error::ReadlineError;
use rustyline::{Editor, completion::Completer};

mod input_parser;
use input_parser::input_parser;

mod handler;
use handler::{redirect_handler, echo_handler, type_handler, cd_handler, general_handler, pwd_handler, SHELL_COMMANDS};

struct MyHelper {
    completions: Vec<String>
}

impl MyHelper {
    fn new() -> Self {
        let mut shell_map : Vec<String> = Vec::new();
        for command in SHELL_COMMANDS {
            let result = format!("{} ",command);
            shell_map.push(result.into());
        }

        Self {
            completions: shell_map
        }
    }

    fn add_entry(&mut self, v:&str) {
        self.completions.push(v.into());
    }

    fn add_path_completions (&mut self, path:PathBuf) {
        if !path.exists() {
            return;
        }

        let files_search_result: std::result::Result<fs::ReadDir, io::Error> = fs::read_dir(path);
        match files_search_result {
            Ok(files) => {
                let files_in_path = files;
                for file in files_in_path {
                    match file {
                        Ok(entry) =>{
                            if entry.path().is_dir() {
                                continue;
                            }
                            if entry.path().is_file() && entry.path().is_executable() {
                                if let Some(filename) = entry.path().file_name().and_then(|n| n.to_str()){
                                    self.add_entry(&format!("{} ",filename));
                                }
                            }
                        }
                        Err(_) => {
                            return;
                        }
                    }
                }
            }
            Err(_) => {
                return;
            }
            }
    }
}

impl Default for MyHelper {
    fn default() -> Self {
        Self::new()
    }
}

impl rustyline::Helper for MyHelper {}
impl Completer for MyHelper {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let prefix: &str = &line[..pos];
        let mut matches: Vec<Pair> = Vec::new();
        for trigger in &self.completions {
            if trigger.starts_with(prefix) {
                matches.push(Pair {
                    display: trigger.clone().trim_end().to_string(),
                    replacement: trigger.clone()
                });
            }
        }
        matches.sort_by(|a, b| a.display.cmp(&b.display));
        
        Ok((pos-prefix.len(), matches))
    }
}

impl rustyline::hint::Hinter for MyHelper {
    type Hint = String;
    fn hint(&self, _: &str, _: usize, _: &rustyline::Context<'_>) -> Option<Self::Hint> {
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
    let mut my_helper = MyHelper::default();
    let config = Config::builder().completion_type(rustyline::CompletionType::List).build();
    let readline_result: Result<Editor<MyHelper, rustyline::history::FileHistory>, ReadlineError> =
        Editor::with_config(config);
    match readline_result {
        Ok(result) => {
            readline = result;
        }
        Err(_) => {
            println!("Sorry editor broke");
            return;
        }
    }
    if let Ok(path_var) = std::env::var("PATH") {
        let paths = std::env::split_paths(&path_var);
        
        for path in paths {
            my_helper.add_path_completions(path);
        }
    }
    
    readline.set_helper(Some(my_helper));
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
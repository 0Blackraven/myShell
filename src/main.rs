use std::{env, fs};
#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;
use anyhow::Result;
use is_executable::IsExecutable;
use rustyline::Config;
use rustyline::completion::Pair;
use rustyline::error::ReadlineError;
use rustyline::{Editor, completion::Completer};
use rustyline::history::{History, FileHistory};
use std::str::FromStr;
use std::fs::OpenOptions;

mod input_parser;
use input_parser::input_parser;

mod handler;
use handler::{redirect_handler, echo_handler, type_handler, cd_handler, general_handler, pwd_handler, SHELL_COMMANDS};

pub struct MyHelper {
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
    let mut last_entry: usize = 0;
    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();
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
    if let Some(location) = env::var_os("HISTFILE") {
        let path = PathBuf::from(location);

        if let Ok(contents) = std::fs::read_to_string(&path) {
            for line in contents.lines() {
                let _ = readline.add_history_entry(line);
            }

            last_entry = readline.history().len();
        }
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
                    let (is_complete, results, redirect, redirects, file_location, file_option) =
                        input_parser(&line);
                    complete = is_complete;

                    let command = results[0].clone();
                    args.extend(results[1..].to_vec());

                    if complete {
                        if redirect {
                            redirect_handler(redirects.clone());
                        }
                        match command.trim() {
                            "" => print!(""),
                            "exit" => {
                                append_history_on_exit(&mut readline, &mut last_entry);
                                break;
                            }
                            "echo" => echo_handler(&args, redirect, redirects),
                            "type" => type_handler(&args, redirect, redirects),
                            "pwd" => pwd_handler(&args, &command, redirect, redirects),
                            "cd" => cd_handler(&args, &command),
                            "history" => history_handler(
                                &mut readline,
                                &args,
                                &file_option,
                                &file_location,
                                &mut last_entry,
                            ),
                            _ => general_handler(&args, &command, redirect, redirects),
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                let _ = readline.clear_history();
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


fn history_handler(
    readline: &mut Editor<MyHelper, FileHistory>,
    args: &Vec<String>,
    file_option: &String,
    file_location: &String,
    last_entry: &mut usize,
) {
    if file_option.is_empty() && file_location.is_empty() {
        if args.len() == 0 {
            for (i, entry) in readline.history().iter().enumerate() {
                println!("    {}  {}", i + 1, entry);
            }
            return;
        }
        if args.len() > 1 {
            println!("Too many arguments provided");
            return;
        }
        let limit_result: Result<usize, <usize as FromStr>::Err> = args[0].parse();
        match limit_result {
            Ok(limit) => {
                let lenght = readline.history().len();
                let start_index = lenght.saturating_sub(limit);

                for (i, entry) in readline.history().iter().enumerate().skip(start_index) {
                    println!("    {}  {}", i + 1, entry);
                }
                return;
            }
            Err(_) => {
                println!("{}: provide correct arguments for command", args[0]);
                return;
            }
        }
    } else {
        // println!("{}   {}", file_location,file_option);
        match file_option.trim() {
            "read" => {
                let result_file = OpenOptions::new().read(true).open(file_location);
                match result_file {
                    Ok(_) => {
                        let result = readline.load_history(file_location);
                        match result {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Sorry could not load history from file {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Sorry erorred out {}", e);
                        return;
                    }
                }
            }
            "write" => {
                let result_file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(file_location);
                match result_file {
                    Ok(mut file) => {
                        for entry in readline.history().iter() {
                            let _ = writeln!(file, "{}", entry);
                        }
                    }
                    Err(e) => {
                        println!("Sorry could not load file {}", e);
                    }
                }
            }

            "append" => {
                let result_file = OpenOptions::new()
                    .read(true)
                    .append(true)
                    .write(true)
                    .create(true)
                    .open(file_location);
                match result_file {
                    Ok(mut file) => {
                        for entry in readline.history().iter().skip(*last_entry) {
                            let _ = writeln!(file, "{}", entry);
                        }
                        *last_entry = readline.history().len();
                    }
                    Err(e) => {
                        println!("Sorry could not append history from file {}", e);
                    }
                }
            }
            _ => {}
        }
    }
}

fn append_history_on_exit (readline: &mut Editor<MyHelper, FileHistory> , last_entry: &mut usize) {
    if let Some(location) = env::var_os("HISTFILE") {
        let path = PathBuf::from(location);

        let file_result = OpenOptions::new().read(true).write(true).append(true).open(path);
        match file_result {
            Ok(mut file) => {
                for command in readline.history().iter().skip(*last_entry) {
                    let _ = writeln!(file, "{}", command);
                }

                *last_entry = readline.history().len();
            }
            Err(_) => {
                println!("Sorry could not save ur cmd history")
            }
        }
    }
}
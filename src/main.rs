#[allow(unused_imports)]
use std::io::{self, Write};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

mod input_parser;
use input_parser::input_parser;

mod handler;
use handler::{redirect_handler, echo_handler, type_handler, cd_handler, general_handler, pwd_handler};

fn main() -> Result<()> {
    let mut input: String = String::new();
    let mut history = String::new();

    enable_raw_mode()?;

    print!("$ ");
    io::stdout().flush()?;
    loop {
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            if modifiers.contains(KeyModifiers::CONTROL) {
                match code {
                    KeyCode::Char('c') => {
                        print!("\r\n^c");
                        break;
                    }
                    KeyCode::Char('l') => {
                        input.clear();
                        print!("\x1b[2J\x1b[H$");
                    }
                    _ => {}
                }
            } else {
                match code {
                    KeyCode::Enter => {
                        print!("\r\n");
                        if !input.trim().is_empty() {
                            history.push_str(input.trim());
                        }
                        let (is_now_complete, _, _, _) = input_parser(&history, true);
                        if is_now_complete {
                            handle_input(&history);
                            history.clear();
                        }
                        // println!("{}",complete);
                        input.clear();
                        if is_now_complete {
                            print!("$ ")
                        } else {
                            print!("> ")
                        };
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                        print!("{}", c);
                    }
                    KeyCode::Tab => {
                        // yaha moye moye ho jaa rha mera thank u 
                    }
                    KeyCode::Esc => {
                        print!("\r\n^c");
                        break;
                    }
                    KeyCode::Backspace => {
                        input.pop();
                        print!("\u{0008} \u{0008}");
                    }
                    _ => {}
                }
            }
        }
        io::stdout().flush()?;
    }

    disable_raw_mode()?;

    Ok(())
}

fn handle_input(input: &str) {
    if input.is_empty() {
        return;
    }
    let (is_complete, results, redirect, redirects) = input_parser(&input, true);

    if is_complete {
        if redirect {
            redirect_handler(redirects.clone());
        }

        let command = results[0].clone();
        let args: Vec<String> = results[1..].to_vec();

        match command.trim() {
            "" => {}
            "exit" => return,
            "echo" => echo_handler(&args, redirect, redirects),
            "type" => type_handler(&args, redirect, redirects),
            "pwd" => pwd_handler(&args, &command, redirect, redirects),
            "cd" => cd_handler(&args, &command),
            _ => general_handler(&args, &command, redirect, redirects),
        }
    }
}
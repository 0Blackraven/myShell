pub fn input_parser(
    input: &str,
) -> (
    bool,
    Vec<String>,
    bool,
    Vec<(String, String)>,
    String,
    String,
) {
    let mut in_double_quotes: bool = false;
    let mut in_single_quotes: bool = false;
    let mut is_complete: bool = true;
    let mut is_escaped: bool = false;
    let mut is_path: bool = false;
    let mut redirect: bool = false;
    let mut file_option: String = String::new();
    let mut file_location: String = String::new();

    let mut current_argument: String = String::new();
    let mut arguments: Vec<String> = Vec::new();
    let mut redirects: Vec<(String, String)> = Vec::new();

    let mut current_character = input.chars().peekable();

    let push_current_char = |current_argument: &mut String, arguments: &mut Vec<String>| {
        if !current_argument.is_empty() {
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
            '<' => {
                if in_double_quotes || in_single_quotes {
                    current_argument.push(character);
                } else {
                    if !current_argument.is_empty() {
                        push_current_char(&mut current_argument, &mut arguments);
                    }
                    is_path = true;
                }
            }
            '\\' => {
                if in_single_quotes {
                    current_argument.push(character);
                    continue;
                } else if in_double_quotes {
                    if let Some(&next_character) = current_character.peek() {
                        if matches!(next_character, '\\' | '"' | '$' | '`') {
                            current_character.next();
                            if next_character != '\n' {
                                current_argument.push(next_character);
                            }
                        } else {
                            current_argument.push(character);
                        }
                    } else {
                        current_argument.push(character);
                        continue;
                    }
                } else {
                    is_escaped = true;
                    continue;
                }
            }
            '\'' => {
                if in_double_quotes {
                    current_argument.push(character);
                    continue;
                } else {
                    in_single_quotes = !in_single_quotes;
                    is_complete = !is_complete;
                    continue;
                }
            }

            '\"' => {
                if in_single_quotes {
                    current_argument.push(character);
                    continue;
                } else {
                    in_double_quotes = !in_double_quotes;
                    is_complete = !is_complete;
                    continue;
                }
            }

            c if c.is_whitespace() => {
                if in_double_quotes || in_single_quotes {
                    current_argument.push(character);
                    continue;
                } else {
                    if !current_argument.is_empty() {
                        push_current_char(&mut current_argument, &mut arguments);
                        continue;
                    }
                }
            }

            _ => {
                if character == '>' {
                    if is_path {
                        continue;
                    }

                    if in_double_quotes || in_single_quotes {
                        current_argument.push(character);
                        continue;
                    }
                    let mut prefix = String::new();
                    if current_argument == "1" || current_argument == "2" {
                        prefix = current_argument.clone();
                        current_argument.clear();
                    }

                    if !current_argument.is_empty() {
                        push_current_char(&mut current_argument, &mut arguments);
                    }

                    current_argument = prefix;
                    current_argument.push(character);

                    while let Some(&next_char) = current_character.peek() {
                        if next_char == '>' {
                            current_argument.push(current_character.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    push_current_char(&mut current_argument, &mut arguments);
                    continue;
                }

                if character == '-' {
                    if in_double_quotes || in_single_quotes {
                        current_argument.push(character);
                        continue;
                    }
                    if let Some(&next_char) = current_character.peek() {
                        if matches!(next_char, 'r' | 'w' | 'a') {
                            if !current_argument.is_empty() {
                                push_current_char(&mut current_argument, &mut arguments);
                            }

                            current_character.next();

                            current_argument.push('-');
                            current_argument.push(next_char);

                            push_current_char(&mut current_argument, &mut arguments);
                            continue;
                        }
                    }
                }
                current_argument.push(character);
            }
        }
    }

    // final check
    if !current_argument.is_empty() {
        push_current_char(&mut current_argument, &mut arguments);
    }

    let mut i = 0;
    while i < arguments.len() {
        if matches!(
            arguments[i].trim(),
            ">" | "1>" | "2>" | ">>" | "1>>" | "2>>"
        ) {
            let redirect_type = arguments.remove(i);
            if i < arguments.len() {
                let redirect_location = arguments.remove(i);

                match redirect_type.trim() {
                    "1>" | ">" => {
                        redirects.push((redirect_location, "replace_output".to_string()));
                    }
                    "1>>" | ">>" => {
                        redirects.push((redirect_location, "append_output".to_string()));
                    }
                    "2>" => {
                        redirects.push((redirect_location, "replace_error".to_string()));
                    }
                    "2>>" => {
                        redirects.push((redirect_location, "append_error".to_string()));
                    }
                    _ => {}
                }
            }
        } else if matches!(arguments[i].trim(), "-r" | "-w" | "-a") {
            let result = arguments.remove(i);
            if i < arguments.len() {
                let file_result = arguments.remove(i);
                match result.trim() {
                    "-a" => {
                        file_option = "append".into();
                        file_location = file_result.into();
                    }
                    "-r" => {
                        file_option = "read".into();
                        file_location = file_result.into();
                    }
                    "-w" => {
                        file_option = "write".into();
                        file_location = file_result.into();
                    }
                    _ => {}
                }
            }
        } else {
            i += 1;
        }
    }

    if !redirects.is_empty() {
        redirect = true;
    }
    return (
        is_complete,
        arguments,
        redirect,
        redirects,
        file_location,
        file_option,
    );
}
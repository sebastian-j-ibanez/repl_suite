// Copyright (c) 2025 Sebastian Ibanez
// Author: Sebastian Ibanez
// Created: 2025-08-29

use repl_lib::Repl;

/// Return line.
fn process_line(line: String) -> repl_lib::Result<String> {
    Ok(line)
}

/// Return `true` if line is larger than 0.
fn line_is_finished(line: String) -> bool {
    match line.len() {
        b if b > 0 => true,
        _ => false,
    }
}

fn main() -> Result<(), ()> {
    let prompt = String::from("> ");
    let banner = String::from(
        r#"
    ____  __________  __       ____  ________  _______ 
   / __ \/ ____/ __ \/ /      / __ \/ ____/  |/  / __ \
  / /_/ / __/ / /_/ / /      / / / / __/ / /|_/ / / / /
 / _, _/ /___/ ____/ /___   / /_/ / /___/ /  / / /_/ / 
/_/ |_/_____/_/   /_____/  /_____/_____/_/  /_/\____/  
    "#,
    );
    let welcome_msg = String::from("Welcome to the REPL demo!");
    let mut repl = match Repl::new(prompt, banner, welcome_msg, process_line, line_is_finished) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("unable to init REPL: {}", e);
            return Err(());
        }
    };

    repl.print_welcome();

    loop {
        repl.print_prompt();
        match repl.get_line() {
            Ok(l) => println!("{}", l),
            Err(e) => {
                eprintln!("error: {}", e);
                return Err(());
            }
        }
    }
}

// Copyright (c) 2025 Sebastian Ibanez
// Author: Sebastian Ibanez
// Created: 2025-08-29

use repl_lib::{LineCompletionFunc, ProcessLineFunc, Repl};

/// Return line.
fn process_line() -> ProcessLineFunc {
    Box::new(|line: String| Ok(line))
}

/// Return `true` if line is larger than 0.
fn line_is_finished() -> LineCompletionFunc {
    Box::new(|line: String| {
        let expression = line.trim();
        let mut open_paren = 0;
        let mut close_paren = 0;

        for e in expression.chars() {
            match e {
                '(' => open_paren += 1,
                ')' => close_paren += 1,
                _ => {}
            }
        }

        let _ = expression.chars().map(|e| match e {
            '(' => open_paren += 1,
            ')' => close_paren += 1,
            _ => {}
        });

        (open_paren == close_paren) || (!expression.starts_with('(') && !expression.ends_with(')'))
    })
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
    let mut repl = match Repl::new(
        prompt,
        banner,
        welcome_msg,
        process_line(),
        line_is_finished(),
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("unable to init REPL: {}", e);
            return Err(());
        }
    };

    repl.print_welcome();

    loop {
        repl.print_prompt();
        repl.process_input()
            .inspect(|l| println!("{}", l))
            .map_err(|e| {
                eprintln!("error: {}", e);
            })?;
    }
}

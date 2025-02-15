#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut input).unwrap();
        
        let trimmed_input = input.trim(); // Avoid redundant `trim()` calls

        match trimmed_input {
            "exit 0" => process::exit(0),
            input if input.starts_with("echo ") => {
                println!("{}", input.strip_prefix("echo ").unwrap());
            }
            input if input.starts_with("type ") => {
                let command = input.strip_prefix("type ").unwrap().trim();
                match command {
                    "echo" => println!("echo is a shell builtin"),
                    "exit" => println!("exit is a shell builtin"),
                    "type" => println!("type is a shell builtin"),
                    _ => println!("{} not found", command),
                }
            }
            _ => println!("{}: command not found", trimmed_input),
        }
    }
}

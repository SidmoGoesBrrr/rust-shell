use std::env;
use std::io::{self, Write};
use std::fs;
use std::path::Path;

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut input).unwrap();

        let trimmed_input = input.trim();

        // Handle exit command
        if trimmed_input == "exit 0" {
            std::process::exit(0);
        }

        // Handle echo command
        if trimmed_input.starts_with("echo ") {
            if let Some(text) = trimmed_input.strip_prefix("echo ") {
                println!("{}", text);
            }
            continue;
        }

        // Handle type command
        if trimmed_input.starts_with("type ") {
            if let Some(command) = trimmed_input.strip_prefix("type ") {
                handle_type_command(command.trim());
            }
            continue;
        }

        // If command is not recognized
        println!("{}: command not found", trimmed_input);
    }
}

/// Handles the `type` command: checks if a command is a built-in or an executable in `PATH`
fn handle_type_command(command: &str) {
    // List of built-in commands
    let builtins = ["echo", "exit","type"];

    // Check if it's a built-in command
    if builtins.contains(&command) {
        println!("{} is a shell builtin", command);
        return;
    }

    // Get the PATH environment variable
    if let Ok(paths) = env::var("PATH") {
        for dir in paths.split(':') {
            let full_path = format!("{}/{}", dir, command);
            if Path::new(&full_path).exists() && is_executable(&full_path) {
                println!("{} is {}", command, full_path);
                return;
            }
        }
    }

    // If the command is neither a built-in nor found in PATH
    println!("{}: not found", command);
}

/// Checks if a file is executable
fn is_executable(path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        return metadata.is_file(); // Simplified check (for Unix, we would check permissions)
    }
    false
}
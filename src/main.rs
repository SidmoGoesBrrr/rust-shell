use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio, ExitStatus};

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut input).unwrap();

        let trimmed_input = input.trim();
        if trimmed_input.is_empty() {
            continue;
        }

        // Split input into command and arguments
        let mut parts = trimmed_input.split_whitespace();
        let command = parts.next().unwrap();
        let args: Vec<&str> = parts.collect();

        // Handle built-in commands
        if handle_builtin(command, &args) {
            continue;
        }

        // Try running an external command
        match run_external_command(command, &args) {
            Ok(status) => {
                if !status.success() {
                    println!("{}: command exited with status {}", command, status);
                }
            }
            Err(_) => {
                println!("{}: command not found", command);
            }
        }
    }
}

/// Handles built-in commands (`exit`, `echo`, `type` , `pwd`)
fn handle_builtin(command: &str, args: &[&str]) -> bool {
    match command {
        "exit" => {
            std::process::exit(0);
        }
        "echo" => {
            println!("{}", args.join(" "));
            return true;
        }
        "type" => {
            if let Some(cmd) = args.first() {
                handle_type_command(cmd);
            }
            return true;

        }
        "pwd" => {
            match env::current_dir(){
                Ok(path) => println!("{}",path.display()),
                Err(e) => eprintln!("pwd: error getting current directory: {}", e),
            }
            return true;
        }
        _ => false,
    }
}

/// Handles `type` command, checking if a command is built-in or an executable
fn handle_type_command(command: &str) {
    let builtins = ["echo", "exit", "type","pwd"];
    if builtins.contains(&command) {
        println!("{} is a shell builtin", command);
        return;
    }

    if let Some(path) = find_executable(command) {
        println!("{} is {}", command, path);
    } else {
        println!("{}: not found", command);
    }
}

/// Finds an executable in `PATH`
fn find_executable(command: &str) -> Option<String> {
    if let Ok(paths) = env::var("PATH") {
        for dir in paths.split(':') {
            let full_path = format!("{}/{}", dir, command);
            if std::path::Path::new(&full_path).exists() {
                return Some(full_path);
            }
        }
    }
    None
}

/// Runs an external command with arguments
fn run_external_command(command: &str, args: &[&str]) -> Result<ExitStatus, std::io::Error> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::inherit()) // Pass user input to the command
        .stdout(Stdio::inherit()) // Print command output directly
        .stderr(Stdio::inherit()) // Print errors directly
        .spawn()?; // Execute command

    let status = child.wait()?; // Wait for command to finish
    Ok(status)
}
use std::env;
use std::path::Path;

pub fn handle_type_command(input: &str) -> bool {
    // Check if the input starts with "type " (with a space)
    if let Some(rest) = input.strip_prefix("type ") {
        let cmd = rest.trim();
        // List of builtins
        let builtins = ["echo", "exit", "type", "pwd", "cd"];
        if builtins.contains(&cmd) {
            println!("{} is a shell builtin", cmd);
        } else if let Some(path) = find_executable(cmd) {
            println!("{} is {}", cmd, path);
        } else {
            println!("{}: not found", cmd);
        }
        return true;
    }
    false
}

pub fn find_executable(command: &str) -> Option<String> {
    if let Ok(paths) = env::var("PATH") {
        for dir in paths.split(':') {
            let full_path = format!("{}/{}", dir, command);
            if Path::new(&full_path).exists() {
                return Some(full_path);
            }
        }
    }
    None
}
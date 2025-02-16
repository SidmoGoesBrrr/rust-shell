use std::env;
use std::process;

pub fn handle_exit_command(input: &str) -> bool {
    // Check if the command is exactly "pwd"
    if input.trim() == "exit 0" {
        process::exit(0);

        return true;
    }
    false
}
use crate::commands::type_cmd::find_executable;
use crate::util::parse_parameters;
use std::process::Command;

pub fn handle_execute_command(input: &str) -> bool {
    if let Some((command, arguments)) = parse_parameters(input).split_first() {
        if find_executable(command).is_some() {
            match Command::new(command).args(arguments).status() {
                Ok(status) => {
                    if !status.success() {
                        eprintln!("Command '{}' failed with status: {}", command, status);
                    }
                }
                Err(_) => eprintln!("{}: command not found", command),
            }
            return true;
        }
    }
    false
}
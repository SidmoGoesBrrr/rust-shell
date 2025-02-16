use crate::commands::type_cmd::find_executable;
use crate::util::parse_parameters;
use std::process::Command;

pub fn handle_execute_command(input: &str) -> bool {
    if let Some((command, arguments)) = parse_parameters(input).split_first() {
        if find_executable(command).is_some() {
            // Run the command and ignore its exit status,
            // letting the command print its own error messages to stderr.
            let _ = Command::new(command).args(arguments).status();
            return true;
        }
    }
    false
}
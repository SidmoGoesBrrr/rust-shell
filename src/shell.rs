use std::io::{self, Write};

use crate::commands::echo::handle_echo_command;
use crate::commands::cd::handle_cd_command;
use crate::commands::type_cmd::handle_type_command;
use crate::commands::pwd::handle_pwd_command;
use crate::commands::execute::handle_execute_command;
use crate::commands::exit::handle_exit_command;
use nix::libc;

pub fn start_shell() {
    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        if stdin.read_line(&mut input).is_err() {
            break;
        }
        let trimmed_input = input.trim();
        if trimmed_input.is_empty() {
            continue;
        }

        // If the command contains a redirection operator, process it.
        if trimmed_input.contains('>') {
            let (cmd_part, file_part) = parse_redirection(trimmed_input);
            run_command_with_redirection(&cmd_part, &file_part);
            continue;
        } else {
            process_command(trimmed_input);
        }
    }
}

/// Process a command without any redirection.
fn process_command(cmd: &str) {
    if handle_echo_command(cmd) {
        return;
    }
    if handle_cd_command(cmd) {
        return;
    }
    if handle_type_command(cmd) {
        return;
    }
    if handle_pwd_command(cmd) {
        return;
    }
    if handle_execute_command(cmd) {
        return;
    }
    if handle_exit_command(cmd) {
        return;
    }
    println!("{}: command not found", cmd);
}

/// Simple parser to split a command into the command part and the output file.
/// It replaces any occurrence of "1>" with ">" so that > is equivalent to 1>.
fn parse_redirection(input: &str) -> (String, String) {
    let input = input.replace("1>", ">");
    if let Some(index) = input.find('>') {
        let cmd = input[..index].trim().to_string();
        let file = input[index + 1..].trim().to_string();
        (cmd, file)
    } else {
        (input.to_string(), String::new())
    }
}

/// Redirect stdout to the given file, run the command, then restore stdout.
fn run_command_with_redirection(command: &str, output_file: &str) {
    use std::fs::OpenOptions;
    use std::os::unix::io::AsRawFd;
    use nix::unistd::{dup, dup2, close};

    // Save the original stdout file descriptor.
    let stdout_fd = dup(libc::STDOUT_FILENO).expect("dup failed");

    // Open the target file for writing (create/truncate).
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_file)
        .expect("failed to open output file");
    let file_fd = file.as_raw_fd();

    // Redirect stdout to the file.
    dup2(file_fd, libc::STDOUT_FILENO).expect("dup2 failed");

    // Run the command normally.
    process_command(command);

    // Flush stdout.
    io::stdout().flush().unwrap();

    // Restore stdout.
    dup2(stdout_fd, libc::STDOUT_FILENO).expect("dup2 restore failed");
    close(stdout_fd).ok();
}
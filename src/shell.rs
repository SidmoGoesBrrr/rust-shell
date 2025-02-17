use nix::libc;
use std::io::{self, Write};

// Import built-in command handlers from your commands modules.
use crate::commands::echo::handle_echo_command;
use crate::commands::cd::handle_cd_command;
use crate::commands::type_cmd::handle_type_command;
use crate::commands::pwd::handle_pwd_command;
use crate::commands::execute::handle_execute_command;
use crate::commands::exit::handle_exit_command;

// ----------------------------------------------------------------
// Redirection Support
// ----------------------------------------------------------------

#[derive(Debug)]
struct RedirectionSpec {
    stdout: Option<(String, bool)>, // (filename, append) for stdout redirection
    stderr: Option<(String, bool)>, // (filename, append) for stderr redirection
}

/// Parses the input command line and extracts redirection operators.
/// It assumes that redirection operators and filenames are separated by whitespace.
fn parse_command_and_redirections(input: &str) -> (String, RedirectionSpec) {
    let mut redir = RedirectionSpec {
        stdout: None,
        stderr: None,
    };
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut cmd_tokens = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            "2>>" => {
                if i + 1 < tokens.len() {
                    redir.stderr = Some((tokens[i + 1].to_string(), true));
                    i += 2;
                } else {
                    break;
                }
            }
            "2>" => {
                if i + 1 < tokens.len() {
                    redir.stderr = Some((tokens[i + 1].to_string(), false));
                    i += 2;
                } else {
                    break;
                }
            }
            "1>>" | ">>" => {
                if i + 1 < tokens.len() {
                    redir.stdout = Some((tokens[i + 1].to_string(), true));
                    i += 2;
                } else {
                    break;
                }
            }
            "1>" | ">" => {
                if i + 1 < tokens.len() {
                    redir.stdout = Some((tokens[i + 1].to_string(), false));
                    i += 2;
                } else {
                    break;
                }
            }
            token => {
                cmd_tokens.push(token);
                i += 1;
            }
        }
    }
    (cmd_tokens.join(" "), redir)
}

/// Executes a command with redirections applied.
/// It saves the current stdout and stderr, opens the target files, replaces the FDs,
/// runs the command, then restores the original FDs.
fn run_command_with_redirections(cmd: &str, redir: RedirectionSpec) {
    use std::fs::OpenOptions;
    use std::os::unix::io::AsRawFd;
    use nix::unistd::{dup, dup2, close};

    // Save original stdout and stderr FDs.
    let stdout_fd = dup(libc::STDOUT_FILENO).expect("dup failed for stdout");
    let stderr_fd = dup(libc::STDERR_FILENO).expect("dup failed for stderr");

    // Redirect stdout if specified.
    if let Some((ref file, append)) = redir.stdout {
        let file_handle = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(file)
            .expect("failed to open stdout redirection file");
        dup2(file_handle.as_raw_fd(), libc::STDOUT_FILENO).expect("dup2 failed for stdout");
    }

    // Redirect stderr if specified.
    if let Some((ref file, append)) = redir.stderr {
        let file_handle = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(file)
            .expect("failed to open stderr redirection file");
        dup2(file_handle.as_raw_fd(), libc::STDERR_FILENO).expect("dup2 failed for stderr");
    }

    // Execute the command.
    process_command(cmd);

    // Flush outputs.
    io::stdout().flush().unwrap();
    io::stderr().flush().unwrap();

    // Restore original stdout and stderr.
    dup2(stdout_fd, libc::STDOUT_FILENO).expect("dup2 restore failed for stdout");
    dup2(stderr_fd, libc::STDERR_FILENO).expect("dup2 restore failed for stderr");
    close(stdout_fd).ok();
    close(stderr_fd).ok();
}

// ----------------------------------------------------------------
// Command Processing
// ----------------------------------------------------------------

/// Dispatches a command (without redirections) to the appropriate builtin or external handler.
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

// ----------------------------------------------------------------
// REPL Loop
// ----------------------------------------------------------------

/// Main entry point for the shell.
/// It reads user input, checks for redirection operators, and dispatches commands.
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

        // If the input contains any redirection operator, parse and run with redirection.
        if trimmed_input.contains('>') {
            let (cmd_part, redir_spec) = parse_command_and_redirections(trimmed_input);
            run_command_with_redirections(&cmd_part, redir_spec);
        } else {
            process_command(trimmed_input);
        }
    }
}
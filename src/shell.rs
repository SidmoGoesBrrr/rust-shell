use nix::libc;
use std::io::{self, Write};

// Import your command handlers.
use crate::commands::echo::handle_echo_command;
use crate::commands::cd::handle_cd_command;
use crate::commands::type_cmd::handle_type_command;
use crate::commands::pwd::handle_pwd_command;
use crate::commands::execute::handle_execute_command;
use crate::commands::exit::handle_exit_command;

// --------------------- Redirection Support ---------------------

#[derive(Debug)]
struct RedirectionSpec {
    stdout: Option<(String, bool)>, // (filename, append) for stdout
    stderr: Option<(String, bool)>, // (filename, append) for stderr
}

/// Parses the input line into the command part and a redirection specification.
/// Assumes that redirection tokens (>, >>, 2>, 2>>) are separated by whitespace.
fn parse_command_and_redirections(input: &str) -> (String, RedirectionSpec) {
    let mut redir = RedirectionSpec { stdout: None, stderr: None };
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut cmd_tokens = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            "2>>" => {
                if i + 1 < tokens.len() {
                    redir.stderr = Some((tokens[i + 1].to_string(), true));
                    i += 2;
                } else { break; }
            },
            "2>" => {
                if i + 1 < tokens.len() {
                    redir.stderr = Some((tokens[i + 1].to_string(), false));
                    i += 2;
                } else { break; }
            },
            "1>>" | ">>" => {
                if i + 1 < tokens.len() {
                    redir.stdout = Some((tokens[i + 1].to_string(), true));
                    i += 2;
                } else { break; }
            },
            "1>" | ">" => {
                if i + 1 < tokens.len() {
                    redir.stdout = Some((tokens[i + 1].to_string(), false));
                    i += 2;
                } else { break; }
            },
            token => {
                cmd_tokens.push(token);
                i += 1;
            }
        }
    }
    (cmd_tokens.join(" "), redir)
}

/// Executes a command with redirections applied.
fn run_command_with_redirections(cmd: &str, redir: RedirectionSpec) {
    use std::fs::OpenOptions;
    use std::os::unix::io::AsRawFd;
    use nix::unistd::{dup, dup2, close};

    // Save original stdout and stderr.
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
        dup2(file_handle.as_raw_fd(), libc::STDOUT_FILENO)
            .expect("dup2 failed for stdout");
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
        dup2(file_handle.as_raw_fd(), libc::STDERR_FILENO)
            .expect("dup2 failed for stderr");
    }

    process_command(cmd);

    io::stdout().flush().unwrap();
    io::stderr().flush().unwrap();

    // Restore original stdout and stderr.
    dup2(stdout_fd, libc::STDOUT_FILENO)
        .expect("dup2 restore failed for stdout");
    dup2(stderr_fd, libc::STDERR_FILENO)
        .expect("dup2 restore failed for stderr");
    close(stdout_fd).ok();
    close(stderr_fd).ok();
}

// --------------------- Command Processing ---------------------

/// Dispatches a command (without redirections) to the appropriate builtin or external handler.
fn process_command(cmd: &str) {
    if handle_echo_command(cmd) { return; }
    if handle_cd_command(cmd) { return; }
    if handle_type_command(cmd) { return; }
    if handle_pwd_command(cmd) { return; }
    if handle_execute_command(cmd) { return; }
    if handle_exit_command(cmd) { return; }
    println!("{}: command not found", cmd);
}

// --------------------- Rustyline Autocompletion ---------------------

// Import rustyline traits from their private submodules.
use rustyline::completion::{Completer, Candidate};
use rustyline::error::ReadlineError;
use rustyline::{Editor, Context, Helper};
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;

#[derive(Debug)]
struct MyCandidate(String);
impl Candidate for MyCandidate {
    fn display(&self) -> &str { &self.0 }
    fn replacement(&self) -> &str { &self.0 }
}

struct MyHelper;
impl Completer for MyHelper {
    type Candidate = MyCandidate;
    fn complete(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<MyCandidate>)> {
        // Only complete when the input is a single token.
        let builtins = ["echo", "exit"];
        if line.contains(' ') {
            return Ok((0, Vec::new()));
        }
        let candidates: Vec<MyCandidate> = builtins.iter()
            .filter(|&&cmd| cmd.starts_with(line) && cmd != line)
            .map(|&cmd| MyCandidate(format!("{} ", cmd)))
            .collect();
        Ok((0, candidates))
    }
}
impl Hinter for MyHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> { None }
}
impl Highlighter for MyHelper {}
impl Validator for MyHelper {}
impl Helper for MyHelper {}

// --------------------- REPL Loop using Rustyline ---------------------

pub fn start_shell() {
    // Note: Editor requires two generic arguments. We use the default history.
    let mut rl = Editor::<MyHelper, rustyline::history::DefaultHistory>::new().unwrap();
    rl.set_helper(Some(MyHelper));
    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let trimmed = line.trim_end_matches('\n');
                if trimmed.is_empty() { continue; }
                if trimmed.contains('>') {
                    let (cmd_part, redir_spec) = parse_command_and_redirections(trimmed);
                    run_command_with_redirections(&cmd_part, redir_spec);
                } else {
                    process_command(trimmed);
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                continue;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
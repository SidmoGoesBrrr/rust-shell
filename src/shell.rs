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

fn process_command(cmd: &str) {
    if handle_echo_command(cmd) { return; }
    if handle_cd_command(cmd) { return; }
    if handle_type_command(cmd) { return; }
    if handle_pwd_command(cmd) { return; }
    if handle_execute_command(cmd) { return; }
    if handle_exit_command(cmd) { return; }
    println!("{}: command not found", cmd);
}

// --------------------- External Command Completion Helpers ---------------------

fn get_external_candidates(prefix: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    if let Ok(path_var) = std::env::var("PATH") {
        for path in path_var.split(':') {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            if let Ok(name) = entry.file_name().into_string() {
                                if name.starts_with(prefix) {
                                    #[cfg(unix)]
                                    {
                                        use std::os::unix::fs::PermissionsExt;
                                        if metadata.permissions().mode() & 0o111 != 0 {
                                            candidates.push(name);
                                        }
                                    }
                                    #[cfg(not(unix))]
                                    {
                                        candidates.push(name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    candidates
}

fn longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() { return "".to_string(); }
    let mut prefix = strings[0].clone();
    for s in strings.iter().skip(1) {
        while !s.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() { break; }
        }
    }
    prefix
}

// --------------------- Rustyline Autocompletion ---------------------

use rustyline::completion::{Completer, Candidate};
use rustyline::error::ReadlineError;
use rustyline::{Editor, Context, Helper};
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use rustyline::history::DefaultHistory;
use std::cell::RefCell;

#[derive(Debug)]
struct MyCandidate(String);
impl Candidate for MyCandidate {
    fn display(&self) -> &str { &self.0 }
    fn replacement(&self) -> &str { &self.0 }
}

struct MyHelper {
    // Store state for autocompletion.
    last_input: RefCell<Option<String>>,
    completion_count: RefCell<usize>,
}
impl MyHelper {
    fn new() -> Self {
        MyHelper { last_input: RefCell::new(None), completion_count: RefCell::new(0) }
    }
}
impl Completer for MyHelper {
    type Candidate = MyCandidate;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &Context<'_>
    ) -> rustyline::Result<(usize, Vec<MyCandidate>)> {

        // Reset state if the line has changed.
        {
            let mut last = self.last_input.borrow_mut();
            if last.as_deref() != Some(line) {
                *last = Some(line.to_string());
                *self.completion_count.borrow_mut() = 0;
            }
        }
        *self.completion_count.borrow_mut() += 1;
        let count = *self.completion_count.borrow();

        // Built-in commands:
        let builtin_candidates: Vec<String> = ["echo", "exit"]
            .iter()
            .filter(|&&cmd| cmd.starts_with(line) && cmd != line)
            .map(|&s| s.to_string())
            .collect();

        // External commands:
        let mut external_candidates = get_external_candidates(line);
        external_candidates.sort();
        external_candidates.dedup();

        // Combine them:
        let mut all_candidates = builtin_candidates;
        all_candidates.extend(external_candidates);
        all_candidates.sort();
        all_candidates.dedup();

        if all_candidates.is_empty() {
            return Ok((0, Vec::new()));
        }

        // If there's exactly one candidate, return it with a trailing space:
        if all_candidates.len() == 1 {
            let candidate = &all_candidates[0];
            // Always append a space to match the test's expectation.
            return Ok((0, vec![MyCandidate(format!("{} ", candidate))]));
        }

        // Otherwise, see if there's a usable longest common prefix:
        let lcp = longest_common_prefix(&all_candidates);
        if lcp.len() > line.len() {
            // Provide the longest common prefix as the single completion
            return Ok((0, vec![MyCandidate(lcp)]));
        }

        // If no progress can be made, handle repeated TAB presses:
        if count == 1 {
            // On first TAB with multiple matches, beep:
            return Ok((0, Vec::new()));
        } else {
            println!(); // Blank line
            println!("{}", all_candidates.join("  "));
            
            // Re‐show prompt and the partial line
            print!("$ {}", line);
            std::io::stdout().flush().unwrap();
        
            *self.completion_count.borrow_mut() = 0;
            return Ok((0, Vec::new()));
        }
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
    let mut rl = Editor::<MyHelper, DefaultHistory>::new().unwrap();
    rl.set_helper(Some(MyHelper::new()));
    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let trimmed = line.trim_end_matches('\n').replace("\u{00A0}", " ");
                if trimmed.is_empty() { continue; }
                if trimmed.contains('>') {
                    let (cmd_part, redir_spec) = parse_command_and_redirections(&trimmed);
                    run_command_with_redirections(&cmd_part, redir_spec);
                } else {
                    process_command(&trimmed);
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
                break;use nix::libc;
                use std::io::{self, Write};
                
                // Import command handlers.
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
                
                /// Splits an input line into the command portion and a redirection specification.
                /// Assumes redirection tokens (>, >>, 2>, 2>>) are separated by whitespace.
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
                                } else { break; }
                            }
                            "2>" => {
                                if i + 1 < tokens.len() {
                                    redir.stderr = Some((tokens[i + 1].to_string(), false));
                                    i += 2;
                                } else { break; }
                            }
                            "1>>" | ">>" => {
                                if i + 1 < tokens.len() {
                                    redir.stdout = Some((tokens[i + 1].to_string(), true));
                                    i += 2;
                                } else { break; }
                            }
                            "1>" | ">" => {
                                if i + 1 < tokens.len() {
                                    redir.stdout = Some((tokens[i + 1].to_string(), false));
                                    i += 2;
                                } else { break; }
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
                fn run_command_with_redirections(cmd: &str, redir: RedirectionSpec) {
                    use std::fs::OpenOptions;
                    use std::os::unix::io::AsRawFd;
                    use nix::unistd::{dup, dup2, close};
                
                    // Save original stdout and stderr.
                    let stdout_fd = dup(libc::STDOUT_FILENO).expect("dup failed for stdout");
                    let stderr_fd = dup(libc::STDERR_FILENO).expect("dup failed for stderr");
                
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
                
                fn process_command(cmd: &str) {
                    if handle_echo_command(cmd) { return; }
                    if handle_cd_command(cmd) { return; }
                    if handle_type_command(cmd) { return; }
                    if handle_pwd_command(cmd) { return; }
                    if handle_execute_command(cmd) { return; }
                    if handle_exit_command(cmd) { return; }
                    println!("{}: command not found", cmd);
                }
                
                // --------------------- External Completion Helpers ---------------------
                
                /// Returns all external executable candidates in PATH matching the given prefix.
                fn get_external_candidates(prefix: &str) -> Vec<String> {
                    let mut candidates: Vec<String> = Vec::new();
                    if let Ok(path_var) = std::env::var("PATH") {
                        for dir in path_var.split(':') {
                            if let Ok(entries) = std::fs::read_dir(dir) {
                                for entry in entries.flatten() {
                                    if let Ok(metadata) = entry.metadata() {
                                        if metadata.is_file() {
                                            if let Ok(name) = entry.file_name().into_string() {
                                                if name.starts_with(prefix) {
                                                    #[cfg(unix)]
                                                    {
                                                        use std::os::unix::fs::PermissionsExt;
                                                        if metadata.permissions().mode() & 0o111 != 0 {
                                                            candidates.push(name);
                                                        }
                                                    }
                                                    #[cfg(not(unix))]
                                                    {
                                                        candidates.push(name);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    candidates
                }
                
                /// Returns the longest common prefix of a list of strings.
                fn longest_common_prefix(strings: &[String]) -> String {
                    if strings.is_empty() { return "".to_string(); }
                    let mut prefix = strings[0].clone();
                    for s in strings.iter().skip(1) {
                        while !s.starts_with(&prefix) {
                            prefix.pop();
                            if prefix.is_empty() { break; }
                        }
                    }
                    prefix
                }
                
                // --------------------- Rustyline Autocompletion ---------------------
                
                use rustyline::completion::{Completer, Candidate};
                use rustyline::error::ReadlineError;
                use rustyline::{Editor, Context, Helper};
                use rustyline::hint::Hinter;
                use rustyline::highlight::Highlighter;
                use rustyline::validate::Validator;
                use rustyline::history::DefaultHistory;
                use std::cell::RefCell;
                
                #[derive(Debug)]
                struct MyCandidate(String);
                impl Candidate for MyCandidate {
                    fn display(&self) -> &str { &self.0 }
                    fn replacement(&self) -> &str { &self.0 }
                }
                
                struct MyHelper {
                    // Store the last input and completion count for repeated TAB presses.
                    last_input: RefCell<Option<String>>,
                    completion_count: RefCell<usize>,
                }
                impl MyHelper {
                    fn new() -> Self {
                        MyHelper { last_input: RefCell::new(None), completion_count: RefCell::new(0) }
                    }
                }
                
                impl Completer for MyHelper {
                    type Candidate = MyCandidate;
                    fn complete(&self, line: &str, _pos: usize, _ctx: &Context<'_>) 
                        -> rustyline::Result<(usize, Vec<MyCandidate>)> 
                    {
                        // If the line is not a single token, do not complete.
                        if line.contains(' ') {
                            return Ok((0, Vec::new()));
                        }
                        // Reset state if input changed.
                        {
                            let mut last = self.last_input.borrow_mut();
                            if last.as_deref() != Some(line) {
                                *last = Some(line.to_string());
                                *self.completion_count.borrow_mut() = 0;
                            }
                        }
                        *self.completion_count.borrow_mut() += 1;
                        let count = *self.completion_count.borrow();
                
                        // Gather builtin candidates.
                        let mut all_candidates: Vec<String> = ["echo", "exit"]
                            .iter()
                            .filter(|&&cmd| cmd.starts_with(line) && cmd != line)
                            .map(|&s| s.to_string())
                            .collect();
                        // Gather external candidates.
                        let mut external_candidates = get_external_candidates(line);
                        all_candidates.append(&mut external_candidates);
                        all_candidates.sort();
                        all_candidates.dedup();
                
                        if all_candidates.is_empty() {
                            return Ok((0, Vec::new()));
                        }
                
                        // Compute the longest common prefix.
                        if all_candidates.len() == 1 {
                            let candidate = &all_candidates[0];
                            // Always append a normal space so the test sees a trailing space.
                            return Ok((0, vec![MyCandidate(format!("{} ", candidate))]));
                        }
                        let lcp = longest_common_prefix(&all_candidates);
                        if lcp.len() > line.len() {
                            // Multiple candidates: complete to the longest common prefix with no trailing space.
                            return Ok((0, vec![MyCandidate(lcp)]));
                        }
                        // Multiple matches with no further extension.
                        if count == 1 {
                            // First TAB: ring bell (return no candidates).
                            return Ok((0, Vec::new()));
                        } else {
                            // Second (or later) TAB: print all candidates, then return the current line.
                            println!();
                            println!("{}", all_candidates.join("  "));
                            *self.completion_count.borrow_mut() = 0;
                            return Ok((0, vec![MyCandidate(line.to_string())]));
                        }
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
                    use rustyline::Config;
                    let config = Config::builder()
                        .build();
                    let mut rl = Editor::<MyHelper, DefaultHistory>::with_config(config).unwrap();
                    rl.set_helper(Some(MyHelper::new()));
                    loop {
                        let readline = rl.readline("$ ");
                        match readline {
                            Ok(line) => {
                                rl.add_history_entry(line.as_str());
                                let trimmed = line.trim_end_matches('\n').replace("\u{00A0}", " ");
                                                                
                                if trimmed.is_empty() {
                                    continue;
                                }
                                if trimmed.contains('>') {
                                    let (cmd_part, redir_spec) = parse_command_and_redirections(&trimmed);
                                    run_command_with_redirections(&cmd_part, redir_spec);
                                } else {
                                    process_command(&trimmed);
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
            }
        }
    }
}
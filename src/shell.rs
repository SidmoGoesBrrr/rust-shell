use std::io::{self, Write};

use crate::commands::echo::handle_echo_command;
use crate::commands::cd::handle_cd_command;
use crate::commands::execute::handle_execute_command;
use crate::commands::type_cmd::handle_type_command;
use crate::commands::pwd::handle_pwd_command;
use crate::commands::exit::handle_exit_command;
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

        if handle_echo_command(trimmed_input) {
            continue;
        }
        if handle_cd_command(trimmed_input) {
            continue;
        }
        if handle_type_command(trimmed_input) {
            continue;
        }
        if handle_pwd_command(trimmed_input) {
            continue;
        }
        if handle_execute_command(trimmed_input) {
            continue;
        }
        if handle_exit_command(trimmed_input){
            continue;
        }

        println!("{}: command not found", trimmed_input);
    }
}
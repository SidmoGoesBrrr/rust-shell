use std::env;

pub fn handle_pwd_command(input: &str) -> bool {
    // Check if the command is exactly "pwd"
    if input.trim() == "pwd" {
        match env::current_dir() {
            Ok(path) => println!("{}", path.display()),
            Err(e) => eprintln!("pwd: error: {}", e),
        }
        return true;
    }
    false
}
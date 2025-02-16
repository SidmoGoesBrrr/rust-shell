use std::env;
use std::path::PathBuf;
use std::io::ErrorKind;

pub fn handle_cd_command(input: &str) -> bool {
    if let Some(rest) = input.strip_prefix("cd ") {
        let target = if rest == "~" || rest.starts_with("~/") {
            match env::var("HOME") {
                Ok(home) => {
                    let mut path = PathBuf::from(home);
                    if rest.len() > 1 {
                        // Skip the "~/" part and append the rest of the path.
                        path.push(&rest[2..]);
                    }
                    path
                }
                Err(_) => {
                    eprintln!("cd: HOME environment variable not set");
                    return true;
                }
            }
        } else {
            PathBuf::from(rest)
        };

        if let Err(e) = env::set_current_dir(&target) {
            if e.kind() == ErrorKind::NotFound {
                // Print a fixed error message that does not include the OS error text.
                eprintln!("cd: {}: No such file or directory", target.display());
            } else {
                // For other kinds of errors, you might want to show the full error.
                eprintln!("cd: {}: {}", target.display(), e);
            }
        }
        return true;
    }
    false
}
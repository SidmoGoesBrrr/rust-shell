#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

fn main() {
    
    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut input).unwrap();
        if input.trim()=="exit 0" {
            process::exit(0);
        }
        if input.trim().contains("echo") {
            println!("{}",input.trim().replace("echo ", ""));
            continue;
        }

        println!("{}: command not found", input.trim());
        

    }

}

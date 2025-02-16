use crate::util::parse_parameters;

pub fn handle_echo_command(input: &str) -> bool {
    if let Some(rest) = input.strip_prefix("echo ") {
        let args = parse_parameters(rest);
        // Join the tokens with a space (like the shell echo builtin)
        let result = args.join(" ");
        println!("{}", result);
        return true;
    }
    false
}
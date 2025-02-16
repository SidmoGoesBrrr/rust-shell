use regex::Regex;

/// Remove quotes (both single and double) from the beginning and end of a string.
pub fn remove_surrounding_quotes(argument: &str) -> String {
    argument.trim_matches(|c| c == '"' || c == '\'').to_string()
}

/// Check if an argument is surrounded by matching quotes.
pub fn is_surrounded_by_quotes(argument: &str) -> bool {
    (matches!(argument.chars().next(), Some('\'')) && matches!(argument.chars().last(), Some('\''))) ||
    (matches!(argument.chars().next(), Some('\"')) && matches!(argument.chars().last(), Some('\"')))
}

/// Parse the input into tokens. If quoted segments are adjacent (i.e. no whitespace between),
/// they are merged together.
pub fn parse_parameters(input: &str) -> Vec<String> {
    // This regex matches:
    //  - a single-quoted substring, or
    //  - a double-quoted substring, or
    //  - an unquoted token (one or more non-whitespace characters)
    let re = Regex::new(r#"'([^']*)'|"([^"]*)"|(\S+)"#).unwrap();
    // Collect tokens as (start_index, end_index, token_string)
    let mut tokens: Vec<(usize, usize, String)> = Vec::new();
    for cap in re.captures_iter(input) {
        let m = cap.get(0).unwrap();
        let start = m.start();
        let end = m.end();
        let token = if let Some(t) = cap.get(1) {
            t.as_str().to_string()
        } else if let Some(t) = cap.get(2) {
            t.as_str().to_string()
        } else if let Some(t) = cap.get(3) {
            t.as_str().to_string()
        } else {
            String::new()
        };
        tokens.push((start, end, token));
    }
    // Merge adjacent tokens (where the next token's start equals the previous token's end)
    let mut merged: Vec<String> = Vec::new();
    if tokens.is_empty() {
        return merged;
    }
    let mut current = tokens[0].2.clone();
    let mut current_end = tokens[0].1;
    for &(start, end, ref token) in tokens.iter().skip(1) {
        if start == current_end {
            // Adjacent tokens, so merge them.
            current.push_str(token);
            current_end = end;
        } else {
            // Tokens are separated by whitespace: push the current token.
            merged.push(current);
            current = token.clone();
            current_end = end;
        }
    }
    merged.push(current);
    merged
}
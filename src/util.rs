pub fn parse_parameters(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_quotes: Option<char> = None; // None = not in quotes; Some(q) = in quotes with q

    while let Some(c) = chars.next() {
        if let Some(q) = in_quotes {
            // When inside quotes, do not treat backslashes specially.
            if c == q {
                in_quotes = None; // closing quote
            } else {
                current.push(c);
            }
        } else {
            // Not inside quotes.
            match c {
                // Begin a quoted segment.
                '"' | '\'' => {
                    in_quotes = Some(c);
                }
                // Outside quotes, backslash escapes the next character.
                '\\' => {
                    if let Some(&next_char) = chars.peek() {
                        current.push(next_char);
                        chars.next(); // consume escaped character
                    }
                }
                // Whitespace: if we have a token, finish it.
                c if c.is_whitespace() => {
                    if !current.is_empty() {
                        tokens.push(current);
                        current = String::new();
                    }
                    // Skip additional whitespace.
                }
                // Regular character.
                other => {
                    current.push(other);
                }
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}
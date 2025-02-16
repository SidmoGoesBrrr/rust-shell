pub fn parse_parameters(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_quotes: Option<char> = None; // None means we're not in a quote; Some(quote) means we are
    
    while let Some(c) = chars.next() {
        if let Some(q) = in_quotes {
            // Inside a quoted segment.
            if c == q {
                // End of quote.
                in_quotes = None;
            } else {
                // In quotes, we do NOT process backslashes; they're literal.
                current.push(c);
            }
        } else {
            // Not inside quotes.
            match c {
                // Begin a quoted segment.
                '"' | '\'' => {
                    in_quotes = Some(c);
                },
                // Backslash escapes the next character.
                '\\' => {
                    if let Some(&next_char) = chars.peek() {
                        current.push(next_char);
                        chars.next(); // consume the escaped character
                    }
                },
                // Whitespace: if token is non-empty, finish it.
                c if c.is_whitespace() => {
                    if !current.is_empty() {
                        tokens.push(current);
                        current = String::new();
                    }
                    // Otherwise, skip consecutive whitespace.
                },
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
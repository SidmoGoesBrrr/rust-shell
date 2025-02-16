pub fn parse_parameters(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_quotes: Option<char> = None; // None means not in quotes; Some(q) means in a quoted segment with delimiter q

    while let Some(c) = chars.next() {
        if let Some(q) = in_quotes {
            // We're inside a quoted segment.
            if c == q {
                // End of quoted segment.
                in_quotes = None;
            } else if q == '"' && c == '\\' {
                // Inside double quotes, a backslash is special if followed by \, $, " or newline.
                if let Some(&next_char) = chars.peek() {
                    if next_char == '\\' || next_char == '$' || next_char == '"' || next_char == '\n' {
                        // Consume next char and push it.
                        current.push(chars.next().unwrap());
                    } else {
                        // Otherwise, the backslash is literal.
                        current.push(c);
                    }
                } else {
                    current.push(c);
                }
            } else {
                // For single quotes, or any other char inside double quotes that isn’t a special backslash, copy literally.
                current.push(c);
            }
        } else {
            // We're not inside any quotes.
            match c {
                '"' | '\'' => {
                    in_quotes = Some(c);
                },
                '\\' => {
                    // Outside quotes, backslash always escapes the next character.
                    if let Some(&next_char) = chars.peek() {
                        current.push(chars.next().unwrap());
                    }
                },
                c if c.is_whitespace() => {
                    if !current.is_empty() {
                        tokens.push(current);
                        current = String::new();
                    }
                    // Skip additional whitespace.
                },
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
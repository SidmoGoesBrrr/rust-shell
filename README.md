# Build Your Own Shell in Rust

[![progress-banner](https://backend.codecrafters.io/progress/shell/de61f23b-35bc-4472-9bff-bf7efd5e28b3)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

Welcome to **Build Your Own Shell in Rust** – a POSIX-compliant shell built in Rust as part of the [Codecrafters Shell Challenge](https://app.codecrafters.io/courses/shell/overview). This project showcases a fully functional shell with support for builtin commands, external command execution, quoting, redirection, and autocompletion.

## Features

- **Interactive REPL:**  
  An interactive prompt with history support and autocompletion.

- **Builtin Commands:**  
  - `cd` – Change directory (supports absolute, relative, and home directory shortcuts).  
  - `pwd` – Print the current working directory.  
  - `echo` – Print text with robust quoting support.  
  - `type` – Determine if a command is a builtin or an external executable.  
  - `exit` – Exit the shell.

- **External Command Execution:**  
  Run commands from your system’s PATH.

- **Quoting Support:**  
  Handles single and double quotes, as well as backslash escaping.

- **Redirection:**  
  Supports redirecting standard output and standard error (with both truncate and append modes) using operators like `>`, `>>`, `2>`, and `2>>`.

- **Autocompletion:**  
  Builtin command autocompletion for commands like `echo` and `exit` using [rustyline](https://crates.io/crates/rustyline).

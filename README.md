# 🐚 Rust Shell

A Unix-like shell implemented in **Rust**, built incrementally to closely match real shell behavior while keeping the implementation understandable and explicit.

This project was developed stage-by-stage, focusing on **correct semantics** rather than shortcuts provided by existing libraries.

---

## ✨ Features

### ✅ Core Execution
- Execute external commands via `$PATH`
- Built-in commands:
  - `echo`
  - `cd`
  - `pwd`
  - `type`
  - `exit`
  - `history`

---

### 📜 History Management
- In-memory command history
- Plain-text history persistence (POSIX-style, no metadata headers)
- Supports:
  - `history` — show full history
  - `history N` — show last `N` entries (correct global numbering)
  - `history -r FILE` — read history from file
  - `history -w FILE` — write history to file
  - `history -a FILE` — append new entries only
- Honors `$HISTFILE` on startup
- Matches bash-style numbering and behavior

---

### 🔗 Pipelines
- Supports **multi-command pipelines**:
  ```sh
  cmd1 | cmd2 | cmd3 | ...


## ⚠️ Current Limitations

The shell currently implements a **subset of basic bash features**.  
The following are still not supported

- **No logical operators**
  - `&&` and `||` are not supported
  - Expressions like `cmd1 && cmd2` or `cmd1 || cmd2` result in error

- **No background execution**
  - `&` is not supported
  - All commands run in the foreground

- **No command substitution**
  - Constructs like `$(command)` or `` `command` `` are not supported

will implement them ass soon ass possible 

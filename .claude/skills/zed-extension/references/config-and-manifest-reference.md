# Configuration and Manifest Reference

Complete reference for the three configuration files in a Zed extension: `extension.toml` (manifest),
`languages/<lang>/config.toml` (language metadata), and `Cargo.toml` (Rust build config).

## Table of Contents

1. [extension.toml](#extensiontoml)
1. [config.toml](#configtoml)
1. [Cargo.toml](#cargotoml)

______________________________________________________________________

## extension.toml

The manifest file. Tells Zed what the extension provides.

### Required Fields

```toml
id = "musa"                        # Unique ID: lowercase, numbers, hyphens only
name = "Musa"                      # Display name
version = "0.1.10"                # Semver
schema_version = 1                # Always 1
authors = ["Jacob Reinhold"]      # Author list
description = "Language support for Musa"
repository = "https://github.com/jcreinhold/musa"
```

**Naming constraints:**

- `id` must match `/^[a-z0-9\-]+$/`
- Neither `id` nor `name` may contain "zed" or "extension"
- Use `-lang` suffix for language extensions if needed to disambiguate

### Grammars

```toml
[grammars.musa]
repository = "https://github.com/jcreinhold/musa"
rev = "abc123def456789..."     # Full commit SHA (required for publishing)
```

With subdirectory path (when grammar lives inside a larger repo):

```toml
[grammars.musa]
repository = "https://github.com/jcreinhold/musa"
rev = "abc123def456789..."
path = "editors/tree-sitter-musa"
```

Local development with `file://` URL:

```toml
[grammars.musa]
repository = "file:///Users/you/Code/musa"
rev = "abc123..."              # Must be a committed SHA
path = "editors/tree-sitter-musa"
```

**What happens at install time:**

1. Zed clones `repository` into `grammars/musa/`
1. Checks out `rev`
1. If `path` is set, enters that subdirectory
1. Compiles `src/parser.c` (and `src/scanner.c` if present) to WASM via wasi-sdk's clang

**Caveats:**

- Branch names work but are unreliable — always use commit SHAs for distribution
- Uncommitted changes are invisible to `file://` clones
- The rev must point at a commit that contains **all** generated files: `src/parser.c`, `src/grammar.json`, and
    `src/node-types.json`. Forgetting `grammar.json` or `node-types.json` causes silent failures.
- The commit sequence is always: (1) `tree-sitter generate`, (2) commit the three generated files + query changes, (3)
    update `extension.toml` rev to that commit SHA, (4) commit `extension.toml`
- Multiple grammars can be declared (e.g., `[grammars.musa]`, `[grammars.musa-markdown]`)

### Language Servers

```toml
[language_servers.musa-lsp]
name = "Musa Language Server"
languages = ["Musa"]              # Must match config.toml `name` field
```

With language ID mapping (for polyglot LSPs):

```toml
[language_servers.my-lsp.language_ids]
"Musa" = "musa"
"Markdown" = "markdown"
```

Multiple language servers can be declared for the same extension.

### Capabilities

```toml
# Execute any command (broad)
[[capabilities]]
kind = "process:exec"
command = "*"
args = ["**"]

# Execute specific command (narrow)
[[capabilities]]
kind = "process:exec"
command = "musa-lsp"
args = ["**"]

# Download from specific host
[[capabilities]]
kind = "download_file"
host = "github.com"
path = ["**"]

# Install npm packages
[[capabilities]]
kind = "npm:install"
package = "typescript-language-server"
```

Prefer narrow capabilities. Zed prompts the user to approve broad capabilities.

### Slash Commands

```toml
[slash_commands.my-cmd]
description = "Does a thing"
requires_argument = true
```

### Context Servers

```toml
[context_servers.my-mcp]
# Registered name — implementation in Extension trait methods
```

### Snippets

```toml
snippets = ["./snippets/musa.json", "./snippets/snippets.json"]
```

File naming: language-specific files use the lowercase language name (`musa.json` for Musa). `snippets.json` provides
universal snippets.

______________________________________________________________________

## config.toml

Lives at `languages/<lang>/config.toml`. Configures language behavior in the editor.

### Required Fields

```toml
name = "Musa"                     # Display name — must match language_servers.*.languages
grammar = "musa"                  # Must match a [grammars.<name>] in extension.toml
path_suffixes = ["musa"]          # File extensions (no dots, no globs)
```

### Comment Syntax

```toml
line_comments = ["// "]          # For Cmd+/ toggle
block_comments = [["/*", "*/"]]  # For block comment toggle
```

Note: include the trailing space in line comments — Zed inserts the string literally.

### Tab Settings

```toml
tab_size = 4                     # Default: 4
hard_tabs = false                # true = tabs, false = spaces
```

### Bracket Pairing

```toml
brackets = [
  { start = "(", end = ")", close = true, newline = true },
  { start = "{", end = "}", close = true, newline = true },
  { start = "[", end = "]", close = true, newline = true },
  { start = "\"", end = "\"", close = true, newline = false, not_in = ["string"] },
  { start = "/*", end = " */", close = true, newline = false, not_in = ["comment", "string"] },
]
```

Fields:

- `start` / `end` — the bracket pair
- `close` — auto-insert `end` when `start` is typed
- `newline` — add a newline between brackets when Enter is pressed
- `not_in` — suppress auto-close inside these syntax scopes (from tree-sitter)

### Indentation Patterns

```toml
autoclose_before = ";:.,=}])>\n\t"    # Only auto-close before these characters
increase_indent_pattern = "\\{\\s*$|=>\\s*$"  # Regex: increase indent after match
decrease_indent_pattern = "^\\s*\\}"          # Regex: decrease indent before match
```

These are fallbacks — `indents.scm` takes precedence when present.

### Other Fields

```toml
first_line_pattern = "^#!.*musa"  # Shebang detection regex
collapsed_placeholder = "/* ... */"  # Text shown for collapsed code blocks
word_characters = ["-"]          # Additional characters treated as part of words
```

### Scope Overrides

```toml
[overrides.string]
word_characters = ["-"]
completion_query_characters = ["-"]
disable_autoclose = true

[overrides.comment]
word_characters = ["-"]
```

Override editor behavior inside specific tree-sitter scopes.

### Debugger Priority

```toml
debuggers = ["lldb", "gdb"]      # Preferred debugger order
```

______________________________________________________________________

## Cargo.toml

Standard Rust project config with Zed-specific requirements.

### Minimal Template

```toml
[package]
name = "zed-musa"
version = "0.1.10"
edition = "2024"
publish = false
license = "MIT OR Apache-2.0"
authors = ["Jacob Reinhold"]
description = "Musa LSP extension for Zed"

# REQUIRED: Isolate from parent workspace
[workspace]

[lib]
crate-type = ["cdylib"]          # REQUIRED: produces WASM module

[dependencies]
zed_extension_api = "0.7.0"     # Only required dependency

[profile.release]
lto = true                       # Link-time optimization
opt-level = "z"                  # Optimize for binary size
strip = true                     # Remove debug symbols
```

### Critical Requirements

| Requirement                    | Why                                                  |
| ------------------------------ | ---------------------------------------------------- |
| `crate-type = ["cdylib"]`      | Produces the `.wasm` module Zed loads                |
| `[workspace]` (empty)          | Prevents Cargo from merging into parent workspace    |
| `zed_extension_api` dependency | Provides Extension trait + register_extension! macro |

### Optional Dependencies

- `serde` + `serde_json` — already re-exported by `zed_extension_api`, but you may need direct access for complex JSON
    manipulation
- No other dependencies are typically needed — keep the WASM binary small

### Build Command

```bash
cargo build --target wasm32-wasip2 --release
```

Output: `target/wasm32-wasip2/release/<crate_name>.wasm`

**Prerequisites:**

- Rust installed via **rustup** (not homebrew — homebrew Rust cannot compile to WASM targets)
- WASM target installed: `rustup target add wasm32-wasip2`

### WASM Target History

| Target          | Status                                       |
| --------------- | -------------------------------------------- |
| `wasm32-wasi`   | Removed in Rust 1.84 (Jan 2025) — do not use |
| `wasm32-wasip1` | Works but deprecated in favor of wasip2      |
| `wasm32-wasip2` | Current target — use this                    |

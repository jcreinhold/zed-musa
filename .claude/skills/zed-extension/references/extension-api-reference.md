# Zed Extension API Reference

Complete reference for `zed_extension_api` v0.7.0 (latest published). The crate wraps WIT-generated bindings into an
idiomatic Rust `Extension` trait.

## Table of Contents

1. [Extension Trait](#extension-trait)
1. [Key Types](#key-types)
1. [Free Functions](#free-functions)
1. [Settings Module](#settings-module)
1. [HTTP Client Module](#http-client-module)
1. [Process Module](#process-module)

______________________________________________________________________

## Extension Trait

One required method (`new`), all others have default implementations that return errors or `None`.

### Required

```rust
fn new() -> Self where Self: Sized;
```

Called once when Zed loads the extension. Initialize any state here.

### Language Server Methods

#### `language_server_command`

```rust
fn language_server_command(
    &mut self,
    language_server_id: &LanguageServerId,
    worktree: &Worktree,
) -> Result<Command>
```

Returns the command to start the language server. Called each time Zed needs to (re)start the LSP for a worktree. The
most commonly implemented method after `new`.

**Typical pattern:**

```rust
fn language_server_command(
    &mut self,
    _id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
) -> Result<zed::Command> {
    // Check user settings first
    let settings = zed::settings::LspSettings::for_worktree("my-lsp", worktree)?;
    let command = settings.binary.as_ref()
        .and_then(|b| b.path.clone())
        .or_else(|| worktree.which("my-lsp"))
        .ok_or_else(|| "my-lsp not found in PATH".to_string())?;

    let args = settings.binary.as_ref()
        .and_then(|b| b.arguments.clone())
        .unwrap_or_default();

    Ok(zed::Command {
        command,
        args,
        env: worktree.shell_env(),
    })
}
```

#### `language_server_initialization_options`

```rust
fn language_server_initialization_options(
    &mut self,
    language_server_id: &LanguageServerId,
    worktree: &Worktree,
) -> Result<Option<serde_json::Value>>
```

Returns JSON `initializationOptions` for the LSP `initialize` request. Called once per LSP session startup.

**Pattern — forward user settings:**

```rust
fn language_server_initialization_options(
    &mut self,
    _id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
) -> Result<Option<serde_json::Value>> {
    let settings = zed::settings::LspSettings::for_worktree("my-lsp", worktree)?;
    Ok(settings.initialization_options)
}
```

#### `language_server_workspace_configuration`

```rust
fn language_server_workspace_configuration(
    &mut self,
    language_server_id: &LanguageServerId,
    worktree: &Worktree,
) -> Result<Option<serde_json::Value>>
```

Returns JSON workspace configuration sent via `workspace/didChangeConfiguration`. Called when Zed's settings change.

**Pattern — forward user settings:**

```rust
fn language_server_workspace_configuration(
    &mut self,
    _id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
) -> Result<Option<serde_json::Value>> {
    let settings = zed::settings::LspSettings::for_worktree("my-lsp", worktree)?;
    Ok(settings.settings)
}
```

#### `language_server_additional_initialization_options`

```rust
fn language_server_additional_initialization_options(
    &mut self,
    language_server_id: &LanguageServerId,
    target_language_server_id: &LanguageServerId,
    worktree: &Worktree,
) -> Result<Option<serde_json::Value>>
```

Lets one extension inject init options into a _different_ language server. Rarely needed — used when an extension needs
to configure another extension's LSP.

#### `language_server_additional_workspace_configuration`

Same as above but for workspace configuration.

### Label Methods

#### `label_for_completion`

```rust
fn label_for_completion(
    &self,
    language_server_id: &LanguageServerId,
    completion: Completion,
) -> Option<CodeLabel>
```

Renders a completion item with syntax highlighting in the completion menu. Without this, completions show as plain text.

**Pattern — from the Haskell extension:**

```rust
fn label_for_completion(
    &self,
    _id: &zed::LanguageServerId,
    completion: zed::lsp::Completion,
) -> Option<zed::CodeLabel> {
    let highlight_name = match completion.kind? {
        CompletionKind::Function | CompletionKind::Method => "function",
        CompletionKind::Keyword => "keyword",
        CompletionKind::Class | CompletionKind::Interface => "type",
        CompletionKind::Constructor | CompletionKind::Enum => "constructor",
        CompletionKind::Field | CompletionKind::Property => "property",
        CompletionKind::Module => "module",
        _ => return None,
    };

    Some(zed::CodeLabel {
        spans: vec![zed::CodeLabelSpan::literal(
            &completion.label,
            Some(highlight_name.to_string()),
        )],
        filter_range: (0..completion.label.len()).into(),
        code: completion.label.clone(),
    })
}
```

#### `label_for_symbol`

```rust
fn label_for_symbol(
    &self,
    language_server_id: &LanguageServerId,
    symbol: Symbol,
) -> Option<CodeLabel>
```

Renders workspace symbols with syntax highlighting. Same `CodeLabel` return type as `label_for_completion`.

### Slash Command Methods

#### `complete_slash_command_argument`

```rust
fn complete_slash_command_argument(
    &self,
    command: SlashCommand,
    args: Vec<String>,
) -> Result<Vec<SlashCommandArgumentCompletion>, String>
```

Provides autocompletion for slash command arguments in the Assistant panel.

#### `run_slash_command`

```rust
fn run_slash_command(
    &self,
    command: SlashCommand,
    args: Vec<String>,
    worktree: Option<&Worktree>,
) -> Result<SlashCommandOutput, String>
```

Executes a slash command and returns text output with labeled sections.

### Context Server Methods

#### `context_server_command`

```rust
fn context_server_command(
    &mut self,
    context_server_id: &ContextServerId,
    project: &Project,
) -> Result<Command>
```

Returns the command to start an MCP (Model Context Protocol) server.

#### `context_server_configuration`

```rust
fn context_server_configuration(
    &mut self,
    context_server_id: &ContextServerId,
    project: &Project,
) -> Result<Option<ContextServerConfiguration>>
```

Returns configuration including installation instructions (Markdown), JSON Schema for settings, and default settings.

### Debug Adapter Methods

Five methods for DAP integration: `get_dap_binary`, `dap_request_kind`, `dap_config_to_scenario`,
`dap_locator_create_scenario`, `run_dap_locator`. See the `zed_extension_api` docs for full signatures — these are only
relevant for debugger extensions.

### Documentation Methods

`suggest_docs_packages` and `index_docs` — for extensions that index documentation into a `KeyValueStore`. Rarely
implemented.

______________________________________________________________________

## Key Types

### Command

```rust
pub struct Command {
    pub command: String,     // Binary path or name
    pub args: Vec<String>,   // Command-line arguments
    pub env: EnvVars,        // Environment variables (Vec<(String, String)>)
}
```

### Worktree

Resource handle with methods:

| Method                 | Returns          | Purpose                               |
| ---------------------- | ---------------- | ------------------------------------- |
| `id()`                 | `u64`            | Unique worktree identifier            |
| `root_path()`          | `String`         | Absolute path to worktree root        |
| `read_text_file(path)` | `Result<String>` | Read a file relative to worktree root |
| `which(binary_name)`   | `Option<String>` | Find binary in shell PATH             |
| `shell_env()`          | `EnvVars`        | Shell environment variables           |

### Project

Resource with `worktree_ids() -> Vec<u64>`.

### CodeLabel

```rust
pub struct CodeLabel {
    pub code: String,                    // Full code text for parsing
    pub spans: Vec<CodeLabelSpan>,       // How to render segments
    pub filter_range: Range<usize>,      // Range for fuzzy matching
}

pub enum CodeLabelSpan {
    CodeRange(Range<usize>),              // Highlight a range of `code`
    Literal(CodeLabelSpanLiteral),        // Literal text with optional highlight
}
```

Helper: `CodeLabelSpan::literal(text, highlight_name)` creates a `Literal` span.

### Completion (from LSP)

```rust
pub struct Completion {
    pub label: String,
    pub label_details: Option<CompletionLabelDetails>,
    pub kind: Option<CompletionKind>,
    pub insert_text_format: Option<InsertTextFormat>,
}
```

### Symbol (from LSP)

```rust
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
}
```

### SlashCommand

```rust
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub tooltip_text: String,
    pub requires_argument: bool,
}
```

### SlashCommandOutput

```rust
pub struct SlashCommandOutput {
    pub text: String,
    pub sections: Vec<SlashCommandOutputSection>,
}

pub struct SlashCommandOutputSection {
    pub range: Range<usize>,
    pub label: String,
}
```

### GithubRelease / GithubReleaseAsset

```rust
pub struct GithubRelease {
    pub version: String,
    pub assets: Vec<GithubReleaseAsset>,
}

pub struct GithubReleaseAsset {
    pub name: String,
    pub download_url: String,
}
```

### LanguageServerInstallationStatus

```rust
pub enum LanguageServerInstallationStatus {
    None,
    Downloading,
    CheckingForUpdate,
    Failed(String),
}
```

### Os / Architecture (from `current_platform()`)

```rust
pub enum Os { Mac, Linux, Windows }
pub enum Architecture { Aarch64, X86, X8664 }
```

______________________________________________________________________

## Free Functions

### Platform

```rust
pub fn current_platform() -> (Os, Architecture)
```

Returns the host OS and architecture. Use for platform-specific binary downloads.

### File Operations

```rust
pub fn download_file(
    url: &str,
    file_path: &str,        // Relative to extension working directory
    file_type: DownloadedFileType,  // Uncompressed, Gzip, GzipTar, Zip
) -> Result<()>
```

Downloads a file into the extension's working directory. The `file_type` determines extraction behavior.

```rust
pub fn make_file_executable(path: &str) -> Result<()>
```

Sets the executable bit on a downloaded file. Required after downloading binary language servers.

### GitHub

```rust
pub fn latest_github_release(
    repo: &str,                    // "owner/repo"
    options: GithubReleaseOptions, // { require_assets: bool, pre_release: bool }
) -> Result<GithubRelease>

pub fn github_release_by_tag_name(
    repo: &str,
    tag: &str,
    options: GithubReleaseOptions,
) -> Result<GithubRelease>
```

### Node.js

```rust
pub fn node_binary_path() -> Result<String>
pub fn npm_install_package(name: &str, version: &str) -> Result<()>
pub fn npm_package_installed_version(name: &str) -> Result<Option<String>>
pub fn npm_package_latest_version(name: &str) -> Result<String>
```

### UI Status

```rust
pub fn set_language_server_installation_status(
    language_server_id: &LanguageServerId,
    status: &LanguageServerInstallationStatus,
)
```

Updates the status indicator shown in Zed while the LSP is being installed or started.

______________________________________________________________________

## Settings Module

### LspSettings

```rust
let settings = zed::settings::LspSettings::for_worktree("server-name", worktree)?;
```

Fields:

```rust
pub struct LspSettings {
    pub binary: Option<BinarySettings>,
    pub initialization_options: Option<serde_json::Value>,
    pub settings: Option<serde_json::Value>,
}

pub struct BinarySettings {
    pub path: Option<String>,
    pub arguments: Option<Vec<String>>,
}
```

Users configure these in Zed's `settings.json`:

```json
{
  "lsp": {
    "musa-lsp": {
      "binary": {
        "path": "/usr/local/bin/musa-lsp",
        "arguments": ["--verbose"]
      },
      "initialization_options": { "checkOnSave": true },
      "settings": { "diagnostics": { "enabled": true } }
    }
  }
}
```

### LanguageSettings

```rust
let lang = zed::settings::LanguageSettings::for_worktree(Some("Kan"), worktree)?;
```

Returns language-scoped editor settings (tab size, formatter, etc.).

### ContextServerSettings

```rust
let cs = zed::settings::ContextServerSettings::for_project("server-name", project)?;
```

______________________________________________________________________

## HTTP Client Module

Full HTTP client for extensions that need to make network requests (e.g., downloading language servers, querying APIs).

```rust
use zed_extension_api::http_client;

let response = http_client::fetch(&http_client::HttpRequest {
    url: "https://api.example.com/data".to_string(),
    method: http_client::HttpMethod::Get,
    headers: vec![("Accept".to_string(), "application/json".to_string())],
    body: None,
    redirect_policy: http_client::RedirectPolicy::FollowAll,
})?;

// response.status: u16
// response.headers: Vec<(String, String)>
// response.body: Vec<u8>
```

Streaming variant: `http_client::fetch_stream()` returns an `HttpResponseStream` with
`fn next_chunk() -> Result<Option<Vec<u8>>>`.

______________________________________________________________________

## Process Module

Execute commands from within the WASM sandbox (requires `process:exec` capability):

```rust
use zed_extension_api::process;

let output = process::Command::new("my-binary")
    .args(&["--version"])
    .env("MY_VAR", "value")
    .output()?;

// output.status: Option<i32>
// output.stdout: String
// output.stderr: String
```

______________________________________________________________________

## The `register_extension!` Macro

Must be called exactly once at the end of `lib.rs`:

```rust
zed::register_extension!(MyExtension);
```

This generates the WASM entry point (`init-extension`) and wires up all trait method dispatch. Without it, the extension
will compile but Zed will fail to load it.

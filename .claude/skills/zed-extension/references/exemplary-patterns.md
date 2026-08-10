# Exemplary Patterns from Zed Extensions

Concrete patterns extracted from mature Zed language extensions: Haskell, Lean4, OCaml, and Typst. Use these as
templates when building similar features.

## Table of Contents

1. [LSP Binary Resolution](#lsp-binary-resolution)
1. [Settings Forwarding](#settings-forwarding)
1. [Binary Caching](#binary-caching)
1. [Auto-Install from GitHub Releases](#auto-install-from-github-releases)
1. [Label Rendering](#label-rendering)
1. [Snippet Patterns](#snippet-patterns)
1. [Multi-Grammar Extensions](#multi-grammar-extensions)
1. [Status Reporting](#status-reporting)

______________________________________________________________________

## LSP Binary Resolution

### Tier 1: User Settings + PATH (Haskell pattern)

The simplest and most common approach. Check user-configured binary first, fall back to PATH search.

```rust
fn language_server_command(
    &mut self,
    _id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
) -> Result<zed::Command> {
    let settings = zed::settings::LspSettings::for_worktree("musa-lsp", worktree)?;

    let command = settings.binary.as_ref()
        .and_then(|b| b.path.clone())
        .or_else(|| worktree.which("musa-lsp"))
        .ok_or_else(|| {
            "musa-lsp not found in PATH.\n\
             Install with: cargo install --path crates/lsp\n\
             Or configure in Zed settings: \
             {\"lsp\": {\"musa-lsp\": {\"binary\": {\"path\": \"/path/to/musa-lsp\"}}}}"
                .to_string()
        })?;

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

**When to use:** When the language server is installed by the user (via cargo install, package manager, etc.). This is
the right pattern for `musa-lsp`.

### Tier 2: PATH + Tool Chain (Lean4 pattern)

When the LSP is part of a toolchain that may be installed in a non-standard location.

```rust
fn language_server_command(
    &mut self,
    _id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
) -> Result<zed::Command> {
    let settings = zed::settings::LspSettings::for_worktree("lean4", worktree)?;

    // Tier 1: User-configured binary
    if let Some(binary) = settings.binary.as_ref().and_then(|b| b.path.clone()) {
        let args = settings.binary.as_ref()
            .and_then(|b| b.arguments.clone())
            .unwrap_or_else(|| vec!["serve".to_string()]);
        return Ok(zed::Command {
            command: binary,
            args,
            env: worktree.shell_env(),
        });
    }

    // Tier 2: `lake` in PATH
    if let Some(lake_path) = worktree.which("lake") {
        return Ok(zed::Command {
            command: lake_path,
            args: vec!["serve".to_string()],
            env: worktree.shell_env(),
        });
    }

    // Tier 3: Toolchain-specific location
    if let Ok(elan_home) = std::env::var("ELAN_HOME") {
        let lake_path = format!("{elan_home}/bin/lake");
        return Ok(zed::Command {
            command: lake_path,
            args: vec!["serve".to_string()],
            env: worktree.shell_env(),
        });
    }

    Err("lake not found. Install elan: https://github.com/leanprover/elan".into())
}
```

**When to use:** When the LSP binary is part of a larger toolchain with its own installer (elan, rustup, ghcup, opam,
etc.).

### Tier 3: PATH + Cache + Download (Typst pattern)

When the extension manages LSP installation itself.

```rust
struct TypstExtension {
    cached_binary_path: Option<String>,
}

fn language_server_command(
    &mut self,
    _id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
) -> Result<zed::Command> {
    // Check PATH first
    if let Some(path) = worktree.which("tinymist") {
        return Ok(zed::Command {
            command: path,
            args: vec![],
            env: worktree.shell_env(),
        });
    }

    // Check cached download
    if let Some(ref path) = self.cached_binary_path {
        return Ok(zed::Command {
            command: path.clone(),
            args: vec![],
            env: worktree.shell_env(),
        });
    }

    // Download from GitHub releases
    let binary_path = self.download_lsp_binary()?;
    self.cached_binary_path = Some(binary_path.clone());

    Ok(zed::Command {
        command: binary_path,
        args: vec![],
        env: worktree.shell_env(),
    })
}
```

**When to use:** When you want to provide a zero-config experience where the extension auto-installs the LSP. Requires
`download_file` capability.

______________________________________________________________________

## Settings Forwarding

### Init Options (Lean4 pattern)

Forward user-configured initialization options to the LSP:

```rust
fn language_server_initialization_options(
    &mut self,
    _id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
) -> Result<Option<serde_json::Value>> {
    let settings = zed::settings::LspSettings::for_worktree("musa-lsp", worktree)?;
    Ok(settings.initialization_options)
}
```

### Workspace Configuration (Typst pattern)

Forward workspace settings, optionally merging defaults:

```rust
fn language_server_workspace_configuration(
    &mut self,
    _id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
) -> Result<Option<serde_json::Value>> {
    let settings = zed::settings::LspSettings::for_worktree("musa-lsp", worktree)?;

    // Return user settings directly, or merge with defaults
    Ok(settings.settings.or_else(|| {
        Some(serde_json::json!({
            "diagnostics": { "enabled": true }
        }))
    }))
}
```

______________________________________________________________________

## Binary Caching

Cache the resolved binary path in extension state to avoid repeated PATH lookups:

```rust
struct KanExtension {
    cached_binary: Option<String>,
}

impl zed::Extension for KanExtension {
    fn new() -> Self {
        Self { cached_binary: None }
    }

    fn language_server_command(
        &mut self,
        _id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let command = match &self.cached_binary {
            Some(path) => path.clone(),
            None => {
                let path = worktree.which("musa-lsp")
                    .ok_or("musa-lsp not found in PATH")?;
                self.cached_binary = Some(path.clone());
                path
            }
        };

        Ok(zed::Command {
            command,
            args: vec![],
            env: worktree.shell_env(),
        })
    }
}
```

**Caveat:** The cache persists for the lifetime of the extension instance (until Zed restarts or reloads the extension).
If the user installs the binary after the first failed lookup, they need to reload the extension.

______________________________________________________________________

## Auto-Install from GitHub Releases

Download a platform-specific binary from GitHub releases:

```rust
fn download_lsp_binary(&self) -> Result<String> {
    let (os, arch) = zed::current_platform();

    let asset_stem = match (os, arch) {
        (zed::Os::Mac, zed::Architecture::Aarch64) => "musa-lsp-aarch64-apple-darwin",
        (zed::Os::Mac, zed::Architecture::X8664) => "musa-lsp-x86_64-apple-darwin",
        (zed::Os::Linux, zed::Architecture::Aarch64) => "musa-lsp-aarch64-unknown-linux-gnu",
        (zed::Os::Linux, zed::Architecture::X8664) => "musa-lsp-x86_64-unknown-linux-gnu",
        _ => return Err("Unsupported platform".into()),
    };

    zed::set_language_server_installation_status(
        &zed::LanguageServerId::new("musa-lsp"),
        &zed::LanguageServerInstallationStatus::Downloading,
    );

    let release = zed::latest_github_release(
        "jcreinhold/musa",
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )?;

    let asset = release.assets.iter()
        .find(|a| a.name.starts_with(asset_stem))
        .ok_or_else(|| format!("No asset for {asset_stem}"))?;

    let binary_path = format!("musa-lsp-{}", release.version);
    zed::download_file(
        &asset.download_url,
        &binary_path,
        zed::DownloadedFileType::Gzip,
    )?;
    zed::make_file_executable(&binary_path)?;

    zed::set_language_server_installation_status(
        &zed::LanguageServerId::new("musa-lsp"),
        &zed::LanguageServerInstallationStatus::None,
    );

    Ok(binary_path)
}
```

______________________________________________________________________

## Label Rendering

### Completion Labels (OCaml pattern)

Rich completion rendering with syntax-highlighted code:

```rust
fn label_for_completion(
    &self,
    _id: &zed::LanguageServerId,
    completion: zed::lsp::Completion,
) -> Option<zed::CodeLabel> {
    match completion.kind? {
        // Functions: show as `fn name : type`
        zed::lsp::CompletionKind::Function => {
            let detail = completion.label_details.as_ref()?.detail.as_ref()?;
            let code = format!("fn {} : {}", completion.label, detail);
            let name_len = completion.label.len();
            Some(zed::CodeLabel {
                code,
                spans: vec![
                    zed::CodeLabelSpan::code_range(3..3 + name_len),  // name
                    zed::CodeLabelSpan::literal(" : ", None),
                    zed::CodeLabelSpan::code_range(3 + name_len + 3..code.len()),  // type
                ],
                filter_range: (0..name_len).into(),
            })
        }

        // Types: show as `type Name`
        zed::lsp::CompletionKind::Class
        | zed::lsp::CompletionKind::Interface
        | zed::lsp::CompletionKind::Struct => {
            let code = format!("data {}", completion.label);
            Some(zed::CodeLabel {
                spans: vec![zed::CodeLabelSpan::code_range(5..code.len())],
                filter_range: (0..completion.label.len()).into(),
                code,
            })
        }

        // Constructors: highlight as constructor
        zed::lsp::CompletionKind::Constructor
        | zed::lsp::CompletionKind::EnumMember => {
            Some(zed::CodeLabel {
                spans: vec![zed::CodeLabelSpan::literal(
                    &completion.label,
                    Some("constructor".to_string()),
                )],
                filter_range: (0..completion.label.len()).into(),
                code: completion.label.clone(),
            })
        }

        // Keywords
        zed::lsp::CompletionKind::Keyword => {
            Some(zed::CodeLabel {
                spans: vec![zed::CodeLabelSpan::literal(
                    &completion.label,
                    Some("keyword".to_string()),
                )],
                filter_range: (0..completion.label.len()).into(),
                code: completion.label.clone(),
            })
        }

        _ => None,
    }
}
```

### Symbol Labels (Haskell pattern)

Render workspace symbols with language-appropriate syntax:

```rust
fn label_for_symbol(
    &self,
    _id: &zed::LanguageServerId,
    symbol: zed::lsp::Symbol,
) -> Option<zed::CodeLabel> {
    let (code, name_start) = match symbol.kind {
        zed::lsp::SymbolKind::Function | zed::lsp::SymbolKind::Variable => {
            (format!("fn {}", symbol.name), 3)
        }
        zed::lsp::SymbolKind::Struct | zed::lsp::SymbolKind::Class => {
            (format!("data {}", symbol.name), 5)
        }
        zed::lsp::SymbolKind::Module | zed::lsp::SymbolKind::Namespace => {
            (format!("mod {}", symbol.name), 4)
        }
        zed::lsp::SymbolKind::Constructor => {
            (format!("| {}", symbol.name), 2)
        }
        _ => return None,
    };

    let name_len = symbol.name.len();
    Some(zed::CodeLabel {
        spans: vec![zed::CodeLabelSpan::code_range(name_start..name_start + name_len)],
        filter_range: (0..name_len).into(),
        code,
    })
}
```

### How CodeLabel Works

`CodeLabel` has three fields:

- **`code`**: A string of synthetic source code. Zed parses this with tree-sitter to determine highlighting. It does not
    need to be valid — just enough for tree-sitter to produce useful captures.
- **`spans`**: Which parts of `code` to actually display (and how). `code_range(range)` uses tree-sitter highlighting
    from that range. `literal(text, highlight)` shows text with a fixed highlight.
- **`filter_range`**: The byte range within the displayed text that the fuzzy matcher searches against.

The key insight: you construct fake source code that, when parsed, gives the right highlighting — then select which
parts to display.

______________________________________________________________________

## Snippet Patterns

### File: `snippets/musa.json`

```json
{
  "fn": {
    "prefix": "fn",
    "body": ["fn ${1:name}(${2:params}) -> ${3:Type} {", "    $0", "}"],
    "description": "Function definition"
  },
  "data": {
    "prefix": "data",
    "body": ["data ${1:Name} where", "    ${2:Constructor} : ${0:Type}"],
    "description": "Data type definition"
  },
  "match": {
    "prefix": "match",
    "body": ["match ${1:expr} {", "    ${2:pattern} => ${0:body},", "}"],
    "description": "Match expression"
  },
  "effect": {
    "prefix": "effect",
    "body": ["effect ${1:Name} where", "    ${2:op} : ${0:Type}"],
    "description": "Effect definition"
  },
  "handle": {
    "prefix": "handle",
    "body": ["handle ${1:expr} {", "    ${2:Op}(${3:args}) => ${0:body}", "}"],
    "description": "Effect handler"
  }
}
```

Register in `extension.toml`:

```toml
snippets = ["./snippets/musa.json"]
```

______________________________________________________________________

## Multi-Grammar Extensions

When an extension supports multiple file types (e.g., OCaml supports `.ml`, `.mli`, Dune files, Menhir, etc.):

```toml
# extension.toml
[grammars.musa]
repository = "https://github.com/jcreinhold/musa"
rev = "..."
path = "editors/tree-sitter-musa"

[grammars.musa-markdown]
repository = "https://github.com/tree-sitter-grammars/tree-sitter-markdown"
rev = "..."
```

Each grammar gets its own `languages/<name>/` directory with independent `config.toml` and query files.

______________________________________________________________________

## Status Reporting

Show download/install progress in Zed's status bar:

```rust
use zed_extension_api::{self as zed, LanguageServerInstallationStatus};

// Before downloading
zed::set_language_server_installation_status(
    &zed::LanguageServerId::new("musa-lsp"),
    &LanguageServerInstallationStatus::Downloading,
);

// On success — clear the status
zed::set_language_server_installation_status(
    &zed::LanguageServerId::new("musa-lsp"),
    &LanguageServerInstallationStatus::None,
);

// On failure — show error
zed::set_language_server_installation_status(
    &zed::LanguageServerId::new("musa-lsp"),
    &LanguageServerInstallationStatus::Failed("Download failed: 404".to_string()),
);
```

Status values: `None` (no status), `Downloading`, `CheckingForUpdate`, `Failed(String)`.

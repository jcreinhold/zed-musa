---
name: zed-extension
description: 'Use for Zed editor extensions: extension.toml, WASM Extension trait, query files (.scm), semantic_token_rules.json, snippets, LSP integration, zed-musa repo, zed_extension_api.'
---

# Zed Extension Development

Zed extensions are Rust crates compiled to WebAssembly (`wasm32-wasip2`). They run sandboxed inside Zed via Wasmtime.
The host-extension boundary is defined by WIT (WebAssembly Interface Types) files; the `zed_extension_api` crate wraps
these into an idiomatic Rust `Extension` trait.

## Critical Rule: Read the Existing Extension First

Before modifying the musa Zed extension, read its current state — `extension.toml`, `Cargo.toml`, `src/lib.rs`,
`languages/musa/config.toml`, and all query files. Understand what the extension already provides before adding or
changing anything. Also read the tree-sitter grammar at `editors/tree-sitter-musa/` in the musa repo — query files
target its committed node names, and `extension.toml` pins it by rev.

## Extension Anatomy

```
zed-musa/                       # standalone repo (not part of the musa workspace)
  extension.toml                # Manifest: grammars, language servers, capabilities
  Cargo.toml                    # Rust build config (cdylib, zed_extension_api dep)
  src/lib.rs                    # Extension trait implementation
  grammars/musa/                # Local clone of the musa repo (gitignored; Zed-managed)
  languages/musa/
    config.toml                 # Language metadata: comments, brackets, indentation
    highlights.scm              # Syntax highlighting captures
    outline.scm                 # Code structure for outline panel
    brackets.scm                # Bracket matching pairs
    indents.scm                 # Auto-indentation rules
    locals.scm                  # Scope/definition tracking
    folds.scm                   # Fold regions
    tags.scm                    # Symbol tags
    injections.scm              # Embedded language injection (if added)
    overrides.scm               # Scope-based setting overrides (if added)
  semantic_token_rules.json     # LSP semantic token styling
  snippets/musa.json            # Code snippets (if added)
  themes/<name>.json            # Color themes (if added)
```

## Quick Reference

### Cargo.toml essentials

| Field | Value | Why |
| --- | --- | --- |
| `crate-type` | `["cdylib"]` | Produces the WASM module |
| `[workspace]` | empty | Isolates from parent workspace |
| `zed_extension_api` | `"0.7.0"` | Latest published version |
| `opt-level` | `"z"` | Minimize WASM binary size |
| `lto` | `true` | Link-time optimization for size |
| `strip` | `true` | Remove debug symbols |

### Extension trait methods worth implementing

| Method | When to implement |
| --- | --- |
| `language_server_command` | Always, if the extension provides an LSP |
| `language_server_initialization_options` | When the LSP accepts init options |
| `language_server_workspace_configuration` | When the LSP reads workspace settings |
| `label_for_completion` | To render completions with syntax highlighting |
| `label_for_symbol` | To render symbols with syntax highlighting |

### CLI commands

| Command | What it does |
| --- | --- |
| `cargo build --target wasm32-wasip2 --release` | Build the extension WASM |
| `rustup target add wasm32-wasip2` | Install the WASM target |
| Zed: `install dev extension` | Load extension from local directory |
| Zed: `open log` | View extension debug output |
| `zed --foreground` | See `println!`/`dbg!` output in terminal |

## Best Practices

### 1. Workspace isolation is mandatory

A Zed extension cannot be a member of a parent Rust workspace. `zed-musa` is its own repository with an empty
`[workspace]` in its `Cargo.toml` — keep it that way, and keep it out of the musa workspace members.

### 2. Find binaries with `worktree.which()`, don't hardcode paths

The WASM sandbox cannot read the filesystem directly. `src/lib.rs` locates `musa-lsp` via `worktree.which("musa-lsp")`
and falls back to `settings::LspSettings::for_worktree("musa-lsp", worktree)` for a user-configured path. The error
message tells the user exactly what to do: build with `cargo build --release -p musa-lsp` in a musa checkout and put
`target/release/musa-lsp` on PATH, or set `lsp.musa-lsp.binary.path`. Keep that message accurate when the build
instructions change.

### 3. Forward user settings to the LSP

Use `settings::LspSettings::for_worktree(server_name, worktree)` to read user-configured binary paths, arguments, init
options, and workspace settings. Forward these to the LSP so users can customize behavior through Zed's settings UI
without modifying the extension.

### 4. Declare capabilities for process execution

If the extension runs commands or downloads files, declare capabilities in `extension.toml`:

```toml
[[capabilities]]
kind = "process:exec"
command = "musa-lsp"
args = ["**"]
```

Without this, Zed may prompt the user or deny execution. Three capability kinds exist: `process:exec`, `download_file`,
`npm:install`.

### 5. Query files target Zed's capture names, not nvim-treesitter's

Zed supports most standard tree-sitter captures but has its own set. The key differences from nvim-treesitter: Zed uses
`@open`/`@close` for brackets (not `@punctuation.bracket`), `@name`/`@item`/`@context` for outline (not tags), and
`@indent`/`@end` for indentation. See `references/query-files-reference.md` for the complete Zed capture taxonomy.

### 6. Keep `outline.scm` separate from `tags.scm`

Zed uses `outline.scm` (not `tags.scm`) for the outline panel and breadcrumbs. The outline query uses `@item` to mark
navigable nodes and `@name` for labels. The `@context` capture adds parent context (e.g., showing `part Strings` above a
voice). This is a Zed-specific file — nvim-treesitter does not use it.

### 7. Grammar references need real commit SHAs — and all generated files

In `extension.toml`, the `rev` field for the grammar must be a full Git commit SHA. `zed-musa` currently uses a
`file://` URL with `path = "editors/tree-sitter-musa"` for local development against a musa checkout; switch to the
HTTPS repository URL before publishing.

**Critical: the rev must point at a commit containing all three generated files** — `src/parser.c`, `src/grammar.json`,
and `src/node-types.json`. After running `tree-sitter generate` in `editors/tree-sitter-musa/`, commit all three before
updating the rev. Forgetting `grammar.json` or `node-types.json` is a common mistake that causes silent failures.

The workflow is always: (1) change the grammar and run `npm test` in `editors/tree-sitter-musa/` (this includes the
drift law against the real lexer — see the tree-sitter-grammar skill), (2) commit the generated files in the musa repo,
(3) update `extension.toml` rev to that commit's SHA, (4) commit `extension.toml`.

### 8. Test as a dev extension before publishing

Use Zed's `install dev extension` command palette action to load the extension from your local directory. Check
`zed: open log` for errors. Run Zed from the terminal with `zed --foreground` to see `println!` and `dbg!` output from
the WASM module.

### 9. Semantic tokens complement tree-sitter highlighting

`semantic_token_rules.json` maps musa-lsp's semantic token types/modifiers to Zed highlight captures, giving the LSP's
context-aware highlighting precedence over tree-sitter's structural highlighting where both apply. Users must opt in via
`"semantic_tokens": "combined"` in Zed settings.

### 10. Size-optimize the WASM binary

Use `opt-level = "z"`, `lto = true`, and `strip = true` in the release profile. Keep dependencies minimal — every crate
added increases the WASM binary. The `zed_extension_api` crate plus `serde_json` is usually all you need.

## Common Mistakes

| Mistake | Fix |
| --- | --- |
| Extension in parent workspace | Add empty `[workspace]` to extension's Cargo.toml |
| Using `wasm32-wasi` target | Use `wasm32-wasip2` (the old target was removed in Rust 1.84) |
| Hardcoded binary paths in `language_server_command` | Use `worktree.which()` or `LspSettings::for_worktree()` |
| Grammar rev set to branch name | Use full commit SHA for reproducibility |
| Rev commit missing `grammar.json` or `node-types.json` | After `tree-sitter generate`, commit all three: `parser.c`, `grammar.json`, `node-types.json` |
| `parser.c` is gitignored | Zed compiles `parser.c` from the cloned repo — it must be tracked in git, not ignored |
| `highlights.scm` uses nvim captures like `@keyword.function` | Use Zed captures: `@keyword` (Zed themes don't read subcategories the same way) |
| Query file in wrong directory | Must be in `languages/musa/`, not the grammar's `queries/` |
| Missing `grammar` field in config.toml | Must match a `[grammars.<name>]` key in extension.toml |
| Forgot `register_extension!` macro | Required at the end of lib.rs to wire up the WASM entry point |
| Trying `std::fs` or `std::net` in WASM | Sandbox blocks these; use `worktree` API or `http_client` module |
| No error message when binary not found | Always return a helpful `.ok_or_else()` with install instructions |
| Publishing without a LICENSE file | Required since Oct 2025: MIT, Apache-2.0, BSD, GPL-3, etc. |
| Extension ID contains "zed" or "extension" | Registry rejects these names |

## Musa-Specific Guidance

The musa Zed extension lives in the standalone `zed-musa` repo. It provides:

- **Tree-sitter grammar** from `editors/tree-sitter-musa/` in the musa repo, pinned by rev in `extension.toml` (with a
  `file://` URL for local development). `grammars/musa/` is a Zed-managed local clone of the musa repo and is gitignored
  — never edit it directly; change the grammar in the musa repo and re-pin.
- **Language server** `musa-lsp` — built from `crates/musa-lsp/` in the musa repo, a thin shell over `musa-project` +
  `musa-language`. Provides diagnostics, hover, completions, document symbols, formatting.
- **Query files** in `languages/musa/` — written against the grammar's committed node names (which mirror
  `syntax_kind.rs`, snake_cased). When the grammar evolves with the language (see `docs/prompts/` for the owning
  prompt), update these queries to match new node types and re-pin the rev.

When modifying the musa extension:

- **The grammar is upstream, the queries are here.** Grammar changes belong in the musa repo (and must pass the drift
  law: `npm test` in `editors/tree-sitter-musa/`); query/capture changes belong in `languages/musa/`. Keep node names in
  sync deliberately — `extension.toml`'s comment records which prompt's grammar the rev pins.
- **The LSP is found via PATH.** Users build `musa-lsp` with `cargo build --release -p musa-lsp`; the extension uses
  `worktree.which("musa-lsp")` plus the `lsp.musa-lsp.binary.path` setting.
- **Notation keywords need highlight coverage.** Musa's statement keywords (`score`, `part`, `voice`, `motif`, `studio`,
  `note`, `rest`, `chord`, `tempo`, `meter`, `key`, `clef`, `dynamic`, `tuplet`, `slur`, `repeat`, `transpose`, ...) and
  elaboration keywords (`import`, `use`, `let`, `fn`, ...) must all appear in `highlights.scm`, matched to the grammar's
  actual node names.
- **Dynamics, articulations, and clefs** are identifier-shaped tokens with fixed vocabularies — highlight them with
  `#any-of?` predicate sets so the list stays easy to extend as the language grows.
- **Pitches and durations** (`c4/8`, `/4.`, tuplets) are lexically distinctive; give them dedicated captures
  (`@constant`, `@number`) so scores read differently from elaboration code.

## Publishing Checklist

To publish to `zed-industries/extensions`:

1. Extension must be its own Git repository (`zed-musa` already is)
1. Include a LICENSE file at the repo root (MIT, Apache-2.0, BSD, GPL-3, etc.)
1. Extension ID: lowercase letters, numbers, hyphens only — no "zed" or "extension" in the name (`musa` qualifies)
1. Replace the `file://` grammar URL with the HTTPS musa repository URL, keeping `path = "editors/tree-sitter-musa"` and
   a full commit SHA
1. Fork `zed-industries/extensions`, add as submodule, add entry to `extensions.toml`
1. Run `pnpm sort-extensions` before submitting the PR
1. Language server binaries must not be bundled — download or detect at runtime

## Supporting References

| Reference | When to read |
| --- | --- |
| `references/extension-api-reference.md` | Implementing Extension trait methods, understanding types |
| `references/config-and-manifest-reference.md` | Editing extension.toml, config.toml, or Cargo.toml |
| `references/query-files-reference.md` | Writing or debugging any .scm query file for Zed |
| `references/exemplary-patterns.md` | Looking for real patterns from mature Zed extensions |

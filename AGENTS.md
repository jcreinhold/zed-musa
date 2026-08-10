# AGENTS.md — zed-musa

Zed extension for the [musa](https://github.com/jcreinhold/musa) language: tree-sitter highlighting, outline,
brackets, indentation, and the `musa-lsp` language server, packaged as a WASM extension (`wasm32-wasip2`).

## The documents that govern this repo

- **`../musa/AGENTS.md`** (a musa checkout) — the architecture and standards. This extension obeys them.
- **`../musa/editors/tree-sitter-musa/`** — the grammar this extension pins by rev in `extension.toml`. Query files in
  `languages/musa/` are written against its committed node names.
- **`.claude/skills/zed-extension/SKILL.md`** — the Zed extension playbook (manifest, WASM build, query taxonomy,
  pinning workflow, publishing). Read it before changing extension code.

If code and these documents disagree, either the code is wrong or the document needs a deliberate repair.

## Layout

| Path | What lives there |
| --- | --- |
| `extension.toml` | manifest: grammar pin (rev = full SHA), musa-lsp declaration |
| `src/lib.rs` | `Extension` trait impl; musa-lsp discovery via `worktree.which` + settings |
| `languages/musa/` | `config.toml` + query files (`highlights.scm`, `outline.scm`, ...) |
| `semantic_token_rules.json` | LSP semantic-token → Zed capture mapping |
| `grammars/` | Zed-managed clone of the grammar repo — gitignored, never edit |

## Commands

```sh
cargo build --target wasm32-wasip2 --release   # build the extension WASM
rustup target add wasm32-wasip2                # one-time target install
```

Then in Zed: `install dev extension` to load this directory; `zed: open log` for errors.

## Standards

- **The grammar is upstream.** Grammar changes happen in the musa repo (`editors/tree-sitter-musa/`), must pass the
  drift law (`npm test` there), and land here only as a new `rev` in `extension.toml`. The rev's commit must contain
  `parser.c`, `grammar.json`, and `node-types.json`.
- **Queries live here.** `languages/musa/*.scm` changes happen in this repo, against the pinned grammar's node names,
  using Zed's capture taxonomy (see the skill's query-files reference).
- **No hardcoded paths.** `musa-lsp` is found via `worktree.which()` or the `lsp.musa-lsp.binary.path` setting, with
  an actionable error message.
- **Size-optimized WASM:** `opt-level = "z"`, `lto = true`, `strip = true`; keep dependencies minimal.
- **Workspace isolation:** this crate has an empty `[workspace]` and is never a member of the musa workspace.

# zed-musa

Language support for [musa](https://github.com/jcreinhold/musa) — the notation-first music language — in
[Zed](https://zed.dev).

The extension embodies the same division the musa repository draws between `musa-language` and `musa-project`:

- **Structure** is the embedded tree-sitter grammar (`editors/tree-sitter-musa` in the musa repository, pinned by
  `rev` in `extension.toml`): highlighting, folding, indentation, outline, brackets, locals, and tags are query
  files in `languages/musa/`, written against the grammar's committed node names.

  Those six files are byte-identical copies of `editors/tree-sitter-musa/queries/` at the pinned `rev`, and are
  meant to stay that way. The grammar and its queries are one artifact, and a query edited here rather than there
  is a second opinion about the language held by the editor that shows it. To follow the language: bump `rev`,
  copy the six files across, and check each one with `tree-sitter query languages/musa/<name>.scm <some>.musa`
  from the grammar directory — a query naming a node the grammar no longer has fails to compile, and Zed's answer
  to that is no highlighting at all.

- **Meaning** is the musa language server (`musa-lsp`): diagnostics, hover, definition, references, rename,
  formatting, completion, quick fixes, and semantic tokens, which refine the grammar's highlighting rather than
  fight it (`semantic_token_rules.json` maps the server's legend onto the same scopes).

## Install (development)

1. Build the server, in a checkout of the musa repository:

   ```sh
   cargo build --release -p musa-lsp
   ```

   Put `target/release/musa-lsp` on your `PATH`, or point the extension at it:

   ```json
   "lsp": { "musa-lsp": { "binary": { "path": "/path/to/musa/target/release/musa-lsp" } } }
   ```

2. Install the extension: clone this repository, then in Zed run `zed: install dev extension` and choose the
   clone.

If the server is missing, the extension says what to install — and the grammar's highlighting, folds, and outline
keep working without it, which is the point of two layers.

## Develop

```sh
cargo build --target wasm32-wasip2   # the extension, sandboxed
```

The grammar is prompt 80's artifact and does not change here; if a query wants a node the grammar does not have,
the grammar is repaired first, in the musa repository, where its drift law can say so.

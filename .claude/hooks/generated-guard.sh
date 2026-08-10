#!/usr/bin/env bash
# .claude/hooks/generated-guard.sh — PreToolUse guard (Edit|Write|MultiEdit).
#
# Blocks edits to generated or vendored files before they happen. Exit 2
# refuses the edit (message on stderr); exit 0 lets it proceed. Paths are
# matched on suffix so this works regardless of checkout location.
#
#   1. grammars/ is a Zed-managed clone of the grammar repo (gitignored).
#      Grammar changes belong in the musa repo; land here as a new rev in
#      extension.toml (see AGENTS.md and the zed-extension skill).
#   2. extension.wasm and target/ are build output.
set -euo pipefail

command -v jq >/dev/null 2>&1 || exit 0
input="$(cat)"
file="$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty')"
[ -n "$file" ] || exit 0

case "$file" in
*/grammars/*)
	echo "Refusing to edit '$file': grammars/ is a Zed-managed clone of the grammar repo. Edit editors/tree-sitter-musa/ in a musa checkout, then update the rev in extension.toml." >&2
	exit 2
	;;
*/target/* | */extension.wasm)
	echo "Refusing to edit '$file': build output. Rebuild with 'cargo build --target wasm32-wasip2 --release'." >&2
	exit 2
	;;
esac

exit 0

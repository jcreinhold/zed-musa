#!/usr/bin/env bash
# .claude/hooks/format.sh — PostToolUse formatter (Edit|Write|MultiEdit).
#
# Best-effort rustfmt on the Rust file Claude just edited. ALWAYS exits 0 —
# formatting must never block or undo an edit; stderr is suppressed because
# a file mid-edit may be incomplete. .scm query files have no formatter.
set -euo pipefail

command -v jq >/dev/null 2>&1 || exit 0
input="$(cat)"
file="$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty')"
[ -n "$file" ] && [ -f "$file" ] || exit 0

case "$file" in
*.rs)
	command -v rustfmt >/dev/null 2>&1 && rustfmt "$file" >/dev/null 2>&1 || true
	;;
esac

exit 0

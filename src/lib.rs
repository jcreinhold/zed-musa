//! The musa extension: start the language server, and nothing else.
//!
//! Zed's model already says the division this repository believes in — the
//! embedded tree-sitter grammar answers "what shape is this text" on every
//! keystroke, and the server answers "what does it mean". This crate is a
//! launcher: the sandbox cannot reach the network or the filesystem beyond
//! the worktree, so a found binary is the whole job, and a missing one is a
//! single honest error naming what to install.

use zed_extension_api::{self as zed, Result};

struct MusaExtension;

impl zed::Extension for MusaExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // The extension's settings first, then PATH. Nowhere else: a found
        // binary is a trust, and there is no auto-download to be one.
        let settings = zed::settings::LspSettings::for_worktree("musa-lsp", worktree)?;
        if let Some(binary) = settings.binary
            && let Some(path) = binary.path
        {
            return Ok(zed::Command {
                command: path,
                args: binary.arguments.unwrap_or_default(),
                env: worktree.shell_env(),
            });
        }

        let command = worktree.which("musa-lsp").ok_or_else(|| {
            "musa-lsp was not found on PATH.\n\
             Build it in a checkout of the musa repository: cargo build --release -p musa-lsp\n\
             and put target/release/musa-lsp on your PATH, or set lsp.musa-lsp.binary.path."
                .to_string()
        })?;

        Ok(zed::Command {
            command,
            args: vec![],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(MusaExtension);

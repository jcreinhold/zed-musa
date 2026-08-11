//! The musa extension: start the language server, and nothing else.
//!
//! Zed's model already says the division this repository believes in — the
//! embedded tree-sitter grammar answers "what shape is this text" on every
//! keystroke, and the server answers "what does it mean". This crate is a
//! launcher: the sandbox cannot reach the network or the filesystem beyond
//! the worktree, so a found binary is the whole job, and a missing one is a
//! single honest error naming what to install.
//!
//! Two exceptions, and they are presentation rather than knowledge. Zed lets
//! an extension say how a completion and a symbol should *look*, and the
//! server already sends the signature the compiler settled on. Handing that
//! signature back as code — for the embedded grammar, the same grammar this
//! repository ships, to highlight — is the whole of what these hooks do. No
//! text is composed here, nothing is parsed here, and a completion the server
//! did not describe gets no label of our making.

use zed_extension_api::{self as zed, CodeLabel, CodeLabelSpan, Result};

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

    /// How one completion reads in the menu.
    ///
    /// A declaration's `detail` is its whole signature — `fn lifted(what:
    /// Music, by: Interval = P5) -> Music` — and a signature is Musa source,
    /// so it is handed to Zed as code and the grammar paints it. Everything
    /// else the server offers describes itself in prose rather than in Musa
    /// (`keyword`, `scale collection`, `Interval = P5`), and prose shown as
    /// code would be painted by a grammar that cannot read it: those keep the
    /// name as the code and carry the description beside it, dimmed.
    ///
    /// The test is whether the name appears in the detail, which is exactly
    /// the invariant the server's own record states — a signature mentions
    /// the declared name, and a class word does not.
    fn label_for_completion(
        &self,
        _language_server_id: &zed::LanguageServerId,
        completion: zed::lsp::Completion,
    ) -> Option<CodeLabel> {
        let detail = completion.detail.unwrap_or_default();
        if let Some(offset) = declaration_detail(&completion.label, &detail) {
            let end = u32::try_from(offset.saturating_add(completion.label.len())).ok()?;
            return Some(CodeLabel {
                spans: vec![CodeLabelSpan::code_range(0..code_end(&detail)?)],
                filter_range: (u32::try_from(offset).ok()?..end).into(),
                code: detail,
            });
        }
        let name = code_end(&completion.label)?;
        let mut spans = vec![CodeLabelSpan::code_range(0..name)];
        if !detail.is_empty() {
            spans.push(CodeLabelSpan::literal(
                format!(" — {detail}"),
                Some("comment".to_string()),
            ));
        }
        Some(CodeLabel {
            spans,
            filter_range: (0..name).into(),
            code: completion.label,
        })
    }

    /// How one symbol reads in the outline and the symbol picker.
    ///
    /// The server names a declaration by its signature and a section by its
    /// own name; both are written in Musa, so both are code.
    fn label_for_symbol(
        &self,
        _language_server_id: &zed::LanguageServerId,
        symbol: zed::lsp::Symbol,
    ) -> Option<CodeLabel> {
        let end = code_end(&symbol.name)?;
        Some(CodeLabel {
            spans: vec![CodeLabelSpan::code_range(0..end)],
            filter_range: (0..end).into(),
            code: symbol.name,
        })
    }
}

/// Where `label` sits inside `detail`, when `detail` is the declaration's own
/// signature rather than a description of it.
fn declaration_detail(label: &str, detail: &str) -> Option<usize> {
    if detail.len() <= label.len() {
        return None;
    }
    detail.find(label)
}

/// The length of `text` as a range end, or `None` if it does not fit — a
/// label longer than four billion bytes is not a label.
fn code_end(text: &str) -> Option<u32> {
    u32::try_from(text.len()).ok()
}

zed::register_extension!(MusaExtension);

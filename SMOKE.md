# Manual smoke: the elaboration language in Zed

The musa repository's protocol laws (`crates/musa-lsp/tests/lsp_laws.rs`) cover every answer the server gives, and this
repository's queries are held to the grammar by the drift tests there. What neither can see is whether *Zed* shows those
answers. That is what this checklist is for. Run it whenever `src/lib.rs`, the queries, or the pinned grammar rev
changes.

## Setup

```sh
cargo build --release -p musa-lsp        # in the musa checkout; put it on PATH
rustup target add wasm32-wasip2
cargo build --target wasm32-wasip2 --release
```

In Zed: **zed: install dev extension**, and choose this directory. Then open
`../musa/examples/module-functor-study.musa`.

## Checklist

- [ ] **The server is running.** No error toast. `debug: open language server logs` shows the handshake.
- [ ] **Highlighting without the server.** Kill `musa-lsp`. Keywords, pitches, durations, strings, and types stay
      coloured — that is the embedded grammar. Restart it.
- [ ] **Diagnostics.** Delete a `;`. A squiggle and a teaching message appear; undo clears them.
- [ ] **Hover, type-aware.** Hover `subject` inside `template structure Canon`: `let subject: Music`, and the comment
      written above it.
- [ ] **Hover, domain distinction.** Hover a `NoteName`-typed binding: the popup says what tells it from `Pc12`.
- [ ] **Hover, standard library.** In a piece importing `std::core`, hover an imported name: *Bundled Musa source ·
      read-only*, with a link.
- [ ] **Completion labels are Musa.** Open the completion menu inside a piece. A declaration's row shows its whole
      signature, highlighted by the same grammar as the buffer — `fn`, the parameter types, and the result type each in
      their own colour. A keyword's row shows the word and a dimmed description beside it.
- [ ] **Completion, site-aware.** Inside `assert realizes(chord c major, ` the three realization policies sort to the
      top. Inside `${` in `../musa/examples/kernel-splice.musa`, the music-typed bindings do.
- [ ] **Sound declaration help.** Open `../musa/examples/glass-mountain.musa`. Hover `lowpass`; the popup gives the
      checked source signature, exact unit/default/range facts, source origin, and registered primitive support. Inside
      its call, `resonance:` completes before the general vocabulary and no `q:` alias appears.
- [ ] **Instrument and media facts.** Hover a selected instrument, room, send, clip, or fixed-media cue. The editor shows
      source declarations and project facts, keeps private graph paths out of the musical contract, and never calls a
      part a track.
- [ ] **Foreign adapter and asset state.** In a project containing an `instrument ... from` declaration, hover an SFZ
      or SoundFont instrument. The popup labels `sfz@1` or `sf2@1`, the checked sample-map result, named refusal classes,
      and the current verified/missing/unlocked/digest asset state. The extension does not parse the foreign file.
- [ ] **Signature help.** Type an open paren after a declared function. The parameters appear; a comma advances the
      highlight.
- [ ] **Outline.** `outline: toggle` lists `signature CanonMaterial`, `template Canon`, the `make` instances, and the
      piece's sections, each rendered as Musa rather than as plain text.
- [ ] **Navigation across a template.** Go-to-definition from a `make` site lands on `template structure Canon`.
      Find-all-references finds both `make` statements.
- [ ] **Rename.** Rename `Canon` from a `make` site: both instances and the template's name change. Renaming an
      imported bundled name is refused with the server's own `read-only` message.
- [ ] **Quick fix, certain only.** A missing `;` offers **add `;`**. A stacked chord with no register offers nothing —
      the octave is the composer's choice.
- [ ] **Kernel documents.** Open a `.musa.kernel` file: highlighting and the outline answer; surface-language
      completion does not, because a kernel document declares none.
- [ ] **Stale semantic answers.** Break the current source after one successful compile. Sound hover/navigation keeps
      answering from the last valid artifact with its stale revision relationship, while asset status reflects the
      current verified closure.

## What Zed does not surface yet

Zed's extension API is a launcher plus label hooks: it has no place to register a command, a code lens, or a virtual
document. So two things the server offers reach VS Code and not Zed today:

- the `Analyze: …` code lenses and the `musa.analyze` report;
- opening `musa-stdlib:` definitions as read-only documents — the hover link names the module and the line, and
  following it requires a checkout of the standard-library sources.

Both are the client's gap, not the server's. Neither needs anything new from `musa-lsp` when Zed grows the surface.

## Recording a run

Note the date, the Zed version, the pinned grammar rev, and the musa commit the server was built from.

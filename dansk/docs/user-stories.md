# User stories

Product-intent notes for the dansk app — the "why" behind behavior
decisions, distinct from code comments which document the "how".

## AI fill preserves fields you've already typed

**As** a user building my vocabulary list, **I want** "AI fill" to treat
whatever I've already typed as ground truth and only fill in the blanks,
**so that** I can seed generation from any field — not just Danish or
English — without it silently rewriting or "correcting" a word or example
sentence I deliberately typed.

Acceptance criteria:
- Any of the four fields (Danish, English, example sentence in each
  language) may be filled in before pressing AI fill — generation isn't
  limited to starting from just Danish or just English.
- Fields that already contain text are sent to the model as context, but
  the frontend never overwrites them with the model's response, regardless
  of what the model returns — see `aiFill()` in
  `src/lib/components/Library.svelte`.
- The model is also instructed in the prompt (`src-tauri/src/ai.rs`) to
  echo already-given fields back unchanged. This is redundant with the
  frontend guard on purpose — two independent layers so a prompt-following
  slip on the model's side can't silently overwrite user input.
- Example: given a Danish word ("derud") and a Danish example sentence
  ("Men allerede på vej derud") but no translations, AI fill produces the
  English word and an English translation of that exact sentence — without
  altering the Danish word or example sentence.

## Deleting a word confirms in place, not via a dialog

**As** a user cleaning up my word list, **I want** delete to require a
second, deliberate click rather than pop up a confirmation dialog, **so
that** I don't lose a word to a misclick, without the app depending on
`window.confirm()` — which Tauri's webview doesn't reliably implement (it
can silently return `false` with no dialog shown and nothing printed to the
console, which is exactly the bug this replaced).

Acceptance criteria:
- Clicking "Delete" on a word arms it: the button's label switches to the
  Danish word for delete, "Slet", rendered uppercase and in the danger
  color via CSS (`text-transform: uppercase`), not a separate translated
  string — see `handleDeleteClick()` in `src/lib/components/Library.svelte`.
- Clicking the same button again while armed performs the delete
  immediately — no dialog, no `window.confirm()`, no Tauri dialog plugin
  dependency.
- Arming auto-expires after 3 seconds (`confirmDeleteTimeout`) so a stray
  second click long after the first doesn't delete something the user no
  longer intends to.
- Only one row can be armed at a time — clicking a different row's delete
  button re-arms that row and implicitly disarms the previous one, since
  `confirmDeleteId` is a single shared value.
- Starting an edit or resetting the form also disarms any pending delete
  (`disarmDelete()` called from `startEdit()`/`resetForm()`), so an armed
  delete never lingers into an unrelated action.

## Theme picker matches the editor app's, without inventing a new pattern

**As** a user who runs both the `editor` (Paperling) and `dansk` apps side by
side, **I want** dansk's theme picker to look and behave like Paperling's,
**so that** switching between the two apps doesn't feel like switching
products, and so that any future palette added to one app is a copy-paste
away from the other instead of requiring a redesign.

Acceptance criteria:
- Same four themes, same names, same trigger: Dark, Light, Paper, Dracula,
  toggled from a gear-icon button that opens a small dropdown — see
  `src/lib/components/SettingsMenu.svelte`, ported from editor's
  `SettingsMenu.svelte`.
- Same mechanism: a `data-theme` attribute on `<html>` selects the active
  CSS-variable set in `src/app.css`, persisted to `localStorage`
  (`dansk-theme`) and initialized from the OS `prefers-color-scheme` when no
  choice has been made yet — see `src/lib/state/theme.svelte.ts`.
- Deliberately narrower than editor's version: no font-family/font-size
  picker, since dansk doesn't bundle the `@fontsource` packages that back
  editor's font list. Adding fonts to satisfy a settings-menu parity that
  nobody asked for would be scope creep, not consistency.
- Dansk-specific tokens (the `--badge-new/learning/review-*` colors used by
  the spaced-repetition status badges) get a value in every theme, not just
  Dark/Light — Paper and Dracula each have their own badge palette so no
  theme is left with badges that clash with its surrounding colors.

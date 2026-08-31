# AGENTS.md

Guidance for AI coding agents (Claude Code, Cursor, Gemini) working in this
repository. Humans: the same rules apply — see also `docs/conventions.md`.

## Commands

```bash
npm run tauri:dev              # run the desktop app (Vite + Rust)
npm run tauri:build:dev        # unsigned dev bundle
npm run lint                   # oxlint --type-aware
npm run format                 # oxfmt --write
npm run typecheck              # tsc --noEmit
npm test                       # vitest run
npm run storybook              # component stories on :6006
cd src-tauri && cargo fmt
cd src-tauri && cargo clippy --all-targets   # --all-targets, or test modules go unlinted
```

The pre-commit hook (lefthook) runs lint, format, typecheck, tests, `cargo fmt`,
`cargo clippy -D warnings` and `cargo test` on staged files. Never bypass it with
`--no-verify`; fix the cause.

## Architecture

Two halves, one app.

**Frontend** (`src/`) — React 19, SCSS modules, i18next. Flat by design:
`components/<Name>/` holds a component with its stylesheet and story; `lib/`
holds everything that is not a component. There is no enforced layer system —
add one only when the app grows enough to need it.

**Backend** (`src-tauri/src/`) — `lib.rs` wires the app up (plugins, invoke
handler) and nothing else; `commands.rs` is the command surface. Commands stay
thin: validate, delegate, return. Business logic goes in its own module.

**The boundary** — `src/lib/tauri.ts` wraps every `invoke` call, once. A
component importing `@tauri-apps/api/core` directly is a bug: it duplicates the
argument types and breaks Storybook, which stubs that module.

Everything on the wire is camelCase (`#[serde(rename_all = "camelCase")]` on
every struct that crosses the boundary).

## Frontend rules

- Arrow functions, always assigned to a named `const`. No anonymous standalone
  functions.
- Descriptive names — 1–3 character names are forbidden, callback parameters
  included.
- No magic numbers — name the constant.
- Comment only what is unclear with the surrounding context in view.
- No barrel files — always import the direct file path.
- `@/` is the alias for `src/`.
- Components never import from `@tauri-apps/*` — they call `@/lib/tauri`.
- All user-facing strings go through i18next (`src/locales/`), never inline.

### if/else

```ts
if (condition) {
} else {
}
```

Always block form. A blank line before `return`, and before and after every
`if`/`else` block.

### Styling

- A `ComponentName.module.scss` next to every component — never import a
  sibling's or a parent's stylesheet.
- Do **not** `@use` `functions`, `variables` or `tokens` by hand — they are
  injected into every SCSS module by Vite and Storybook (`SCSS_ADDITIONAL_DATA`
  in `vite.config.ts`).
- Sizes go through `fs()` / `sp()` / `radius()` / `rem()`, never raw `px`
  (borders excepted).
- Colors come from the semantic tokens in `_tokens.scss`. Never hardcode a hex
  or rgba in a component; add a token instead.
- No inline styles — conditional styling uses a named class. (Exception:
  genuinely data-driven values, e.g. a computed `grid-template`.)
- Flat camelCase class names; modifier classes declared after their base class.

### Components

- One component per folder, with its stylesheet and (when it has visual states)
  a `*.stories.tsx`.
- Keep them small — decompose any self-contained section, and anything that
  would push the parent past ~150 lines.
- Props are typed with an `interface`; extend the DOM props type when the
  component wraps a native element.

## Rust rules

- All imports at the top of the file; no fully-qualified paths in logic.
- Every command gets a unit test in the same file's `mod tests`.
- Never panic in a command — return `Result<T, String>` and let the frontend
  handle it.
- New permission? Add it to `src-tauri/capabilities/default.json` — a plugin
  without its capability fails silently at runtime.

## Testing

- Vitest for pure logic (`src/**/*.test.ts`); `cargo test` for Rust.
- Storybook for visual states — a story per meaningful variant, seeded through
  `args`.

## Workflow

- Object if a proposed solution seems wrong — give the argument and wait for
  confirmation before starting.
- After finishing, suggest a Conventional Commits message; do **not** run
  `git commit` unless asked.
- Adding a dependency is a decision, not a detail — say why.

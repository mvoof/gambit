# Conventions

The rules the linter cannot check. The ones it can are in `.oxlintrc.json` and
`lefthook.yml` — if a rule is enforceable, enforce it there instead of writing
it down here.

## Naming and structure

- Arrow functions assigned to a named `const`, everywhere.
- Descriptive names. `e`, `el`, `idx` are out; `event`, `element`, `index` are
  in — callback parameters included.
- No magic numbers: `const POLL_INTERVAL_MS = 250;` beats a bare `250`.
- No barrel (`index.ts`) files — import the direct path. A barrel hides what a
  module actually costs to import.
- One component per folder: `components/<Name>/<Name>.tsx`, its
  `<Name>.module.scss`, and its `<Name>.stories.tsx`.
- A helper used by one component lives with it; a helper used by two or more
  moves to `src/lib/`.

## When to grow the structure

The template is flat on purpose — `components/` and `lib/` are enough for a
small app. Add structure only when a real pressure appears:

| Pressure                                | Add                                                      |
| --------------------------------------- | -------------------------------------------------------- |
| Shared state read by many components    | `src/store/` (MobX, Zustand — pick one and stay with it) |
| Business logic drifting into components | `src/services/`, called by the store, not by the UI      |
| Several routes                          | `src/pages/` + react-router                              |
| A component folder past ~8 files        | split it into sub-components                             |

Don't add a layer before the second consumer exists. Do add one the moment a
component reaches past its own folder for something.

## Backend

- `lib.rs` wires the app up; it does not contain logic.
- `commands.rs` holds the command surface. Commands validate their input,
  delegate, and return — logic belongs in a module of its own once it grows
  beyond a few lines.
- Return `Result<T, String>` and let the frontend decide; never panic in a
  command.
- Every struct crossing the boundary carries
  `#[serde(rename_all = "camelCase")]`, so the frontend never sees snake_case.
- Every command gets a unit test in the same file.

## The frontend/backend boundary

`src/lib/tauri.ts` is the only module that calls `invoke`. It exports one typed
function per command. This is what makes the argument shapes checkable, lets
Storybook stub the backend with one alias, and turns "which commands does the UI
actually use?" into a file you can read.

## Testing

- **Vitest** for pure logic — anything in `src/lib/` that is not a Tauri call.
  Rendering tests are welcome but not the point of the template; add
  `@testing-library/react` and switch `test.environment` to `jsdom` if you want
  them.
- **cargo test** for Rust, next to the code.
- **Storybook** for visual states. A story per meaningful variant, seeded via
  `args` — not a screenshot suite, a place to build a component without running
  the whole app.

## Git

- Conventional Commits (`feat:`, `fix:`, `chore:` ...), see `CONTRIBUTING.md`.
- Version bumps go in their own commit, not attached to a feature.
- Never `--no-verify`. If the hook fails, the hook is right until proven
  otherwise.

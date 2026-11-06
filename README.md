<p align="center">
  <img src="src/assets/brand/gambit-icon-original.png" alt="Gambit" width="128" height="128">
</p>

# Gambit

<p align="center">
  <a href="https://github.com/mvoof/gambit/releases"><img src="https://img.shields.io/github/package-json/v/mvoof/gambit?label=version" alt="Version"></a>
  <a href="https://github.com/mvoof/gambit/actions/workflows/release.yml"><img src="https://github.com/mvoof/gambit/actions/workflows/release.yml/badge.svg?event=push" alt="Release"></a>
  <img src="https://img.shields.io/badge/platforms-Windows%20%7C%20Linux-informational" alt="Platforms: Windows and Linux">
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&amp;logoColor=white" alt="Tauri 2">
  <img src="https://img.shields.io/badge/engines-UCI-informational" alt="UCI engines">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0--or--later-blue" alt="License: GPL-3.0-or-later"></a>
  <a href="https://github.com/mvoof/gambit/releases"><img src="https://img.shields.io/github/downloads/mvoof/gambit/total" alt="Downloads"></a>
</p>

Chess against a real engine, and training to get better. See `docs/spec.md`, and
`CHANGELOG.md` for what changed.

![Home screen: new game, recent games and statistics](docs/screenshots/home.png)

![A game against Stockfish with the move list](docs/screenshots/game.png)

## Scripts

| Command                               | What it does                                              |
| ------------------------------------- | --------------------------------------------------------- |
| `npm run tauri:dev`                   | Run the desktop app in dev mode (Vite + Rust, hot reload) |
| `npm run dev`                         | Vite only — the web part, without the Tauri shell         |
| `npm run tauri:build:dev`             | Unsigned dev build (`nsis`, `deb`), for sharing a preview |
| `npm run tauri:build:release`         | Release build with all bundle targets                     |
| `npm run lint` / `lint:fix`           | oxlint, type-aware                                        |
| `npm run format` / `format:check`     | oxfmt                                                     |
| `npm run typecheck`                   | `tsc --noEmit`                                            |
| `npm test` / `test:watch`             | Vitest                                                    |
| `npm run storybook`                   | Component stories on `:6006`                              |
| `npm run cargo:format` / `cargo:lint` | `cargo fmt` / `cargo clippy -D warnings`                  |
| `npm run rename`                      | Rename the app (see Quick start)                          |

---

## What is set up

**Tooling**

- **oxlint** (`.oxlintrc.json`) — the linter, type-aware, with the React,
  TypeScript and jsx-a11y plugins. Replaces ESLint; it is ~50× faster.
- **oxfmt** (`.oxfmtrc.json`) — the formatter. Replaces Prettier, same defaults
  (80 cols, single quotes, semicolons).
- **lefthook** (`lefthook.yml`) — pre-commit hook, installed by `npm install`.
  Runs on **staged files only**, in parallel: oxlint `--fix`, oxfmt, `tsc
  --noEmit`, `vitest related`, and for Rust `cargo fmt`, `cargo clippy -D
  warnings`, `cargo test`.
- **Vitest** — configured inside `vite.config.ts`, picks up `src/**/*.test.ts(x)`.
- **Storybook 10** — `.storybook/`, shares the Vite aliases and the SCSS token
  injection with the app, and stubs `@tauri-apps/api/core` so stories run in a
  plain browser.
- **VS Code** — `.vscode/` sets oxc as the formatter and recommends the Tauri,
  rust-analyzer and oxc extensions.

**CI (GitHub Actions)**

| Workflow                                        | Trigger          | Does                                                            |
| ----------------------------------------------- | ---------------- | --------------------------------------------------------------- |
| `pr-quality-check.yml` → `reusable-quality.yml` | PR to `main`     | lint, format check, typecheck, Vitest, `cargo fmt/clippy/test`  |
| `pr-build.yml`                                  | PR to `main`     | builds the Windows and Linux bundles, uploads them as artifacts |
| `artifact-comment.yml`                          | after a PR build | comments the download links on the PR                           |
| `release.yml`                                   | tag `v*`         | builds, signs and drafts a GitHub release                       |

Jobs are skipped when nothing relevant changed — the path filters live in
`.github/filters.yml`.

---

## Project layout

```
src/
  components/<Name>/      component + its .module.scss (+ .stories.tsx)
  lib/                    non-UI helpers; tauri.ts wraps every backend command
  locales/                i18next translation files
  styles/                 design tokens, functions, reset
  mocks/                  browser stubs used by Storybook
  App.tsx  main.tsx  i18n.ts

src-tauri/
  src/lib.rs              app wiring: plugins + invoke handler
  src/commands.rs         the command surface (+ unit tests)
  tauri.conf.json         app config;  tauri.dev.conf.json overrides it for dev builds
  capabilities/           permissions granted to each window
```

Flat and deliberately small. Grow it as the app needs — see
[docs/conventions.md](./docs/conventions.md) for how, and
[docs/styling.md](./docs/styling.md) for the token system.

---

## Frontend ↔ backend

A Tauri command is declared in `src-tauri/src/commands.rs`, registered in
`lib.rs`, and wrapped **once** in `src/lib/tauri.ts`:

```rust
#[tauri::command]
pub fn greet(name: String) -> String { /* … */ }
```

```ts
export const greet = (name: string) => invoke<string>('greet', { name });
```

Components import from `@/lib/tauri`, never `@tauri-apps/api` directly — that is
what keeps the types in one place and the stories runnable in a browser.

Rust returns camelCase to the frontend (`#[serde(rename_all = "camelCase")]`).

---

## Styling

SCSS modules with an auto-injected token layer — no `@use` lines in a component:

```scss
.card {
  padding: sp('xl'); // spacing scale
  font-size: fs('md'); // type scale
  border-radius: radius('lg'); // radius scale
  background: $bg-card; // semantic color token
}
```

`fs()`, `sp()`, `radius()`, `rem()` come from `src/styles/_functions.scss`;
colors from `src/styles/_tokens.scss`. Never hardcode a hex in a component.

---

## Docs

- [docs/getting-started.md](./docs/getting-started.md) — first hour with the template
- [docs/conventions.md](./docs/conventions.md) — code style and structure rules
- [docs/styling.md](./docs/styling.md) — the token system in detail
- [AGENTS.md](./AGENTS.md) — the same rules, addressed to AI coding agents
- [CONTRIBUTING.md](./CONTRIBUTING.md) — branches, commits, PRs

## License

[GPL-3.0-or-later](LICENSE)

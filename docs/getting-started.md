# Getting started

## 1. Prerequisites

- **Node 20+**
- **Rust stable** (`rustup toolchain install stable`)
- The [Tauri system dependencies](https://tauri.app/start/prerequisites/) for
  your OS. On Windows that is the WebView2 runtime (already present on Win 11)
  and the MSVC build tools.

## 2. Clone and rename

```sh
git clone https://github.com/mvoof/tauri-app-template my-app
cd my-app
npm install
npm run rename "My App" com.example.my-app
```

`npm install` also installs the git hooks (`lefthook install` runs from the
`prepare` script). `npm run rename` rewrites the template's name everywhere it
is baked in:

| File                            | What changes                           |
| ------------------------------- | -------------------------------------- |
| `package.json`                  | `name`                                 |
| `src-tauri/Cargo.toml`          | package name, `[lib] name` (the crate) |
| `src-tauri/src/main.rs`         | the `*_lib::run()` call                |
| `src-tauri/tauri.conf.json`     | `productName`, `identifier`            |
| `src-tauri/tauri.dev.conf.json` | dev `identifier`                       |
| `index.html`                    | `<title>`                              |

Then replace the icons in `src-tauri/icons/` (`npx tauri icon path/to/icon.png`
generates the whole set from one 1024x1024 PNG), and delete the demo components
you do not need.

## 3. Run it

```sh
npm run tauri:dev
```

The Rust side compiles on first run — that takes a few minutes once, then
seconds. The frontend hot-reloads; changing Rust triggers a rebuild and a
restart.

Frontend-only work is faster in `npm run dev` (Vite alone, on
`http://localhost:1420`) — but anything calling `@/lib/tauri` will fail there,
because there is no backend. Use Storybook for isolated component work: the
Tauri module is stubbed there (`src/mocks/`).

## 4. Add a feature

**A backend command:**

1. Write it in `src-tauri/src/commands.rs`, with a test in `mod tests`.
2. Register it in the `generate_handler!` list in `src-tauri/src/lib.rs`.
3. Wrap it in `src/lib/tauri.ts` with its argument and return types.
4. Call the wrapper from a component.

**A component:**

1. `src/components/MyThing/MyThing.tsx` + `MyThing.module.scss`.
2. Use the token functions (`sp()`, `fs()`, `radius()`) and the color tokens.
3. Add `MyThing.stories.tsx` if it has more than one visual state.
4. Put user-facing strings in `src/locales/*.json`.

**A Tauri plugin:**

1. `npm run tauri add <plugin>` (adds both the npm package and the crate).
2. Add its permission to `src-tauri/capabilities/default.json` — without it the
   plugin fails silently at runtime.

## 5. Ship it

```sh
npm run tauri:build:dev      # unsigned local bundle, for testing
npm run tauri:build:release  # full release build
```

For signed GitHub releases, generate an updater keypair
(`npx tauri signer generate`) and set `TAURI_PRIVATE_KEY` and
`TAURI_KEY_PASSWORD` as repository secrets in the `release` environment — that
is what `.github/workflows/release.yml` expects. Then push a tag:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

The workflow builds every platform in the matrix and drafts a release.

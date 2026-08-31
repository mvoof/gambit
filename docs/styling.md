# Styling

## How the tokens reach a component

`vite.config.ts` exports `SCSS_ADDITIONAL_DATA` and injects it into every SCSS
module, and `.storybook/main.ts` reuses the same constant:

```scss
@use '@/styles/functions' as *;
@use '@/styles/variables' as *;
@use '@/styles/tokens' as *;
```

So a component's stylesheet starts with its own rules — no `@use` lines to
maintain, and no way for a file to accidentally miss the token layer.

## The three files

| File              | Holds                                                  |
| ----------------- | ------------------------------------------------------ |
| `_functions.scss` | `rem()`, and the `fs()` / `sp()` / `radius()` scales   |
| `_variables.scss` | fonts, transitions, layout constants                   |
| `_tokens.scss`    | the palette, and the semantic color tokens built on it |

`_reset.scss` (pulled in by `index.scss`, imported once from `main.tsx`) sets
box-sizing, the base font and the app background.

## Scales, not numbers

```scss
.panel {
  padding: sp('lg'); // 12px
  gap: sp('sm'); // 8px
  font-size: fs('md'); // 15px
  border-radius: radius('lg'); // 6px
}
```

Steps are named `xxxs` through `xxxl`. They avoid a leading digit on purpose:
Sass parses `2xs` as a number, not an identifier.

An unknown step is a build error, not a silent `null` — the functions `@error`.

**Why a scale?** Tuning the app's density becomes one edit in one map instead of
a hunt through forty stylesheets, and two components written a month apart end
up on the same rhythm without anyone coordinating.

Something genuinely off-scale (a one-off 37px illustration slot) uses `rem(37)`.
That is the escape hatch; if you reach for it three times, the scale is wrong.

## Colors

`_tokens.scss` has two halves. The palette (`$palette-ink-900`,
`$palette-blue-500`, ...) is raw values — **never used in a component**. The
semantic tokens are:

| Group    | Tokens                                            |
| -------- | ------------------------------------------------- |
| Surfaces | `$bg-app`, `$bg-panel`, `$bg-card`, `$bg-raised`  |
| Lines    | `$border-main`, `$border-sub`                     |
| Text     | `$text-primary`, `$text-secondary`, `$text-muted` |
| Status   | `$accent`, `$success`, `$warning`, `$danger`      |

A component names the role (`background: $bg-card`), not the value. Retheming is
then a change to the palette, and a token name tells the next reader what the
color is _for_ — which `#15161a` never will.

Need a color no token covers? Add a token. A hex in a component is a bug.

## Adding a theme

The tokens are Sass variables, so they are resolved at build time — good for one
theme, not enough for a runtime switch. To add light/dark:

1. Emit the tokens as CSS custom properties on `:root` in `_reset.scss`.
2. Redefine them under `[data-theme='light']`.
3. Point the Sass tokens at the custom properties
   (`$bg-card: var(--bg-card);`), so components need no change at all.

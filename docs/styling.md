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

| File              | Holds                                                |
| ----------------- | ---------------------------------------------------- |
| `_functions.scss` | `rem()`, and the `fs()` / `sp()` / `radius()` scales |
| `_variables.scss` | fonts, transitions, layout constants                 |
| `_tokens.scss`    | the semantic color tokens                            |
| `_themes.scss`    | their values, one map per theme                      |

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

The semantic tokens are:

| Group    | Tokens                                            |
| -------- | ------------------------------------------------- |
| Surfaces | `$bg-app`, `$bg-panel`, `$bg-card`, `$bg-raised`  |
| Lines    | `$border-main`, `$border-sub`                     |
| Text     | `$text-primary`, `$text-secondary`, `$text-muted` |
| Status   | `$accent`, `$success`, `$warning`, `$danger`      |

A component names the role (`background: $bg-card`), not the value. Retheming is
then a change to `_themes.scss`, and a token name tells the next reader what the
color is _for_ — which `#15161a` never will.

Need a color no token covers? Add a token. A hex in a component is a bug.

## Themes

`_tokens.scss` names the roles as CSS custom properties, and `_themes.scss` holds
one map of values per theme, emitted under `[data-theme]`. `start-sync.ts` sets
the attribute on `<html>`, following the system for the `system` theme. Nothing
in a component knows which theme is on. A new theme is one more map.

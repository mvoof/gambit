#!/usr/bin/env node
/**
 * Turns the template into a project of its own: rewrites every place the
 * template name is baked in (package.json, Cargo.toml, the Tauri configs, the
 * Rust crate name and index.html) in one pass.
 *
 * Usage:
 *   node scripts/rename-app.mjs "My App" [com.example.my-app]
 */
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = resolve(dirname(fileURLToPath(import.meta.url)), '..');

const [, , productNameArg, identifierArg] = process.argv;

if (!productNameArg) {
  console.error(
    'Usage: node scripts/rename-app.mjs "My App" [com.example.my-app]'
  );
  process.exit(1);
}

const productName = productNameArg;
const slug = productName
  .toLowerCase()
  .replace(/[^a-z0-9]+/g, '-')
  .replace(/^-|-$/g, '');
const crateName = slug.replace(/-/g, '_');
const identifier = identifierArg ?? `com.example.${slug}`;

const OLD_SLUG = 'tauri-app-template';
const OLD_CRATE = 'tauri_app_template';

const replaceInFile = (relativePath, replacements) => {
  const filePath = resolve(rootDir, relativePath);
  let content = readFileSync(filePath, 'utf8');

  for (const [pattern, replacement] of replacements) {
    content = content.replaceAll(pattern, replacement);
  }

  writeFileSync(filePath, content);
  console.log(`updated ${relativePath}`);
};

replaceInFile('package.json', [[OLD_SLUG, slug]]);
replaceInFile('src-tauri/Cargo.toml', [
  [OLD_SLUG, slug],
  [OLD_CRATE, crateName],
]);
replaceInFile('src-tauri/src/main.rs', [[OLD_CRATE, crateName]]);
replaceInFile('src-tauri/tauri.conf.json', [
  [`"productName": "${OLD_SLUG}"`, `"productName": "${productName}"`],
  [`"identifier": "com.voof.${OLD_SLUG}"`, `"identifier": "${identifier}"`],
  [OLD_SLUG, slug],
]);
replaceInFile('src-tauri/tauri.dev.conf.json', [
  [`com.voof.${OLD_SLUG}`, identifier],
]);
replaceInFile('index.html', [['Tauri App Template', productName]]);

console.log(
  `\nDone. Product "${productName}", crate "${crateName}", identifier "${identifier}".`
);
console.log(
  'Next: delete src-tauri/Cargo.lock target caches if the crate name changed, then `npm run tauri:dev`.'
);

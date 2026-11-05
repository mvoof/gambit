# Gambit: product and technical spec

Chess against a real engine, and training to get better at it. Everything runs
offline on the user's machine. The name is **Gambit**, with no "Marble": this
project has nothing to do with Marble Trace.

## The principle that decides everything

**Every fact comes from code.** The rules code decides legality, the UCI engine
decides strength and evaluation, and there is no language model. One was tried
through LM Studio and removed: it invented moves and opening names, it only
repeated what the engine had found, and the engine can show that directly.

## Architecture

- Tauri 2. **All logic lives in Rust; the frontend holds UI state only.** No
  chess.js: rules, SAN, PGN, clocks, evaluation and persistence are backend work.
- Backend: a Cargo workspace, one crate per domain, under `src-tauri/crates/`.
  Only the app crate sees Tauri.
- Frontend: strict FSD, `app → pages → widgets → features → entities → shared`,
  held by `scripts/lint/layers-plugin.mjs`. React 19, MobX, SCSS Modules. Stores
  through context hooks. Components read stores, never `@tauri-apps/*`;
  `shared/api/*.service.ts` is the only place that calls `invoke`.
- Contract: specta generates `src/shared/contracts/generated/bindings.ts` from
  `gb-model` under the `dev` feature. Generated files are never edited. Specta is
  pinned exactly (`=2.0.0-rc.28`, typescript `=0.0.15`): an rc may change the
  contract.
- Licence GPL-3.0-or-later (shakmaty is GPL-3, Stockfish is GPLv3).

### Crates

| Crate        | Holds                                                         |
| ------------ | ------------------------------------------------------------- |
| `gb-model`   | every wire type and event name, nothing else                  |
| `gb-chess`   | shakmaty game state, draw rules, stable piece ids, PGN, clock |
| `gb-engine`  | UCI process driver, Stockfish installer, score parsing        |
| `gb-archive` | saved games in SQLite                                         |
| `gb-pieces`  | custom piece import, sprite slicing (to add, `image` crate)   |
| app          | session orchestration, Tauri commands, settings file          |

### Wire design

- `GameView` is what the frontend renders: `pieces` with **stable ids** tracked by
  Rust through captures, castling, en passant and promotion, so the frontend
  animates by key with no chess knowledge. Also FEN, turn, status, legal moves,
  last move, check square, move list, captured pieces, material difference, clock
  state and an engine-thinking flag.
- Each move carries its sound kind (move, capture, castle, check, promote), so the
  frontend does not infer it.
- Replay: Rust returns all frames at once and the frontend only indexes them.
- Clock on the wire: white and black milliseconds, the running side and a sync
  time. The frontend counts down for display only. Rust stays authoritative and
  decides flag fall.
- Evaluation event: white-POV centipawns or mate, plus a win percentage computed
  in Rust, so the bar only sets a height.
- Everything crossing the boundary is camelCase. Payloads are finite: `Option`,
  never NaN or infinity.

## Stage 1: a playable game

**Board**, top-down view:

- click and drag to move, legal-move dots, last-move and check highlights,
  coordinates, flip, resizable board, piece animation
- promotion picker, premoves, right-drag arrows and square marks
- captured pieces and material difference, two-column move list, keyboard
  navigation

**Rules**, automatic: checkmate, stalemate, threefold repetition (zobrist hash),
fifty-move rule, insufficient material. Also resign, draw offer and takeback.

**Clocks**, switching by themselves:

- presets plus custom base and increment; unlimited is an option
- Rust owns the clock, adds the increment and runs the flag-fall timer
- the clock starts on White's first move
- on flag fall, a draw if the opponent has insufficient mating material (FIDE)
- the engine gets real `wtime/btime/winc/binc`; `movetime` when unlimited
- a takeback restores the clock snapshot of that ply

**Engine**, any UCI binary:

- Default: Stockfish 19 downloaded on demand, never in the installer.
  - URL: `https://github.com/official-stockfish/Stockfish/releases/download/sf_19/stockfish-windows-x86-64-universal.zip`
  - 81431614 bytes
  - sha256 `3c8bf1f9ea66a09350a40df4f632288285ac206d99f33ab5842c408fc30b48a7`
- A settings tab offers: download Stockfish, pick your own `.exe`, links to other
  UCI engines.
- Name and the `UCI_Elo` range are read from the engine's `uci` reply. Strength
  goes through `UCI_LimitStrength`; an engine without it gets a depth or time
  limit.
- Stale guard: a generation number, so a reply for a reset game is dropped.

**Evaluation bar**, toggled in settings: a **second engine process at full
strength**, separate from the Elo-limited opponent. It also works on replay
frames.

**Sounds**: synthesised with Web Audio (original, no files, no AGPL samples).

**Archive**: finished games autosave. PGN with a `TimeControl` header, FEN setup,
import and export. Replay with stepping, autoplay and keyboard.

**Custom pieces**:

- bundled sets: cburnett, merida, chessnut, fantasy (`src/assets/pieces`, see
  `LICENSES.md`; avoid sets under CC BY-NC-SA or AGPL)
- import a sprite sheet (PNG, WebP or JPG, 6×2): Rust validates and slices it into
  12 normalised PNGs, with a column-order option (`KQRBNP`, `KQBNRP`) and a
  preview
- or import a folder of `wK…bP` files (SVG or PNG)
- the frontend receives data URLs, so bundled and custom sets look the same to it

**Board customisation**: colours, coordinates, highlights, sound volume.

## Design

It must look nothing like Marble Trace: this is a beautiful game, not a telemetry
tool. Direction: warm, dark, premium board-room feel, stone and brass, generous
space, the board as the hero. Self-hosted fonts with Cyrillic:
`@fontsource-variable/playfair-display` for display, `manrope` for UI,
`jetbrains-mono` for clocks. Replace the template tokens.

## Stage 2: best move and mistakes

A setting shows the engine's best move as an arrow on the board. After each move
of the player the engine measures what it cost, and the move is marked as an
inaccuracy, a mistake or a blunder, with the better move next to it.

## Stage 3: game review

Stockfish scores every move of a finished game: an evaluation graph, accuracy, and
a walk through the player's mistakes. Hints go by level: piece, then move.

## Stage 4: opening trainer

Openings come from an open database, whose licence has to be checked first; it also
names the opening of a game by its moves. The player picks an opening and a colour,
the app answers from the tree, and a wrong move is taken back and the right one
shown. Spaced repetition brings back what went wrong, and a repertoire can be
imported from PGN.

## Stage 5: puzzles

Puzzles from the player's own mistakes, with spaced repetition, and a daily tactics
set from an open puzzle database downloaded on demand, with a puzzle rating.

## Stage 6: lessons

Lessons as data: text, a position and a task whose move is checked by the rules
code. Rules of the game, basic mates against Stockfish, tactics by theme, endgames.

## Later ideas

Games against a friend, with `shared/ui/Chat` kept for them. Statistics of the
player's openings and kinds of mistake, play from any position, blindfold mode.

## Status

Stage 1 is built and was checked in the real app against Stockfish 19: download
with the pinned hash, a timed game, premove, takeback, draw offer, resignation,
autosave, archive, replay and the evaluation bar. The window has no native title
bar: the app header drags it and holds its buttons. Icons come from
`tauri icon`, the original is in `src/assets/brand`.

Versions are the latest stable releases, chosen on their own merit. Tauri stays
on 2.x while 3.x is alpha.

The dev build (`tauri.dev.conf.json`) has its own identifier, so its data never
mixes with a release install, and `withGlobalTauri`, which the MCP Bridge needs.
`npm run contract` regenerates the TypeScript contract.

Updates go through `tauri-plugin-updater` and GitHub Releases: checked at
startup, offered in the header and in settings, installed only on request. The
signing public key still has to be put into `plugins.updater.pubkey`.

The language model and the voice plans were dropped. `shared/ui/Chat` stays, unused,
for games against a friend. `UciEngine::candidates` (`MultiPV`) and `san_line` stay
for the stages below.

Next, in order: Stages 2 to 6 above.

APIs already checked. shakmaty 0.30: `Chess`, `Position::{legal_moves,
play_unchecked, is_check, is_checkmate, is_stalemate, has_insufficient_material,
halfmoves, zobrist_hash::<Zobrist64>(EnPassantMode::Legal)}`, `UciMove::to_move`,
`san::SanPlus::from_move_and_play_unchecked`, `fen::Fen::{from_position,
into_position}`. pgn-reader 0.29: `Visitor` with `begin_tags`, `tag`,
`begin_movetext`, `san`, `end_game`, and `Reader::read_game`.

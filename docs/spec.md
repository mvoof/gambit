# Gambit: product and technical spec

Chess against a real engine, with a local AI companion that talks by text and
voice. Everything runs offline on the user's machine. The name is **Gambit**,
with no "Marble": this project has nothing to do with Marble Trace.

## The principle that decides everything

**An LLM never plays or calculates.** Language models lose track of the position
and invent lines. The rules code decides legality, the UCI engine decides
strength and evaluation, and the LLM only turns measured facts into words.

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

Later: `gb-llm` (LM Studio) and `gb-voice` (copied from Marble Trace `mt-voice`
first, shared only once it is clear what is really reused).

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

## Stage 1: a playable game, no LLM

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
  UCI engines. LM Studio cannot be an engine: it serves language models.
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

## Stage 2: companion

LM Studio over its OpenAI-compatible API on `localhost:1234`, with streaming.
Personalities: Stockfish gives 3–5 candidates and the persona picks by style, so
moves stay legal. A "model plays alone" fun mode picks from the legal list and is
validated.

## Stage 3: analysis

Stockfish scores each move and classifies mistakes; the LLM explains only those
facts. The user can ask the model while paused on a replay, and it answers with an
engine line and arrows. Hints go by level: idea, piece, move.

## Stage 4: voice

Piper for speech: split the LLM stream on sentences so speech starts at once, and
stop when the user talks. Vosk **grammar mode** for moves ("конь эф три"),
whisper.cpp for free dialogue, since grammar mode cannot do dialogue. Models are
downloaded on demand. No Edge TTS, no Silero (CC BY-NC).

## Later ideas

Puzzles from your own blunders with spaced repetition, memory of recurring
weaknesses, blindfold voice-only mode, opening trainer.

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

Next, in order:

1. Stage 2: `gb-llm` over the LM Studio API, streaming chat, the roles below.
2. Stage 3: engine-measured game review, then the LLM explaining it.
3. Stage 4: `gb-voice`, speaking with the model both ways.

The LLM side, always under the principle above, with the engine supplying every
fact:

- **Roles**, each a persona with its own prompt and style: opponent, coach,
  sparring partner, commentator.
- **Teaching**: lessons on ideas and openings, at the player's level.
- **Trainer**: positions from the player's own games, puzzles from their
  blunders with spaced repetition.
- **Game review**: walks through a finished game, mistake by mistake.
- **Advisor**: hints during a game, by level: idea, piece, move.
- **Open chat** about the position on screen, by text and by voice.

APIs already checked. shakmaty 0.30: `Chess`, `Position::{legal_moves,
play_unchecked, is_check, is_checkmate, is_stalemate, has_insufficient_material,
halfmoves, zobrist_hash::<Zobrist64>(EnPassantMode::Legal)}`, `UciMove::to_move`,
`san::SanPlus::from_move_and_play_unchecked`, `fen::Fen::{from_position,
into_position}`. pgn-reader 0.29: `Visitor` with `begin_tags`, `tag`,
`begin_movetext`, `san`, `end_game`, and `Reader::read_game`.

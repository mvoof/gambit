# Changelog

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-11-06

The first release.

### Play

- Games against any UCI engine, with its strength set by Elo, or by search depth
  when the engine has no Elo setting.
- Legal-move dots, premoves, arrows and square marks, a promotion picker, a flip
  and a resizable board.
- Automatic checkmate, stalemate, threefold repetition, fifty-move rule and
  insufficient material; resignation and draw offers.
- Clocks with presets, custom base and increment, or no limit.
- Every move is saved as it is played, so closing the app loses no game.

### Training and review

- Training games: the engine's lines, evaluation bar and best-move arrow, and the
  player's mistakes marked as they happen, with the better move beside them.
- Replays of saved games with stepping, autoplay and the keyboard, marks for both
  sides and an evaluation graph.

### Engines

- A catalogue of eight pinned engines, installed in one click and checked against
  their SHA-256: Stockfish, Reckless, PlentyChess, Viridithas, Caissa, Alexandria,
  Berserk and Clover. Only engines with a build for the machine are offered.
- Any other UCI engine can be chosen from a file.

### App

- Windows and Linux (x86-64) builds.
- A game archive with PGN import and export, and statistics.
- Bundled and custom piece sets, board colours, and light, dark and system themes.
- Moves written with figurines or letters.
- English and Russian.
- Updates through GitHub Releases, offered in the app and installed on request.

[1.0.0]: https://github.com/mvoof/gambit/releases/tag/v1.0.0

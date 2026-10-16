# ChillBlocks

A quiet block puzzle in Rust: place pieces on an 8×8 grid, complete rows and columns,
and that is the whole game. It is deliberately not a dopamine loop.

Browser-first (WASM), touch-first, with a native iOS/Android path prepared for later.

## The chill contract

These are enforced in code, not left to convention:

- **No streaks, no multipliers, no combo chains.** Scoring is flat: +1 per placed cell,
  +10 per cleared line. A bigger clear is worth more only because it is more lines.
- **No flashy feedback.** No particles, no confetti, no screen shake, no score popups.
- **Motion budget:** max 300 ms, ease-out cubic only. Fade in 180 ms, fade out 250 ms,
  lift 150 ms. No bounce, no overshoot.
- **Muted palette**, one accent, no alarm red. An invalid placement is a dim outline.
- **Audio off by default.** One soft sine tone on a clear only, gain capped at 0.06.
- **No dark patterns:** no daily streaks, achievements, leaderboards, ads, IAP,
  notifications, telemetry, or network calls. Fully offline.
- **End of game** is a calm summary, not a celebration.

## Running it

```sh
./build.sh                       # builds the wasm module and generates the bindings
uv run --no-project python -m http.server -d web 8080
```

Then open <http://127.0.0.1:8080/index.html>.

The toolchain is pinned in `rust-toolchain.toml` to 1.92.0 (the default 1.77.2 is below
the MSRV of current wasm-bindgen). The `wasm-bindgen` CLI lives in `tools/` and is
gitignored; `cargo install wasm-bindgen-cli` works too.

## Controls

- Drag a piece from the tray onto the board. The piece is drawn so the cell you grabbed
  by stays under your finger, at board size, inside and outside the board alike. On touch
  it is lifted above the finger, otherwise the finger covers it.
- Tap a tray piece to select it, then tap the board to place it.
- Keyboard: `1`/`2`/`3` select a piece, arrows move the cursor, `Enter` places,
  `R` starts a new game, `M` toggles sound.

## Hosting

The game is static files, so GitHub Pages works. `.github/workflows/pages.yml` builds the
wasm module, assembles `index.html`, `style.css`, `main.js` and the generated bindings into
`dist/`, and deploys that as the Pages artifact.

Two notes:

- `web/pkg` is generated and gitignored, so Pages builds it in CI rather than serving a
  committed copy.
- GitHub Pages does not always serve `.wasm` as `application/wasm`. The wasm-bindgen loader
  falls back to `WebAssembly.instantiate` when the content type is wrong, so it runs either
  way.

## Architecture

```
crates/
  chillblocks-core/       pure game logic, zero deps, deterministic, fully testable
  chillblocks-ui/         Painter trait, theme, layout, view state, draw
  chillblocks-platform/   Store / Clock / AudioSink traits + web impls + WebPainter
  chillblocks-app/        main loop, input handling, wasm entry point
web/
  index.html, style.css, main.js, pkg/ (generated)
```

Dependency direction is `core` ← `ui` ← `platform` ← `app`. `chillblocks-core` knows
nothing about rendering, input, storage or time, so a game can be reproduced from its
seed and tested headlessly. The same code will back the future native builds.

`Painter`, `Store`, `Clock`, `AudioSink` and the unified `PointerEvent` model are the
only seams a native backend has to implement.

## Tests

```sh
cargo test
```

- Unit tests: placement validity, single and multi-line clears, game-over detection,
  RNG determinism, snapshot round-trip, layout bounds, motion budget.
- A headless simulation harness plays 200 games and asserts the rules hold.

## Roadmap

See [PLAN.md](PLAN.md). The next milestone is the native mobile harness: a wgpu backend
plus `aarch64-apple-ios` and `aarch64-linux-android` targets, which needs nothing new
from the game logic.

# ChillBlocks — Implementation Plan

Block Blast-like puzzle in Rust, browser-first (WASM), touch-first, prepared for native
iOS/Android later. The point of the project is the *chill* variant: no dopamine-loop
mechanics, no flashy feedback, no dark patterns.

This file is the durable record of the plan. Keep it updated when decisions change.

## Locked decisions

- **Rendering:** draw-primitive abstraction (`ui::Painter` trait), canvas 2D backend now,
  wgpu backend later for native.
- **Scoring:** minimal. +1 per placed cell, +10 per cleared line. No streaks, no
  multipliers, no combo chains, no score popups.
- **Mobile:** native wgpu builds later. Core stays platform-agnostic from day one.
- **v1 features:** resume saved board, local personal best, optional soft audio.
- **Deferred to v2:** accessibility settings (reduced motion, high contrast, dyslexia-friendly
  font), break nudge / parent session cap. The animation system already carries a
  `motion_scale` factor so reduced motion is a one-line setting later.

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

`chillblocks-core` must never depend on platform code. It is the source of truth for
parity across web and future native builds.

Dependency direction: `core` <- `ui` <- `platform` <- `app`.

## Game rules

- 8x8 grid, tray of 3 pieces, refills when the tray is empty.
- Full row or full column clears. Multi-line clears count, but only as `10 * lines`.
- Game over when no remaining tray piece fits anywhere.
- Piece set: 23 shapes — bars 1x1..4x4, 2x2 square, corner trominoes, L, T, S, Z, plus.
  No rotation. Uniform random draw from the fixed set: no weighting, no near-miss
  generation, no streak-based piece feeding.
- RNG: SplitMix64, seeded. Same seed => same game. Reproducible and testable.

## The chill contract (enforced in code, not vibes)

- Motion: max 300 ms, ease-out cubic only. Fade in 180 ms, fade out 250 ms, lift 150 ms.
  No bounce, no overshoot, no particles, no confetti, no screen shake.
- Colour: low-saturation muted palette, one accent. Invalid placement is a dim outline,
  never saturated red.
- Audio: off by default. One soft sine tone on line clear only, capped gain.
- No daily streaks, achievements, leaderboards, ads, IAP, notifications, telemetry,
  or any network calls. Fully offline.
- End of game: calm summary, "Play again" and "Stop for today" given equal visual weight.

## Input

Unified pointer model (`Down / Move / Up / Cancel`) shared by mouse and touch.

- Drag from tray -> ghost preview on the board.
- Grab anchor: the cell of the shape the press lands on is recorded on `Down`, and the
  piece is drawn so that cell stays under the pointer at board size, inside the board and
  outside it alike. Crossing the board edge only changes whether it snaps to a cell. A
  press on empty space in the bounding box falls back to the nearest filled cell.
- Touch anchor offset: piece renders above the finger, otherwise the finger occludes it.
- Tap-to-select then tap-to-place fallback for small screens.
- Keyboard: 1/2/3 select piece, arrows move cursor, Enter places, R restarts.
- Draw order is board, tray, footer, then the piece in hand, so a dragged piece is never
  covered by the slot it came from.

## Persistence

`Store` trait. Web -> localStorage. Native -> platform storage later.
Key `chillblocks.v1` holds the game snapshot; `chillblocks.best` the personal best;
`chillblocks.settings` the settings. No accounts, no server.

## Testing

- Unit tests: placement validity, single and multi-line clears, game-over detection,
  RNG determinism, snapshot round-trip.
- Property test: board invariants hold after any placement.
- Headless simulation harness: scripted greedy AI plays many games, asserts no panics
  and sane clear rates.

## Toolchain

- Toolchain pinned in `rust-toolchain.toml` to **1.92.0** (the default 1.77.2 is below the
  MSRV of current wasm-bindgen). wasm32-unknown-unknown target installed for it.
- `wasm-bindgen` 0.2.129 CLI lives in `tools/` (gitignored).
- Build: `cargo build --target wasm32-unknown-unknown -p chillblocks-app` then
  `tools/wasm-bindgen --target web --out-dir web/pkg crates/chillblocks-app`.
- Serve `web/` with any static server (`python3 -m http.server`).
- CI: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`.

## Milestones

Status legend: ✅ done, 🔨 in progress, ⬜ not started.

1. **M0** — workspace scaffold, toolchain, build script. ✅
2. **M1** — `chillblocks-core` + tests. ✅
3. **M2** — canvas 2D painter, pointer input, responsive layout, playable in browser. ✅
4. **M3** — calm visual language, animation system, settings. ✅
5. **M4** — persistence (resume, personal best), optional audio. ✅
6. **M5** — native mobile harness: wgpu backend, iOS/Android targets, storage impls. ⬜

## Verified

- `cargo fmt`, `cargo clippy -D warnings`, `cargo test` (89 tests) and the wasm release
  build are green.
- Checked in a browser: placement, tap-to-select, touch drag with the lift offset, the
  grab anchor holding the same size and pivot inside and outside the board, the piece in
  hand drawing above the slot panel, line clears, the game-over card, resume from
  `localStorage`, personal best, and the layout at 1200×1000, 390×844 and 320×480.
- Audio: the suspended-context bug is fixed by `AudioSink::wake()` on the first press and
  on the sound toggle, and toggling sound on now plays one confirmation tone. Audible
  output cannot be confirmed from the harness, so this is verified by code path only.

## M5 notes (not started)

- Targets: `aarch64-apple-ios`, `aarch64-linux-android`.
- iOS: wgpu + Metal + an Objective-C lifecycle shim.
- Android: `android-activity` + winit + wgpu.
- Cheaper interim route to the stores is a webview wrapper (Capacitor/TWA) around the same
  wasm build. Decide when mobile work starts.
- Everything M5 needs is already isolated: `Painter`, `Store`, `Clock`, `AudioSink`,
  `PointerEvent`, and safe-area insets in `Layout`.

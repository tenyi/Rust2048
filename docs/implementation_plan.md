# Implementation Plan — Rust 2048

This file tracks the implementation progress with checkboxes (`- [ ]` → `- [x]`).
Each item corresponds to a bounded unit of work. Check off immediately after verification passes.

---

## Phase P0: Project Scaffold & Dependencies ✅ COMPLETED

- [x] Add `egui`, `eframe`, `serde`, `serde_json`, `dirs-next` to Cargo.toml
- [x] Set up directory structure (`src/game/`, `src/gui/`, `src/persistence/`)
- [x] Create module `mod.rs` files with re-exports
- [x] Verify project compiles: `cargo build` succeeds

## Phase P1: Game Data Model — Tile & Grid ✅ COMPLETED

- [x] Implement `game::tile::Tile { value, newly_placed }` + constructor + validation
- [x] Implement `game::grid::Grid { cells }` with empty initialization
- [x] Implement `Grid::add_random_tile()` (90% chance of 2, 10% chance of 4)
- [x] Verify: grid starts empty, first add places a tile at random position

## Phase P2: Move Engine — Slide & Merge Algorithm ✅ COMPLETED

- [x] Define `Direction { Up, Down, Left, Right }` enum
- [x] Implement `slide_line()` for one row/column (compress → merge → pad)
- [x] Implement `Grid::slide_and_merge(direction)` applying slide to all rows/cols
- [x] Return `bool` indicating whether the grid actually changed
- [x] Verify: single-row slides, multi-row merges in all four directions

## Phase P3: Score Tracker ✅ COMPLETED

- [x] Implement `ScoreTracker { current_score, best_score }` with default values
- [x] Add `add_to_current(amount)` — increments by merge result score
- [x] Add `update_best_if_higher()` — compares to persisted best and returns new value if higher
- [x] Verify: scores increment correctly; best updates only when surpassed

## Phase P4: Unit Tests for Game Logic ✅ COMPLETED

- [x] Write test: empty row slide → no change, no score gain
- [x] Write test: two equal tiles merge → value doubles, second removed, score += new_value
- [x] Write test: three consecutive equal tiles → only leading pair merges (no chain)
- [x] Write test: `is_game_over()` on full grid with no possible merges returns true
- [x] Write test: `has_won(2048)` returns true when 2048 tile exists
- [x] Run `cargo test` — all pass (18 tests passing)

## Phase P5: GUI Scaffold — GameWindow (eframe::App) ✅ COMPLETED

- [x] Create `GameWindow` struct implementing `eframe::App` trait
- [x] Initialize with empty Grid + ScoreTracker in `new(cc)`
- [x] Implement basic `update(ctx, frame)` method
- [x] Verify: window opens with egui default styling

## Phase P6: Connect GUI to game logic — render tiles & numbers ✅ COMPLETED

- [x] Draw 4×4 grid background with cell borders in `show_grid_ui`
- [x] For each non-empty tile, draw rounded rectangle + value text using standard palette colors
- [x] Color tiles by value using the standard 2048 palette (#eee4da through #edc22e)
- [x] Show "New Game" button at top of window

## Phase P7: Input handling — keyboard directions → slide & merge ✅ COMPLETED

- [x] Handle keyboard input in `update()` for arrow keys via egui input system
- [x] Map each key to a `Direction` and call `grid.slide_and_merge(direction)`
- [x] If move succeeds (returns true), add random tile + update score
- [x] Re-render the grid after every successful move

## Phase P8: Win / Game Over detection & overlay UI ✅ COMPLETED

- [x] After each move, check `has_won(2048)` → show modal overlay with "You Win!" + "New Game" button
- [x] Check `is_game_over()` → disable further moves, show "Game Over" message + "New Game" button inside grid area
- [x] Track `game_won: bool` flag to prevent re-triggering the win overlay on subsequent moves

## Phase P9: High score persistence ✅ COMPLETED

- [x] Load high score from disk on startup (using `dirs-next` for path)
- [x] Seed loaded value into `ScoreTracker.best_score` if present
- [x] Handle corrupted/missing file gracefully (treat as no high score — default to 0)
- [x] Save new best when current score beats persisted best via update_best_if_higher() integration

## Phase P10: Polish & Configuration ✅ COMPLETED

- [x] Game title displayed at top of window ("Rust 2048") in large bold text
- [x] Improved UI layout: score panel with title, scores centered, New Game button aligned right
- [x] Responsive sizing: grid auto-scales to available width (minimum 300px)
- [ ] Custom font support deferred to post-MVP (default system font used)
- [x] Final `cargo test` — all 18 tests pass ✅
- [x] Final `cargo clippy -- -D warnings` — clean build ✅
- [x] Final `cargo build --release` — optimized binary at target/release/rust-2048 (9.1 MB) ✅

## Phase P11: Custom Font Support — Noto Sans TC/JP/KR ✅ COMPLETED

- [x] Implemented font loading from system paths (Linux OpenType & TrueType, user fonts)
- [x] Used `dirs_next::home_dir()` for proper tilde expansion in font paths
- [x] Set up egui's font system with Noto Sans CJK (includes Traditional Chinese support)
- [x] Applied custom font to all UI text elements via proportional font family
- [x] Fallback mechanism: uses default system font if custom font not found
- [x] Verified `cargo clippy` — no warnings ✅

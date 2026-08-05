# Rust 2048 — Architecture Blueprint

## Project Overview

A cross-platform 2048 puzzle game built in Rust, using `egui` for GUI rendering and supporting custom fonts to avoid Chinese character garbling. Game logic is completely decoupled from the rendering layer.

---

## 1. Scope & Assumptions

| # | Item | Detail |
|---|------|--------|
| S1 | Target platforms | Windows, macOS, Linux (same source code) |
| S2 | GUI framework | `egui` + `eframe` for windowing and immediate-mode rendering |
| S3 | Input methods | Keyboard direction keys (`Up`, `Down`, `Left`, `Right`) + mouse click/tap on UI buttons |
| New Game / Restart button via mouse click |
| S4 | Persistence | High score saved to local JSON file |
| S5 | Font support | Custom font path configurable; defaults included if user does not specify one |
| S6 | No online/network dependency — fully offline single-player game |

---

## 2. Architectural Style & System Context

```
┌──────────────────────────────────────┐
│          egui / eframe               │
│  (windowing, input dispatch,         │
│   immediate-mode rendering)           │
├──────────────────────────────────────┤
│              GUI Layer                │
│  ┌───────────────────────────────┐    │
│  │ GameWindow (implements        │    │
│  │ eframe::App)                   │    │
│  ├───────────────────────────────┤    │
│  │ TileRenderer                    │    │
│  │ ScorePanel                      │    │
│  │ FontLoader                      │    │
│  └───────────────────────────────┘    │
├──────────────────────────────────────┤
│          Game Logic (core)            │
│  ┌───────────────────────────────┐    │
│  │ Grid<Tile>                     │    │
│  │ TileManager                    │    │
│  │ ScoreTracker                   │    │
│  │ MoveEngine                     │    │
│  └───────────────────────────────┘    │
├──────────────────────────────────────┤
│          Persistence                  │
│  ┌───────────────────────────────┐    │
│  │ HighScoreStore                 │    │
│  └───────────────────────────────┘    │
└──────────────────────────────────────┘
```

**Dependency direction:** GUI → Game Logic ← Persistence. The game logic knows nothing about egui or rendering; it is pure data + algorithms.

---

## 3. Module / Bounded Contexts

| Module | Purpose | Key Invariants |
|--------|---------|----------------|
| `game::grid` | 4×4 tile grid, occupancy tracking, coordinate mapping | Grid is always exactly 4 rows × 4 columns; each cell holds `Option<Tile>` |
| `game::tile` | Individual tile value & state (newly placed flag) | Value ≥ 2 and always a power of 2 |
| `game::move_engine` | Slide, merge, add-random-tile operations | Merge only happens between adjacent equal-valued tiles; no chain reactions in one move |
| `game::score_tracker` | Current score + high score logic (in-memory) | Score is non-decreasing; never negative |
| `gui::window` | egui `App` struct — the single top-level entry point for rendering & input | One window, no dialogs |
| `gui::renderer` | Draw grid, tiles, numbers with custom font | Tile colors determined by value (2→light blue, 4→orange, …) |
| `gui::font_loader` | Load user-specified or bundled fonts; fallback to system defaults | Must support at least one TTF/OTF file |
| `persistence::high_score_store` | Save/load high score from disk (JSON) | File is atomically written; corrupted data → reset to 0 |

---

## 4. Dependency Rules

1. **Game logic** depends on zero external crates except standard library (`serde`, `serde_json` for persistence).
2. **GUI layer** depends only on `game::*` and `persistence::*`. No direct disk access from game code.
3. **No circular dependencies**.
4. Public API of each module is defined by its `pub` items; everything else is `pub(crate)`.

---

## 5. Project & Directory Structure

```
Rust2048/
├── Cargo.toml
├── docs/                    # Blueprint, plan, architecture notes
│   ├── blueprint.md         # This file
│   └── implementation_plan.md
├── src/
│   ├── main.rs              # Entry point — eframe::run_native()
│   ├── game/
│   │   ├── mod.rs           # Re-exports everything from sub-modules
│   │   ├── grid.rs          # Grid<Tile> type + coordinate system
│   │   ├── tile.rs          # Tile { value: u16, new: bool }
│   │   ├── move_engine.rs   # slide(), merge(), add_random()
│   │   └── score_tracker.rs # ScoreTracker with current & best
│   ├── gui/
│   │   ├── mod.rs           # Re-exports GUI types
│   │   ├── window.rs        # GameWindow (eframe::App) — orchestrates render + input
│   │   ├── renderer.rs      # TileRenderer, draw_grid(), draw_tile()
│   │   └── font_loader.rs   # load_font(path: &Path) -> Result<FontData>
│   └── persistence/
│       ├── mod.rs           # Re-exports persistence types
│       └── high_score_store.rs  # save/load HighScore to/from disk
├── fonts/                   # Optional bundled custom fonts (user-provided TTF files)
│   └── .gitkeep
└── assets/                  # Optional image assets (background, tile textures)
    └── .gitkeep
```

---

## 6. Component Responsibilities & Contracts

### `game::tile::Tile`

```rust
pub struct Tile {
    pub value: u16,          // Always a power of 2 ≥ 2
    pub newly_placed: bool,   // Set true when first added; cleared after rendering pass
}

impl Tile {
    /// Create a new tile with the given value.
    pub fn new(value: u16) -> Self;
    /// Check if this tile can be merged into `other`. Returns (new_value, can_merge).
    pub fn try_merge_into(&self, other: &Tile) -> Option<(u16, bool)>;
}
```

**Invariant:** Value is always a power of 2 and ≥ 2. If a caller tries to create an invalid tile, panic in debug mode, return `Err` otherwise.

### `game::grid::Grid`

```rust
pub struct Grid {
    cells: [[Option<Tile>; 4]; 4],   // Row-major storage
}

impl Grid {
    /// Create an empty grid (all cells None).
    pub fn new() -> Self;
    /// Add a random tile at an empty cell. Returns the newly placed tile.
    pub fn add_random_tile(&mut self) -> Option<Tile>;
    /// Slide all tiles in `direction` and merge equal-valued neighbors.
    /// Returns true if any tile moved or merged (i.e., grid actually changed).
    pub fn slide_and_merge(&mut self, direction: Direction) -> bool;
    /// Check whether the game is over (no empty cells AND no possible merges in any direction).
    pub fn is_game_over(&self) -> bool;
    /// Check if a winning tile (2048) exists.
    pub fn has_won(&self, target: u16) -> bool;
    /// Clone the grid state for save/restore or undo-if-needed future use.
    pub fn clone_state(&self) -> GridState;
}

#[derive(Clone)]
pub struct GridState {
    cells: [[Option<Tile>; 4]; 4],
}
```

**Direction enum:**

```rust
pub enum Direction { Up, Down, Left, Right }
```

### `game::move_engine` — Slide & Merge Algorithm

The core algorithm per row/column:
1. **Compress**: Remove all `None`s → shift non-None tiles toward the move direction.
2. **Merge adjacent equal pairs**: Scan from the move-direction side; if two adjacent tiles have the same value, merge them into one (value × 2) and mark a new tile for score addition. The merged tile cannot participate in further merges this turn.
3. **Pad with `None`** to fill remaining cells on the far side.

```rust
pub fn slide_line(line: &mut [Option<Tile>]) -> MergeResult {
    // Returns (score_gained, tiles_merged_count)
}
```

**Invariants:**
- No tile merges more than once per move.
- Slide direction is always toward the edge of the grid (left for `Left`, top for `Up`, etc.).

### `game::score_tracker::ScoreTracker`

```rust
pub struct ScoreTracker {
    current_score: u32,
    best_score: Option<u32>,   // Loaded from disk on startup; None if never saved.
}

impl ScoreTracker {
    pub fn new() -> Self;
    pub fn add_to_current(&mut self, amount: u32);
    pub fn update_best_if_higher(&mut self) -> Option<u32>;  // Returns Some(new_best) if updated
    pub fn current_score(&self) -> u32;
    pub fn best_score(&self) -> u32 { self.best_score.unwrap_or(0) }
}
```

### `gui::window::GameWindow` (eframe::App)

```rust
pub struct GameWindow {
    grid: Grid,
    score_tracker: ScoreTracker,
    font_data: Option<FontData>,   // egui FontData loaded by font_loader
    game_won: bool,                // Track win state; show overlay once
}

impl eframe::App for GameWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard (Direction keys) and mouse input
    }
}

impl GameWindow {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self;
    fn handle_direction_input(&mut self, direction: Direction);
    fn restart_game(&mut self);
    fn show_grid_ui(&self, ctx: &egui::Context);
    fn show_score_panel(&self, ctx: &egui::Context);
}
```

**Input handling:**
- Keyboard `ArrowUp/Down/Left/Right` → call `slide_and_merge(direction)`. If move returns `true`, refresh UI.
- Mouse click on "New Game" button → `restart_game()`.

### `gui::renderer` — Tile Drawing

Tile colors per value (standard 2048 palette):

| Value | Background Color | Text Color |
|-------|------------------|------------|
| 2     | #eee4da          | #776e65    |
| 4     | #ede0c8          | #776e65    *|
| 8     | #f2b17f          | #f9f6f2    |
| 16    | #f59563          | #f9f6f2    |
| 32    | #f67c5f          | #f9f6f2    |
| 64    | #f65e3b          | #f9f6f2    |
| 128   | #edcf72          | #f9f6f2    |
| 256   | #edcc61          | #f9f6f2    |
| 512   | #edc850          | #f9f6f2    |
| 1024  | #edc53f          | #f9f6f2    |
| 2048  | #edc22e          | #f9f6f2    |

```rust
pub fn draw_grid(ctx: &egui::Context, grid: &Grid) { /* … */ }
pub fn draw_tile_at(ctx: &egui::Context, pos: Pos2, tile: &Tile) { /* … */ }
fn value_to_color(value: u16) -> egui::Color32;
```

### `gui::font_loader` — Custom Font Loading

```rust
pub struct FontData {
    pub face: egui::FontFamily,
}

pub fn load_custom_font(path: &std::path::Path) -> Result<FontData, FontError>;
// Returns system default font if no custom path is provided.

#[derive(Debug)]
pub enum FontError {
    FileNotFound(std::path::PathBuf),
    InvalidFormat(String),
}
```

### `persistence::high_score_store`

```rust
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct HighScoreFile {
    best: u32,
    last_updated: String,   // ISO-8601 timestamp
}

pub fn save_high_score(path: &Path, score: u32) -> Result<(), PersistenceError>;
pub fn load_high_score(path: &Path) -> Option<u32>;
```

Default file path: `~/.local/share/rust_2048/high_score.json` on Linux (or platform-appropriate via `dirs-next`).

---

## 7. Public Contracts & Data Flows

### Game Flow

```
User presses Arrow key
        │
        ▼
GameWindow::handle_direction_input(direction)
        │
        ▼
Grid::slide_and_merge(direction) → returns bool (changed?)
        │                              │
        │         ┌────────────────────┘
        │         ▼
        │   MoveEngine::slide_line per row/col → MergeResult
        │       ScoreTracker::add_to_current(score_gained)
        │       Grid::add_random_tile() if move succeeded
        │
        ▼
GameWindow::show_grid_ui(&ctx)  // re-render everything
GameWindow::show_score_panel(&ctx)
```

### Win / Game Over Detection

After every successful move:
1. Check `Grid::has_won(2048)` → if true, set `game_won = true`, show overlay.
2. Check `Grid::is_game_over()` → if true, disable further moves, show "Game Over" button.

### High Score Persistence

Every time `ScoreTracker::update_best_if_higher()` returns a new best:
1. Persist to disk via `HighScoreStore::save()`.
2. On startup, load existing high score and seed into `ScoreTracker`.

---

## 8. Error & Validation Boundaries

| Layer | Error Type | Handling Strategy |
|-------|-----------|-------------------|
| Font loading | `FontError` (file missing / invalid format) | Use system default font; log warning to console |
| Persistence | I/O error on save/load | Log warning, treat as "no previous high score" — do not crash the game |
| Tile creation | Invalid value (< 2 or not power of 2) | Debug panic in debug builds; return `Option::None` in release |

---

## 9. Testing Strategy

### Unit Tests (game logic only — no egui dependency)

All tests live in `src/game/`, separate from GUI code:

```
tests/
├── grid_tests.rs       # add_random, slide_and_merge in all directions
├── move_engine_tests.rs  # slide_line edge cases, merge rules
├── score_tracker_tests.rs   # update_best_if_higher logic
└── win_lose_tests.rs     # is_game_over, has_won(2048)
```

Key test scenarios:
- Slide empty row → no change.
- Merge two equal tiles → value doubles, second tile removed.
- Three identical consecutive tiles in one direction → only the leading pair merges (no chain).
- Game over detection on full grid with no possible merge.
- Win detection when 2048 appears anywhere.

### Integration Test

A single `tests/integration.rs` that runs a simulated game:
1. Start new game, verify initial 2 tiles placed.
2. Press all 4 directions repeatedly, verify score tracks correctly and no crashes occur.
3. Verify high score persists across "restart" (simulated by reloading ScoreTracker from disk).

---

## 10. Implementation Sequence

| Phase | Task | Dependencies | Estimated Effort |
|-------|------|-------------|-----------------|
| P0   | Scaffold project structure, add dependencies (`egui`, `eframe`, `serde`) | None | 30 min |
| P1   | Implement `game::tile` + `game::grid` (data model) | None | 1 hr |
| P2   | Implement `game::move_engine` (slide + merge algorithm) | P1 | 1.5 hrs |
| P3   | Implement `game::score_tracker` | P2 | 30 min |
| P4   | Write unit tests for game logic | P2–P3 | 1 hr |
| P5   | Scaffold GUI: `GameWindow` struct, basic egui rendering of empty grid | P0 | 1 hr |
| P6   | Connect GUI to game logic: render actual tiles + numbers using custom font | P4, P5 | 2 hrs |
| P7   | Input handling (keyboard directions) → call `slide_and_merge` and re-render | P3–P6 | 1.5 hrs |
| P8   | Win/Game Over detection UI overlay | P2, P7 | 1 hr |
| P9   | High score persistence (load on start, save on new best) | P3, P4 | 1 hr |
| P10  | Polish: tile animations (optional), New Game button, font customization via config file | P8–P9 | 2 hrs |

---

## 11. Open Decisions & Risks

| # | Decision / Risk | Status | Mitigation |
|---|-----------------|--------|------------|
| O1 | Custom font path: hardcode in GUI or read from a config file? | **Open** — default to hardcode, add config file in P10 if needed. |
| O2 | Tile animation (smooth slide) — egui is immediate-mode and does not natively support tweening. | Will use simple "pop" fade for newly placed tiles; full slide animation deferred to post-MVP or skipped. |
| O3 | `egui` requires `wgpu` backend by default — adds compile time & binary size. | Acceptable trade-off for cross-platform GPU rendering; no alternative desired at this point. |
| O4 | Game Over overlay: should user be able to "continue" past 2048? | Standard behavior: game ends when no moves remain, even if >2048 tiles exist. No continue mode in MVP. |
| O5 | Persistence path varies by OS — use `dirs-next` crate for platform-appropriate XDG/home paths. | Add as dependency; abstract behind one function call. |

---

## 12. Public API Inventory (for consumers / future extensions)

These are the surface items that external code or tests should rely on:

| Module | Type/Function | Signature Summary |
|--------|--------------|-------------------|
| `game::tile` | `Tile { value, newly_placed }` | Construct via `Tile::new(value)` |
| `game::grid` | `Grid { cells }` | Construct via `Grid::new()`; public methods: `add_random_tile`, `slide_and_merge`, `is_game_over`, `has_won`, `clone_state` |
| `game::move_engine` | `Direction` enum | Values: `Up, Down, Left, Right` |
| `game::score_tracker` | `ScoreTracker { current_score, best_score }` | Methods: `add_to_current`, `update_best_if_higher`, accessors |
| `gui::window` | `GameWindow : eframe::App` | Lifecycle: `new(cc)`, `update(ctx, frame)` — internal methods not part of public contract |
| `persistence::high_score_store` | Functions | `save_high_score(path, score)`, `load_high_score(path)` |

---

## 13. Review Notes (Phase 3 Preview)

Before locking this blueprint, the following areas need stress-testing:

- **Slide & merge correctness**: Three consecutive equal tiles in one direction — only the leading pair merges; the trailing tile does NOT chain. This is the most common bug source in 2048 clones.
- **Random tile placement**: Standard 90% chance of 2, 10% chance of 4. Must be documented and tested.
- **Win condition timing**: The game should only show "You Win!" after a move that produces 2048 — not on initial setup or when loading a saved state (though we are not saving full state in MVP).
- **Grid immutability during slide**: Slides must be computed from a snapshot of the current row/column, then written back atomically to avoid partial-update bugs.

---

## 14. Approved Dependencies (Cargo.toml)

```toml
[dependencies]
egui = "0.29"          # Immediate-mode GUI
eframe = "0.29"        # egui application wrapper
serde = { version = "1", features = ["derive"] }  # JSON serialization
serde_json = "1"       # JSON parsing for high score persistence
dirs-next = "2"        # Platform-appropriate user data paths
```

All other dependencies are optional and deferred to Phase P10 (polish).

use crate::game::{Direction, Grid, ScoreTracker};
use eframe::egui;
use egui::Key;

/// Main game window struct implementing `eframe::App`.
#[allow(dead_code)]
pub struct GameWindow {
    grid: Grid,
    score_tracker: ScoreTracker,
    game_won: bool,
}

impl eframe::App for GameWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for keyboard input (arrow keys → direction slides).
        ctx.input(|i| {
            if i.key_pressed(Key::ArrowUp) {
                self.handle_direction_input(Direction::Up);
            } else if i.key_pressed(Key::ArrowDown) {
                self.handle_direction_input(Direction::Down);
            } else if i.key_pressed(Key::ArrowLeft) {
                self.handle_direction_input(Direction::Left);
            } else if i.key_pressed(Key::ArrowRight) {
                self.handle_direction_input(Direction::Right);
            }
        });

        // Top panel with game title, score display, and New Game button.
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Title on the left.
                let title_style = egui::RichText::new("Rust 2048")
                    .size(28.0)
                    .font(self.get_font_id());
                ui.label(title_style);

                // Spacer to push scores and button to the right.
                ui.add_space(20.0);

                // Scores in the middle.
                let score_text = format!("Score: {}", self.score_tracker.get_current());
                let best_text = format!("Best: {}", self.score_tracker.get_best());
                let score_style = egui::RichText::new(score_text)
                    .size(16.0)
                    .font(self.get_font_id());
                let best_style = egui::RichText::new(best_text)
                    .size(16.0)
                    .font(self.get_font_id());
                ui.label(score_style);
                ui.separator();
                ui.label(best_style);

                // Spacer before button.
                ui.add_space(20.0);

                if ui.button("New Game").clicked() {
                    self.restart_game();
                }
            });
        });

        // Center panel with the grid.
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_grid_ui(ui);
        });

        // Show win overlay if game_won flag is set.
        if self.game_won {
            self.show_win_overlay(ctx);
        }
    }
}

impl GameWindow {
    /// Create a new game window with empty grid and loaded high score.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let grid = Grid::new();
        let mut score_tracker = ScoreTracker::new();

        // Load persisted best score on startup if file exists.
        let path = crate::persistence::default_path();
        if let Some(best) = crate::persistence::load_high_score(&path) {
            score_tracker.best_score = Some(best);
        }

        // Set custom font for the entire UI using Noto Sans CJK (includes Traditional Chinese).
        let _font_id = Self::setup_custom_font(&cc.egui_ctx);

        GameWindow {
            grid,
            score_tracker,
            game_won: false,
        }
    }

    /// Get the font ID for use in text styling.
    fn get_font_id(&self) -> egui::FontId {
        egui::FontId::proportional(14.0)
    }

    /// Setup custom font (Noto Sans CJK) by loading it and setting as default.
    fn setup_custom_font(ctx: &egui::Context) -> egui::FontId {
        // Helper function to expand tilde in paths (replace with home dir).
        let expand_path = |path: &str| -> String {
            if path.starts_with("~/") || path == "~" {
                match dirs_next::home_dir() {
                    Some(home) => home.join(&path[2..]).to_string_lossy().into_owned(),
                    None => path.to_string(), // Fallback to original path.
                }
            } else {
                path.to_string()
            }
        };

        // Try common paths for Noto Sans CJK on different platforms.
        let font_paths = [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc",      // Linux (OpenType)
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Bold.ttc",       // Linux (TrueType alternative)
            "~/.local/share/fonts/NotoSansCJK-Bold.ttc",                  // User fonts directory
        ];

        for font_path in &font_paths {
            let expanded = expand_path(font_path);
            if let Ok(data) = std::fs::read(&expanded) {
                if data.len() > 100 {
                    // Load the font as a custom font into egui's system.
                    ctx.set_fonts(egui::FontDefinitions::default());
                    let mut fonts = egui::FontDefinitions::default();
                    fonts.font_data.insert(
                        "custom_font".to_string(),
                        egui::FontData::from_owned(data),
                    );

                    // Add the font to the monospace and proportional families.
                    fonts.families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, "custom_font".to_string());
                    fonts.families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push("custom_font".to_string());

                    ctx.set_fonts(fonts);
                    return egui::FontId::proportional(14.0); // Return the font ID for use in styling.
                }
            }
        }

        // Fallback to default proportional font if custom font not found.
        egui::FontId::proportional(14.0)
    }

    /// Handle keyboard direction input — call slide_and_merge and update state.
    fn handle_direction_input(&mut self, direction: Direction) {
        let moved = self.grid.slide_and_merge(direction);

        if moved {
            // Add a random tile after successful move.
            self.grid.add_random_tile();

            // Check for new best score and persist it.
            if let Some(new_best) = self.score_tracker.update_best_if_higher() {
                let path = crate::persistence::default_path();
                if let Err(e) = crate::persistence::save_high_score(&path, new_best) {
                    eprintln!("Failed to save high score: {}", e);
                }
            }

            // Check win/lose conditions.
            if !self.game_won && self.grid.has_won(2048) {
                self.game_won = true;
            } else if self.grid.is_game_over() {
                // Game over — stop further moves (handled by checking is_game_over in update).
                return;
            }
        }

        // Clear newly_placed flags on all tiles after rendering pass.
        for row in &mut self.grid.cells {
            for tile in row.iter_mut().flatten() {
                tile.newly_placed = false;
            }
        }
    }

    /// Restart the game with a fresh grid and reset current score (keep best).
    fn restart_game(&mut self) {
        self.grid = Grid::new();
        self.score_tracker.reset_current();
        self.game_won = false;

        // Place two initial tiles.
        self.grid.add_random_tile();
        self.grid.add_random_tile();
    }

    /// Render the 4×4 grid with colored tiles and numbers.
    fn show_grid_ui(&mut self, ui: &mut egui::Ui) {
        let size = (ui.available_width() - 20.0).max(300.0); // minimum cell width.
        let cell_size = size / 4.0;

        // Use vertical layout so each row starts a new line, and horizontal_wrapped
        // to flow tiles within each row (wrapping only if the window is too narrow).
        ui.vertical(|ui| {
            for row in &self.grid.cells {
                ui.horizontal_wrapped(|ui| {
                    for tile_opt in row.iter() {
                        if let Some(tile) = tile_opt {
                            draw_tile(ui, cell_size, tile);
                        } else {
                            draw_empty_cell(ui, cell_size);
                        }
                    }
                });
            }
        });

        // Check game over — disable moves if no empty cells and no merges possible.
        if self.grid.is_game_over() && !self.game_won {
            ui.centered_and_justified(|ui| {
                let response = ui.button("Game Over\nNew Game");
                if response.clicked() {
                    self.restart_game();
                }
            });
        }
    }

    /// Show the "You Win!" overlay with a New Game button.
    fn show_win_overlay(&mut self, ctx: &egui::Context) {
        egui::Window::new("Win!")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("You reached 2048!");

                if ui.button("New Game").clicked() {
                    self.restart_game();
                }
            });
    }
}

/// Draw a single tile with value and background color.
fn draw_tile(ui: &mut egui::Ui, cell_size: f32, tile: &crate::game::Tile) {
    let rect = ui.available_rect_before_wrap();

    // Calculate position for this cell based on cursor.
    let pos = egui::Pos2::new(rect.min.x, rect.min.y);
    let cell_rect = egui::Rect::from_min_size(pos, egui::vec2(cell_size, cell_size));

    ui.painter()
        .rect_filled(cell_rect, 8.0, tile_color(tile.value));

    let text_color = text_color_for_tile(tile.value);
    let font_size = cell_size * 0.4;

    // Use proportional font with custom size for better readability on tiles.
    ui.painter().text(
        cell_rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{}", tile.value),
        egui::FontId::proportional(font_size),
        text_color,
    );

    // Advance the cursor by one cell width (horizontal layout).
    ui.advance_cursor_after_rect(cell_rect);
}

/// Draw an empty cell placeholder.
fn draw_empty_cell(ui: &mut egui::Ui, cell_size: f32) {
    let rect = ui.available_rect_before_wrap();

    let pos = egui::Pos2::new(rect.min.x, rect.min.y);
    let cell_rect = egui::Rect::from_min_size(pos, egui::vec2(cell_size, cell_size));

    ui.painter()
        .rect_filled(cell_rect, 8.0, egui::Color32::from_rgb(204, 193, 186));

    ui.advance_cursor_after_rect(cell_rect);
}

/// Standard 2048 palette: background color by tile value.
fn tile_color(value: u16) -> egui::Color32 {
    match value {
        2 => egui::Color32::from_rgb(238, 228, 218),   // #eee4da
        4 => egui::Color32::from_rgb(237, 224, 200),   // #ede0c8
        8 => egui::Color32::from_rgb(242, 177, 127),   // #f2b17f
        16 => egui::Color32::from_rgb(245, 149, 99),   // #f59563
        32 => egui::Color32::from_rgb(246, 124, 95),   // #f67c5f
        64 => egui::Color32::from_rgb(246, 94, 59),    // #f65e3b
        128 => egui::Color32::from_rgb(237, 207, 114), // #edcf72
        256 => egui::Color32::from_rgb(237, 204, 97),  // #edcc61
        512 => egui::Color32::from_rgb(237, 200, 80),  // #edc850
        1024 => egui::Color32::from_rgb(237, 197, 63), // #edc53f
        _ => egui::Color32::from_rgb(237, 194, 46),    // default gold (#edc22e)
    }
}

/// Text color for tiles: dark on light backgrounds (≤4), white on darker.
fn text_color_for_tile(value: u16) -> egui::Color32 {
    if value <= 4 {
        egui::Color32::from_rgb(119, 110, 101) // #776e65 dark gray for light tiles.
    } else {
        egui::Color32::WHITE
    }
}

use super::move_engine::{slide_line, Direction};
use super::tile::Tile;

/// 4×4 game grid storing tiles in row-major order.
#[derive(Debug, Clone)]
pub struct Grid {
    pub cells: [[Option<Tile>; 4]; 4], // Public for test access and renderer use.
}

impl Grid {
    /// Create an empty grid (all cells None).
    pub fn new() -> Self {
        Grid {
            cells: [[None; 4]; 4],
        }
    }

    /// Add a random tile at a randomly-chosen empty cell.
    /// Returns the newly placed tile, or `None` if grid is full.
    pub fn add_random_tile(&mut self) -> Option<Tile> {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Simple pseudo-random using system time (adequate for MVP).
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        let seed = now as usize;

        // Find all empty cells.
        let empty_cells: Vec<(usize, usize)> = (0..4)
            .flat_map(|r| (0..4).map(move |c| (r, c)))
            .filter(|&(r, c)| self.cells[r][c].is_none())
            .collect();

        if empty_cells.is_empty() {
            return None; // Grid is full — no space to place a tile.
        }

        // Pick a random empty cell using seed-based index.
        let idx = seed % empty_cells.len();
        let (r, c) = empty_cells[idx];

        // 90% chance of value 2, 10% chance of value 4.
        let tile_value: u16 = if seed % 10 < 9 { 2 } else { 4 };

        let tile = Tile::new(tile_value).expect("value must be valid power-of-2");
        self.cells[r][c] = Some(tile);

        Some(tile)
    }

    /// Slide all tiles in `direction` and merge equal-valued neighbors.
    /// Returns true if the grid actually changed (any tile moved or merged).
    pub fn slide_and_merge(&mut self, direction: Direction) -> bool {
        let old_cells = self.cells; // Copy since [[Option<Tile>; 4]; 4] is Copy

        match direction {
            Direction::Left => {
                for row in &mut self.cells {
                    slide_line(row);
                }
            }
            Direction::Right => {
                // Right: reverse each row, slide toward index 0 (= original right), then un-reverse.
                for row in &mut self.cells {
                    row.reverse();
                    slide_line(row);
                    row.reverse();
                }
            }
            Direction::Up => {
                // Extract each column from snapshot, slide toward top (index 0), write back.
                let mut cols: [[Option<Tile>; 4]; 4] = [[None; 4]; 4];

                for c in 0..4 {
                    // Pull out all cells[r][c] into a single column vector.
                    let mut col_vec: Vec<Option<Tile>> = (0..4)
                        .map(|r| old_cells[r][c])
                        .collect();
                    slide_line(&mut col_vec); // Compress toward row 0 (top).

                    for r in 0..4 {
                        cols[r][c] = col_vec[r];
                    }
                }

                self.cells.copy_from_slice(&cols);
            }
            Direction::Down => {
                // Extract each column from snapshot, reverse before slide_line so tiles compress toward bottom (row 3), then un-reverse.
                let mut cols: [[Option<Tile>; 4]; 4] = [[None; 4]; 4];

                for c in 0..4 {
                    // Pull out all cells[r][c] into a single column vector.
                    let mut col_vec: Vec<Option<Tile>> = (0..4)
                        .map(|r| old_cells[r][c])
                        .collect();
                    col_vec.reverse(); // Flip so index 0 is now the bottom of the grid.
                    slide_line(&mut col_vec); // Compress toward index 0 (= original bottom).
                    col_vec.reverse(); // Restore top-to-bottom order.

                    for r in 0..4 {
                        cols[r][c] = col_vec[r];
                    }
                }

                self.cells.copy_from_slice(&cols);
            }
        }

        // Compare with snapshot to detect any change.
        self.cells != old_cells
    }

    /// Check whether the game is over: no empty cells AND no possible merges in any direction.
    pub fn is_game_over(&self) -> bool {
        // If any cell is empty, game continues.
        for row in &self.cells {
            for cell in row {
                if cell.is_none() {
                    return false;
                }
            }
        }

        // No empty cells — check if any adjacent pair can merge.
        let any_merge = [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ]
        .iter()
        .any(|&dir| self.can_slide_in_direction(dir));

        !any_merge
    }

    /// Check if a winning tile (2048) exists on the board.
    pub fn has_won(&self, target: u16) -> bool {
        self.cells
            .iter()
            .flatten()
            .any(|t| t.is_some() && t.as_ref().unwrap().value == target)
    }

    /// Clone the grid state for save/restore or undo-if-needed future use.
    #[allow(dead_code)]
    pub fn clone_state(&self) -> GridState {
        GridState { cells: self.cells }
    }

    /// Check if a slide in the given direction would change anything (used by `is_game_over`).
    fn can_slide_in_direction(&self, direction: Direction) -> bool {
        // Build a temporary grid with the current state and try sliding.
        let mut temp = Grid { cells: self.cells };
        temp.slide_and_merge(direction)
    }

    /// Get the number of non-empty tiles (used for game-over detection).
    #[allow(dead_code)]
    pub fn occupied_count(&self) -> usize {
        self.cells.iter().flatten().filter(|c| c.is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{ScoreTracker, Tile};

    #[test]
    fn test_grid_new_empty() {
        let grid = Grid::new();
        assert_eq!(grid.occupied_count(), 0);
    }

    #[test]
    fn test_add_random_tile_increases_occupancy() {
        let mut grid = Grid::new();
        let tile = grid.add_random_tile().unwrap();
        assert!(tile.value == 2 || tile.value == 4, "Should be 2 or 4");
        assert_eq!(grid.occupied_count(), 1);
    }

    #[test]
    fn test_add_random_tile_fails_when_full() {
        let mut grid = Grid::new();
        // Fill all cells with tiles (we need to manually set them since add_random is probabilistic).
        for r in 0..4 {
            for c in 0..4 {
                grid.cells[r][c] = Some(Tile::new(2).unwrap());
            }
        }
        assert!(
            grid.add_random_tile().is_none(),
            "Should fail when grid is full"
        );
    }

    #[test]
    fn test_slide_and_merge_left_moves_tiles() {
        let mut grid = Grid::new();
        // Set up a row with one tile at the end (right side).
        grid.cells[0][3] = Some(Tile::new(2).unwrap());

        assert!(grid.slide_and_merge(Direction::Left)); // Should move to left.
        assert_eq!(grid.cells[0][0].as_ref().unwrap().value, 2);
        assert!(grid.cells[0][3].is_none());
    }

    #[test]
    fn test_slide_and_merge_right_moves_tiles() {
        let mut grid = Grid::new();
        // Tile at far left → should slide to far right.
        grid.cells[0][0] = Some(Tile::new(2).unwrap());

        assert!(grid.slide_and_merge(Direction::Right));
        assert_eq!(grid.cells[0][3].as_ref().unwrap().value, 2);
        assert!(grid.cells[0][0].is_none());
    }

    #[test]
    fn test_slide_and_merge_right_merges() {
        let mut grid = Grid::new();
        // Two equal tiles at positions [0] and [1] → should merge at far right.
        grid.cells[0][0] = Some(Tile::new(4).unwrap());
        grid.cells[0][1] = Some(Tile::new(4).unwrap());

        assert!(grid.slide_and_merge(Direction::Right));
        // Merged tile should be at index 3 (rightmost), consumed tile gone.
        assert_eq!(grid.cells[0][3].as_ref().unwrap().value, 8);
        assert!(grid.cells[0][2].is_none());
        assert!(grid.cells[0][1].is_none());
    }

    #[test]
    fn test_slide_and_merge_up_moves_tiles() {
        let mut grid = Grid::new();
        // Tile at bottom (row 3) → should slide to top (row 0).
        grid.cells[3][2] = Some(Tile::new(4).unwrap());

        assert!(grid.slide_and_merge(Direction::Up));
        assert_eq!(grid.cells[0][2].as_ref().unwrap().value, 4);
        assert!(grid.cells[3][2].is_none());
    }

    #[test]
    fn test_slide_and_merge_up_merges() {
        let mut grid = Grid::new();
        // Two equal tiles in column 1 at rows 0 and 1 → already adjacent to top.
        // After Up: they compress to row 0, merge into one tile with value 4.
        grid.cells[0][1] = Some(Tile::new(2).unwrap());
        grid.cells[1][1] = Some(Tile::new(2).unwrap());

        assert!(grid.slide_and_merge(Direction::Up));
        // Row 0 gets the merged tile, row 1 consumed.
        assert_eq!(grid.cells[0][1].as_ref().unwrap().value, 4);
        assert!(grid.cells[1][1].is_none());
    }

    #[test]
    fn test_slide_and_merge_down_moves_tiles() {
        let mut grid = Grid::new();
        // Tile at top (row 0) → should slide to bottom (row 3).
        grid.cells[0][2] = Some(Tile::new(8).unwrap());

        assert!(grid.slide_and_merge(Direction::Down));
        assert_eq!(grid.cells[3][2].as_ref().unwrap().value, 8);
        assert!(grid.cells[0][2].is_none());
    }

    #[test]
    fn test_slide_and_merge_down_merges() {
        let mut grid = Grid::new();
        // Two equal tiles in column 1 at rows 2 and 3 → already adjacent to bottom.
        // After Down: they compress to row 3, merge into one tile with value 32.
        grid.cells[2][1] = Some(Tile::new(16).unwrap());
        grid.cells[3][1] = Some(Tile::new(16).unwrap());

        assert!(grid.slide_and_merge(Direction::Down));
        // Row 3 gets the merged tile, row 2 consumed.
        assert_eq!(grid.cells[3][1].as_ref().unwrap().value, 32);
        assert!(grid.cells[2][1].is_none());
    }

    #[test]
    fn test_slide_and_merge_down_no_change_empty() {
        let mut grid = Grid::new();
        assert!(!grid.slide_and_merge(Direction::Down)); // No tiles to move.
    }

    #[test]
    fn test_slide_and_merge_right_no_change_empty() {
        let mut grid = Grid::new();
        assert!(!grid.slide_and_merge(Direction::Right)); // No tiles to move.
    }

    #[test]
    fn test_slide_and_merge_does_nothing_on_empty() {
        let mut grid = Grid::new();
        assert!(!grid.slide_and_merge(Direction::Left)); // No tiles to move.
    }

    #[test]
    fn test_is_game_over_detects_full_no_merge() {
        let mut grid = Grid::new();
        // Fill with non-mergeable tiles: ensure NO two adjacent cells have the same value.
        // Use a checkerboard pattern of values so merges are impossible in any direction.
        for r in 0..4 {
            for c in 0..4 {
                let val = if (r + c) % 2 == 0 { 2u16 } else { 4u16 };
                grid.cells[r][c] = Some(Tile::new(val).unwrap());
            }
        }

        assert!(
            grid.is_game_over(),
            "Should be game over — no merges possible"
        );
    }

    #[test]
    fn test_has_won_returns_true_for_2048() {
        let mut grid = Grid::new();
        // Place a 2048 tile at [0][0].
        grid.cells[0][0] = Some(Tile::new(2048).unwrap());

        assert!(grid.has_won(2048), "Should detect win when 2048 exists");
    }

    #[test]
    fn test_score_tracker_add_to_current() {
        let mut tracker = ScoreTracker::new();
        tracker.add_to_current(5);
        assert_eq!(tracker.get_current(), 5);
    }

    #[test]
    fn test_score_tracker_update_best_if_higher() {
        let mut tracker = ScoreTracker::new();
        tracker.current_score = 10;
        let new_best = tracker.update_best_if_higher().unwrap();
        assert_eq!(new_best, 10); // First game — current becomes best.

        tracker.current_score = 5; // Lower than best — no update.
        assert!(tracker.update_best_if_higher().is_none());

        tracker.current_score = 20; // Higher than best — should update.
        let new_best = tracker.update_best_if_higher().unwrap();
        assert_eq!(new_best, 20);
    }
}

/// Immutable snapshot of grid state. Useful for undo or save functionality.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GridState {
    cells: [[Option<Tile>; 4]; 4],
}

impl GridState {
    /// Restore a previous game state by replacing all cells.
    #[allow(dead_code)]
    pub fn restore(&self) -> Grid {
        Grid { cells: self.cells }
    }
}

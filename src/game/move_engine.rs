use super::tile::Tile;

/// Direction of a slide/merge operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Result of a single line (row or column) slide-and-merge pass.
#[derive(Debug)]
#[allow(dead_code)]
pub struct MergeResult {
    /// Total score gained from merges in this line.
    pub score_gained: u32,
    /// Number of tiles merged.
    pub tiles_merged_count: usize,
}

/// Slide a single row or column toward index 0 (the "move direction" side).
///
/// Algorithm per line:
///   1. **Compress** — remove all `None` values, shifting non-None tiles to the front.
///   2. **Merge adjacent equal pairs** — scan from move-direction side; if two adjacent
///      tiles have the same value and neither has been merged this turn, merge them into
///      one (value × 2) and record score. The merged tile is marked so it cannot be used
///      again in this pass.
///   3. **Pad with `None`** — fill remaining cells at the end of the line with `None`.
pub fn slide_line(line: &mut [Option<Tile>]) -> MergeResult {
    let mut score_gained = 0u32;
    let mut tiles_merged_count = 0usize;

    // Step 1: Compress — move all non-None tiles to the front.
    let compressed_tiles: Vec<Tile> = line.iter().filter_map(|c| *c).collect();

    // Convert to Option<Vec> form for easier manipulation.
    let mut working: Vec<Option<Tile>> = compressed_tiles.into_iter().map(Some).collect();

    // Step 2: Merge adjacent equal pairs from the front (move-direction side).
    let mut merged_indices: Vec<bool> = vec![false; working.len()];
    let mut i = 0;
    while i < working.len().saturating_sub(1) {
        if !merged_indices[i] && working[i].is_some() && working[i + 1].is_some() {
            let a_val = working[i].as_ref().unwrap().value;
            let b_val = working[i + 1].as_ref().unwrap().value;

            if a_val == b_val {
                // Merge: combine into one tile with doubled value.
                let new_value = a_val * 2;
                let merged_score = new_value as u32;

                working[i] = Tile::new(new_value).map(|mut t| {
                    t.newly_placed = false; // merged tile is no longer "new"
                    t
                });
                working[i + 1] = None; // remove the consumed tile.

                merged_indices[i] = true;
                score_gained += merged_score;
                tiles_merged_count += 1;

                i += 2; // skip past the just-merged pair (no chain).
                continue;
            }
        }
        i += 1;
    }

    // Step 3: Pad with `None` to fill remaining cells.
    working.resize(4, None);

    line.copy_from_slice(&working[..4]);

    MergeResult {
        score_gained,
        tiles_merged_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slide_line_empty() {
        let mut line = vec![None; 4];
        let result = slide_line(&mut line);
        assert_eq!(result.score_gained, 0);
        assert_eq!(line, [None, None, None, None]);
    }

    #[test]
    fn test_slide_line_two_equal_merge() {
        let mut line = vec![
            Some(Tile::new(2).unwrap()),
            Some(Tile::new(2).unwrap()),
            None,
            None,
        ];
        let result = slide_line(&mut line);

        assert_eq!(result.score_gained, 4); // Merged to 4 → score += 4.
        assert_eq!(line[0].as_ref().unwrap().value, 4);
        assert_eq!(line[1], None); // Second tile consumed.
    }

    #[test]
    fn test_slide_line_three_equal_no_chain() {
        let mut line = vec![
            Some(Tile::new(2).unwrap()),
            Some(Tile::new(2).unwrap()),
            Some(Tile::new(2).unwrap()),
            None,
        ];
        let result = slide_line(&mut line);

        // Only leading pair merges → one merge, score += 4. The third tile does NOT chain.
        assert_eq!(result.score_gained, 4);
        assert_eq!(line[0].as_ref().unwrap().value, 4);
        assert_eq!(line[1], None);
        assert_eq!(line[2].as_ref().unwrap().value, 2); // Third tile remains.
    }

    #[test]
    fn test_slide_line_compress_shifts_tiles() {
        let mut line = vec![
            None,
            Some(Tile::new(8).unwrap()),
            None,
            Some(Tile::new(16).unwrap()),
        ];
        slide_line(&mut line);

        // Tiles should shift to the front.
        assert_eq!(line[0].as_ref().unwrap().value, 8);
        assert_eq!(line[1].as_ref().unwrap().value, 16);
        assert_eq!(line[2], None);
        assert_eq!(line[3], None);
    }
}

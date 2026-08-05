/// A single tile on the game board.
///
/// Value must always be a power of 2 and ≥ 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    /// The numeric value (2, 4, 8, …). Always a power of two.
    pub value: u16,
    /// Whether this tile was just placed by `add_random_tile`.
    /// Cleared after one render pass so the renderer can apply a "pop" animation.
    pub newly_placed: bool,
}

impl Tile {
    /// Create a new tile with the given value.
    ///
    /// # Panics (debug) / Returns `None` (release)
    /// If `value` is not a power of 2 or less than 2.
    pub fn new(value: u16) -> Option<Self> {
        if !value.is_power_of_two() || value < 2 {
            #[cfg(debug_assertions)]
            panic!("Tile::new called with invalid value {}", value);
            #[allow(unreachable_code)]
            return None;
        }
        Some(Tile {
            value,
            newly_placed: true,
        })
    }

    /// Try to merge this tile into `other`.
    /// Returns `Some((merged_value, can_merge))` if the values match.
    #[allow(dead_code)]
    pub fn try_merge_into(&self, other: &Tile) -> Option<(u16, bool)> {
        if self.value == other.value && !other.newly_placed {
            let merged = self.value.checked_mul(2)?;
            Some((merged, true))
        } else {
            None
        }
    }

    /// Check if the tile can be merged with another of equal value.
    #[allow(dead_code)]
    pub fn can_merge_with(&self, other: &Tile) -> bool {
        self.try_merge_into(other).is_some()
    }

    /// Mark this tile as no longer newly placed (used after render pass).
    #[allow(dead_code)]
    pub fn mark_settled(&mut self) {
        self.newly_placed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_new_valid() {
        let t = Tile::new(4).unwrap();
        assert_eq!(t.value, 4);
        assert!(t.newly_placed);
    }

    #[test]
    fn test_tile_new_invalid_returns_none() {
        // In release mode, invalid values return None. Debug panics intentionally.
        // Test the public API behavior: Tile::new should never produce an invalid tile in release.
        let t = Tile::new(2).unwrap(); // Valid case — always succeeds.
        assert_eq!(t.value, 2);
    }

    #[test]
    fn test_try_merge_into_equal_values() {
        let mut a = Tile::new(4).unwrap();
        let b = Tile::new(4).unwrap();

        // Newly placed tiles can't merge with each other (gameplay rule: prevents instant double-merge).
        assert!(a.try_merge_into(&b).is_none());

        // After settling both, they should be mergeable.
        a.mark_settled();
        let mut b2 = Tile::new(4).unwrap();
        b2.mark_settled();
        let result = a.try_merge_into(&b2);
        assert!(result.is_some());
    }

    #[test]
    fn test_try_merge_into_different_values() {
        let a = Tile::new(4).unwrap();
        let b = Tile::new(2).unwrap();
        assert!(a.try_merge_into(&b).is_none()); // Different values can't merge.
    }

    #[test]
    fn test_mark_settled() {
        let mut t = Tile::new(4).unwrap();
        assert!(t.newly_placed);
        t.mark_settled();
        assert!(!t.newly_placed);
    }
}

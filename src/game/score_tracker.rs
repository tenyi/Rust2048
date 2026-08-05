/// Tracks current game score and best (high) score.
#[derive(Debug, Clone)]
pub struct ScoreTracker {
    /// Current session score — increases when tiles merge.
    pub current_score: u32,
    /// Best score ever achieved, loaded from disk on startup.
    pub best_score: Option<u32>,
}

impl ScoreTracker {
    /// Create a new tracker with zero scores and no previously saved best.
    pub fn new() -> Self {
        ScoreTracker {
            current_score: 0,
            best_score: None,
        }
    }

    /// Add `amount` to the current score (from merges).
    #[allow(dead_code)]
    pub fn add_to_current(&mut self, amount: u32) {
        self.current_score = self.current_score.saturating_add(amount);
    }

    /// Compare current score against best. If it exceeds the best, update and return `Some(new_best)`.
    /// Otherwise returns `None` (no change).
    pub fn update_best_if_higher(&mut self) -> Option<u32> {
        let new_best = match self.best_score {
            None => Some(self.current_score), // First game — current score becomes best.
            Some(best) if self.current_score > best => Some(self.current_score),
            _ => None,
        };

        if let Some(val) = new_best {
            self.best_score = Some(val);
        }
        new_best
    }

    /// Get the current score (always returns a value).
    pub fn get_current(&self) -> u32 {
        self.current_score
    }

    /// Get the best score, defaulting to 0 if no record exists yet.
    pub fn get_best(&self) -> u32 {
        self.best_score.unwrap_or(0)
    }

    /// Reset scores for a new game (keep best score).
    pub fn reset_current(&mut self) {
        self.current_score = 0;
    }
}

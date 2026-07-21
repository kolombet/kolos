use chrono::{DateTime, Utc};
use rs_fsrs::{Card, Rating, ReviewLog, FSRS};

pub struct Engine {
    fsrs: FSRS,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            fsrs: FSRS::default(),
        }
    }

    /// Applies a single rating and returns the updated card + the log entry to persist.
    pub fn schedule(&self, card: Card, rating: Rating, now: DateTime<Utc>) -> (Card, ReviewLog) {
        let info = self.fsrs.next(card, now, rating);
        (info.card, info.review_log)
    }

    /// Days until the next review under each of the four ratings, so the review UI can
    /// label its buttons (e.g. "Good — 3d") before the user picks one.
    pub fn preview_days(&self, card: Card, now: DateTime<Utc>) -> [i64; 4] {
        let record_log = self.fsrs.repeat(card, now);
        [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy]
            .map(|r| record_log[&r].card.scheduled_days)
    }
}

pub fn rating_from_u8(v: u8) -> Result<Rating, String> {
    match v {
        1 => Ok(Rating::Again),
        2 => Ok(Rating::Hard),
        3 => Ok(Rating::Good),
        4 => Ok(Rating::Easy),
        other => Err(format!("invalid rating: {other}")),
    }
}

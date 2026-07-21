use chrono::{DateTime, Utc};
use rs_fsrs::State as FsrsState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardState {
    New,
    Learning,
    Review,
    Relearning,
}

impl CardState {
    pub fn as_i64(self) -> i64 {
        match self {
            CardState::New => 0,
            CardState::Learning => 1,
            CardState::Review => 2,
            CardState::Relearning => 3,
        }
    }

    pub fn from_i64(v: i64) -> Self {
        match v {
            1 => CardState::Learning,
            2 => CardState::Review,
            3 => CardState::Relearning,
            _ => CardState::New,
        }
    }
}

impl From<FsrsState> for CardState {
    fn from(s: FsrsState) -> Self {
        match s {
            FsrsState::New => CardState::New,
            FsrsState::Learning => CardState::Learning,
            FsrsState::Review => CardState::Review,
            FsrsState::Relearning => CardState::Relearning,
        }
    }
}

impl From<CardState> for FsrsState {
    fn from(s: CardState) -> Self {
        match s {
            CardState::New => FsrsState::New,
            CardState::Learning => FsrsState::Learning,
            CardState::Review => FsrsState::Review,
            CardState::Relearning => FsrsState::Relearning,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Word {
    pub id: i64,
    pub danish: String,
    pub english: String,
    pub example_da: Option<String>,
    pub example_en: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A word plus just enough of its card state to badge it in the library list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordWithCard {
    #[serde(flatten)]
    pub word: Word,
    pub due: DateTime<Utc>,
    pub state: CardState,
    pub stability: f64,
}

/// A word plus its full FSRS card state, used by the review queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCard {
    #[serde(flatten)]
    pub word: Word,
    pub due: DateTime<Utc>,
    pub stability: f64,
    pub difficulty: f64,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
    pub reps: i32,
    pub lapses: i32,
    pub state: CardState,
    pub last_review: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntervalPreview {
    pub again_days: i64,
    pub hard_days: i64,
    pub good_days: i64,
    pub easy_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub due_today: i64,
    pub new_today: i64,
    pub reviewed_today: i64,
    pub total_words: i64,
}

/// Result of an AI-fill request: the model echoes back the complete word so the
/// frontend can just overwrite all four fields with a consistent set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiFillResult {
    pub danish: String,
    pub english: String,
    pub example_da: String,
    pub example_en: String,
}

use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rs_fsrs::Card as FsrsCard;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::types::{CardState, ReviewCard, Stats, Word, WordWithCard};

const SCHEMA_V1: &str = r#"
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS words (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  danish        TEXT NOT NULL,
  english       TEXT NOT NULL,
  example_da    TEXT,
  example_en    TEXT,
  notes         TEXT,
  created_at    TEXT NOT NULL
);

-- 1:1 with words; created alongside each word via FsrsCard::new().
CREATE TABLE IF NOT EXISTS cards (
  word_id         INTEGER PRIMARY KEY REFERENCES words(id) ON DELETE CASCADE,
  due             TEXT NOT NULL,
  stability       REAL NOT NULL DEFAULT 0,
  difficulty      REAL NOT NULL DEFAULT 0,
  elapsed_days    INTEGER NOT NULL DEFAULT 0,
  scheduled_days  INTEGER NOT NULL DEFAULT 0,
  reps            INTEGER NOT NULL DEFAULT 0,
  lapses          INTEGER NOT NULL DEFAULT 0,
  state           INTEGER NOT NULL DEFAULT 0,
  last_review     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_cards_due ON cards(due);

CREATE TABLE IF NOT EXISTS review_log (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  word_id         INTEGER NOT NULL REFERENCES words(id) ON DELETE CASCADE,
  rating          INTEGER NOT NULL,
  elapsed_days    INTEGER NOT NULL,
  scheduled_days  INTEGER NOT NULL,
  state           INTEGER NOT NULL,
  reviewed_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_review_log_word ON review_log(word_id);
"#;

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create db dir {parent:?}"))?;
        }
        let conn = Connection::open(path).with_context(|| format!("opening {path:?}"))?;
        conn.execute_batch(SCHEMA_V1).context("running schema")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    #[cfg(test)]
    fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("open in-memory db")?;
        conn.execute_batch(SCHEMA_V1).context("running schema")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn add_word(
        &self,
        danish: &str,
        english: &str,
        example_da: Option<&str>,
        example_en: Option<&str>,
        notes: Option<&str>,
    ) -> Result<Word> {
        let conn = self.conn.lock();
        let now = Utc::now();
        conn.execute(
            "INSERT INTO words (danish, english, example_da, example_en, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![danish, english, example_da, example_en, notes, now.to_rfc3339()],
        )
        .context("insert word")?;
        let id = conn.last_insert_rowid();

        let card = FsrsCard::new();
        conn.execute(
            "INSERT INTO cards
                (word_id, due, stability, difficulty, elapsed_days, scheduled_days, reps, lapses, state, last_review)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                card.due.to_rfc3339(),
                card.stability,
                card.difficulty,
                card.elapsed_days,
                card.scheduled_days,
                card.reps,
                card.lapses,
                CardState::from(card.state).as_i64(),
                card.last_review.to_rfc3339(),
            ],
        )
        .context("insert card")?;

        Ok(Word {
            id,
            danish: danish.to_string(),
            english: english.to_string(),
            example_da: example_da.map(str::to_string),
            example_en: example_en.map(str::to_string),
            notes: notes.map(str::to_string),
            created_at: now,
        })
    }

    pub fn update_word(
        &self,
        id: i64,
        danish: &str,
        english: &str,
        example_da: Option<&str>,
        example_en: Option<&str>,
        notes: Option<&str>,
    ) -> Result<Word> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE words SET danish = ?1, english = ?2, example_da = ?3, example_en = ?4, notes = ?5
             WHERE id = ?6",
            params![danish, english, example_da, example_en, notes, id],
        )
        .context("update word")?;
        conn.query_row(
            "SELECT id, danish, english, example_da, example_en, notes, created_at
             FROM words WHERE id = ?1",
            params![id],
            word_from_row,
        )
        .context("reload updated word")
    }

    pub fn delete_word(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM words WHERE id = ?1", params![id])
            .context("delete word")?;
        Ok(())
    }

    pub fn get_word(&self, id: i64) -> Result<Option<Word>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, danish, english, example_da, example_en, notes, created_at
             FROM words WHERE id = ?1",
            params![id],
            word_from_row,
        )
        .optional()
        .context("get word")
    }

    pub fn list_words(&self) -> Result<Vec<WordWithCard>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT w.id, w.danish, w.english, w.example_da, w.example_en, w.notes, w.created_at,
                    c.due, c.state, c.stability
             FROM words w JOIN cards c ON c.word_id = w.id
             ORDER BY w.created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(WordWithCard {
                word: word_from_row(row)?,
                due: parse_dt(row.get(7)?),
                state: CardState::from_i64(row.get(8)?),
                stability: row.get(9)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("list words")
    }

    pub fn get_card(&self, word_id: i64) -> Result<Option<FsrsCard>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT due, stability, difficulty, elapsed_days, scheduled_days, reps, lapses, state, last_review
             FROM cards WHERE word_id = ?1",
            params![word_id],
            card_from_row,
        )
        .optional()
        .context("get card")
    }

    pub fn save_card(&self, word_id: i64, card: &FsrsCard) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE cards SET
                due = ?1, stability = ?2, difficulty = ?3, elapsed_days = ?4, scheduled_days = ?5,
                reps = ?6, lapses = ?7, state = ?8, last_review = ?9
             WHERE word_id = ?10",
            params![
                card.due.to_rfc3339(),
                card.stability,
                card.difficulty,
                card.elapsed_days,
                card.scheduled_days,
                card.reps,
                card.lapses,
                CardState::from(card.state).as_i64(),
                card.last_review.to_rfc3339(),
                word_id,
            ],
        )
        .context("save card")?;
        Ok(())
    }

    pub fn insert_review_log(&self, word_id: i64, rating: u8, log: &rs_fsrs::ReviewLog) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO review_log (word_id, rating, elapsed_days, scheduled_days, state, reviewed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                word_id,
                rating as i64,
                log.elapsed_days,
                log.scheduled_days,
                CardState::from(log.state).as_i64(),
                log.reviewed_date.to_rfc3339(),
            ],
        )
        .context("insert review log")?;
        Ok(())
    }

    /// Cards due for review (state != New, due <= now) ordered soonest-first, followed by up
    /// to `new_limit` never-reviewed cards. No persisted "introduced today" counter yet — see
    /// CLAUDE.md-equivalent note in the project plan.
    pub fn get_due_queue(&self, now: DateTime<Utc>, new_limit: i64) -> Result<Vec<ReviewCard>> {
        let conn = self.conn.lock();
        let due_now = now.to_rfc3339();

        let mut stmt = conn.prepare(
            "SELECT w.id, w.danish, w.english, w.example_da, w.example_en, w.notes, w.created_at,
                    c.due, c.stability, c.difficulty, c.elapsed_days, c.scheduled_days, c.reps, c.lapses,
                    c.state, c.last_review
             FROM words w JOIN cards c ON c.word_id = w.id
             WHERE c.state != 0 AND c.due <= ?1
             ORDER BY c.due ASC",
        )?;
        let mut queue: Vec<ReviewCard> = stmt
            .query_map(params![due_now], review_card_from_row)?
            .collect::<rusqlite::Result<_>>()
            .context("due cards")?;

        let mut stmt_new = conn.prepare(
            "SELECT w.id, w.danish, w.english, w.example_da, w.example_en, w.notes, w.created_at,
                    c.due, c.stability, c.difficulty, c.elapsed_days, c.scheduled_days, c.reps, c.lapses,
                    c.state, c.last_review
             FROM words w JOIN cards c ON c.word_id = w.id
             WHERE c.state = 0
             ORDER BY w.created_at ASC
             LIMIT ?1",
        )?;
        let new_cards: Vec<ReviewCard> = stmt_new
            .query_map(params![new_limit], review_card_from_row)?
            .collect::<rusqlite::Result<_>>()
            .context("new cards")?;

        queue.extend(new_cards);
        Ok(queue)
    }

    pub fn get_stats(&self) -> Result<Stats> {
        let conn = self.conn.lock();
        let total_words: i64 = conn.query_row("SELECT COUNT(*) FROM words", [], |r| r.get(0))?;
        let now = Utc::now();
        let due_today: i64 = conn.query_row(
            "SELECT COUNT(*) FROM cards WHERE state != 0 AND due <= ?1",
            params![now.to_rfc3339()],
            |r| r.get(0),
        )?;
        let new_today: i64 = conn.query_row("SELECT COUNT(*) FROM cards WHERE state = 0", [], |r| r.get(0))?;
        let day_start = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .expect("midnight is a valid time")
            .and_utc();
        let reviewed_today: i64 = conn.query_row(
            "SELECT COUNT(*) FROM review_log WHERE reviewed_at >= ?1",
            params![day_start.to_rfc3339()],
            |r| r.get(0),
        )?;
        Ok(Stats {
            due_today,
            new_today,
            reviewed_today,
            total_words,
        })
    }
}

fn parse_dt(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .expect("timestamps are always written as rfc3339")
        .with_timezone(&Utc)
}

fn word_from_row(row: &Row) -> rusqlite::Result<Word> {
    Ok(Word {
        id: row.get(0)?,
        danish: row.get(1)?,
        english: row.get(2)?,
        example_da: row.get(3)?,
        example_en: row.get(4)?,
        notes: row.get(5)?,
        created_at: parse_dt(row.get(6)?),
    })
}

fn card_from_row(row: &Row) -> rusqlite::Result<FsrsCard> {
    Ok(FsrsCard {
        due: parse_dt(row.get(0)?),
        stability: row.get(1)?,
        difficulty: row.get(2)?,
        elapsed_days: row.get(3)?,
        scheduled_days: row.get(4)?,
        reps: row.get(5)?,
        lapses: row.get(6)?,
        state: CardState::from_i64(row.get(7)?).into(),
        last_review: parse_dt(row.get(8)?),
    })
}

fn review_card_from_row(row: &Row) -> rusqlite::Result<ReviewCard> {
    Ok(ReviewCard {
        word: word_from_row(row)?,
        due: parse_dt(row.get(7)?),
        stability: row.get(8)?,
        difficulty: row.get(9)?,
        elapsed_days: row.get(10)?,
        scheduled_days: row.get(11)?,
        reps: row.get(12)?,
        lapses: row.get(13)?,
        state: CardState::from_i64(row.get(14)?),
        last_review: parse_dt(row.get(15)?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsrs_engine::Engine;
    use rs_fsrs::{Rating, State as FsrsState};

    #[test]
    fn word_lifecycle_and_scheduling() {
        let db = Db::open_in_memory().unwrap();
        let word = db
            .add_word("hus", "house", Some("Jeg bor i et hus."), Some("I live in a house."), None)
            .unwrap();

        let words = db.list_words().unwrap();
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].state, CardState::New);

        let now = Utc::now();
        let queue = db.get_due_queue(now, 20).unwrap();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].word.id, word.id);

        let engine = Engine::new();
        let card = db.get_card(word.id).unwrap().unwrap();

        // Rating::Easy on a fresh card must not schedule sooner than Rating::Again.
        let (again_card, _) = engine.schedule(card.clone(), Rating::Again, now);
        let (easy_card, _) = engine.schedule(card, Rating::Easy, now);
        assert!(easy_card.scheduled_days >= again_card.scheduled_days);

        // Persist a Good outcome and confirm it round-trips through the DB.
        let card = db.get_card(word.id).unwrap().unwrap();
        let (new_card, log) = engine.schedule(card, Rating::Good, now);
        db.save_card(word.id, &new_card).unwrap();
        db.insert_review_log(word.id, 3, &log).unwrap();

        let saved = db.get_card(word.id).unwrap().unwrap();
        assert_eq!(saved.reps, 1);
        assert_ne!(saved.state, FsrsState::New);

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_words, 1);
        assert_eq!(stats.reviewed_today, 1);
    }
}

use chrono::Utc;
use tauri::State;

use crate::fsrs_engine::rating_from_u8;
use crate::types::{IntervalPreview, ReviewCard, Stats, Word, WordWithCard};
use crate::AppState;

#[tauri::command]
pub fn add_word(
    state: State<'_, AppState>,
    danish: String,
    english: String,
    example_da: Option<String>,
    example_en: Option<String>,
    notes: Option<String>,
) -> Result<Word, String> {
    state
        .db
        .add_word(&danish, &english, example_da.as_deref(), example_en.as_deref(), notes.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_word(
    state: State<'_, AppState>,
    id: i64,
    danish: String,
    english: String,
    example_da: Option<String>,
    example_en: Option<String>,
    notes: Option<String>,
) -> Result<Word, String> {
    state
        .db
        .update_word(id, &danish, &english, example_da.as_deref(), example_en.as_deref(), notes.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_word(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_word(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_words(state: State<'_, AppState>) -> Result<Vec<WordWithCard>, String> {
    state.db.list_words().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_due_queue(state: State<'_, AppState>, new_limit: i64) -> Result<Vec<ReviewCard>, String> {
    state.db.get_due_queue(Utc::now(), new_limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn preview_review(state: State<'_, AppState>, word_id: i64) -> Result<IntervalPreview, String> {
    let card = state
        .db
        .get_card(word_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no card for word {word_id}"))?;
    let [again_days, hard_days, good_days, easy_days] = state.fsrs.preview_days(card, Utc::now());
    Ok(IntervalPreview {
        again_days,
        hard_days,
        good_days,
        easy_days,
    })
}

#[tauri::command]
pub fn submit_review(state: State<'_, AppState>, word_id: i64, rating: u8) -> Result<ReviewCard, String> {
    let fsrs_rating = rating_from_u8(rating)?;
    let card = state
        .db
        .get_card(word_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no card for word {word_id}"))?;
    let word = state
        .db
        .get_word(word_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no word {word_id}"))?;

    let now = Utc::now();
    let (new_card, log) = state.fsrs.schedule(card, fsrs_rating, now);

    state.db.save_card(word_id, &new_card).map_err(|e| e.to_string())?;
    state
        .db
        .insert_review_log(word_id, rating, &log)
        .map_err(|e| e.to_string())?;

    Ok(ReviewCard {
        word,
        due: new_card.due,
        stability: new_card.stability,
        difficulty: new_card.difficulty,
        elapsed_days: new_card.elapsed_days,
        scheduled_days: new_card.scheduled_days,
        reps: new_card.reps,
        lapses: new_card.lapses,
        state: new_card.state.into(),
        last_review: new_card.last_review,
    })
}

#[tauri::command]
pub fn get_stats(state: State<'_, AppState>) -> Result<Stats, String> {
    state.db.get_stats().map_err(|e| e.to_string())
}

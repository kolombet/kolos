use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;

use crate::types::AiFillResult;

const CLI_TIMEOUT: Duration = Duration::from_secs(60);

/// Fills in whichever of {danish, english, example sentence in each language} the
/// user left blank, given at least one of `danish` / `english`. Shells out to the
/// `claude` CLI (`claude -p`) instead of calling the Anthropic API directly, so this
/// reuses the user's existing Claude Code login — no API key to manage in-app.
pub async fn fill_word(danish: Option<&str>, english: Option<&str>) -> Result<AiFillResult> {
    if danish.is_none() && english.is_none() {
        bail!("provide at least one of danish or english");
    }

    let known = match (danish, english) {
        (Some(d), Some(e)) => format!("Danish word: \"{d}\"\nEnglish translation: \"{e}\""),
        (Some(d), None) => format!("Danish word: \"{d}\"\nEnglish translation: (fill this in)"),
        (None, Some(e)) => format!("English word: \"{e}\"\nDanish translation: (fill this in)"),
        (None, None) => unreachable!(),
    };

    let prompt = format!(
        "You are helping build a Danish vocabulary flashcard. Given the following, \
         return the complete word pair plus one natural, simple example sentence \
         demonstrating the word in context.\n\n{known}\n\n\
         Rules:\n\
         - Danish nouns must include their grammatical gender article (en/et) if applicable, \
           written as part of the danish field (e.g. \"et hus\", \"en bil\").\n\
         - The example sentence must actually use the word.\n\
         - Keep the example sentence short (under 12 words) and at a beginner level.\n\
         - exampleEn is a natural translation of exampleDa, not a literal word-for-word gloss.\n\n\
         Respond with ONLY a single JSON object, no markdown code fences, no explanation, in \
         exactly this shape: {{\"danish\": \"...\", \"english\": \"...\", \"exampleDa\": \"...\", \"exampleEn\": \"...\"}}"
    );

    let run = Command::new("claude")
        .arg("-p")
        .arg(&prompt)
        .args([
            "--output-format",
            "json",
            "--model",
            "sonnet",
            "--no-session-persistence",
            "--strict-mcp-config",
            "--disable-slash-commands",
        ])
        .kill_on_drop(true)
        .output();

    let output = timeout(CLI_TIMEOUT, run)
        .await
        .context("claude CLI timed out")?
        .context("failed to run the claude CLI (is it installed and on PATH?)")?;

    if !output.status.success() {
        bail!("claude CLI exited with an error: {}", String::from_utf8_lossy(&output.stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: Value = serde_json::from_str(&stdout).context("parsing claude CLI JSON output")?;

    if envelope.get("subtype").and_then(|s| s.as_str()) != Some("success") {
        let message = envelope
            .get("result")
            .and_then(|r| r.as_str())
            .unwrap_or("unknown error");
        bail!("claude CLI reported an error: {message}");
    }

    let result_text = envelope
        .get("result")
        .and_then(|r| r.as_str())
        .context("no result field in claude CLI output")?;

    let json_text = strip_code_fence(result_text);
    serde_json::from_str(json_text).with_context(|| format!("parsing structured AI-fill output: {json_text}"))
}

/// The prompt asks for bare JSON, but strip a ```json fence defensively in case the
/// model wraps its answer in one anyway.
fn strip_code_fence(text: &str) -> &str {
    let trimmed = text.trim();
    for prefix in ["```json", "```"] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return rest.trim().strip_suffix("```").unwrap_or(rest).trim();
        }
    }
    trimmed
}

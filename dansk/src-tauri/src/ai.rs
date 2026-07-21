use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;

use crate::types::AiFillResult;

const CLI_TIMEOUT: Duration = Duration::from_secs(60);

/// Fills in whichever of {danish, english, example sentence in each language} the
/// user left blank, using whatever fields they already filled in as context. Fields
/// the user already provided are never sent to be overwritten by the model — the
/// prompt instructs the model to echo them back unchanged, and the caller (see
/// `commands::ai_fill_word` / the frontend) additionally never applies the response
/// over a field that was already non-empty. Shells out to the `claude` CLI
/// (`claude -p`) instead of calling the Anthropic API directly, so this reuses the
/// user's existing Claude Code login — no API key to manage in-app.
pub async fn fill_word(
    danish: Option<&str>,
    english: Option<&str>,
    example_da: Option<&str>,
    example_en: Option<&str>,
) -> Result<AiFillResult> {
    if danish.is_none() && english.is_none() && example_da.is_none() && example_en.is_none() {
        bail!("provide at least one field to generate from");
    }

    let field = |label: &str, value: Option<&str>| match value {
        Some(v) => format!("{label}: \"{v}\""),
        None => format!("{label}: (blank — fill this in)"),
    };

    let known = [
        field("Danish word", danish),
        field("English translation", english),
        field("Example sentence (Danish)", example_da),
        field("Example sentence (English translation)", example_en),
    ]
    .join("\n");

    let prompt = format!(
        "You are helping complete a Danish vocabulary flashcard. Some fields below are \
         already filled in by the user; others are blank. Fill in ONLY the blank fields.\n\n\
         {known}\n\n\
         Rules:\n\
         - Any field already given above must be returned completely unchanged, character \
           for character — never correct, rephrase, or add to it.\n\
         - When you fill in the Danish word yourself, include its grammatical gender article \
           (en/et) if applicable, as part of the danish field (e.g. \"et hus\", \"en bil\"). \
           Never add an article to a Danish word field the user already gave.\n\
         - If the example sentence (Danish) is blank, write one short (under 12 words), \
           natural, beginner-level sentence that actually uses the Danish word.\n\
         - If the example sentence (Danish) is given but its translation is blank, translate \
           that exact given sentence naturally — not word-for-word.\n\
         - If the Danish word is blank but an example sentence is given, infer the word from \
           that sentence.\n\n\
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

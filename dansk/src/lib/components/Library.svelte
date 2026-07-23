<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { wordsState } from "../state/words.svelte";
  import type { AiFillResult, WordInput, WordWithCard } from "../types";

  onMount(() => {
    wordsState.refresh();
  });

  let editingId = $state<number | null>(null);
  let danish = $state("");
  let english = $state("");
  let exampleDa = $state("");
  let exampleEn = $state("");
  let error = $state("");
  let aiLoading = $state(false);
  let aiError = $state("");

  // Delete is a two-click "arm" pattern instead of a confirm() dialog — Tauri's
  // webview doesn't reliably show window.confirm(), so the click-to-arm button below
  // ("Delete" -> red "SLET" -> click again) is the confirmation UI.
  let confirmDeleteId = $state<number | null>(null);
  let confirmDeleteTimeout: ReturnType<typeof setTimeout> | null = null;

  function disarmDelete() {
    if (confirmDeleteTimeout) clearTimeout(confirmDeleteTimeout);
    confirmDeleteTimeout = null;
    confirmDeleteId = null;
  }

  function resetForm() {
    editingId = null;
    danish = "";
    english = "";
    exampleDa = "";
    exampleEn = "";
    error = "";
    aiError = "";
    disarmDelete();
  }

  function startEdit(word: WordWithCard) {
    editingId = word.id;
    danish = word.danish;
    english = word.english;
    exampleDa = word.exampleDa ?? "";
    exampleEn = word.exampleEn ?? "";
    error = "";
    aiError = "";
    disarmDelete();
  }

  async function submit(event: Event) {
    event.preventDefault();
    if (!danish.trim() || !english.trim()) {
      error = "Danish and English are required.";
      return;
    }
    const input: WordInput = {
      danish: danish.trim(),
      english: english.trim(),
      exampleDa: exampleDa.trim() || null,
      exampleEn: exampleEn.trim() || null,
    };
    if (editingId !== null) {
      await wordsState.update(editingId, input);
    } else {
      await wordsState.add(input);
    }
    resetForm();
  }

  async function handleDeleteClick(id: number) {
    if (confirmDeleteId !== id) {
      confirmDeleteId = id;
      if (confirmDeleteTimeout) clearTimeout(confirmDeleteTimeout);
      confirmDeleteTimeout = setTimeout(disarmDelete, 3000);
      return;
    }
    disarmDelete();
    await wordsState.remove(id);
    if (editingId === id) resetForm();
  }

  async function aiFill() {
    if (!danish.trim() && !english.trim() && !exampleDa.trim() && !exampleEn.trim()) return;
    aiLoading = true;
    aiError = "";
    // Only fields that were empty before the call get overwritten — whatever the user
    // already typed is used as context but never replaced, even if the model echoes
    // something different back.
    const hadDanish = !!danish.trim();
    const hadEnglish = !!english.trim();
    const hadExampleDa = !!exampleDa.trim();
    const hadExampleEn = !!exampleEn.trim();
    try {
      const result = await invoke<AiFillResult>("ai_fill_word", {
        danish: danish.trim() || null,
        english: english.trim() || null,
        exampleDa: exampleDa.trim() || null,
        exampleEn: exampleEn.trim() || null,
      });
      if (!hadDanish) danish = result.danish;
      if (!hadEnglish) english = result.english;
      if (!hadExampleDa) exampleDa = result.exampleDa;
      if (!hadExampleEn) exampleEn = result.exampleEn;
    } catch (e) {
      aiError = String(e);
    } finally {
      aiLoading = false;
    }
  }

  function stateBadgeClass(state: WordWithCard["state"]) {
    switch (state) {
      case "new":
        return "bg-[var(--badge-new-bg)] text-[var(--badge-new-text)]";
      case "learning":
      case "relearning":
        return "bg-[var(--badge-learning-bg)] text-[var(--badge-learning-text)]";
      default:
        return "bg-[var(--badge-review-bg)] text-[var(--badge-review-text)]";
    }
  }

  function dueLabel(word: WordWithCard): string {
    if (word.state === "new") return "new";
    const due = new Date(word.due).getTime();
    const diffDays = Math.ceil((due - Date.now()) / 86_400_000);
    if (diffDays <= 0) return "due now";
    if (diffDays === 1) return "due in 1 day";
    return `due in ${diffDays} days`;
  }

  let search = $state("");

  let filteredWords = $derived.by(() => {
    const query = search.trim().toLowerCase();
    if (!query) return wordsState.words;
    return wordsState.words.filter(
      (word) =>
        word.danish.toLowerCase().includes(query) ||
        word.english.toLowerCase().includes(query) ||
        (word.exampleDa?.toLowerCase().includes(query) ?? false) ||
        (word.exampleEn?.toLowerCase().includes(query) ?? false),
    );
  });
</script>

<div class="mx-auto flex max-w-2xl flex-col gap-6 p-6">
  <form onsubmit={submit} class="flex flex-col gap-3 rounded-lg border border-[var(--border)] bg-[var(--bg-secondary)] p-4">
    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1 text-sm text-[var(--text-secondary)]">
        Danish
        <input
          class="rounded-md border border-[var(--border)] bg-[var(--bg-input)] px-2 py-1.5 text-[var(--text-primary)]"
          bind:value={danish}
          placeholder="hus"
        />
      </label>
      <label class="flex flex-col gap-1 text-sm text-[var(--text-secondary)]">
        English
        <input
          class="rounded-md border border-[var(--border)] bg-[var(--bg-input)] px-2 py-1.5 text-[var(--text-primary)]"
          bind:value={english}
          placeholder="house"
        />
      </label>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1 text-sm text-[var(--text-secondary)]">
        Example sentence (Danish)
        <input
          class="rounded-md border border-[var(--border)] bg-[var(--bg-input)] px-2 py-1.5 text-[var(--text-primary)]"
          bind:value={exampleDa}
          placeholder="Jeg bor i et hus."
        />
      </label>
      <label class="flex flex-col gap-1 text-sm text-[var(--text-secondary)]">
        Example translation
        <input
          class="rounded-md border border-[var(--border)] bg-[var(--bg-input)] px-2 py-1.5 text-[var(--text-primary)]"
          bind:value={exampleEn}
          placeholder="I live in a house."
        />
      </label>
    </div>

    <div class="flex items-center gap-2">
      <button
        type="button"
        onclick={aiFill}
        disabled={aiLoading || (!danish.trim() && !english.trim() && !exampleDa.trim() && !exampleEn.trim())}
        class="rounded-md border border-[var(--border)] px-3 py-1.5 text-sm text-[var(--text-primary)] hover:bg-[var(--bg-hover)] disabled:cursor-not-allowed disabled:opacity-50"
      >
        {aiLoading ? "Filling…" : "AI fill"}
      </button>
      <span class="text-xs text-[var(--text-muted)]">
        Fill in any field above, then let AI fill the rest — it won't touch what you've already typed.
      </span>
    </div>
    {#if aiError}
      <p class="text-sm text-[var(--danger)]">{aiError}</p>
    {/if}
    {#if error}
      <p class="text-sm text-[var(--danger)]">{error}</p>
    {/if}
    <div class="flex gap-2">
      <button
        type="submit"
        class="rounded-md bg-[var(--accent)] px-3 py-1.5 text-sm font-medium text-[var(--accent-text)]"
      >
        {editingId !== null ? "Save changes" : "Add word"}
      </button>
      {#if editingId !== null}
        <button
          type="button"
          onclick={resetForm}
          class="rounded-md border border-[var(--border)] px-3 py-1.5 text-sm text-[var(--text-secondary)]"
        >
          Cancel
        </button>
      {/if}
    </div>
  </form>

  <div class="flex flex-col gap-3">
    {#if wordsState.words.length > 0}
      <input
        type="search"
        bind:value={search}
        placeholder="Search words…"
        aria-label="Search words"
        class="rounded-md border border-[var(--border)] bg-[var(--bg-input)] px-2 py-1.5 text-sm text-[var(--text-primary)]"
      />
    {/if}

    {#if wordsState.loading && wordsState.words.length === 0}
      <p class="text-sm text-[var(--text-muted)]">Loading…</p>
    {:else if wordsState.words.length === 0}
      <p class="text-sm text-[var(--text-muted)]">No words yet — add your first one above.</p>
    {:else if filteredWords.length === 0}
      <p class="text-sm text-[var(--text-muted)]">No words match "{search}".</p>
    {:else}
      {#each filteredWords as word (word.id)}
        <div class="flex items-center gap-3 rounded-md border border-[var(--border-subtle)] px-3 py-2">
          <div class="flex-1">
            <div class="font-medium text-[var(--text-primary)]">{word.danish}</div>
            <div class="text-sm text-[var(--text-secondary)]">{word.english}</div>
          </div>
          <span class="rounded-full px-2 py-0.5 text-xs {stateBadgeClass(word.state)}">
            {dueLabel(word)}
          </span>
          <button
            onclick={() => startEdit(word)}
            class="rounded-md px-2 py-1 text-sm text-[var(--text-secondary)] hover:bg-[var(--bg-hover)]"
          >
            Edit
          </button>
          <button
            onclick={() => handleDeleteClick(word.id)}
            class="rounded-md px-2 py-1 text-sm hover:bg-[var(--bg-hover)] {confirmDeleteId === word.id
              ? 'font-bold uppercase text-[var(--danger)]'
              : 'text-[var(--text-secondary)]'}"
          >
            {confirmDeleteId === word.id ? "Slet" : "Delete"}
          </button>
        </div>
      {/each}
    {/if}
  </div>
</div>

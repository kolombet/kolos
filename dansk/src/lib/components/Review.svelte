<script lang="ts">
  import { onMount } from "svelte";
  import { reviewState } from "../state/review.svelte";
  import { RATINGS, type Rating } from "../types";

  onMount(() => {
    reviewState.loadQueue();
  });

  function intervalLabel(days: number): string {
    if (days < 1) return "<10m";
    if (days === 1) return "1d";
    if (days < 30) return `${days}d`;
    if (days < 365) return `${Math.round(days / 30)}mo`;
    return `${(days / 365).toFixed(1)}y`;
  }

  function previewDays(rating: Rating): number {
    const p = reviewState.preview;
    if (!p) return 0;
    return { 1: p.againDays, 2: p.hardDays, 3: p.goodDays, 4: p.easyDays }[rating];
  }

  function onKeydown(event: KeyboardEvent) {
    if (!reviewState.current) return;
    if (!reviewState.flipped) {
      if (event.code === "Space" || event.key === "Enter") {
        event.preventDefault();
        reviewState.flip();
      }
      return;
    }
    const key = { "1": 1, "2": 2, "3": 3, "4": 4 } as const;
    if (event.key in key) {
      event.preventDefault();
      reviewState.rate(key[event.key as keyof typeof key] as Rating);
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="mx-auto flex max-w-xl flex-col items-center gap-8 p-6">
  {#if reviewState.loading}
    <p class="text-sm text-[var(--text-muted)]">Loading…</p>
  {:else if reviewState.done}
    <div class="flex flex-col items-center gap-2 py-24 text-center">
      <p class="text-lg font-medium text-[var(--text-primary)]">All done for today.</p>
      <p class="text-sm text-[var(--text-muted)]">Come back later, or add more words in the Library.</p>
    </div>
  {:else if reviewState.current}
    <p class="text-sm text-[var(--text-muted)]">{reviewState.remaining} left</p>

    <button
      type="button"
      onclick={() => (reviewState.flipped ? undefined : reviewState.flip())}
      class="flex min-h-56 w-full flex-col items-center justify-center gap-4 rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-8 text-center"
    >
      <span class="text-3xl font-semibold text-[var(--text-primary)]">{reviewState.current.danish}</span>
      {#if reviewState.flipped}
        <div class="flex flex-col gap-2">
          <span class="text-xl text-[var(--text-secondary)]">{reviewState.current.english}</span>
          {#if reviewState.current.exampleDa}
            <p class="mt-2 text-sm italic text-[var(--text-secondary)]">{reviewState.current.exampleDa}</p>
          {/if}
          {#if reviewState.current.exampleEn}
            <p class="text-sm text-[var(--text-muted)]">{reviewState.current.exampleEn}</p>
          {/if}
        </div>
      {:else}
        <span class="text-xs text-[var(--text-muted)]">click, space, or enter to flip</span>
      {/if}
    </button>

    {#if reviewState.flipped}
      <div class="grid w-full grid-cols-4 gap-2">
        {#each RATINGS as { rating, label }}
          <button
            onclick={() => reviewState.rate(rating)}
            class="flex flex-col items-center gap-1 rounded-md border border-[var(--border)] px-2 py-2 text-sm hover:bg-[var(--bg-hover)]"
          >
            <span class="text-[var(--text-primary)]">{label}</span>
            <span class="text-xs text-[var(--text-muted)]">{intervalLabel(previewDays(rating))}</span>
          </button>
        {/each}
      </div>
    {/if}
  {/if}
</div>

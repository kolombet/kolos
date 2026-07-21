<script lang="ts">
  import { onMount } from "svelte";
  import Library from "$lib/components/Library.svelte";
  import Review from "$lib/components/Review.svelte";
  import SettingsMenu from "$lib/components/SettingsMenu.svelte";
  import { statsState } from "$lib/state/stats.svelte";

  let tab = $state<"review" | "library">("review");

  onMount(() => {
    statsState.refresh();
  });

  $effect(() => {
    tab;
    statsState.refresh();
  });
</script>

<div class="flex h-full flex-col">
  <header class="flex items-center justify-between border-b border-[var(--border)] px-4 py-2">
    <div class="flex gap-1">
      <button
        onclick={() => (tab = "review")}
        class="rounded-md px-3 py-1.5 text-sm {tab === 'review'
          ? 'bg-[var(--bg-hover)] text-[var(--text-primary)]'
          : 'text-[var(--text-secondary)]'}"
      >
        Review
      </button>
      <button
        onclick={() => (tab = "library")}
        class="rounded-md px-3 py-1.5 text-sm {tab === 'library'
          ? 'bg-[var(--bg-hover)] text-[var(--text-primary)]'
          : 'text-[var(--text-secondary)]'}"
      >
        Library
      </button>
    </div>
    <div class="flex items-center gap-3">
      {#if statsState.stats}
        <div class="text-sm text-[var(--text-muted)]">
          {statsState.stats.dueToday} due · {statsState.stats.newToday} new · {statsState.stats.totalWords} words
        </div>
      {/if}
      <SettingsMenu />
    </div>
  </header>

  <main class="flex-1 overflow-y-auto">
    {#if tab === "review"}
      <Review />
    {:else}
      <Library />
    {/if}
  </main>
</div>

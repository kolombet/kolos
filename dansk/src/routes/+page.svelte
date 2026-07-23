<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
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

  async function handleTitleBarMouseDown(event: MouseEvent) {
    const target = event.target as HTMLElement;

    if (
      event.button !== 0 ||
      (target && target.closest("button, a, input, textarea, select, [role='button'], [role='menu'], [role='menuitem']"))
    ) {
      return;
    }

    try {
      const appWindow = getCurrentWindow();
      if (event.detail === 2) {
        await appWindow.toggleMaximize();
      } else {
        await appWindow.startDragging();
      }
    } catch (e) {
      console.error("Window drag failed:", e);
    }
  }

  async function handleMinimize() {
    try {
      await getCurrentWindow().minimize();
    } catch (e) {
      console.error("Minimize failed:", e);
    }
  }

  async function handleMaximize() {
    try {
      await getCurrentWindow().toggleMaximize();
    } catch (e) {
      console.error("Maximize failed:", e);
    }
  }

  async function handleClose() {
    try {
      await getCurrentWindow().close();
    } catch (e) {
      console.error("Close failed:", e);
    }
  }
</script>

<div class="flex h-full flex-col">
  <header
    role="none"
    onmousedown={handleTitleBarMouseDown}
    class="drag-region no-select flex h-[24px] shrink-0 items-center justify-between border-b border-[var(--border)] pl-[6px] pr-[2px] transition-colors"
  >
    <div class="flex h-full items-center gap-1 no-drag">
      <button
        onclick={() => (tab = "review")}
        class="flex h-[20px] items-center rounded-[3px] px-2 text-xs {tab === 'review'
          ? 'bg-[var(--bg-hover)] text-[var(--text-primary)]'
          : 'text-[var(--text-secondary)]'}"
      >
        Review
      </button>
      <button
        onclick={() => (tab = "library")}
        class="flex h-[20px] items-center rounded-[3px] px-2 text-xs {tab === 'library'
          ? 'bg-[var(--bg-hover)] text-[var(--text-primary)]'
          : 'text-[var(--text-secondary)]'}"
      >
        Library
      </button>
    </div>
    <div class="flex h-full items-center gap-2 no-drag">
      {#if statsState.stats}
        <div class="text-[11px] text-[var(--text-muted)]">
          {statsState.stats.dueToday} due · {statsState.stats.newToday} new · {statsState.stats.totalWords} words
        </div>
      {/if}
      <SettingsMenu />
      <div class="w-[1px] h-[14px] bg-[var(--border)] mx-1"></div>
      <button
        onclick={handleMinimize}
        aria-label="Minimize"
        class="btn-press flex items-center justify-center w-[26px] h-[22px] rounded-[3px] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
        title="Minimize"
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M5 12h14" />
        </svg>
      </button>
      <button
        onclick={handleMaximize}
        aria-label="Maximize"
        class="btn-press flex items-center justify-center w-[26px] h-[22px] rounded-[3px] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
        title="Maximize"
      >
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round">
          <rect x="4" y="4" width="16" height="16" rx="1" />
        </svg>
      </button>
      <button
        onclick={handleClose}
        aria-label="Close"
        class="btn-press flex items-center justify-center w-[26px] h-[22px] rounded-[3px] hover:bg-[var(--danger)] text-[var(--text-secondary)] hover:text-white transition-colors"
        title="Close"
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M6 6l12 12M18 6L6 18" />
        </svg>
      </button>
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

<script lang="ts">
  import { themeState, type Theme } from "$lib/state/theme.svelte";

  let isOpen = $state(false);
  let menuRef: HTMLDivElement | null = $state(null);

  const themes: Array<{ id: Theme; name: string; colors: [string, string] }> = [
    { id: "dark", name: "Dark", colors: ["#0a0a0a", "#141414"] },
    { id: "light", name: "Light", colors: ["#ffffff", "#f4f2ee"] },
    { id: "paper", name: "Paper", colors: ["#f5f0e6", "#ebe5d8"] },
    { id: "dracula", name: "Dracula", colors: ["#282a36", "#44475a"] },
  ];

  $effect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef && !menuRef.contains(e.target as Node)) {
        isOpen = false;
      }
    };
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") isOpen = false;
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleKey);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleKey);
    };
  });
</script>

<div bind:this={menuRef} class="relative">
  <button
    onclick={() => (isOpen = !isOpen)}
    aria-label="Settings"
    aria-expanded={isOpen}
    aria-haspopup="true"
    class="btn-press flex items-center justify-center w-[26px] h-[22px] rounded-[3px] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
    title="Settings"
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z" />
    </svg>
  </button>

  {#if isOpen}
    <div role="menu" aria-label="Settings" class="absolute right-0 top-full mt-2 w-64 bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl shadow-2xl overflow-y-auto max-h-[calc(100vh-5rem)] z-[70] animate-fade-in-down">
      <div class="p-4">
        <div class="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider mb-3">
          Theme
        </div>
        <div class="grid grid-cols-2 gap-2">
          {#each themes as t}
            <button
              onclick={() => (themeState.theme = t.id)}
              class={`flex-1 flex flex-col items-center gap-2 p-3 rounded-lg transition-all ${
                themeState.theme === t.id
                  ? "ring-2 ring-[var(--accent)] bg-[var(--bg-hover)]"
                  : "hover:bg-[var(--bg-hover)]"
              }`}
              title={t.name}
            >
              <div class="w-10 h-10 rounded-lg overflow-hidden border border-[var(--border)] flex shadow-sm">
                <div class="w-1/2 h-full" style={`background-color: ${t.colors[0]}`}></div>
                <div class="w-1/2 h-full" style={`background-color: ${t.colors[1]}`}></div>
              </div>
              <span class="text-[11px] font-medium text-[var(--text-primary)]">
                {t.name}
              </span>
            </button>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>

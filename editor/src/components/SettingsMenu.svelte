<script lang="ts">
    import { themeState, type Theme, type FontFamily, type FontSize } from "../state/theme.svelte.ts";

    let isOpen = $state(false);
    let menuRef: HTMLDivElement | null = $state(null);

    const themes: Array<{ id: Theme; name: string; colors: [string, string] }> = [
        { id: "dark", name: "Dark", colors: ["#0a0a0a", "#141414"] },
        { id: "light", name: "Light", colors: ["#ffffff", "#f4f2ee"] },
        { id: "paper", name: "Paper", colors: ["#f5f0e6", "#ebe5d8"] },
        { id: "dracula", name: "Dracula", colors: ["#282a36", "#44475a"] },
    ];

    const fonts: Array<{ id: FontFamily; name: string; family: string }> = [
        { id: "inter", name: "Inter", family: "'Inter', sans-serif" },
        { id: "merriweather", name: "Merriweather", family: "'Merriweather', serif" },
        { id: "lora", name: "Lora", family: "'Lora', serif" },
        { id: "source-serif", name: "Source Serif", family: "'Source Serif 4', serif" },
        { id: "fira-sans", name: "Fira Sans", family: "'Fira Sans', sans-serif" },
    ];

    const fontSizes: Array<{ id: FontSize; name: string; size: string }> = [
        { id: "small", name: "Small", size: "14px" },
        { id: "medium", name: "Medium", size: "16px" },
        { id: "large", name: "Large", size: "18px" },
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

<div bind:this={menuRef} class="relative no-drag">
    <!-- Settings Button -->
    <button
        onclick={() => isOpen = !isOpen}
        aria-label="Settings"
        aria-expanded={isOpen}
        aria-haspopup="true"
        data-tour="settings"
        class="btn-press flex items-center justify-center w-[26px] h-[22px] rounded-[3px] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
        title="Settings"
    >
        <span class="material-symbols-outlined text-[12px]">settings</span>
    </button>

    <!-- Dropdown Menu -->
    {#if isOpen}
        <div role="menu" aria-label="Settings" class="absolute right-0 top-full mt-2 w-80 bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl shadow-2xl overflow-y-auto max-h-[calc(100vh-5rem)] z-[70] animate-fade-in-down">
            <!-- Theme Section -->
            <div class="p-4 border-b border-[var(--border)]">
                <div class="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider mb-3">
                    Theme
                </div>
                <div class="grid grid-cols-4 gap-2">
                    {#each themes as t}
                        <button
                            onclick={() => themeState.theme = t.id}
                            class={`flex-1 flex flex-col items-center gap-2 p-3 rounded-lg transition-all ${themeState.theme === t.id
                                ? "ring-2 ring-[var(--accent)] bg-[var(--bg-hover)]"
                                : "hover:bg-[var(--bg-hover)]"
                            }`}
                            title={t.name}
                        >
                            <div class="w-10 h-10 rounded-lg overflow-hidden border border-[var(--border)] flex items-center justify-center shadow-sm" style={`background-color: ${t.colors[0]}`}>
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

            <!-- Font Section -->
            <div class="p-4 border-b border-[var(--border)]">
                <div class="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider mb-3">
                    Font
                </div>
                <div class="flex flex-col gap-1">
                    {#each fonts as f}
                        <button
                            onclick={() => themeState.font = f.id}
                            class={`w-full text-left px-3 py-2 rounded-lg transition-all text-sm ${themeState.font === f.id
                                ? "bg-[var(--accent)] text-[var(--accent-text)] font-medium"
                                : "text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
                            }`}
                            style={`font-family: ${f.family}`}
                        >
                            {f.name}
                        </button>
                    {/each}
                </div>
            </div>

            <!-- Font Size Section -->
            <div class="p-4">
                <div class="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider mb-3">
                    Font Size
                </div>
                <div class="flex gap-2">
                    {#each fontSizes as s}
                        <button
                            onclick={() => themeState.fontSize = s.id}
                            class={`flex-1 px-3 py-2 rounded-lg transition-all text-sm text-center ${themeState.fontSize === s.id
                                ? "bg-[var(--accent)] text-[var(--accent-text)] font-medium"
                                : "text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
                            }`}
                        >
                            {s.name}
                        </button>
                    {/each}
                </div>
            </div>

            <!-- More settings -->
            <div class="p-2 border-t border-[var(--border)]">
                <button
                    onclick={() => { isOpen = false; window.dispatchEvent(new CustomEvent("paperling:open-settings")); }}
                    class="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-colors"
                >
                    <span class="material-symbols-outlined text-[18px]">tune</span>
                    More settings…
                </button>
            </div>
        </div>
    {/if}
</div>

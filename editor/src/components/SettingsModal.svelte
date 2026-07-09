<script lang="ts">
    import { themeState, type Theme, type FontFamily, type FontSize } from "../state/theme.svelte.ts";
    import { settingsState } from "../state/settings.svelte.ts";
    import { attachFocusTrap } from "../utils/focusTrap";
    import mascotWave from "../assets/mascot/mascot-wave.png";

    interface Props {
        isOpen: boolean;
        onClose: () => void;
    }

    let { isOpen, onClose }: Props = $props();

    type Section = "appearance" | "editor" | "about";

    const sections: Array<{ id: Section; label: string; icon: string }> = [
        { id: "appearance", label: "Appearance", icon: "palette" },
        { id: "editor", label: "Editor", icon: "edit" },
        { id: "about", label: "About", icon: "info" },
    ];

    const themes: Array<{ id: Theme; name: string; colors: [string, string]; textColor: string }> = [
        { id: "dark", name: "Dark", colors: ["#0a0a0a", "#141414"], textColor: "#ffffff" },
        { id: "light", name: "Light", colors: ["#ffffff", "#f4f2ee"], textColor: "#171717" },
        { id: "paper", name: "Paper", colors: ["#f5f0e6", "#ebe5d8"], textColor: "#3d3d3d" },
        { id: "dracula", name: "Dracula", colors: ["#282a36", "#44475a"], textColor: "#f8f8f2" },
    ];

    const fonts: Array<{ id: FontFamily; name: string; kind: string; stack: string }> = [
        { id: "inter", name: "Inter", kind: "Sans-serif", stack: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif" },
        { id: "merriweather", name: "Merriweather", kind: "Serif", stack: "'Merriweather', Georgia, 'Times New Roman', serif" },
        { id: "lora", name: "Lora", kind: "Serif", stack: "'Lora', Georgia, 'Times New Roman', serif" },
        { id: "source-serif", name: "Source Serif", kind: "Serif", stack: "'Source Serif 4', Georgia, 'Times New Roman', serif" },
        { id: "fira-sans", name: "Fira Sans", kind: "Sans-serif", stack: "'Fira Sans', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif" },
    ];

    const fontSizes: Array<{ id: FontSize; name: string; sample: number }> = [
        { id: "small", name: "Small", sample: 13 },
        { id: "medium", name: "Medium", sample: 16 },
        { id: "large", name: "Large", sample: 19 },
    ];

    let section = $state<Section>("appearance");
    let filter = $state("");
    let dialogRef: HTMLDivElement | null = $state(null);

    // Escape to close + focus trap
    $effect(() => {
        if (!isOpen) return;
        let detachTrap: (() => void) | undefined;
        if (dialogRef) {
            detachTrap = attachFocusTrap(dialogRef);
        }
        const onKey = (e: KeyboardEvent) => {
            if (e.key === "Escape") {
                e.preventDefault();
                onClose();
            }
        };
        document.addEventListener("keydown", onKey);
        return () => {
            if (detachTrap) detachTrap();
            document.removeEventListener("keydown", onKey);
        };
    });

    function matches(text: string) {
        return !filter || text.toLowerCase().includes(filter.toLowerCase());
    }
</script>

{#if isOpen}
    <div class="fixed inset-0 z-[100] flex items-center justify-center" role="dialog" aria-modal="true" aria-label="Settings">
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" onclick={onClose} aria-hidden="true" role="presentation"></div>

        <div
            bind:this={dialogRef}
            class="relative z-10 w-[820px] max-w-[95vw] h-[600px] max-h-[90vh] flex bg-[var(--bg-primary)] border border-[var(--border)] rounded-[var(--radius-lg)] shadow-2xl overflow-hidden animate-fade-in"
            role="presentation"
        >
            <!-- Sidebar -->
            <aside class="w-36 sm:w-48 shrink-0 bg-[var(--bg-secondary)] border-r border-[var(--border)] flex flex-col">
                <div class="px-4 py-3 border-b border-[var(--border)]">
                    <input
                        type="text"
                        bind:value={filter}
                        placeholder="Search…"
                        aria-label="Search settings"
                        class="w-full px-2 py-1 text-sm bg-[var(--bg-input)] border border-[var(--border)] rounded-[var(--radius-md)] text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
                    />
                </div>
                <nav class="flex-1 py-2">
                    {#each sections as s}
                        <button
                            onclick={() => section = s.id}
                            class={`w-full flex items-center gap-2 px-4 py-2 text-sm text-left transition-colors ${section === s.id
                                ? "bg-[var(--bg-hover)] text-[var(--text-primary)] font-medium"
                                : "text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
                            }`}
                        >
                            <span class="material-symbols-outlined text-[12px]">{s.icon}</span>
                            {s.label}
                        </button>
                    {/each}
                </nav>
            </aside>

            <!-- Body -->
            <div class="flex-1 flex flex-col min-w-0">
                <header class="flex items-center justify-between px-6 py-3 border-b border-[var(--border)]">
                    <h2 class="text-base font-semibold text-[var(--text-primary)]">
                        {sections.find((s) => s.id === section)?.label ?? "Settings"}
                    </h2>
                    <button onclick={onClose} aria-label="Close settings" class="w-7 h-7 rounded-[var(--radius-sm)] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] flex items-center justify-center transition-colors">
                        <span class="material-symbols-outlined text-[12px]">close</span>
                    </button>
                </header>

                <div class="flex-1 overflow-y-auto px-6 py-4 space-y-6">
                    {#if section === "appearance"}
                        {#if matches("theme")}
                            <section>
                                <h3 class="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider mb-2">Theme</h3>
                                <div class="grid grid-cols-4 gap-2">
                                    {#each themes as t}
                                        <button
                                            onclick={() => themeState.theme = t.id}
                                            class={`flex flex-col items-center gap-2 p-3 rounded-[var(--radius-md)] transition-all ${themeState.theme === t.id
                                                ? "ring-2 ring-[var(--accent)] bg-[var(--bg-hover)]"
                                                : "hover:bg-[var(--bg-hover)]"
                                            }`}
                                            title={t.name}
                                        >
                                            <div class="w-12 h-12 rounded-[var(--radius-md)] overflow-hidden border border-[var(--border)] flex items-center justify-center" style={`background-color: ${t.colors[0]}`}>
                                                <div class="w-1/2 h-full" style={`background-color: ${t.colors[0]}`}></div>
                                                <div class="w-1/2 h-full" style={`background-color: ${t.colors[1]}`}></div>
                                            </div>
                                            <span class="text-[11px] text-[var(--text-primary)]">{t.name}</span>
                                        </button>
                                    {/each}
                                </div>
                            </section>
                        {/if}

                        {#if matches("font")}
                            <section>
                                <h3 class="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider mb-2">Font</h3>
                                <div class="grid grid-cols-2 gap-2">
                                    {#each fonts as f}
                                        {@const active = themeState.font === f.id}
                                        <button
                                            onclick={() => themeState.font = f.id}
                                            aria-pressed={active}
                                            class={`flex items-center justify-between gap-2 px-3 py-2.5 rounded-[var(--radius-md)] border text-left transition-all ${active
                                                ? "border-[var(--accent)] bg-[var(--bg-hover)] ring-1 ring-[var(--accent)]"
                                                : "border-[var(--border)] hover:border-[var(--text-muted)] hover:bg-[var(--bg-hover)]"
                                            }`}
                                        >
                                            <span class="min-w-0">
                                                <span class="block text-[15px] leading-tight text-[var(--text-primary)] truncate" style={`font-family: ${f.stack}`}>{f.name}</span>
                                                <span class="block text-[10px] text-[var(--text-muted)] mt-0.5">{f.kind}</span>
                                            </span>
                                            {#if active}
                                                <span class="material-symbols-outlined text-[12px] text-[var(--accent)] shrink-0">check</span>
                                            {/if}
                                        </button>
                                    {/each}
                                </div>
                            </section>
                        {/if}

                        {#if matches("size")}
                            <section>
                                <h3 class="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider mb-2">Font size</h3>
                                <div class="grid grid-cols-3 gap-2">
                                    {#each fontSizes as s}
                                        {@const active = themeState.fontSize === s.id}
                                        <button
                                            onclick={() => themeState.fontSize = s.id}
                                            aria-pressed={active}
                                            class={`flex flex-col items-center justify-center gap-1 py-3 rounded-[var(--radius-md)] border transition-all ${active
                                                ? "border-[var(--accent)] bg-[var(--bg-hover)] ring-1 ring-[var(--accent)]"
                                                : "border-[var(--border)] hover:border-[var(--text-muted)] hover:bg-[var(--bg-hover)]"
                                            }`}
                                        >
                                            <span class="leading-none text-[var(--text-primary)]" style={`font-size: ${s.sample}px`}>Aa</span>
                                            <span class="text-[11px] text-[var(--text-secondary)]">{s.name}</span>
                                        </button>
                                    {/each}
                                </div>
                            </section>
                        {/if}
                    {/if}

                    {#if section === "editor"}
                        <div class="rounded-[var(--radius-lg)] border border-[var(--border)] divide-y divide-[var(--border-subtle)] overflow-hidden">
                            {#if matches("toolbar")}
                                <button
                                    role="switch"
                                    aria-checked={settingsState.toolbarEnabled}
                                    onclick={() => settingsState.toolbarEnabled = !settingsState.toolbarEnabled}
                                    class="group w-full flex items-center justify-between gap-4 px-3.5 py-3 hover:bg-[var(--bg-hover)] transition-colors text-left"
                                >
                                    <div class="flex flex-col items-start min-w-0">
                                        <span class="text-sm font-medium text-[var(--text-primary)]">Show formatting toolbar</span>
                                        <span class="text-[11px] text-[var(--text-muted)] mt-0.5">Toolbar above the editor</span>
                                    </div>
                                    <span class={`relative inline-block w-[42px] h-[24px] rounded-full shrink-0 transition-colors duration-200 ${settingsState.toolbarEnabled ? "bg-[var(--accent)]" : "bg-[var(--text-muted)]/45 group-hover:bg-[var(--text-muted)]/60"}`}>
                                        <span class={`absolute top-[3px] left-[3px] w-[18px] h-[18px] rounded-full bg-white shadow-[0_1px_2px_rgba(0,0,0,0.3)] transition-transform duration-200 ease-out ${settingsState.toolbarEnabled ? "translate-x-[18px]" : "translate-x-0"}`}></span>
                                    </span>
                                </button>
                            {/if}
                            {#if matches("word wrap")}
                                <button
                                    role="switch"
                                    aria-checked={settingsState.wordWrap}
                                    onclick={() => settingsState.wordWrap = !settingsState.wordWrap}
                                    class="group w-full flex items-center justify-between gap-4 px-3.5 py-3 hover:bg-[var(--bg-hover)] transition-colors text-left"
                                >
                                    <div class="flex flex-col items-start min-w-0">
                                        <span class="text-sm font-medium text-[var(--text-primary)]">Word wrap</span>
                                        <span class="text-[11px] text-[var(--text-muted)] mt-0.5">Wrap long lines instead of horizontal scroll</span>
                                    </div>
                                    <span class={`relative inline-block w-[42px] h-[24px] rounded-full shrink-0 transition-colors duration-200 ${settingsState.wordWrap ? "bg-[var(--accent)]" : "bg-[var(--text-muted)]/45 group-hover:bg-[var(--text-muted)]/60"}`}>
                                        <span class={`absolute top-[3px] left-[3px] w-[18px] h-[18px] rounded-full bg-white shadow-[0_1px_2px_rgba(0,0,0,0.3)] transition-transform duration-200 ease-out ${settingsState.wordWrap ? "translate-x-[18px]" : "translate-x-0"}`}></span>
                                    </span>
                                </button>
                            {/if}
                            {#if matches("spell check")}
                                <button
                                    role="switch"
                                    aria-checked={settingsState.spellCheck}
                                    onclick={() => settingsState.spellCheck = !settingsState.spellCheck}
                                    class="group w-full flex items-center justify-between gap-4 px-3.5 py-3 hover:bg-[var(--bg-hover)] transition-colors text-left"
                                >
                                    <div class="flex flex-col items-start min-w-0">
                                        <span class="text-sm font-medium text-[var(--text-primary)]">Spell check</span>
                                        <span class="text-[11px] text-[var(--text-muted)] mt-0.5">Underline misspelled words while you type</span>
                                    </div>
                                    <span class={`relative inline-block w-[42px] h-[24px] rounded-full shrink-0 transition-colors duration-200 ${settingsState.spellCheck ? "bg-[var(--accent)]" : "bg-[var(--text-muted)]/45 group-hover:bg-[var(--text-muted)]/60"}`}>
                                        <span class={`absolute top-[3px] left-[3px] w-[18px] h-[18px] rounded-full bg-white shadow-[0_1px_2px_rgba(0,0,0,0.3)] transition-transform duration-200 ease-out ${settingsState.spellCheck ? "translate-x-[18px]" : "translate-x-0"}`}></span>
                                    </span>
                                </button>
                            {/if}
                            {#if matches("autosave")}
                                <button
                                    role="switch"
                                    aria-checked={settingsState.autoSave}
                                    onclick={() => settingsState.autoSave = !settingsState.autoSave}
                                    class="group w-full flex items-center justify-between gap-4 px-3.5 py-3 hover:bg-[var(--bg-hover)] transition-colors text-left"
                                >
                                    <div class="flex flex-col items-start min-w-0">
                                        <span class="text-sm font-medium text-[var(--text-primary)]">Autosave</span>
                                        <span class="text-[11px] text-[var(--text-muted)] mt-0.5">Save automatically a moment after you stop typing</span>
                                    </div>
                                    <span class={`relative inline-block w-[42px] h-[24px] rounded-full shrink-0 transition-colors duration-200 ${settingsState.autoSave ? "bg-[var(--accent)]" : "bg-[var(--text-muted)]/45 group-hover:bg-[var(--text-muted)]/60"}`}>
                                        <span class={`absolute top-[3px] left-[3px] w-[18px] h-[18px] rounded-full bg-white shadow-[0_1px_2px_rgba(0,0,0,0.3)] transition-transform duration-200 ease-out ${settingsState.autoSave ? "translate-x-[18px]" : "translate-x-0"}`}></span>
                                    </span>
                                </button>
                            {/if}
                            {#if matches("vim")}
                                <button
                                    role="switch"
                                    aria-checked={settingsState.vimMode}
                                    onclick={() => settingsState.vimMode = !settingsState.vimMode}
                                    class="group w-full flex items-center justify-between gap-4 px-3.5 py-3 hover:bg-[var(--bg-hover)] transition-colors text-left"
                                >
                                    <div class="flex flex-col items-start min-w-0">
                                        <span class="text-sm font-medium text-[var(--text-primary)]">Vim mode</span>
                                        <span class="text-[11px] text-[var(--text-muted)] mt-0.5">Enable Vim keybindings in the code editor</span>
                                    </div>
                                    <span class={`relative inline-block w-[42px] h-[24px] rounded-full shrink-0 transition-colors duration-200 ${settingsState.vimMode ? "bg-[var(--accent)]" : "bg-[var(--text-muted)]/45 group-hover:bg-[var(--text-muted)]/60"}`}>
                                        <span class={`absolute top-[3px] left-[3px] w-[18px] h-[18px] rounded-full bg-white shadow-[0_1px_2px_rgba(0,0,0,0.3)] transition-transform duration-200 ease-out ${settingsState.vimMode ? "translate-x-[18px]" : "translate-x-0"}`}></span>
                                    </span>
                                </button>
                            {/if}
                        </div>
                    {/if}

                    {#if section === "about"}
                        <div class="text-sm text-[var(--text-secondary)] space-y-2">
                            <div class="flex items-center gap-3">
                                <img src="/icon.svg" alt="Paperling" class="w-10 h-10" />
                                <div>
                                    <div class="text-[var(--text-primary)] font-semibold">Paperling</div>
                                    <div class="text-[11px]">A minimal markdown editor</div>
                                </div>
                            </div>
                            <p>Built with Tauri + Svelte 5 + TypeScript.</p>
                            <button
                                type="button"
                                onclick={() => {
                                    onClose();
                                    window.dispatchEvent(new CustomEvent("paperling:replay-tour"));
                                }}
                                class="btn-press mt-3 w-full flex items-center gap-3 px-3.5 py-3 rounded-[var(--radius-md)] border border-[var(--border)] bg-[var(--bg-secondary)] hover:bg-[var(--bg-hover)] transition-colors text-left"
                            >
                                <img src={mascotWave} alt="" aria-hidden="true" draggable="false" class="w-10 h-10 object-contain select-none shrink-0" />
                                <span class="flex flex-col items-start min-w-0">
                                    <span class="text-sm font-medium text-[var(--text-primary)]">Replay the welcome tour</span>
                                    <span class="text-[11px] text-[var(--text-muted)] mt-0.5">A 30-second walkthrough of the editor, views, and shortcuts</span>
                                </span>
                                <span class="material-symbols-outlined ml-auto text-[12px] text-[var(--text-muted)]">arrow_forward</span>
                            </button>
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>
{/if}

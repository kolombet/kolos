<script lang="ts">
    import { attachFocusTrap } from "../utils/focusTrap";
    import iconKeyboard from "../assets/mascot/icon-keyboard.png";

    interface Props {
        isOpen: boolean;
        onClose: () => void;
    }

    let { isOpen, onClose }: Props = $props();

    interface Shortcut {
        keys: string;
        description: string;
    }

    interface ShortcutGroup {
        title: string;
        items: Shortcut[];
    }

    const isMac = typeof navigator !== "undefined" && /Mac|iPod|iPhone|iPad/.test(navigator.platform);
    const cmd = isMac ? "⌘" : "Ctrl";

    const groups: ShortcutGroup[] = [
        {
            title: "File",
            items: [
                { keys: `${cmd}+O`, description: "Open file" },
                { keys: `${cmd}+N`, description: "New file (new tab)" },
                { keys: `${cmd}+W`, description: "Close tab" },
                { keys: `${cmd}+S`, description: "Save" },
                { keys: `${cmd}+Shift+S`, description: "Save As…" },
            ],
        },
        {
            title: "Tabs",
            items: [
                { keys: `${cmd}+N`, description: "New tab" },
                { keys: `${cmd}+W`, description: "Close tab" },
                { keys: `${cmd}+Shift+T`, description: "Reopen closed tab" },
                { keys: `${cmd}+Tab`, description: "Next tab" },
                { keys: `${cmd}+Shift+Tab`, description: "Previous tab" },
                { keys: "Alt+←/→", description: "Previous / next tab" },
                { keys: `${cmd}+1-8`, description: "Jump to tab N" },
                { keys: `${cmd}+9`, description: "Jump to last tab" },
            ],
        },
        {
            title: "View",
            items: [
                { keys: `${cmd}+E`, description: "Toggle Reader / Code" },
                { keys: `${cmd}+\\`, description: "Toggle split view" },
                { keys: "F11", description: "Toggle fullscreen" },
                { keys: `${cmd}+Shift+E`, description: "Toggle file explorer" },
                { keys: `${cmd}+Shift+F`, description: "Search across files" },
                { keys: `${cmd}+Shift+O`, description: "Toggle outline" },
                { keys: `${cmd}+P`, description: "Command palette" },
                { keys: `${cmd}+,`, description: "Open settings" },
                { keys: "?", description: "Show this cheatsheet" },
            ],
        },
        {
            title: "Editor — Formatting",
            items: [
                { keys: `${cmd}+B`, description: "Bold (toggle)" },
                { keys: `${cmd}+I`, description: "Italic (toggle)" },
                { keys: `${cmd}+K`, description: "Insert link" },
                { keys: `${cmd}+/`, description: "Toggle blockquote on line" },
            ],
        },
        {
            title: "Editor — Navigation",
            items: [
                { keys: "Tab", description: "Indent line / selection" },
                { keys: "Shift+Tab", description: "Outdent line / selection" },
                { keys: "Enter", description: "Continue list, blockquote, or task item" },
            ],
        },
        {
            title: "Editor — Auto-pair",
            items: [
                { keys: "( [ { ` \" '", description: "Wrap selection or insert pair" },
                { keys: ") ] } ` \" '", description: "Type past matching closer" },
                { keys: "Backspace", description: "Removes empty pair atomically" },
            ],
        },
        {
            title: "Slash & Smart Paste",
            items: [
                { keys: "/", description: "Slash menu (at line start)" },
                { keys: "Paste URL on selection", description: "Wraps selection as link" },
                { keys: "Paste rich HTML", description: "Converts to markdown" },
                { keys: "Paste tab-separated", description: "Converts to GFM table" },
            ],
        },
    ];

    let dialogRef: HTMLDivElement | null = $state(null);
    let filter = $state("");

    $effect(() => {
        if (!isOpen) return;
        let detachTrap: (() => void) | undefined;
        
        const handleKey = (e: KeyboardEvent) => {
            if (e.key === "Escape") {
                e.preventDefault();
                onClose();
            }
        };
        document.addEventListener("keydown", handleKey);
        
        if (dialogRef) {
            detachTrap = attachFocusTrap(dialogRef);
            const input = dialogRef.querySelector<HTMLInputElement>("input");
            input?.focus();
        }
        
        return () => {
            if (detachTrap) detachTrap();
            document.removeEventListener("keydown", handleKey);
        };
    });

    let filtered = $derived((() => {
        const q = filter.trim().toLowerCase();
        if (!q) return groups;
        return groups
            .map((g) => ({
                ...g,
                items: g.items.filter((it) => it.description.toLowerCase().includes(q) || it.keys.toLowerCase().includes(q)),
            }))
            .filter((g) => g.items.length > 0);
    })());
</script>

{#if isOpen}
    <div class="fixed inset-0 z-[100] flex items-center justify-center" role="dialog" aria-modal="true" aria-labelledby="cheatsheet-title">
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" onclick={onClose} aria-hidden="true" role="presentation"></div>

        <div
            bind:this={dialogRef}
            class="relative z-10 w-[640px] max-h-[80vh] flex flex-col bg-[var(--bg-primary)] border border-[var(--border)] rounded-[var(--radius-lg)] shadow-2xl overflow-hidden animate-fade-in"
            role="presentation"
        >
            <div class="flex items-center gap-3 px-5 py-4 border-b border-[var(--border)]">
                <img src={iconKeyboard} alt="" aria-hidden="true" draggable="false" class="w-8 h-8 object-contain select-none" />
                <h2 id="cheatsheet-title" class="text-base font-semibold text-[var(--text-primary)]">Keyboard Shortcuts</h2>
                <input
                    type="text"
                    bind:value={filter}
                    placeholder="Filter shortcuts…"
                    aria-label="Filter shortcuts"
                    class="ml-auto px-2 py-1 text-sm bg-[var(--bg-input)] border border-[var(--border)] rounded-[var(--radius-md)] text-[var(--text-primary)] outline-none focus:border-[var(--accent)] w-48"
                />
                <button
                    onclick={onClose}
                    aria-label="Close cheatsheet"
                    class="w-7 h-7 rounded-[var(--radius-sm)] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] flex items-center justify-center transition-colors"
                >
                    <span class="material-symbols-outlined text-[18px]">close</span>
                </button>
            </div>

            <div class="flex-1 overflow-y-auto px-5 py-4 grid grid-cols-2 gap-x-6 gap-y-5">
                {#if filtered.length === 0}
                    <div class="col-span-2 text-center text-[var(--text-secondary)] py-8 text-sm">
                        No shortcuts match "{filter}"
                    </div>
                {:else}
                    {#each filtered as g}
                        <section>
                            <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-secondary)] mb-2">
                                {g.title}
                            </h3>
                            <ul class="space-y-1.5">
                                {#each g.items as it}
                                    <li class="flex items-center justify-between gap-3">
                                        <span class="text-sm text-[var(--text-primary)]">{it.description}</span>
                                        <span class="flex items-center gap-1 shrink-0">
                                            {#each it.keys.split(/\s+/) as part, i}
                                                <span class="inline-flex items-center">
                                                    {#if i > 0}
                                                        <span class="mx-0.5 text-[var(--text-muted)]">+</span>
                                                    {/if}
                                                    <kbd class="px-1.5 py-0.5 text-[11px] font-mono rounded border border-[var(--border)] bg-[var(--bg-input)] text-[var(--text-primary)] shadow-sm">
                                                        {part}
                                                    </kbd>
                                                </span>
                                            {/each}
                                        </span>
                                    </li>
                                {/each}
                            </ul>
                        </section>
                    {/each}
                {/if}
            </div>

            <div class="px-5 py-2 text-[11px] text-[var(--text-muted)] border-t border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
                Press <kbd class="px-1 py-0.5 font-mono rounded border border-[var(--border)] bg-[var(--bg-input)]">Esc</kbd> to close
            </div>
        </div>
    </div>
{/if}

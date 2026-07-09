<script lang="ts">
    import { attachFocusTrap } from "../utils/focusTrap";

    export interface PaletteCommand {
        id: string;
        label: string;
        hint?: string;
        section: string;
        icon?: string;
        keywords?: string;
        run: () => void;
    }

    interface Props {
        isOpen: boolean;
        items: PaletteCommand[];
        onClose: () => void;
    }

    let { isOpen, items, onClose }: Props = $props();

    let query = $state("");
    let activeIdx = $state(0);
    let dialogRef: HTMLDivElement | null = $state(null);
    let listRef: HTMLUListElement | null = $state(null);

    // Reset state on open
    $effect(() => {
        if (isOpen) {
            query = "";
            activeIdx = 0;
        }
    });

    // Focus input + trap, Escape to close
    $effect(() => {
        if (!isOpen) return;
        let detachTrap: (() => void) | undefined;
        if (dialogRef) {
            detachTrap = attachFocusTrap(dialogRef);
            const input = dialogRef.querySelector<HTMLInputElement>("input");
            if (input) input.focus();
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

    function matchIndices(needle: string, haystack: string): number[] | null {
        if (!needle) return null;
        const n = needle.toLowerCase();
        const h = haystack.toLowerCase();
        const sub = h.indexOf(n);
        if (sub !== -1) {
            return Array.from({ length: n.length }, (_, i) => sub + i);
        }
        const out: number[] = [];
        let hi = 0;
        for (let ni = 0; ni < n.length; ni++) {
            const found = h.indexOf(n[ni], hi);
            if (found === -1) return null;
            out.push(found);
            hi = found + 1;
        }
        return out;
    }

    function fuzzyScore(needle: string, haystack: string): number {
        if (!needle) return 0;
        const n = needle.toLowerCase();
        const h = haystack.toLowerCase();
        if (h.includes(n)) return h.indexOf(n);
        let hi = 0;
        let score = 0;
        let lastIdx = -1;
        for (let ni = 0; ni < n.length; ni++) {
            const ch = n[ni];
            const found = h.indexOf(ch, hi);
            if (found === -1) return -1;
            if (lastIdx !== -1) score += (found - lastIdx);
            lastIdx = found;
            hi = found + 1;
        }
        return 1000 + score;
    }

    function getHighlightedParts(label: string, queryText: string) {
        const q = queryText.trim();
        if (!q) return [{ text: label, highlight: false }];
        const idx = matchIndices(q, label);
        if (!idx) return [{ text: label, highlight: false }];
        
        const matched = new Set(idx);
        const parts: { text: string; highlight: boolean }[] = [];
        let buf = "";
        let bufMatched = matched.has(0);
        
        const flush = () => {
            if (!buf) return;
            parts.push({ text: buf, highlight: bufMatched });
            buf = "";
        };
        
        for (let i = 0; i < label.length; i++) {
            const m = matched.has(i);
            if (i > 0 && m !== bufMatched) {
                flush();
                bufMatched = m;
            }
            buf += label[i];
        }
        flush();
        return parts;
    }

    let ranked = $derived((() => {
        if (!isOpen) return [];
        if (!query.trim()) return items;
        return items
            .map((it) => {
                const candidates = [it.label, it.hint ?? "", it.keywords ?? "", it.section];
                let best = -1;
                for (const c of candidates) {
                    const s = fuzzyScore(query, c);
                    if (s !== -1 && (best === -1 || s < best)) best = s;
                }
                return { item: it, score: best };
            })
            .filter((r) => r.score !== -1)
            .sort((a, b) => a.score - b.score)
            .map((r) => r.item);
    })());

    let grouped = $derived((() => {
        const out: Array<{ section: string; items: PaletteCommand[] }> = [];
        const seen = new Map<string, number>();
        for (const it of ranked) {
            const idx = seen.get(it.section);
            if (idx === undefined) {
                seen.set(it.section, out.length);
                out.push({ section: it.section, items: [it] });
            } else {
                out[idx].items.push(it);
            }
        }
        return out;
    })());

    // Keep activeIdx valid
    $effect(() => {
        if (activeIdx >= ranked.length) {
            activeIdx = 0;
        }
    });

    // Auto-scroll active row
    $effect(() => {
        if (listRef && activeIdx >= 0) {
            const active = listRef.querySelector<HTMLElement>(`[data-idx="${activeIdx}"]`);
            if (active) active.scrollIntoView({ block: "nearest" });
        }
    });

    function onKeyDown(e: KeyboardEvent) {
        if (e.key === "ArrowDown") {
            e.preventDefault();
            activeIdx = ranked.length === 0 ? 0 : (activeIdx + 1) % ranked.length;
        } else if (e.key === "ArrowUp") {
            e.preventDefault();
            activeIdx = ranked.length === 0 ? 0 : (activeIdx - 1 + ranked.length) % ranked.length;
        } else if (e.key === "Enter") {
            e.preventDefault();
            const cmd = ranked[activeIdx];
            if (cmd) {
                onClose();
                cmd.run();
            }
        }
    }
</script>

{#if isOpen}
<div class="fixed inset-0 z-[110] flex items-start justify-center pt-[12vh]" role="dialog" aria-modal="true" aria-label="Command palette">
    <div class="absolute inset-0 bg-black/40 backdrop-blur-sm" onclick={onClose} aria-hidden="true" role="presentation"></div>

    <div
        bind:this={dialogRef}
        class="relative z-10 w-[640px] max-w-[92vw] flex flex-col bg-[var(--bg-secondary)] border border-[var(--border)] rounded-[var(--radius-lg)] shadow-2xl overflow-hidden animate-fade-in"
        onkeydown={onKeyDown}
        role="presentation"
    >
        <div class="flex items-center gap-3 px-4 py-3 border-b border-[var(--border)]">
            <span class="material-symbols-outlined text-[var(--text-secondary)]">search</span>
            <input
                type="text"
                bind:value={query}
                placeholder="Type a command, file, or heading…"
                aria-label="Search commands"
                class="flex-1 bg-transparent text-[var(--text-primary)] outline-none text-sm placeholder:text-[var(--text-muted)]"
            />
            <kbd class="px-1.5 py-0.5 text-[11px] font-mono rounded border border-[var(--border)] bg-[var(--bg-input)] text-[var(--text-muted)]">Esc</kbd>
        </div>

        <ul bind:this={listRef} class="max-h-[420px] overflow-y-auto py-1" role="listbox">
            {#if ranked.length === 0}
                <li class="px-4 py-6 text-center text-sm text-[var(--text-secondary)]">No results</li>
            {:else}
                {#each grouped as g}
                    <li>
                        <div class="px-4 py-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
                            {g.section}
                        </div>
                        <ul>
                            {#each g.items as cmd}
                                {@const idx = ranked.indexOf(cmd)}
                                {@const active = idx === activeIdx}
                                <li>
                                    <button
                                        role="option"
                                        aria-selected={active}
                                        data-idx={idx}
                                        onmouseenter={() => activeIdx = idx}
                                        onclick={() => { onClose(); cmd.run(); }}
                                        class={`w-full flex items-center gap-3 px-4 py-2 text-left transition-colors ${active ? "bg-[var(--bg-hover)]" : ""}`}
                                    >
                                        <span class={`material-symbols-outlined text-[19px]! shrink-0 ${active ? "text-[var(--accent)]" : "text-[var(--text-secondary)]"}`}>
                                            {cmd.icon ?? "chevron_right"}
                                        </span>
                                        <span class="flex-1 min-w-0 text-sm text-[var(--text-primary)] truncate">
                                            {#each getHighlightedParts(cmd.label, query) as part}
                                                {#if part.highlight}
                                                    <mark class="bg-transparent text-[var(--accent)] font-semibold">{part.text}</mark>
                                                {:else}
                                                    <span>{part.text}</span>
                                                {/if}
                                            {/each}
                                        </span>
                                        {#if cmd.hint}
                                            <span class="text-[11px] text-[var(--text-muted)] tabular-nums truncate ml-2 shrink-0">
                                                {cmd.hint}
                                            </span>
                                        {/if}
                                    </button>
                                </li>
                            {/each}
                        </ul>
                    </li>
                {/each}
            {/if}
        </ul>

        <div class="px-4 py-1.5 text-[10px] text-[var(--text-muted)] border-t border-[var(--border-subtle)] bg-[var(--bg-titlebar)] flex items-center gap-3">
            <span><kbd class="px-1 font-mono rounded border border-[var(--border)] bg-[var(--bg-input)]">↑↓</kbd> navigate</span>
            <span><kbd class="px-1 font-mono rounded border border-[var(--border)] bg-[var(--bg-input)]">↵</kbd> run</span>
            <span class="ml-auto">{ranked.length} {ranked.length === 1 ? "result" : "results"}</span>
        </div>
    </div>
</div>
{/if}

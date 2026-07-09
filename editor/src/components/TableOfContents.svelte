<script lang="ts">
    import { attachFocusTrap } from "../utils/focusTrap";
    import mascotReading from "../assets/mascot/mascot-reading.png";
    import mascotMagnify from "../assets/mascot/mascot-magnify.png";

    interface TocItem {
        id: string;
        text: string;
        level: number;
        line: number;
    }

    interface Props {
        isOpen: boolean;
        content: string;
        onClose: () => void;
        activeLine?: number;
    }

    let { isOpen, content, onClose, activeLine = 1 }: Props = $props();

    let panelRef: HTMLElement | null = $state(null);
    let filter = $state("");

    let headings = $derived((() => {
        if (!isOpen || !content) return [];
        const normalized = content.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
        const lines = normalized.split("\n");
        const items: TocItem[] = [];

        lines.forEach((line, index) => {
            const trimmed = line.trim();
            const match = trimmed.match(/^(#{1,6})\s+(.+)$/);
            if (match) {
                const level = match[1].length;
                const text = match[2].trim();
                const id = `heading-${index}-${text.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "")}`;
                items.push({ id, text, level, line: index + 1 });
            }
        });
        return items;
    })());

    let activeHeadingIdx = $derived((() => {
        let idx = -1;
        for (let i = 0; i < headings.length; i++) {
            if (headings[i].line <= activeLine) idx = i;
            else break;
        }
        return idx;
    })());

    let visible = $derived((() => {
        if (!filter.trim()) return headings.map((h, i) => ({ h, i }));
        const q = filter.toLowerCase();
        return headings.map((h, i) => ({ h, i })).filter(({ h }) => h.text.toLowerCase().includes(q));
    })());

    // Scroll active item into view
    $effect(() => {
        if (activeHeadingIdx !== -1 && panelRef) {
            const el = panelRef.querySelector<HTMLElement>(`[data-toc-idx="${activeHeadingIdx}"]`);
            if (el) el.scrollIntoView({ block: "nearest" });
        }
    });

    // Escape and focus trap
    $effect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape") {
                e.preventDefault();
                onClose();
            }
        };

        document.addEventListener("keydown", handleKeyDown);
        if (panelRef) panelRef.focus();
        let detachTrap: (() => void) | undefined;
        if (panelRef) detachTrap = attachFocusTrap(panelRef);

        return () => {
            document.removeEventListener("keydown", handleKeyDown);
            if (detachTrap) detachTrap();
        };
    });

    function handleHeadingClick(line: number) {
        window.dispatchEvent(new CustomEvent("paperling:goto-line", { detail: { line } }));
    }

    function getIndent(level: number): string {
        const indents = ["", "pl-4", "pl-8", "pl-12", "pl-16", "pl-20"];
        return indents[level - 1] || "";
    }

    function getIcon(level: number): string {
        if (level === 1) return "title";
        if (level === 2) return "format_h2";
        return "format_h3";
    }
</script>

<aside
    bind:this={panelRef}
    role="navigation"
    aria-label="Table of contents"
    tabindex="-1"
    class={`fixed left-0 top-12 bottom-7 w-72 bg-[var(--bg-secondary)] border-r border-[var(--border)] z-50 shadow-2xl flex flex-col overflow-hidden transition-transform duration-200 ease-out ${isOpen ? "translate-x-0" : "-translate-x-full"}`}
>
    <!-- Header -->
    <div class="h-10 shrink-0 px-4 flex items-center justify-between border-b border-[var(--border)] bg-[var(--bg-titlebar)]">
        <div class="flex items-center gap-2 text-sm font-semibold text-[var(--text-primary)] no-select">
            <span class="material-symbols-outlined text-[16px]!">format_list_bulleted</span>
            <span>Outline</span>
        </div>
        <button
            onclick={onClose}
            aria-label="Close outline"
            class="btn-press flex items-center justify-center w-7 h-7 rounded-[var(--radius-sm)] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
        >
            <span class="material-symbols-outlined text-[16px]!">close</span>
        </button>
    </div>

    <!-- Filter -->
    {#if headings.length > 5}
        <div class="px-3 py-2 shrink-0 border-b border-[var(--border-subtle)]">
            <input
                type="text"
                bind:value={filter}
                placeholder="Filter headings…"
                aria-label="Filter headings"
                class="w-full px-2 py-1 text-xs bg-[var(--bg-input)] border border-[var(--border)] rounded-[var(--radius-sm)] text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
            />
        </div>
    {/if}

    <!-- Content -->
    <nav class="flex-1 min-h-0 overflow-y-auto" aria-label="Document headings">
        {#if headings.length === 0}
            <div class="flex flex-col items-center justify-center py-8 text-[var(--text-secondary)] text-sm gap-2 px-4 text-center">
                <img src={mascotReading} alt="" aria-hidden="true" draggable="false" class="w-20 h-20 object-contain select-none opacity-90" />
                <span>No headings yet.</span>
                <span class="text-[11px] text-[var(--text-muted)]">Type <code class="font-mono">#</code> to add one.</span>
            </div>
        {:else if visible.length === 0}
            <div class="flex flex-col items-center justify-center py-8 gap-2 text-[var(--text-secondary)] text-sm">
                <img src={mascotMagnify} alt="" aria-hidden="true" draggable="false" class="w-20 h-20 object-contain select-none opacity-90" />
                <span>No matches</span>
            </div>
        {:else}
            <ul class="py-2">
                {#each visible as { h: heading, i: index } (heading.id + "-" + index)}
                    {@const isActive = index === activeHeadingIdx}
                    <li data-toc-idx={index}>
                        <button
                            onclick={() => handleHeadingClick(heading.line)}
                            aria-label={`Go to heading: ${heading.text}`}
                            aria-current={isActive ? "location" : undefined}
                            class={`btn-press w-full px-4 py-1.5 text-left text-sm flex items-center gap-2 transition-colors ${getIndent(heading.level)} ${isActive
                                ? "bg-[var(--bg-hover)] text-[var(--text-primary)] border-l-2 border-[var(--accent)] -ml-px"
                                : "text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
                            }`}
                        >
                            <span class={`material-symbols-outlined text-[12px]! ${heading.level === 1 ? "text-[var(--text-primary)]" : "opacity-60"}`}>
                                {getIcon(heading.level)}
                            </span>
                            <span class={`truncate ${heading.level === 1 ? "font-semibold" : heading.level === 2 ? "font-medium" : ""}`}>
                                {heading.text}
                            </span>
                        </button>
                    </li>
                {/each}
            </ul>
        {/if}
    </nav>
</aside>

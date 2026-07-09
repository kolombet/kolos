<script lang="ts">
    import { attachFocusTrap } from "../utils/focusTrap";
    import { computeStats } from "../utils/documentStats";
    import iconClock from "../assets/mascot/icon-clock.png";

    interface Props {
        isOpen: boolean;
        content: string;
        onClose: () => void;
    }

    let { isOpen, content, onClose }: Props = $props();

    let dialogRef: HTMLDivElement | null = $state(null);
    let stats = $derived(isOpen ? computeStats(content) : null);

    const formatReadingTime = (min: number): string => {
        if (min < 1) return "< 1 min";
        if (min < 60) return `${Math.round(min)} min`;
        const hours = Math.floor(min / 60);
        const rem = Math.round(min % 60);
        return rem === 0 ? `${hours}h` : `${hours}h ${rem}m`;
    };

    let rows = $derived(stats ? [
        ["Words", stats.words.toLocaleString()],
        ["Characters", stats.chars.toLocaleString()],
        ["Characters (no spaces)", stats.charsNoSpaces.toLocaleString()],
        ["Sentences", stats.sentences.toLocaleString()],
        ["Paragraphs", stats.paragraphs.toLocaleString()],
        ["Lines", stats.lines.toLocaleString()],
        ["Headings", stats.headings.toLocaleString()],
        ["Links", stats.links.toLocaleString()],
        ["Images", stats.images.toLocaleString()],
        ["Code blocks", stats.codeBlocks.toLocaleString()],
        ["Reading time", formatReadingTime(stats.readingTimeMin)],
    ] : []);

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
</script>

{#if isOpen && stats}
    <div class="fixed inset-0 z-[100] flex items-center justify-center" role="dialog" aria-modal="true" aria-label="Document statistics">
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" onclick={onClose} aria-hidden="true" role="presentation"></div>
        <div
            bind:this={dialogRef}
            class="relative z-10 w-[420px] max-w-[92vw] bg-[var(--bg-primary)] border border-[var(--border)] rounded-[var(--radius-lg)] shadow-2xl overflow-hidden animate-fade-in"
            role="presentation"
        >
            <header class="flex items-center justify-between px-5 py-3 border-b border-[var(--border)]">
                <div class="flex items-center gap-3">
                    <img src={iconClock} alt="" aria-hidden="true" draggable="false" class="w-8 h-8 object-contain select-none" />
                    <h2 class="text-base font-semibold text-[var(--text-primary)]">Document statistics</h2>
                </div>
                <button
                    type="button"
                    onclick={onClose}
                    aria-label="Close statistics"
                    class="w-7 h-7 rounded-[var(--radius-sm)] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] flex items-center justify-center transition-colors"
                >
                    <span class="material-symbols-outlined text-[19px]!">close</span>
                </button>
            </header>
            <dl class="px-5 py-4 grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
                {#each rows as [label, value]}
                    <div class="contents">
                        <dt class="text-[var(--text-secondary)]">{label}</dt>
                        <dd class="text-right font-mono text-[var(--text-primary)] tabular-nums">{value}</dd>
                    </div>
                {/each}
            </dl>
            <footer class="px-5 py-2 text-[11px] text-[var(--text-muted)] border-t border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
                Reading time assumes ~200 words per minute. Word and sentence counts exclude code blocks, frontmatter, and Markdown syntax.
            </footer>
        </div>
    </div>
{/if}

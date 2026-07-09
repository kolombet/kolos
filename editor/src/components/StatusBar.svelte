<script lang="ts">
    interface Props {
        isSaved?: boolean;
        lineNumber?: number;
        columnNumber?: number;
        mode?: "preview" | "code" | "split";
        showFileExplorer?: boolean;
        showTOC?: boolean;
        onToggleFileExplorer?: () => void;
        onToggleTOC?: () => void;
        wordCount?: number;
        charCount?: number;
        readingTimeMin?: number;
        selectionLength?: number;
        selectionWordCount?: number;
    }

    let {
        isSaved = true,
        lineNumber = 1,
        columnNumber = 1,
        mode = "preview",
        showFileExplorer = false,
        showTOC = false,
        onToggleFileExplorer,
        onToggleTOC,
        wordCount,
        charCount,
        readingTimeMin,
        selectionLength = 0,
        selectionWordCount = 0,
    }: Props = $props();

    let hasSelection = $derived(selectionLength > 0);

    function formatReadingTime(min: number): string {
        if (min < 1) return "< 1 min read";
        if (min < 60) return `${Math.round(min)} min read`;
        const hours = Math.floor(min / 60);
        const rem = Math.round(min % 60);
        return rem === 0 ? `${hours}h read` : `${hours}h ${rem}m read`;
    }
</script>

<footer
    role="status"
    class="h-7 shrink-0 bg-[var(--bg-titlebar)] border-t border-[var(--border)] px-4 flex items-center justify-between text-[11px] font-medium tracking-wide text-[var(--text-secondary)] no-select transition-colors"
>
    <div class="flex items-center gap-1">
        <button
            data-tour="file-explorer"
            onclick={onToggleFileExplorer}
            title="Files (Ctrl+Shift+E)"
            aria-label={showFileExplorer ? "Close file explorer" : "Open file explorer"}
            aria-pressed={showFileExplorer}
            class={`btn-press flex items-center justify-center w-8 h-6 rounded transition-colors ${showFileExplorer ? "bg-[var(--accent)] text-[var(--accent-text)]" : "hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"}`}
        >
            <span class="material-symbols-outlined text-[12px]!">folder_open</span>
        </button>

        <button
            data-tour="toc"
            onclick={onToggleTOC}
            title="Table of Contents (Ctrl+Shift+O)"
            aria-label={showTOC ? "Close table of contents" : "Open table of contents"}
            aria-pressed={showTOC}
            class={`btn-press flex items-center justify-center w-8 h-6 rounded transition-colors ${showTOC ? "bg-[var(--accent)] text-[var(--accent-text)]" : "hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"}`}
        >
            <span class="material-symbols-outlined text-[12px]!">format_list_bulleted</span>
        </button>
    </div>
    <div class="flex items-center gap-4">
        <div class="flex items-center gap-1.5" aria-label={isSaved ? "File saved" : "File has unsaved changes"}>
            <span
                class={`w-2 h-2 rounded-full transition-all ${isSaved ? "bg-[var(--status-saved)] shadow-[0_0_4px_rgba(80,250,123,0.4)]" : "bg-[var(--status-unsaved)] shadow-[0_0_4px_rgba(255,184,108,0.4)] status-dot-unsaved"}`}
            ></span>
            <span class="transition-colors">{isSaved ? "Saved" : "Unsaved"}</span>
        </div>
        {#if mode === "code" || mode === "split"}
            <div class="hover:text-[var(--text-primary)] cursor-default transition-colors">
                Ln {lineNumber}, Col {columnNumber}
            </div>
        {/if}
        {#if wordCount !== undefined}
            <div
                class={`flex items-center gap-1 cursor-default transition-colors ${hasSelection ? "text-[var(--accent)]" : "hover:text-[var(--text-primary)]"}`}
                title={hasSelection ? `Selection: ${selectionWordCount.toLocaleString()} words, ${selectionLength.toLocaleString()} characters` : (charCount !== undefined ? `${charCount.toLocaleString()} characters` : undefined)}
            >
                <span class="material-symbols-outlined text-[12px]! opacity-70">text_fields</span>
                {#if hasSelection}
                    {selectionWordCount.toLocaleString()} / {wordCount.toLocaleString()} words
                {:else}
                    {wordCount.toLocaleString()} words
                {/if}
            </div>
        {/if}
        {#if readingTimeMin !== undefined && readingTimeMin > 0}
            <div
                class="flex items-center gap-1 hover:text-[var(--text-primary)] cursor-default transition-colors"
                title="Estimated reading time at 200 wpm"
            >
                <span class="material-symbols-outlined text-[12px]! opacity-70">schedule</span>
                {formatReadingTime(readingTimeMin)}
            </div>
        {/if}
    </div>
</footer>

<script lang="ts">
    export interface TabBarItem {
        id: string;
        name: string;
        label: string;
        dirty: boolean;
    }

    interface Props {
        tabs: TabBarItem[];
        activeId: string | null;
        onSelect: (id: string) => void;
        onClose: (id: string) => void;
        onNewTab: () => void;
        onReorder?: (fromIndex: number, toIndex: number) => void;
        onContextMenu?: (id: string, x: number, y: number) => void;
    }

    let { tabs, activeId, onSelect, onClose, onNewTab, onReorder, onContextMenu }: Props = $props();

    let listRef: HTMLElement | null = $state(null);
    let tabRefs = new Map<string, HTMLElement>();
    let dragIndex: number | null = $state(null);
    let overIndex: number | null = $state(null);

    $effect(() => {
        if (!activeId) return;
        tabs.length;
        tabRefs.get(activeId)?.scrollIntoView({ block: "nearest", inline: "nearest" });
    });

    function registerTab(node: HTMLElement, id: string) {
        tabRefs.set(id, node);
        return {
            destroy() {
                tabRefs.delete(id);
            }
        };
    }

    function onWheel(e: WheelEvent) {
        if (!listRef || e.deltaY === 0) return;
        listRef.scrollLeft += e.deltaY;
    }

    function onKeyDown(e: KeyboardEvent, index: number) {
        const focusAndSelect = (i: number) => {
            const t = tabs[i];
            if (!t) return;
            onSelect(t.id);
            tabRefs.get(t.id)?.focus();
        };
        if (e.key === "ArrowRight") {
            e.preventDefault();
            focusAndSelect((index + 1) % tabs.length);
        } else if (e.key === "ArrowLeft") {
            e.preventDefault();
            focusAndSelect((index - 1 + tabs.length) % tabs.length);
        } else if (e.key === "Home") {
            e.preventDefault();
            focusAndSelect(0);
        } else if (e.key === "End") {
            e.preventDefault();
            focusAndSelect(tabs.length - 1);
        } else if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onSelect(tabs[index].id);
        } else if (e.key === "Delete" || e.key === "Backspace") {
            e.preventDefault();
            onClose(tabs[index].id);
        }
    }
</script>

<div
    bind:this={listRef}
    role="tablist"
    aria-label="Open files"
    onwheel={onWheel}
    class="h-full shrink-0 flex items-stretch overflow-x-auto no-select w-full"
>
    {#each tabs as tab, index (tab.id)}
        {@const isActive = tab.id === activeId}
        {@const isDropTarget = overIndex === index && dragIndex !== null && dragIndex !== index}
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div
            use:registerTab={tab.id}
            role="tab"
            aria-selected={isActive}
            tabindex={isActive ? 0 : -1}
            title={tab.name}
            draggable={!!onReorder}
            onkeydown={(e) => onKeyDown(e, index)}
            ondragstart={(e) => {
                dragIndex = index;
                if (e.dataTransfer) {
                    e.dataTransfer.effectAllowed = "move";
                    e.dataTransfer.setData("text/plain", tab.id);
                }
            }}
            ondragover={(e) => {
                if (dragIndex === null) return;
                e.preventDefault();
                if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
                if (overIndex !== index) overIndex = index;
            }}
            ondrop={(e) => {
                e.preventDefault();
                if (dragIndex !== null && dragIndex !== index) onReorder?.(dragIndex, index);
                dragIndex = null;
                overIndex = null;
            }}
            ondragend={() => { dragIndex = null; overIndex = null; }}
            onmousedown={(e) => {
                if (e.button === 1) { e.preventDefault(); onClose(tab.id); }
                else if (e.button === 0) onSelect(tab.id);
            }}
            oncontextmenu={(e) => {
                if (!onContextMenu) return;
                e.preventDefault();
                onContextMenu(tab.id, e.clientX, e.clientY);
            }}
            class={`group/tab relative flex items-center gap-[4px] pl-[8px] pr-[4px] shrink-0 min-w-[80px] max-w-[150px] cursor-pointer border-r border-[var(--border)] transition-colors outline-none ${isActive ? "bg-[var(--bg-primary)] text-[var(--text-primary)]" : "text-[var(--text-secondary)] hover:bg-[var(--bg-hover)]"} ${isDropTarget ? "ring-1 ring-inset ring-[var(--accent)]" : ""}`}
        >
            {#if isActive}
                <span class="absolute left-0 top-0 h-[2px] w-full bg-[var(--accent)]" aria-hidden="true"></span>
            {/if}
            <span class="material-symbols-outlined text-[8px] shrink-0 opacity-70">description</span>
            <span class="truncate text-[10px] font-mono">{tab.label}</span>
            <button
                onmousedown={(e) => e.stopPropagation()}
                onclick={(e) => { e.stopPropagation(); onClose(tab.id); }}
                tabindex="-1"
                aria-label={`Close ${tab.name}`}
                title={tab.dirty ? "Unsaved changes — click to close" : "Close"}
                class="shrink-0 w-[14px] h-[14px] flex items-center justify-center rounded-[2px] hover:bg-[var(--danger)] text-[var(--text-muted)] hover:text-white ml-auto transition-colors"
            >
                {#if tab.dirty}
                    <span class="w-[4px] h-[4px] rounded-full bg-[var(--status-unsaved)] group-hover/tab:hidden" aria-hidden="true"></span>
                {/if}
                <span class={`material-symbols-outlined text-[8px] leading-none ${tab.dirty ? "hidden group-hover/tab:inline" : "opacity-0 group-hover/tab:opacity-100"}`} aria-hidden="true">close</span>
            </button>
        </div>
    {/each}
    <button
        onclick={onNewTab}
        aria-label="New tab"
        title="New tab (Ctrl+N)"
        class="shrink-0 flex items-center justify-center w-[24px] text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-colors"
    >
        <span class="material-symbols-outlined text-[9px]">add</span>
    </button>
</div>

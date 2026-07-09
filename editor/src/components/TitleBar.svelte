<script lang="ts">
    import { Window } from "@tauri-apps/api/window";
    import SettingsMenu from "./SettingsMenu.svelte";
    import ExportMenu from "./ExportMenu.svelte";

    interface Props {
        fileName?: string;
        isDirty?: boolean;
        filePath?: string;
        onOpenFile?: () => void;
        onNewFile?: () => void;
        getExportHtml?: () => string;
        onExportSuccess?: (format: string) => void;
        onExportError?: (format: string) => void;
        isFullscreen?: boolean;
        onToggleFullscreen?: () => void;
        children?: import('svelte').Snippet;
    }

    let {
        fileName,
        isDirty = false,
        filePath,
        onOpenFile,
        onNewFile,
        getExportHtml,
        onExportSuccess,
        onExportError,
        isFullscreen = false,
        onToggleFullscreen,
        children
    }: Props = $props();

    async function handleMinimize() {
        try {
            const appWindow = Window.getCurrent();
            await appWindow.minimize();
        } catch (e) {
            console.error("Minimize failed:", e);
        }
    }

    async function handleMaximize() {
        if (isFullscreen) {
            onToggleFullscreen?.();
            return;
        }
        try {
            const appWindow = Window.getCurrent();
            await appWindow.toggleMaximize();
        } catch (e) {
            console.error("Maximize failed:", e);
        }
    }

    async function handleTitleBarMouseDown(event: MouseEvent) {
        const target = event.target as HTMLElement;

        if (
            event.button !== 0 ||
            (target && target.closest("button, a, input, textarea, select, [role='button'], [role='menu'], [role='menuitem']"))
        ) {
            return;
        }

        try {
            const appWindow = Window.getCurrent();
            if (event.detail === 2) {
                if (isFullscreen) onToggleFullscreen?.();
                else await appWindow.toggleMaximize();
            } else {
                await appWindow.startDragging();
            }
        } catch (e) {
            console.error("Window drag failed:", e);
        }
    }

    async function handleCloseClick() {
        try {
            const appWindow = Window.getCurrent();
            await appWindow.close();
        } catch (e) {
            console.error("Close failed:", e);
        }
    }

    let parentFolder = $derived((() => {
        if (!filePath) return null;
        const parts = filePath.replace(/\\/g, "/").split("/");
        if (parts.length >= 2) return parts.slice(-2, -1)[0];
        return null;
    })());

    let hasFile = $derived(!!fileName);
</script>

<header
    role="none"
    onmousedown={handleTitleBarMouseDown}
    class="h-[24px] shrink-0 flex items-center justify-between pl-[6px] pr-[2px] bg-[var(--bg-titlebar)] border-b border-[var(--border)] no-select drag-region transition-colors"
>
    <!-- Left: Icon & Actions -->
    <div class="flex items-center gap-[4px] no-drag h-full shrink-0">
        <div class="flex items-center justify-center w-[14px] h-[14px] ml-[2px]">
            <img src="/icon.svg" alt="Paperling" class="w-full h-full opacity-80" />
        </div>

        <!-- Open File / New Button -->
        {#if onOpenFile}
            <div class="w-[1px] h-[14px] bg-[var(--border)] ml-1 mr-1"></div>
            {#if onNewFile}
                <button
                    onclick={onNewFile}
                    aria-label="New file"
                    class="flex items-center justify-center w-[22px] h-[20px] rounded-[3px] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
                    title="New File (Ctrl+N)"
                >
                    <span class="material-symbols-outlined text-[12px]">edit_note</span>
                </button>
            {/if}
            <button
                onclick={onOpenFile}
                aria-label="Open file"
                class="flex items-center justify-center w-[22px] h-[20px] rounded-[3px] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
                title="Open File (Ctrl+O)"
            >
                <span class="material-symbols-outlined text-[12px]">folder_open</span>
            </button>
            {#if hasFile}
                <ExportMenu {fileName} {getExportHtml} onSuccess={onExportSuccess} onError={onExportError} />
            {/if}
        {/if}
    </div>

    <!-- Center: Tabs -->
    <div class="flex-1 overflow-hidden h-full flex items-end ml-[6px] no-drag relative">
        <div class="absolute inset-0 drag-region" style="z-index: -1;"></div>
        {@render children?.()}
    </div>

    <!-- Right: Settings & Window Controls -->
    <div class="flex items-center gap-[1px] ml-[2px] no-drag h-full">
        <SettingsMenu />
        <div class="w-[1px] h-[14px] bg-[var(--border)] mx-1"></div>
        <button
            onclick={handleMinimize}
            aria-label="Minimize"
            class="flex items-center justify-center w-[26px] h-[22px] rounded-[3px] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
        >
            <span class="text-[12px] leading-none">—</span>
        </button>
        <button
            onclick={handleMaximize}
            aria-label={isFullscreen ? "Exit fullscreen" : "Maximize"}
            title={isFullscreen ? "Exit fullscreen (F11)" : "Maximize"}
            class="flex items-center justify-center w-[26px] h-[22px] rounded-[3px] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
        >
            <span class="text-[12px] leading-none">▢</span>
        </button>
        <button
            onclick={handleCloseClick}
            aria-label="Close"
            class="flex items-center justify-center w-[26px] h-[22px] rounded-[3px] hover:bg-[var(--danger)] hover:text-white text-[var(--text-secondary)] transition-colors"
        >
            <span class="text-[12px] leading-none">✕</span>
        </button>
    </div>
</header>

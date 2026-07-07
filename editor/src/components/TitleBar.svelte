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
        onToggleFullscreen
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
    class="h-12 shrink-0 flex items-center justify-between px-4 bg-[var(--bg-titlebar)] border-b border-[var(--border)] no-select drag-region transition-colors"
>
    <!-- Left: Icon & Title -->
    <div class="flex items-center gap-3 no-drag">
        <div class="flex items-center justify-center w-5 h-5">
            <img src="/icon.svg" alt="Paperling" class="w-full h-full" />
        </div>
        <div class="flex items-center gap-2 text-sm text-[var(--text-secondary)] min-w-0">
            {#if parentFolder}
                <span class="opacity-60 hidden md:inline">{parentFolder} /</span>
            {/if}
            <span class="text-[var(--text-primary)] font-semibold tracking-tight truncate max-w-[28vw]">
                {fileName || "Paperling"}
            </span>
            {#if !fileName}
                <span class="text-[var(--text-muted)] text-xs ml-1 hidden sm:inline">— drop a .md file or Ctrl+O</span>
            {/if}
            {#if isDirty}
                <span class="text-[var(--status-unsaved)] ml-1 italic text-xs">— Edited</span>
            {/if}
        </div>

        <!-- Open File / New Button -->
        {#if hasFile && onOpenFile}
            <div class="w-[1px] h-4 bg-[var(--border)] ml-2"></div>
            {#if onNewFile}
                <button
                    onclick={onNewFile}
                    aria-label="New file"
                    class="flex items-center gap-1 px-2 py-1 rounded-[var(--radius-md)] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors text-xs"
                    title="New File (Ctrl+N)"
                >
                    <span class="material-symbols-outlined text-[16px]">edit_note</span>
                    <span class="hidden sm:inline">New</span>
                </button>
            {/if}
            <button
                onclick={onOpenFile}
                aria-label="Open file"
                class="flex items-center gap-1 px-2 py-1 rounded-[var(--radius-md)] hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors text-xs"
                title="Open File (Ctrl+O)"
            >
                <span class="material-symbols-outlined text-[16px]">folder_open</span>
                <span class="hidden sm:inline">Open</span>
            </button>
            <ExportMenu {fileName} {getExportHtml} onSuccess={onExportSuccess} onError={onExportError} />
        {/if}
    </div>

    <!-- Right: Settings & Window Controls -->
    <div class="flex items-center gap-1 no-drag">
        <SettingsMenu />
        <div class="w-[1px] h-4 bg-[var(--border)] mx-1"></div>
        <button
            onclick={handleMinimize}
            aria-label="Minimize"
            class="flex items-center justify-center w-8 h-8 rounded-lg hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
        >
            <span class="material-symbols-outlined text-[18px]">remove</span>
        </button>
        <button
            onclick={handleMaximize}
            aria-label={isFullscreen ? "Exit fullscreen" : "Maximize"}
            title={isFullscreen ? "Exit fullscreen (F11)" : "Maximize"}
            class="flex items-center justify-center w-8 h-8 rounded-lg hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
        >
            <span class="material-symbols-outlined text-[16px]">{isFullscreen ? "fullscreen_exit" : "crop_square"}</span>
        </button>
        <button
            onclick={handleCloseClick}
            aria-label="Close"
            class="flex items-center justify-center w-8 h-8 rounded-lg hover:bg-[var(--danger)] text-[var(--text-secondary)] hover:text-[var(--accent-text)] transition-colors"
        >
            <span class="material-symbols-outlined text-[18px]">close</span>
        </button>
    </div>
</header>

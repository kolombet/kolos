<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { attachFocusTrap } from "../utils/focusTrap";
    import mascotCarry from "../assets/mascot/mascot-carry.png";
    import mascotShrug from "../assets/mascot/mascot-shrug.png";

    interface FileEntry {
        name: string;
        path: string;
    }

    interface Props {
        isOpen: boolean;
        currentFilePath: string | null;
        onFileSelect: (path: string) => void;
        onClose: () => void;
    }

    let { isOpen, currentFilePath, onFileSelect, onClose }: Props = $props();

    let files = $state<FileEntry[]>([]);
    let isLoading = $state(false);
    let error = $state<string | null>(null);
    let panelRef: HTMLElement | null = $state(null);

    // Derived directory
    let directory = $derived((() => {
        if (!currentFilePath) return null;
        const normalized = currentFilePath.replace(/\\/g, "/");
        const lastSlash = normalized.lastIndexOf("/");
        return lastSlash > 0 ? currentFilePath.substring(0, lastSlash) : null;
    })());

    let directoryName = $derived(directory ? directory.replace(/\\/g, "/").split("/").pop() : "Files");

    async function loadFiles(dir: string) {
        isLoading = true;
        error = null;
        try {
            const entries = await invoke<FileEntry[]>("list_directory_files", { directory: dir });
            files = entries;
        } catch (err) {
            console.error("Failed to load directory:", err);
            error = "Failed to load files";
        } finally {
            isLoading = false;
        }
    }

    // Effect: Reload files when directory or isOpen changes
    $effect(() => {
        if (isOpen && directory) {
            loadFiles(directory);
        }
    });

    onMount(() => {
        const onFocus = () => {
            if (isOpen && directory) loadFiles(directory);
        };
        window.addEventListener("focus", onFocus);
        return () => window.removeEventListener("focus", onFocus);
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

    function handleFileClick(path: string) {
        onFileSelect(path);
        onClose();
    }
</script>

<aside
    bind:this={panelRef}
    role="navigation"
    aria-label="File explorer"
    tabindex="-1"
    class={`fixed left-0 top-12 bottom-7 w-72 bg-[var(--bg-secondary)] border-r border-[var(--border)] z-50 shadow-2xl flex flex-col overflow-hidden transition-transform duration-200 ease-out ${isOpen ? "translate-x-0" : "-translate-x-full"}`}
>
    <!-- Header -->
    <div class="h-10 shrink-0 px-4 flex items-center justify-between border-b border-[var(--border)] bg-[var(--bg-titlebar)]">
        <div class="flex items-center gap-2 text-sm font-semibold text-[var(--text-primary)] no-select">
            <span class="material-symbols-outlined text-[19px]!">folder_open</span>
            <span class="truncate max-w-[180px]">{directoryName}</span>
        </div>
        <div class="flex items-center gap-1">
            <button
                onclick={() => { if (directory) loadFiles(directory); }}
                aria-label="Refresh file list"
                title="Refresh"
                class="btn-press flex items-center justify-center w-7 h-7 rounded-lg hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            >
                <span class="material-symbols-outlined text-[19px]!">refresh</span>
            </button>
            <button
                onclick={onClose}
                aria-label="Close file explorer"
                class="btn-press flex items-center justify-center w-7 h-7 rounded-lg hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            >
                <span class="material-symbols-outlined text-[19px]!">close</span>
            </button>
        </div>
    </div>

    <!-- Content -->
    <div class="flex-1 min-h-0 overflow-y-auto">
        {#if isLoading}
            <div class="flex items-center justify-center h-32 text-[var(--text-secondary)] text-sm">
                Loading...
            </div>
        {:else if error}
            <div class="flex flex-col items-center justify-center gap-3 py-10 text-sm" role="alert">
                <img src={mascotShrug} alt="" aria-hidden="true" draggable="false" class="w-20 h-20 object-contain select-none opacity-90" />
                <span class="text-[var(--danger)]">{error}</span>
            </div>
        {:else if files.length === 0}
            <div class="flex flex-col items-center justify-center gap-3 py-10 text-sm text-[var(--text-secondary)]">
                <img src={mascotCarry} alt="" aria-hidden="true" draggable="false" class="w-20 h-20 object-contain select-none opacity-90" />
                <span>No markdown files here</span>
            </div>
        {:else}
            <ul class="py-2" role="listbox" aria-label="Markdown files">
                {#each files as file, index}
                    {@const isActive = file.path === currentFilePath}
                    <li class="stagger-item" style="animation-delay: {index * 0.03}s">
                        <button
                            onclick={() => handleFileClick(file.path)}
                            role="option"
                            aria-selected={isActive}
                            class={`btn-press w-full px-4 py-2 text-left text-sm flex items-center gap-2 transition-colors ${isActive
                                ? "bg-[var(--accent)] text-[var(--accent-text)]"
                                : "text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
                            }`}
                        >
                            <span class="material-symbols-outlined text-[18px]!">description</span>
                            <span class="truncate">{file.name}</span>
                        </button>
                    </li>
                {/each}
            </ul>
        {/if}
    </div>
</aside>

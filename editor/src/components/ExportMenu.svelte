<script lang="ts">
    import { themeState } from '../state/theme.svelte.ts';
    import iconExportPdf from '../assets/mascot/icon-export-pdf.png';
    import iconPaperPlane from '../assets/mascot/icon-paper-plane.png';

    type ExportModule = typeof import('../utils/exportUtils');
    let exportModulePromise: Promise<ExportModule> | null = null;
    const loadExportModule = (): Promise<ExportModule> => {
        if (!exportModulePromise) {
            exportModulePromise = import('../utils/exportUtils');
        }
        return exportModulePromise;
    };

    interface Props {
        fileName?: string;
        getExportHtml?: () => string;
        onSuccess?: (format: string) => void;
        onError?: (format: string) => void;
    }

    let { fileName = "Document", getExportHtml, onSuccess, onError }: Props = $props();

    let isOpen = $state(false);
    let isExporting = $state(false);
    let menuRef: HTMLDivElement | null = $state(null);

    let disabled = $derived(!getExportHtml);

    $effect(() => {
        if (!isOpen) return;
        
        const handleClickOutside = (e: MouseEvent) => {
            if (menuRef && !menuRef.contains(e.target as Node)) {
                isOpen = false;
            }
        };
        const handleKey = (e: KeyboardEvent) => {
            if (e.key === 'Escape') isOpen = false;
        };

        document.addEventListener('mousedown', handleClickOutside);
        document.addEventListener('keydown', handleKey);

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            document.removeEventListener('keydown', handleKey);
        };
    });

    async function handleExport(format: 'html' | 'pdf' | 'docx') {
        if (isExporting || !getExportHtml) return;

        const htmlContent = getExportHtml();
        if (!htmlContent) return;

        isExporting = true;
        isOpen = false;

        try {
            const mod = await loadExportModule();
            if (format === 'html') {
                if (await mod.exportToHTML(htmlContent, fileName, themeState.theme, themeState.font, themeState.fontSize)) {
                    onSuccess?.('HTML');
                }
            } else if (format === 'docx') {
                if (await mod.exportToDocx(htmlContent, fileName, themeState.theme, themeState.font, themeState.fontSize)) {
                    onSuccess?.('DOCX');
                }
            } else {
                const result = await mod.exportToPDF(htmlContent, fileName, themeState.theme, themeState.font, themeState.fontSize);
                if (result === 'saved') onSuccess?.('PDF');
            }
        } catch (error) {
            console.error(`Failed to export ${format}:`, error);
            onError?.(format.toUpperCase());
        } finally {
            isExporting = false;
        }
    }
</script>

<div bind:this={menuRef} class="relative no-drag">
    <!-- Export Button -->
    <button
        onclick={() => { if (!disabled) isOpen = !isOpen; }}
        disabled={disabled || isExporting}
        aria-label="Export document"
        aria-expanded={isOpen}
        aria-haspopup="true"
        class={`btn-press flex items-center gap-[4px] px-[6px] h-[20px] rounded-[3px] hover:bg-[var(--bg-hover)] transition-colors text-[10px] font-mono ${
            disabled
                ? 'cursor-not-allowed text-[var(--text-muted)]'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
        }`}
        title="Export document"
    >
        {#if isExporting}
            <span class="material-symbols-outlined text-[8px] animate-spin">progress_activity</span>
            <span class="hidden sm:inline">Exporting...</span>
        {:else}
            <span class="material-symbols-outlined text-[8px]">ios_share</span>
            <span class="hidden sm:inline">Export</span>
        {/if}
    </button>

    <!-- Dropdown Menu -->
    {#if isOpen && !disabled}
        <div role="menu" aria-label="Export formats" class="absolute left-0 top-full mt-1 w-40 bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg shadow-xl overflow-hidden z-[70] animate-fade-in-down">
            <button
                role="menuitem"
                onclick={() => handleExport('html')}
                class="w-full flex items-center gap-2.5 px-3 py-2 text-sm text-left hover:bg-[var(--bg-hover)] transition-colors"
            >
                <img src={iconPaperPlane} alt="" aria-hidden="true" draggable="false" class="w-6 h-6 object-contain select-none" />
                <span>HTML</span>
            </button>
            <button
                role="menuitem"
                onclick={() => handleExport('pdf')}
                class="w-full flex items-center gap-2.5 px-3 py-2 text-sm text-left hover:bg-[var(--bg-hover)] transition-colors"
            >
                <img src={iconExportPdf} alt="" aria-hidden="true" draggable="false" class="w-6 h-6 object-contain select-none" />
                <span>PDF</span>
            </button>
            <button
                role="menuitem"
                onclick={() => handleExport('docx')}
                class="w-full flex items-center gap-2.5 px-3 py-2 text-sm text-left hover:bg-[var(--bg-hover)] transition-colors"
            >
                <span class="material-symbols-outlined text-[15px] w-6 text-center text-[var(--accent)]" aria-hidden="true">description</span>
                <span>Word (.docx)</span>
            </button>
        </div>
    {/if}
</div>

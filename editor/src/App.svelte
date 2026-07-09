<script lang="ts">
    import { onMount } from 'svelte';
    import { themeState } from './state/theme.svelte.ts';
    import { tabsState } from './state/tabs.svelte.ts';
    import { settingsState } from './state/settings.svelte.ts';
    import { handleOpenFile, handleNewFile, handleSaveFile, loadFile } from './utils/fileOps';
    import TitleBar from './components/TitleBar.svelte';
    import TabBar from './components/TabBar.svelte';
    import StatusBar from './components/StatusBar.svelte';
    import CodeEditor from './components/CodeEditor.svelte';
    import MarkdownPreview from './components/MarkdownPreview.svelte';
    import FileExplorer from './components/FileExplorer.svelte';
    import TableOfContents from './components/TableOfContents.svelte';
    import CommandPalette from './components/CommandPalette.svelte';
    import SettingsModal from './components/SettingsModal.svelte';
    import StatsDialog from './components/StatsDialog.svelte';
    import ShortcutCheatsheet from './components/ShortcutCheatsheet.svelte';
    import Toast from './components/Toast.svelte';
    import { toastState } from './state/toast.svelte';

    onMount(() => {
        const cleanup = themeState.initListener();

        // Keyboard shortcuts
        const handleKeyDown = (e: KeyboardEvent) => {
            const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
            const modifier = isMac ? e.metaKey : e.ctrlKey;

            if (modifier && e.key.toLowerCase() === 's') {
                e.preventDefault();
                if (activeTab) {
                    handleSaveFile(activeTab.id);
                }
            } else if (modifier && e.key.toLowerCase() === 'o') {
                e.preventDefault();
                handleOpenFile();
            } else if (modifier && e.key.toLowerCase() === 'n') {
                e.preventDefault();
                handleNewFile();
            } else if (modifier && e.key.toLowerCase() === 'w') {
                e.preventDefault();
                if (activeTab) {
                    tabsState.closeTab(activeTab.id);
                }
            } else if (modifier && (e.key.toLowerCase() === 'k' || e.key.toLowerCase() === 'p')) {
                e.preventDefault();
                showCommandPalette = true;
            }
        };

        const handleOpenSettings = () => showSettings = true;
        const handleReplayTour = () => console.log("Replay tour not implemented yet");

        window.addEventListener('keydown', handleKeyDown);
        window.addEventListener('paperling:open-settings', handleOpenSettings);
        window.addEventListener('paperling:replay-tour', handleReplayTour);

        return () => {
            if (cleanup) cleanup();
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('paperling:open-settings', handleOpenSettings);
            window.removeEventListener('paperling:replay-tour', handleReplayTour);
        };
    });

    let activeTab = $derived(tabsState.activeTab);
    let isDirty = $derived(activeTab ? activeTab.content !== activeTab.originalContent : false);

    let tabItems = $derived(tabsState.tabs.map(t => ({
        id: t.id,
        name: t.fileName,
        label: tabsState.labels.get(t.id) || t.fileName,
        dirty: t.content !== t.originalContent
    })));

    let showFileExplorer = $state(false);
    let showTOC = $state(false);
    let showCommandPalette = $state(false);
    let showSettings = $state(false);
    let showStats = $state(false);
    let showCheatsheet = $state(false);

    let paletteItems = $derived((() => {
        const items: any[] = [];
        
        items.push({ id: "file.new", label: "New file", hint: "Ctrl+N", section: "File", icon: "edit_note", run: handleNewFile });
        items.push({ id: "file.open", label: "Open file…", hint: "Ctrl+O", section: "File", icon: "folder_open", run: handleOpenFile });
        
        if (activeTab) {
            items.push({ id: "file.save", label: "Save", hint: "Ctrl+S", section: "File", icon: "save", run: () => handleSaveFile(activeTab!.id) });
            items.push({ id: "tab.close", label: "Close tab", hint: "Ctrl+W", section: "File", icon: "tab_close", run: () => tabsState.closeTab(activeTab!.id) });
            items.push({ id: "doc.stats", label: "Show document statistics", section: "File", icon: "analytics", run: () => showStats = true });
            
            items.push({ id: "view.preview", label: "Switch to Reader mode", section: "View", icon: "visibility", run: () => settingsState.viewMode = 'preview' });
            items.push({ id: "view.code", label: "Switch to Code editor", section: "View", icon: "code", run: () => settingsState.viewMode = 'code' });
            items.push({ id: "view.split", label: "Toggle Split view", section: "View", icon: "vertical_split", run: () => settingsState.viewMode = 'split' });
            items.push({ id: "view.explorer", label: "Toggle file explorer", section: "View", icon: "folder", run: () => showFileExplorer = !showFileExplorer });
            items.push({ id: "view.toc", label: "Toggle outline", section: "View", icon: "format_list_bulleted", run: () => showTOC = !showTOC });
            
            items.push({ id: "settings.open", label: "Open Settings…", hint: "Ctrl+,", section: "Toggles", icon: "settings", run: () => showSettings = true });
            items.push({ id: "help.cheatsheet", label: "Show keyboard shortcuts", hint: "?", section: "Help", icon: "keyboard", run: () => showCheatsheet = true });
            
            // Add headings from current tab
            const content = activeTab.content;
            const lines = content.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
            lines.forEach((line, idx) => {
                const m = line.match(/^(#{1,6})\s+(.+)$/);
                if (m) {
                    const level = m[1].length;
                    items.push({
                        id: `head.${idx}`,
                        label: m[2].trim(),
                        hint: `H${level}`,
                        section: "Headings",
                        icon: level === 1 ? "title" : level === 2 ? "format_h2" : "format_h3",
                        run: () => window.dispatchEvent(new CustomEvent("paperling:goto-line", { detail: { line: idx + 1 } }))
                    });
                }
            });
        }
        
        return items;
    })());

    function handleEditorChange(newContent: string) {
        if (activeTab) {
            tabsState.updateTab(activeTab.id, { content: newContent });
        }
    }

    const getExportHtml = () => {
        const el = document.getElementById('markdown-preview-container');
        return el ? el.innerHTML : "";
    };

    const handleExportSuccess = (format: string) => toastState.show(`Exported to ${format} successfully`, "success");
    const handleExportError = (format: string) => toastState.show(`Failed to export to ${format}`, "error");
</script>

<div class="h-screen w-screen flex flex-col overflow-hidden bg-[var(--bg-primary)] text-[var(--text-primary)] font-sans">
    <TitleBar 
        fileName={activeTab?.fileName}
        isDirty={isDirty}
        filePath={activeTab?.filePath || undefined}
        onOpenFile={handleOpenFile}
        onNewFile={handleNewFile}
        {getExportHtml}
        onExportSuccess={handleExportSuccess}
        onExportError={handleExportError}
    >
        {#if tabsState.tabs.length > 0}
            <TabBar
                tabs={tabItems}
                activeId={tabsState.activeTabId}
                onSelect={(id) => tabsState.setActiveTab(id)}
                onClose={(id) => tabsState.closeTab(id)}
                onNewTab={handleNewFile}
                onReorder={(from, to) => tabsState.reorderTabs(from, to)}
            />
        {/if}
    </TitleBar>

    <main class="flex-1 relative min-h-0 flex flex-row">
        <FileExplorer 
            isOpen={showFileExplorer}
            currentFilePath={activeTab?.filePath || null}
            onFileSelect={loadFile}
            onClose={() => showFileExplorer = false}
        />
        <TableOfContents 
            isOpen={showTOC}
            content={activeTab?.content || ''}
            onClose={() => showTOC = false}
        />

        {#if activeTab}
            {#if settingsState.viewMode === 'code' || settingsState.viewMode === 'split'}
                <div class="h-full flex-shrink-0" style={`width: ${settingsState.viewMode === 'split' ? settingsState.splitRatio * 100 + '%' : '100%'}`}>
                    <CodeEditor 
                        content={activeTab.content} 
                        onChange={handleEditorChange} 
                    />
                </div>
            {/if}

            {#if settingsState.viewMode === 'preview' || settingsState.viewMode === 'split'}
                <div class="h-full flex-1 min-w-0">
                    <MarkdownPreview content={activeTab.content} />
                </div>
            {/if}
        {:else}
            <div class="w-full h-full flex flex-col items-center justify-center text-[var(--text-muted)] text-sm">
                <span class="material-symbols-outlined text-6xl! opacity-20 mb-4">description</span>
                <p>Press <kbd class="font-mono bg-[var(--bg-hover)] px-1 rounded">Ctrl+N</kbd> or <kbd class="font-mono bg-[var(--bg-hover)] px-1 rounded">⌘+N</kbd> to open a new tab.</p>
                <button 
                    class="mt-4 px-4 py-2 bg-[var(--accent)] text-[var(--accent-text)] rounded hover:opacity-90"
                    onclick={() => {
                        handleNewFile();
                        settingsState.viewMode = 'split';
                    }}
                >
                    Create New File
                </button>
            </div>
        {/if}
    </main>

    <StatusBar 
        isSaved={!isDirty}
        mode={settingsState.viewMode}
        wordCount={activeTab?.content.split(/\s+/).filter(w => w.length > 0).length || 0}
        showFileExplorer={showFileExplorer}
        showTOC={showTOC}
        onToggleFileExplorer={() => showFileExplorer = !showFileExplorer}
        onToggleTOC={() => showTOC = !showTOC}
    />

    <CommandPalette 
        isOpen={showCommandPalette}
        items={paletteItems}
        onClose={() => showCommandPalette = false}
    />
    
    <SettingsModal 
        isOpen={showSettings} 
        onClose={() => showSettings = false} 
    />
    
    <StatsDialog 
        isOpen={showStats} 
        content={activeTab?.content ?? ""}
        onClose={() => showStats = false} 
    />
    
    <ShortcutCheatsheet 
        isOpen={showCheatsheet} 
        onClose={() => showCheatsheet = false} 
    />

    <Toast />
</div>

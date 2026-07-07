import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { tabsState } from '../state/tabs.svelte.ts';
import { toastState } from '../state/toast.svelte.ts';

interface FileData {
    path: string;
    name: string;
    content: string;
    size: number;
    line_count: number;
    modified: number;
}

export async function loadFile(path: string) {
    // If already open, just switch to it
    const existing = tabsState.findByPath(path);
    if (existing) {
        tabsState.setActiveTab(existing.id);
        return;
    }

    try {
        const fileData = await invoke<FileData>('read_file', { path });
        const id = crypto.randomUUID();
        tabsState.addTab({
            id,
            filePath: fileData.path,
            fileName: fileData.name,
            content: fileData.content,
            originalContent: fileData.content,
            fileSize: fileData.size,
            knownMtime: fileData.modified ?? 0
        });
    } catch (err) {
        console.error("Failed to load file:", err);
        toastState.show("Failed to open file", "error");
    }
}

export async function handleOpenFile() {
    try {
        const selected = await open({
            multiple: true,
            filters: [{ name: "Markdown & text", extensions: ["md", "markdown", "txt", "text"] }]
        });
        if (typeof selected === 'string') {
            await loadFile(selected);
        } else if (Array.isArray(selected)) {
            for (const p of selected) await loadFile(p);
        }
    } catch (err) {
        console.error("Failed to open file:", err);
    }
}

export async function handleSaveAs(tabId: string) {
    const tab = tabsState.tabs.find(t => t.id === tabId);
    if (!tab) return;
    try {
        const selected = await save({
            filters: [{ name: "Markdown", extensions: ["md"] }],
            defaultPath: tab.fileName || undefined
        });
        if (!selected) return;
        const mtime = await invoke<number>('save_file', { path: selected, content: tab.content });
        const name = selected.replace(/\\/g, "/").split("/").pop() || "Untitled";
        tabsState.updateTab(tabId, {
            filePath: selected,
            fileName: name,
            originalContent: tab.content,
            knownMtime: mtime
        });
    } catch (err) {
        console.error("Failed to save as:", err);
        toastState.show("Failed to save file", "error");
    }
}

export async function handleSaveFile(tabId: string) {
    const tab = tabsState.tabs.find(t => t.id === tabId);
    if (!tab) return;
    if (!tab.filePath) {
        return handleSaveAs(tabId);
    }
    try {
        const mtime = await invoke<number>('save_file', { path: tab.filePath, content: tab.content });
        tabsState.updateTab(tabId, {
            originalContent: tab.content,
            knownMtime: mtime
        });
    } catch (err) {
        console.error("Failed to save file:", err);
        toastState.show("Failed to save file", "error");
    }
}

export function handleNewFile() {
    const reusable = tabsState.getReusableUntitled();
    if (reusable) {
        tabsState.setActiveTab(reusable.id);
        return;
    }
    const id = crypto.randomUUID();
    tabsState.addTab({
        id,
        filePath: null,
        fileName: tabsState.getNextUntitledName(),
        content: '',
        originalContent: '',
        fileSize: 0,
        knownMtime: Date.now()
    });
}

import { TabState, findTabByPath, nextActiveAfterClose, nextUntitledName, findReusableUntitledTab, computeTabLabels, moveTab } from '../utils/tabsModel';
import { setSession } from '../utils/persistence';

class TabsState {
    #tabs = $state<TabState[]>([]);
    #activeTabId = $state<string | null>(null);
    #docSwapId = $state<number>(0);
    
    // Derived value for tab labels using Svelte 5 Runes
    #labels = $derived(computeTabLabels(this.#tabs));

    get tabs() { return this.#tabs; }
    get activeTabId() { return this.#activeTabId; }
    get activeTab() { return this.#tabs.find(t => t.id === this.#activeTabId) || null; }
    get docSwapId() { return this.#docSwapId; }
    get labels() { return this.#labels; }

    addTab(tab: TabState, activate = true) {
        this.#tabs = [...this.#tabs, tab]; // reassign for reactivity
        if (activate) {
            this.setActiveTab(tab.id);
        } else {
            this.#persistSession();
        }
    }

    setActiveTab(id: string | null) {
        if (this.#activeTabId !== id) {
            this.#activeTabId = id;
            this.bumpDocSwap();
            this.#persistSession();
        }
    }

    updateTab(id: string, updates: Partial<TabState>) {
        const idx = this.#tabs.findIndex(t => t.id === id);
        if (idx !== -1) {
            this.#tabs[idx] = { ...this.#tabs[idx], ...updates };
            if (updates.filePath !== undefined || updates.cursorLine !== undefined) {
                this.#persistSession();
            }
        }
    }

    closeTab(id: string) {
        const nextId = nextActiveAfterClose(this.#tabs, id);
        this.#tabs = this.#tabs.filter(t => t.id !== id);
        if (this.#activeTabId === id) {
            this.setActiveTab(nextId);
        } else {
            this.#persistSession();
        }
    }

    closeAll() {
        this.#tabs = [];
        this.#activeTabId = null;
        this.bumpDocSwap();
        this.#persistSession();
    }

    bumpDocSwap() {
        this.#docSwapId++;
    }

    reorderTabs(fromIndex: number, toIndex: number) {
        this.#tabs = moveTab(this.#tabs, fromIndex, toIndex);
        this.#persistSession();
    }

    getReusableUntitled() {
        return findReusableUntitledTab(this.#tabs);
    }

    getNextUntitledName() {
        return nextUntitledName(this.#tabs);
    }

    findByPath(path: string | null) {
        return findTabByPath(this.#tabs, path);
    }

    setTabs(tabs: TabState[], activeTabId: string | null) {
        this.#tabs = tabs;
        if (activeTabId) {
            this.setActiveTab(activeTabId);
        } else {
            this.setActiveTab(null);
        }
    }

    #persistSession() {
        if (typeof window === 'undefined') return;
        const sessionTabs = this.#tabs.filter(t => t.filePath !== null).map(t => ({
            path: t.filePath!,
            cursorLine: t.cursorLine
        }));
        
        let activePath = this.activeTab?.filePath;
        let persistedActiveIndex = sessionTabs.findIndex(t => t.path === activePath);

        setSession(sessionTabs.length > 0 ? {
            tabs: sessionTabs,
            activeIndex: persistedActiveIndex >= 0 ? persistedActiveIndex : 0
        } : null);
    }
}

export const tabsState = new TabsState();

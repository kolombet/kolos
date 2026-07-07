import {
    getAutoSave, setAutoSave,
    getSpellCheck, setSpellCheck,
    getWordWrap, setWordWrap,
    getToolbarEnabled, setToolbarEnabled,
    getTourDone, setTourDone,
    getSavedViewMode, setSavedViewMode,
    getSplitRatio, setSplitRatio,
    getVimMode, setVimMode
} from '../utils/persistence';

class SettingsState {
    #autoSave = $state(getAutoSave());
    #spellCheck = $state(getSpellCheck());
    #wordWrap = $state(getWordWrap());
    #toolbarEnabled = $state(getToolbarEnabled());
    #tourDone = $state(getTourDone());
    #viewMode = $state(getSavedViewMode());
    #splitRatio = $state(getSplitRatio());
    #vimMode = $state(getVimMode());

    get autoSave() { return this.#autoSave; }
    set autoSave(v) { this.#autoSave = v; setAutoSave(v); }

    get spellCheck() { return this.#spellCheck; }
    set spellCheck(v) { this.#spellCheck = v; setSpellCheck(v); }

    get wordWrap() { return this.#wordWrap; }
    set wordWrap(v) { this.#wordWrap = v; setWordWrap(v); }

    get toolbarEnabled() { return this.#toolbarEnabled; }
    set toolbarEnabled(v) { this.#toolbarEnabled = v; setToolbarEnabled(v); }

    get tourDone() { return this.#tourDone; }
    set tourDone(v) { this.#tourDone = v; setTourDone(v); }

    get viewMode() { return this.#viewMode; }
    set viewMode(v) { this.#viewMode = v; setSavedViewMode(v); }

    get splitRatio() { return this.#splitRatio; }
    set splitRatio(v) { this.#splitRatio = v; setSplitRatio(v); }

    get vimMode() { return this.#vimMode; }
    set vimMode(v) { this.#vimMode = v; setVimMode(v); }
}

export const settingsState = new SettingsState();

import { ensureFontLoaded } from '../fonts';

export type Theme = 'dark' | 'light' | 'paper' | 'dracula';
export type FontFamily = 'inter' | 'merriweather' | 'lora' | 'source-serif' | 'fira-sans';
export type FontSize = 'small' | 'medium' | 'large';

const THEME_STORAGE_KEY = 'paperling-theme';
const FONT_STORAGE_KEY = 'paperling-font';
const FONT_SIZE_STORAGE_KEY = 'paperling-font-size';

const VALID_THEMES: Theme[] = ['dark', 'light', 'paper', 'dracula'];
const VALID_FONTS: FontFamily[] = ['inter', 'merriweather', 'lora', 'source-serif', 'fira-sans'];
const VALID_FONT_SIZES: FontSize[] = ['small', 'medium', 'large'];

function getValidated<T extends string>(key: string, validValues: T[], fallback: T): T {
    const stored = localStorage.getItem(key);
    if (stored && validValues.includes(stored as T)) return stored as T;
    return fallback;
}

function prefersLight(): boolean {
    return typeof window !== 'undefined'
        && typeof window.matchMedia === 'function'
        && window.matchMedia('(prefers-color-scheme: light)').matches;
}

function getInitialTheme(): Theme {
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (stored && VALID_THEMES.includes(stored as Theme)) return stored as Theme;
    return prefersLight() ? 'light' : 'dark';
}

class ThemeState {
    #theme = $state<Theme>('dark');
    #font = $state<FontFamily>('inter');
    #fontSize = $state<FontSize>('medium');

    constructor() {
        if (typeof window !== 'undefined') {
            this.#theme = getInitialTheme();
            this.#font = getValidated(FONT_STORAGE_KEY, VALID_FONTS, 'inter');
            this.#fontSize = getValidated(FONT_SIZE_STORAGE_KEY, VALID_FONT_SIZES, 'medium');
        }
    }

    get theme() { return this.#theme; }
    set theme(v: Theme) {
        this.#theme = v;
        localStorage.setItem(THEME_STORAGE_KEY, v);
        document.documentElement.setAttribute('data-theme', v);
    }

    get font() { return this.#font; }
    set font(v: FontFamily) {
        this.#font = v;
        localStorage.setItem(FONT_STORAGE_KEY, v);
        document.documentElement.setAttribute('data-font', v);
        ensureFontLoaded(v);
    }

    get fontSize() { return this.#fontSize; }
    set fontSize(v: FontSize) {
        this.#fontSize = v;
        localStorage.setItem(FONT_SIZE_STORAGE_KEY, v);
        document.documentElement.setAttribute('data-font-size', v);
    }

    initListener() {
        if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return;
        
        document.documentElement.setAttribute('data-theme', this.#theme);
        document.documentElement.setAttribute('data-font', this.#font);
        document.documentElement.setAttribute('data-font-size', this.#fontSize);
        ensureFontLoaded(this.#font);

        const mq = window.matchMedia('(prefers-color-scheme: light)');
        const onChange = (e: MediaQueryListEvent) => {
            if (!localStorage.getItem(THEME_STORAGE_KEY)) {
                this.theme = e.matches ? 'light' : 'dark';
                localStorage.removeItem(THEME_STORAGE_KEY);
            }
        };
        mq.addEventListener('change', onChange);
        return () => mq.removeEventListener('change', onChange);
    }
}

export const themeState = new ThemeState();

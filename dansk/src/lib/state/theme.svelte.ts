export type Theme = "dark" | "light" | "paper" | "dracula";

const THEME_STORAGE_KEY = "dansk-theme";

const VALID_THEMES: Theme[] = ["dark", "light", "paper", "dracula"];

function prefersLight(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-color-scheme: light)").matches
  );
}

function getInitialTheme(): Theme {
  const stored = localStorage.getItem(THEME_STORAGE_KEY);
  if (stored && VALID_THEMES.includes(stored as Theme)) return stored as Theme;
  return prefersLight() ? "light" : "dark";
}

class ThemeState {
  #theme = $state<Theme>("dark");

  constructor() {
    if (typeof window !== "undefined") {
      this.#theme = getInitialTheme();
    }
  }

  get theme() {
    return this.#theme;
  }
  set theme(v: Theme) {
    this.#theme = v;
    localStorage.setItem(THEME_STORAGE_KEY, v);
    document.documentElement.setAttribute("data-theme", v);
  }

  initListener() {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") return;

    document.documentElement.setAttribute("data-theme", this.#theme);

    const mq = window.matchMedia("(prefers-color-scheme: light)");
    const onChange = (e: MediaQueryListEvent) => {
      if (!localStorage.getItem(THEME_STORAGE_KEY)) {
        this.theme = e.matches ? "light" : "dark";
        localStorage.removeItem(THEME_STORAGE_KEY);
      }
    };
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }
}

export const themeState = new ThemeState();

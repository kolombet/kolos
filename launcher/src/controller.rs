// Controller mode: a gamepad-driven grid of apps, split into tabs.
//
// Tab 0 is "Fav": pinned apps (in pin order, saved to
// $XDG_DATA_HOME/kolos-launcher/pinned, one name per line), then every app
// launched before, most-used first (the launch history kickoff already keeps).
// The other tabs are one per first letter of the app's name (only letters that
// have apps), each sorted most-used first, then alphabetically. LB/RB switch tabs.
//
// "Apps" are visible desktop entries (so games, Waydroid apps, etc. show up even
// though they aren't commands on PATH), plus anything already launched or pinned.
// Desktop entries are matched to PATH commands and history by their file stem
// (`org.mozilla.firefox.desktop` <-> the flatpak's `org.mozilla.firefox`
// command), and their `Name=` is what the tile shows.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use image::{Rgba, RgbaImage};
use xdg::BaseDirectories;

use crate::app::App;
use crate::color::Color;
use crate::gamepad::PadAction;
use crate::selection::Element;

const FAV: &str = "Fav";
const TERMINAL: &str = "foot -e";

fn pinned_path() -> Option<PathBuf> {
    BaseDirectories::with_prefix("kolos-launcher").place_data_file("pinned").ok()
}

pub fn load_pinned() -> Vec<String> {
    pinned_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .map(|s| s.lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect())
        .unwrap_or_default()
}

fn save_pinned(pinned: &[String]) {
    if let Some(path) = pinned_path() {
        let mut body = pinned.join("\n");
        body.push('\n');
        if let Err(e) = fs::write(path, body) {
            log::error!("saving pinned apps: {e}");
        }
    }
}

pub struct DesktopApp {
    pub id: String,   // file stem
    pub name: String, // Name=
    pub exec: String, // Exec= with field codes removed, ready for `sh -c`
    pub command: Option<String>, // basename of Exec's program
}

/// Exec= minus its %-field codes (%u, %F, ...), with %% unescaped.
fn clean_exec(exec: &str) -> String {
    let mut out = String::new();
    let mut chars = exec.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            if chars.next() == Some('%') {
                out.push('%');
            }
        } else {
            out.push(c);
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every visible (not NoDisplay/Hidden) desktop entry, higher-priority dirs winning.
pub fn desktop_apps() -> Vec<DesktopApp> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let xdg = BaseDirectories::new();
    if let Some(home) = xdg.get_data_home() {
        dirs.push(home);
    }
    dirs.extend(xdg.get_data_dirs());
    dirs.push("/var/lib/flatpak/exports/share".into());

    let mut seen = HashSet::new();
    let mut apps = Vec::new();
    for dir in &dirs {
        let Ok(entries) = fs::read_dir(dir.join("applications")) else { continue };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let Some(id) = path.file_stem().and_then(|s| s.to_str()).map(String::from) else { continue };
            if path.extension().is_none_or(|e| e != "desktop") || !seen.insert(id.clone()) {
                continue; // a same-named entry in an earlier dir overrides this one
            }
            let Ok(text) = fs::read_to_string(&path) else { continue };
            let (mut name, mut exec, mut terminal, mut hidden, mut in_main) = (None, None, false, false, false);
            for line in text.lines() {
                if line.starts_with('[') {
                    in_main = line.trim() == "[Desktop Entry]";
                    continue;
                }
                if !in_main {
                    continue;
                }
                match line.split_once('=').map(|(k, v)| (k.trim(), v.trim())) {
                    Some(("Name", v)) => name = name.or(Some(v.to_string())),
                    Some(("Exec", v)) => exec = exec.or(Some(v.to_string())),
                    Some(("Terminal", v)) => terminal = v == "true",
                    Some(("NoDisplay" | "Hidden", "true")) => hidden = true,
                    Some(("Type", v)) if v != "Application" => hidden = true,
                    _ => {}
                }
            }
            let (Some(name), Some(exec), false) = (name, exec, hidden) else { continue };
            let mut exec = clean_exec(&exec);
            let command = exec
                .split_whitespace()
                .next()
                .map(|p| p.trim_matches('"').rsplit('/').next().unwrap_or(p).to_string());
            if terminal {
                exec = format!("{TERMINAL} {exec}");
            }
            apps.push(DesktopApp { id, name, exec, command });
        }
    }
    apps
}

fn fill(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, c: &Color) {
    // Colors are premultiplied (see color.rs): out = src + dst * (1 - a).
    let inv = 255 - u32::from(c.3);
    for py in y..(y + h).min(img.height()) {
        for px in x..(x + w).min(img.width()) {
            let Rgba(d) = *img.get_pixel(px, py);
            let mix = |s: u8, d: u8| (u32::from(s) + u32::from(d) * inv / 255).min(255) as u8;
            img.put_pixel(px, py, Rgba([mix(c.0, d[0]), mix(c.1, d[1]), mix(c.2, d[2]), mix(c.3, d[3])]));
        }
    }
}

/// False for a history entry whose program is gone (e.g. an uninstalled
/// flatpak), so the grid doesn't offer something that can't start.
fn installed(command: &str) -> bool {
    let Some(program) = command.split_whitespace().next() else { return false };
    if program.contains('/') {
        return Path::new(program.trim_matches('"')).exists();
    }
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
}

fn tab_letter(label: &str) -> String {
    match label.chars().next() {
        Some(c) if c.is_alphabetic() => c.to_uppercase().collect(),
        _ => "#".into(),
    }
}

impl App {
    /// Adds desktop entries to `all_entries` and records their display names.
    pub fn load_desktop_apps(&mut self) {
        for app in desktop_apps() {
            // A PATH command or history entry with the same id is the same app:
            // keep its launch count (and command), just give it the nicer name.
            self.all_entries.ensure(&app.id, &app.exec);
            if let Some(cmd) = &app.command {
                self.desktop_names.entry(cmd.clone()).or_insert_with(|| app.name.clone());
            }
            self.apps.insert(app.id.clone());
            self.desktop_names.insert(app.id, app.name);
        }
    }

    pub fn label<'a>(&'a self, elem: &'a Element) -> &'a str {
        self.desktop_names
            .get(&elem.name)
            .or_else(|| self.desktop_names.get(&elem.value))
            .map_or(&elem.name, String::as_str)
    }

    /// Rebuilds the tab list and the current tab's items.
    pub fn build_tabs(&mut self) {
        let entries = self.all_entries.as_ref_vec();
        let mut seen_labels = HashSet::new();
        // Most-used first, so when two entries share a label (a PATH command and
        // its desktop entry) the one with history wins.
        let mut order: Vec<usize> = (0..entries.len()).collect();
        order.sort_by(|&a, &b| entries[b].base_score.cmp(&entries[a].base_score));
        let mut candidates = Vec::new();
        for i in order {
            let e = entries[i];
            let pinned = self.pinned.contains(&e.name);
            let is_app = self.apps.contains(&e.name);
            if !(is_app || pinned || e.base_score > 0) || !(is_app || installed(&e.value)) {
                continue;
            }
            if seen_labels.insert(self.label(e).to_lowercase()) {
                candidates.push(i);
            }
        }

        let current = self.tabs.get(self.tab).cloned().or_else(|| self.args.tab.clone());
        let mut letters: Vec<String> =
            candidates.iter().map(|&i| tab_letter(self.label(entries[i]))).collect::<HashSet<_>>().into_iter().collect();
        letters.sort_by(|a, b| (a == "#").cmp(&(b == "#")).then(a.cmp(b)));
        self.tabs = std::iter::once(FAV.to_string()).chain(letters).collect();
        self.candidates = candidates;
        self.tab = current.and_then(|c| self.tabs.iter().position(|t| *t == c)).unwrap_or(0);
        self.fill_tab();
    }

    fn fill_tab(&mut self) {
        let entries = self.all_entries.as_ref_vec();
        let tab = &self.tabs[self.tab];
        let items: Vec<usize> = if tab == FAV {
            let mut favs: Vec<usize> = self
                .pinned
                .iter()
                .filter_map(|name| self.candidates.iter().copied().find(|&i| &entries[i].name == name))
                .collect();
            let used: Vec<usize> =
                self.candidates.iter().copied().filter(|&i| entries[i].base_score > 0 && !favs.contains(&i)).collect();
            favs.extend(used);
            favs
        } else {
            let mut items: Vec<usize> =
                self.candidates.iter().copied().filter(|&i| tab_letter(self.label(entries[i])) == *tab).collect();
            items.sort_by(|&a, &b| {
                entries[b]
                    .base_score
                    .cmp(&entries[a].base_score)
                    .then_with(|| self.label(entries[a]).to_lowercase().cmp(&self.label(entries[b]).to_lowercase()))
            });
            items
        };
        self.favorites = items;
        self.grid_index = self.grid_index.min(self.favorites.len().saturating_sub(1));
    }

    fn switch_tab(&mut self, step: isize) {
        let n = self.tabs.len() as isize;
        self.tab = (self.tab as isize + step).rem_euclid(n) as usize;
        self.grid_index = 0;
        self.grid_scroll = 0;
        self.fill_tab();
    }

    fn selected_element(&self) -> Option<Element> {
        let entries = self.all_entries.as_ref_vec();
        let index = if self.grid {
            *self.favorites.get(self.grid_index)?
        } else if self.select_input {
            return None;
        } else {
            *self.last_search_result.get(self.select_index)?
        };
        entries.get(index).map(|e| (*e).clone())
    }

    fn toggle_pin(&mut self) {
        let Some(elem) = self.selected_element() else { return };
        if let Some(pos) = self.pinned.iter().position(|n| n == &elem.name) {
            self.pinned.remove(pos);
        } else {
            self.pinned.push(elem.name);
        }
        save_pinned(&self.pinned);
        let keep = self.favorites.get(self.grid_index).copied();
        self.build_tabs();
        // Keep the cursor on the same app if it's still in this tab.
        if let Some(pos) = keep.and_then(|k| self.favorites.iter().position(|&f| f == k)) {
            self.grid_index = pos;
        }
    }

    /// Applies one gamepad action. Returns true when the launcher should close.
    pub fn pad(&mut self, action: PadAction) -> bool {
        let cols = self.grid_columns.max(1);
        let len = self.favorites.len();
        match action {
            PadAction::Exit | PadAction::Back => return true,
            PadAction::Confirm => {
                if self.grid {
                    if let Some(elem) = self.selected_element() {
                        self.run(&elem);
                        return true;
                    }
                } else if !self.last_search_result.is_empty() || self.select_input {
                    self.execute();
                    return true;
                }
            }
            PadAction::Switch => {
                self.grid = !self.grid;
                if !self.grid {
                    self.query.clear();
                    self.search();
                }
            }
            PadAction::Pin => self.toggle_pin(),
            _ if !self.grid => match action {
                PadAction::Up => self.nav_up(1),
                PadAction::Down => self.nav_down(1),
                PadAction::PageUp | PadAction::Left => self.nav_up(self.page_size.max(1)),
                PadAction::PageDown | PadAction::Right => {
                    let room = self.last_search_result.len().saturating_sub(1 + self.select_index);
                    self.nav_down(self.page_size.max(1).min(room).max(1));
                }
                _ => {}
            },
            PadAction::PageUp => self.switch_tab(-1),
            PadAction::PageDown => self.switch_tab(1),
            PadAction::Left if self.grid_index > 0 => self.grid_index -= 1,
            PadAction::Right if self.grid_index + 1 < len => self.grid_index += 1,
            PadAction::Up => self.grid_index = self.grid_index.saturating_sub(cols),
            PadAction::Down => {
                if self.grid_index + cols < len {
                    self.grid_index += cols;
                } else if self.grid_index / cols < len.saturating_sub(1) / cols {
                    self.grid_index = len - 1; // partial last row
                }
            }
            _ => {}
        }
        false
    }

    fn draw_tab_strip(&self, img: &mut RgbaImage, width: u32, padding: u32, font_size: f32) {
        let colors = &self.config.colors;
        // Noto Sans Mono: every glyph is ~0.6em wide.
        let char_w = font_size * 0.6;
        let pad_x = (font_size * 0.35) as u32;
        let widths: Vec<u32> =
            self.tabs.iter().map(|t| (t.chars().count() as f32 * char_w) as u32 + 2 * pad_x).collect();
        let avail = width.saturating_sub(2 * padding);
        // Scroll the strip so the current tab stays on screen, roughly centred.
        let mut start = 0;
        let total: u32 = widths.iter().sum();
        while total > avail && start < self.tab {
            let before: u32 = widths[start..self.tab].iter().sum();
            if before + widths[self.tab] <= avail / 2 + widths[self.tab] {
                break;
            }
            start += 1;
        }
        let tab_h = (font_size * 1.4) as u32;
        let mut x = padding;
        for (i, tab) in self.tabs.iter().enumerate().skip(start) {
            if x + widths[i] > padding + avail {
                break;
            }
            let selected = i == self.tab;
            if selected {
                fill(img, x, padding, widths[i], tab_h, &colors.text_selected);
            }
            let color = if selected { Color(14, 15, 18, 255) } else { colors.prompt.clone() };
            self.font.render(tab, &color, img, x + pad_x, padding + (font_size * 0.1) as u32, None);
            x += widths[i];
        }
    }

    pub fn draw_grid(&mut self, img: &mut RgbaImage, width: u32, height: u32, scale: i32) {
        let padding = self.config.padding * scale as u32 / 2;
        let font_size = self.config.font_size * scale as f32;
        let colors = self.config.colors.clone();

        // The list view's translucent background is too busy behind big tiles.
        fill(img, 0, 0, width, height, &Color(14, 15, 18, 215));
        self.draw_tab_strip(img, width, padding, font_size);
        let footer = padding + font_size as u32;
        self.font.render(
            "A open  B close  X pin  LB/RB tab  Y search",
            &colors.prompt,
            img,
            padding,
            height.saturating_sub(footer),
            Some(width.saturating_sub(2 * padding) as usize),
        );

        let top = padding + (font_size * 2.2) as u32;
        if self.favorites.is_empty() {
            self.font.render("Nothing here yet. RB: browse apps, X to pin one.", &colors.text, img, padding, top, None);
            return;
        }

        let avail_w = width.saturating_sub(2 * padding);
        let gap = (font_size * 0.5) as u32;
        let min_tile = (font_size * 9.) as u32;
        let cols = ((avail_w + gap) / (min_tile + gap)).clamp(1, 6) as usize;
        self.grid_columns = cols;
        let tile_w = (avail_w + gap) / cols as u32 - gap;
        let tile_h = (font_size * 3.) as u32;
        let bottom = footer + (font_size * 0.8) as u32;
        let rows_visible = (((height.saturating_sub(top + bottom)) + gap) / (tile_h + gap)).max(1) as usize;

        let sel_row = self.grid_index / cols;
        if sel_row < self.grid_scroll {
            self.grid_scroll = sel_row;
        } else if sel_row >= self.grid_scroll + rows_visible {
            self.grid_scroll = sel_row + 1 - rows_visible;
        }

        let tile_bg = Color(28, 28, 32, 60);
        let sel_bg = Color(colors.text_selected.0 / 3, colors.text_selected.1 / 3, colors.text_selected.2 / 3, 110);
        let marker = (font_size * 0.35) as u32;
        let entries = self.all_entries.as_ref_vec();
        for (n, &index) in self.favorites.iter().enumerate().skip(self.grid_scroll * cols) {
            let (row, col) = (n / cols - self.grid_scroll, n % cols);
            if row >= rows_visible {
                break;
            }
            let x = padding + col as u32 * (tile_w + gap);
            let y = top + row as u32 * (tile_h + gap);
            let selected = n == self.grid_index;
            fill(img, x, y, tile_w, tile_h, if selected { &sel_bg } else { &tile_bg });
            if selected {
                let edge = (2 * scale) as u32;
                let c = &colors.text_selected;
                fill(img, x, y, tile_w, edge, c);
                fill(img, x, y + tile_h - edge, tile_w, edge, c);
                fill(img, x, y, edge, tile_h, c);
                fill(img, x + tile_w - edge, y, edge, tile_h, c);
            }
            let elem = entries[index];
            if self.pinned.contains(&elem.name) {
                fill(img, x + tile_w - 2 * marker, y + marker, marker, marker, &colors.text_query);
            }
            let text_color = if selected { &colors.text_selected } else { &colors.text };
            let inner = (font_size * 0.6) as u32;
            self.font.render(
                self.label(elem),
                text_color,
                img,
                x + inner,
                // Glyph boxes sit below their y; nudge up to look centered.
                y + ((tile_h as f32 - font_size * 1.3) / 2.) as u32,
                Some((tile_w - 2 * inner - 2 * marker) as usize),
            );
        }
        // More rows below: a thin bar under the last visible row.
        let total_rows = self.favorites.len().div_ceil(cols);
        if self.grid_scroll + rows_visible < total_rows {
            let y = top + rows_visible as u32 * (tile_h + gap) - gap / 2;
            fill(img, padding, y, avail_w, (2 * scale) as u32, &colors.prompt);
        }
    }
}

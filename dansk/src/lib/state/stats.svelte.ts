import { invoke } from "@tauri-apps/api/core";
import type { Stats } from "../types";

class StatsState {
  #stats = $state<Stats | null>(null);

  get stats() {
    return this.#stats;
  }

  async refresh() {
    this.#stats = await invoke<Stats>("get_stats");
  }
}

export const statsState = new StatsState();

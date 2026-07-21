import { invoke } from "@tauri-apps/api/core";
import type { Word, WordInput, WordWithCard } from "../types";

class WordsState {
  #words = $state<WordWithCard[]>([]);
  #loading = $state(false);

  get words() {
    return this.#words;
  }

  get loading() {
    return this.#loading;
  }

  async refresh() {
    this.#loading = true;
    try {
      this.#words = await invoke<WordWithCard[]>("list_words");
    } finally {
      this.#loading = false;
    }
  }

  async add(input: WordInput): Promise<Word> {
    const word = await invoke<Word>("add_word", { ...input });
    await this.refresh();
    return word;
  }

  async update(id: number, input: WordInput): Promise<Word> {
    const word = await invoke<Word>("update_word", { id, ...input });
    await this.refresh();
    return word;
  }

  async remove(id: number) {
    await invoke("delete_word", { id });
    await this.refresh();
  }
}

export const wordsState = new WordsState();

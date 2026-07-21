import { invoke } from "@tauri-apps/api/core";
import type { IntervalPreview, Rating, ReviewCard } from "../types";

const NEW_CARDS_PER_SESSION = 20;

class ReviewState {
  #queue = $state<ReviewCard[]>([]);
  #index = $state(0);
  #flipped = $state(false);
  #preview = $state<IntervalPreview | null>(null);
  #loading = $state(false);

  get current(): ReviewCard | null {
    return this.#queue[this.#index] ?? null;
  }

  get remaining(): number {
    return Math.max(this.#queue.length - this.#index, 0);
  }

  get flipped() {
    return this.#flipped;
  }

  get preview() {
    return this.#preview;
  }

  get loading() {
    return this.#loading;
  }

  get done(): boolean {
    return !this.#loading && this.current === null;
  }

  async loadQueue() {
    this.#loading = true;
    try {
      this.#queue = await invoke<ReviewCard[]>("get_due_queue", { newLimit: NEW_CARDS_PER_SESSION });
      this.#index = 0;
      this.#flipped = false;
      await this.#loadPreview();
    } finally {
      this.#loading = false;
    }
  }

  async #loadPreview() {
    const card = this.current;
    this.#preview = card ? await invoke<IntervalPreview>("preview_review", { wordId: card.id }) : null;
  }

  flip() {
    this.#flipped = true;
  }

  async rate(rating: Rating) {
    const card = this.current;
    if (!card) return;
    await invoke("submit_review", { wordId: card.id, rating });
    this.#index += 1;
    this.#flipped = false;
    await this.#loadPreview();
  }
}

export const reviewState = new ReviewState();

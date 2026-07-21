export type CardState = "new" | "learning" | "review" | "relearning";

export type Rating = 1 | 2 | 3 | 4; // Again, Hard, Good, Easy — matches rs-fsrs::Rating

export const RATINGS: { rating: Rating; label: string }[] = [
  { rating: 1, label: "Again" },
  { rating: 2, label: "Hard" },
  { rating: 3, label: "Good" },
  { rating: 4, label: "Easy" },
];

export interface Word {
  id: number;
  danish: string;
  english: string;
  exampleDa: string | null;
  exampleEn: string | null;
  createdAt: string;
}

export interface WordWithCard extends Word {
  due: string;
  state: CardState;
  stability: number;
}

export interface ReviewCard extends Word {
  due: string;
  stability: number;
  difficulty: number;
  elapsedDays: number;
  scheduledDays: number;
  reps: number;
  lapses: number;
  state: CardState;
  lastReview: string;
}

export interface IntervalPreview {
  againDays: number;
  hardDays: number;
  goodDays: number;
  easyDays: number;
}

export interface Stats {
  dueToday: number;
  newToday: number;
  reviewedToday: number;
  totalWords: number;
}

export interface WordInput {
  danish: string;
  english: string;
  exampleDa?: string | null;
  exampleEn?: string | null;
}

export interface AiFillResult {
  danish: string;
  english: string;
  exampleDa: string;
  exampleEn: string;
}

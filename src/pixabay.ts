import type { PixabayImageHit } from "./types";

export const normalizeKeyword = (value: string) => value.trim().replace(/\s+/gu, " ").toLowerCase();
export const normalizeQuery = (value: string) => value.trim().replace(/\s+/gu, " ");
export interface PixabayReference { hit: PixabayImageHit; position: number }
export interface PixabaySuggestion {
  key: string;
  text: string;
  count: number;
  relevance: "high" | "medium" | "low";
}
export function rankKeywords(references: Iterable<PixabayReference>): PixabaySuggestion[] {
  const unique = new Map<number, PixabayReference>();
  for (const item of [...references].sort((a, b) => a.position - b.position)) {
    if (!unique.has(item.hit.id)) unique.set(item.hit.id, item);
  }
  const images = [...unique.values()];
  const tags = new Map<string, PixabaySuggestion>();
  for (const { hit } of images) {
    const seen = new Set<string>();
    for (const text of hit.tags) {
      const key = normalizeKeyword(text);
      if (!key || seen.has(key)) continue;
      seen.add(key);
      const tag = tags.get(key) ?? { key, text: text.trim().replace(/\s+/gu, " "), count: 0, relevance: "low" };
      tag.count++;
      tags.set(key, tag);
    }
  }
  for (const tag of tags.values()) {
    const ratio = tag.count / images.length;
    tag.relevance = ratio >= 0.75 ? "high" : ratio >= 0.4 ? "medium" : "low";
  }
  return [...tags.values()].sort((a, b) => b.count - a.count);
}
export function appendKeywords(existing: string[], suggestions: PixabaySuggestion[], chosen: ReadonlySet<string>) {
  const keywords = [...existing];
  const seen = new Set(existing.map(normalizeKeyword));
  let added = 0;
  let duplicates = 0;
  for (const suggestion of suggestions) {
    if (!chosen.has(suggestion.key)) continue;
    if (seen.has(suggestion.key)) { duplicates++; continue; }
    seen.add(suggestion.key);
    keywords.push(suggestion.text);
    added++;
  }
  return { keywords, added, duplicates };
}

import { messages, type MessageKey } from "./messages";
import type { Language, Theme } from "./types";

let language: Language = "id";
export function t(key: MessageKey, ...params: unknown[]): string {
  return messages[key][language].replace(/\{(\d+)\}/g, (_, index) => String(params[Number(index)] ?? ""));
}
export interface AppError { key: string | null; params: string[]; detail: string | null }
export function errorText(error: unknown): string {
  if (error && typeof error === "object" && "key" in error) {
    const value = error as AppError;
    if (value.key && value.key in messages) return t(value.key as MessageKey, ...value.params);
    return value.detail ?? "";
  }
  return String(error);
}
export function applyTranslations(next: Language, theme: Theme) {
  // WebKit can reset selection when translated option text is replaced.
  const selections = [...document.querySelectorAll("select")].map((select) => [select, select.value] as const);
  language = next;
  document.documentElement.lang = next;
  document.documentElement.dataset.theme = theme;
  for (const node of document.querySelectorAll<HTMLElement>("[data-i18n]")) {
    node.textContent = t(node.dataset.i18n as MessageKey);
  }
  for (const [select, value] of selections) select.value = value;
  for (const attribute of ["title", "aria-label", "placeholder"]) {
    for (const node of document.querySelectorAll<HTMLElement>(`[data-i18n-${attribute}]`)) {
      node.setAttribute(attribute, t(node.getAttribute(`data-i18n-${attribute}`) as MessageKey));
    }
  }
}

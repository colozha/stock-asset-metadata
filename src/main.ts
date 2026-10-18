import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, confirm } from "@tauri-apps/plugin-dialog";
import type {
  AppPreferences,
  FolderStatus,
  Inspection,
  AssetDraft,
  AiGenerationRequest,
  AiModelOption,
  AiSettings,
  GeneratedKeywords,
  GeneratedTitleDescription,
  Metadata,
  MetadataPatch,
  Progress,
  ProviderDraft,
  ProviderSetting,
  SaveMode,
  Language,
  Theme,
  SaveResult,
} from "./types";
import "./styles.css";
import { t, errorText, applyTranslations } from "./i18n";
import { setupPanelResize } from "./layout";

const el = <T extends HTMLElement = HTMLElement>(id: string) =>
  document.getElementById(id) as T;
const assets = new Map<string, AssetDraft>();
const selected = new Set<string>();
const native = isTauri();
let active: string | null = null;
let busy = false;
let aiBusy = false;
let settingsBusy = false;
let previewCacheBusy = false;
let previewCacheBytes: number | null = null;
let previewCacheStatus: { key: string; params: unknown[]; state: "error" | "success" | "neutral" } | null = null;
let preferences: AppPreferences | null = null;
let outputDir: string | null = null;
const folders = new Map<string, FolderStatus>();
const explicitFiles = new Set<string>();
const disclosures = new Map<string, boolean>();
let previewUrl: string | null = null;
let previewGeneration = 0;
let aiSettings: AiSettings | null = null;
let selectedProvider = "GPT";
let hasSelectedApiKey = false;
let clearSelectedApiKey = false;
let settingsOpen = false;
let pendingPreferenceWrites = 0;
let preferenceWriteQueue: Promise<void> = Promise.resolve();
const discoveredModels = new Map<string, AiModelOption[]>();
let messageTimer: ReturnType<typeof setTimeout>;

const providerNames = [
  "GPT",
  "Gemini",
  "Claude",
  "Groq",
  "DeepSeek",
  "GLM Z.ai",
  "Databytes",
  "Ollama",
  "Custom",
];
const providerLabels: Record<string, string> = {
  GPT: "OpenAI",
  Gemini: "Gemini",
  Claude: "Claude",
  Groq: "Groq",
  DeepSeek: "DeepSeek",
  "GLM Z.ai": "GLM Z.ai",
  Databytes: "Databytes",
  Ollama: "Ollama",
  Custom: "Custom / OpenAI-compatible",
};
const currentModelCatalog: Record<string, AiModelOption[]> = {
  GPT: [
    { modelName: "gpt-6-astra", label: "GPT-6 Astra", supportsVision: true },
    { modelName: "gpt-6.1-sol", label: "GPT-6.1 Sol", supportsVision: true },
    { modelName: "gpt-6-luna", label: "GPT-6 Luna", supportsVision: true },
  ],
  Gemini: [
    { modelName: "gemini-3.8-flash", label: "Gemini 3.8 Flash", supportsVision: true },
    { modelName: "gemini-3.7-flash", label: "Gemini 3.7 Flash", supportsVision: true },
    { modelName: "gemini-3.6-flash", label: "Gemini 3.6 Flash", supportsVision: true },
    { modelName: "gemini-3.5-flash-lite", label: "Gemini 3.5 Flash-Lite", supportsVision: null },
  ],
  Claude: [
    { modelName: "claude-fable-5", label: "Claude Fable 5", supportsVision: true },
    { modelName: "claude-opus-5", label: "Claude Opus 5", supportsVision: true },
    { modelName: "claude-sonnet-5", label: "Claude Sonnet 5", supportsVision: true },
    { modelName: "claude-opus-4-8", label: "Claude Opus 4.8", supportsVision: true },
    { modelName: "claude-opus-4-7", label: "Claude Opus 4.7", supportsVision: true },
    { modelName: "claude-opus-4-6", label: "Claude Opus 4.6", supportsVision: true },
    { modelName: "claude-sonnet-4-6", label: "Claude Sonnet 4.6", supportsVision: true },
    { modelName: "claude-haiku-4-5-20251001", label: "Claude Haiku 4.5", supportsVision: true },
  ],
  Groq: [
    { modelName: "qwen/qwen3.8-27b", label: "Qwen 3.8 27B", supportsVision: true },
    { modelName: "openai/gpt-oss-120b", label: "GPT OSS 120B", supportsVision: null },
  ],
  DeepSeek: [
    { modelName: "deepseek-flash", label: "DeepSeek V4.1 Flash", supportsVision: true },
    { modelName: "deepseek-v4-pro", label: "DeepSeek V4 Pro", supportsVision: false },
  ],
  "GLM Z.ai": [
    { modelName: "glm-5v-turbo", label: "GLM-5V Turbo", supportsVision: true },
    { modelName: "glm-4.6v", label: "GLM-4.6V", supportsVision: true },
    { modelName: "glm-4.6v-flashx", label: "GLM-4.6V FlashX", supportsVision: true },
    { modelName: "glm-4.6v-flash", label: "GLM-4.6V Flash", supportsVision: true },
    { modelName: "glm-4.5v", label: "GLM-4.5V", supportsVision: true },
    { modelName: "glm-5.2", label: "GLM-5.2", supportsVision: false },
  ],
  Databytes: [
    { modelName: "databyte-m1", label: "Databyte M1", supportsVision: false },
  ],
  Ollama: [
    { modelName: "qwen3-vl:2b", label: "Qwen3-VL 2B", supportsVision: true },
    { modelName: "qwen3-vl:4b", label: "Qwen3-VL 4B", supportsVision: true },
    { modelName: "qwen3-vl:8b", label: "Qwen3-VL 8B", supportsVision: true },
    { modelName: "gemma3:4b", label: "Gemma 3 4B", supportsVision: true },
    { modelName: "gemma3:12b", label: "Gemma 3 12B", supportsVision: true },
    { modelName: "gemma3:27b", label: "Gemma 3 27B", supportsVision: true },
    { modelName: "llama4:scout", label: "Llama 4 Scout", supportsVision: true },
    { modelName: "mistral-small3.1", label: "Mistral Small 3.1", supportsVision: true },
    { modelName: "llama3.1", label: "Llama 3.1", supportsVision: false },
  ],
  Custom: [],
};

const copy = (metadata: Metadata): Metadata => ({
  ...metadata,
  keywords: [...metadata.keywords],
});
const same = (a: Metadata, b: Metadata) =>
  a.title === b.title &&
  a.description === b.description &&
  JSON.stringify(a.keywords) === JSON.stringify(b.keywords);
const dirty = (asset: AssetDraft) =>
  !same(
    (preferences?.saveMode ?? "copy") === "copy"
      ? (asset.lastSavedCopy ?? asset.metadata)
      : asset.metadata,
    asset.draft,
  );
const current = () => (active ? assets.get(active) : undefined);
const targets = () =>
  [...selected]
    .map((path) => assets.get(path)!)
    .filter((asset) => asset && !asset.error && dirty(asset));
const sizeText = (size: number) =>
  size < 1024
    ? `${size} B`
    : size < 1024 * 1024
      ? `${(size / 1024).toFixed(0)} KB`
      : `${(size / 1024 / 1024).toFixed(1)} MB`;
const keywords = (value: string) => {
  const seen = new Set<string>();
  return value
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean)
    .filter((word) => {
      const key = word.toLowerCase();
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    });
};

function icon(name: string) {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("class", "icon");
  svg.setAttribute("aria-hidden", "true");
  const use = document.createElementNS("http://www.w3.org/2000/svg", "use");
  use.setAttribute("href", `#i-${name}`);
  svg.append(use);
  return svg;
}

function message(text: string, error = false) {
  clearTimeout(messageTimer);
  const node = el("message");
  node.textContent = text;
  node.hidden = false;
  node.classList.toggle("is-error", error);
  messageTimer = setTimeout(
    () => {
      node.hidden = true;
    },
    error ? 14000 : 7000,
  );
}

function requireDesktop() {
  if (native) return true;
  message(t("Buka aplikasi desktop untuk mengakses dan menyimpan file lokal."));
  return false;
}

function assetRow(asset: AssetDraft) {
    const row = document.createElement("div");
    row.className = `asset-row${active === asset.path ? " active" : ""}`;
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.checked = selected.has(asset.path);
    checkbox.disabled = busy || aiBusy;
    checkbox.setAttribute("aria-label", t("Pilih {0}", asset.name));
    checkbox.addEventListener("change", () => {
      if (checkbox.checked) selected.add(asset.path);
      else selected.delete(asset.path);
      if (!active || !selected.has(active))
        active = selected.values().next().value ?? asset.path;
      render(true);
      void loadPreview();
    });
    const button = document.createElement("button");
    button.type = "button";
    button.className = "asset-item";
    button.disabled = busy || aiBusy;
    button.title = asset.path;
    button.setAttribute("aria-pressed", String(active === asset.path));
    const name = document.createElement("strong");
    name.textContent = asset.name;
    const detail = document.createElement("span");
    detail.textContent = `${asset.format === "jpeg" ? "JPG" : asset.format.toUpperCase()} · ${sizeText(asset.size)}`;
    const status = document.createElement("small");
    const hasError = asset.error || asset.saveError;
    status.textContent = hasError
      ? t("Gagal · lihat detail")
      : dirty(asset)
        ? t("Belum disimpan")
        : asset.savedCopy
          ? t("Salinan tersimpan")
          : asset.backupPath
            ? t("Tersimpan + backup")
            : t("Siap disunting");
    status.className = hasError
      ? "error-text"
      : dirty(asset)
        ? "modified-text"
        : "asset-status";
    button.append(name, detail, status);
    button.addEventListener("click", () => {
      active = asset.path;
      selected.clear();
      selected.add(asset.path);
      render(true);
      void loadPreview();
    });
    row.append(checkbox, button);
    return row;
}

const inside = (path: string, root: string) => path.startsWith(`${root.replace(/\/$/, "")}/`);
const groupFormat = (format: string) => format === "jpeg" ? "jpg" : format;
function owner(path: string) {
  return [...folders.keys()].filter((root) => inside(path, root)).sort((a, b) => b.length - a.length)[0];
}

function disclosure(key: string, title: string, count: number, folder = false) {
  const node = document.createElement("details");
  node.className = "asset-group";
  node.open = disclosures.get(key) ?? true;
  const summary = document.createElement("summary");
  summary.title = title;
  if (folder) summary.append(icon("folder"));
  const label = document.createElement("span");
  label.className = "tree-label";
  label.textContent = title;
  const badge = document.createElement("small");
  badge.textContent = String(count);
  summary.append(label, badge);
  node.append(summary);
  node.addEventListener("toggle", () => { if (node.isConnected) disclosures.set(key, node.open); });
  return node;
}

function appendTree(container: HTMLElement, directory: string, items: AssetDraft[]) {
  const subfolders = new Map<string, AssetDraft[]>();
  const formats = new Map<string, AssetDraft[]>();
  for (const asset of items) {
    const relative = directory ? asset.path.slice(directory.replace(/\/$/, "").length + 1) : asset.name;
    const slash = relative.indexOf("/");
    const groups = slash < 0 ? formats : subfolders;
    const key = slash < 0 ? groupFormat(asset.format) : relative.slice(0, slash);
    groups.set(key, [...(groups.get(key) ?? []), asset]);
  }
  for (const [name, children] of [...subfolders].sort(([a], [b]) => a.localeCompare(b))) {
    const path = `${directory.replace(/\/$/, "")}/${name}`;
    const node = disclosure(`folder:${path}`, name, children.length, true);
    appendTree(node, path, children);
    container.append(node);
  }
  for (const [format, children] of [...formats].sort(([a], [b]) => a.localeCompare(b))) {
    const node = disclosure(`format:${directory}:${format}`, format.toUpperCase(), children.length);
    children.sort((a, b) => a.name.localeCompare(b.name)).forEach((asset) => node.append(assetRow(asset)));
    container.append(node);
  }
}

function renderList() {
  const list = el("asset-list");
  const scroll = list.scrollTop;
  list.replaceChildren();
  for (const folder of folders.values()) {
    const items = [...assets.values()].filter((asset) => owner(asset.path) === folder.path);
    const name = folder.path.split("/").filter(Boolean).pop() ?? folder.path;
    const node = disclosure(`root:${folder.path}`, name, items.length, true);
    const summary = node.querySelector("summary")!;
    summary.title = folder.path;
    const remove = document.createElement("button");
    remove.className = "remove-folder icon-button";
    remove.title = t("Keluarkan folder dari daftar");
    remove.setAttribute("aria-label", t("Keluarkan folder {0} dari daftar", name));
    remove.disabled = busy || aiBusy || settingsBusy;
    remove.append(icon("x"));
    remove.addEventListener("click", (event) => { event.preventDefault(); event.stopPropagation(); void removeFolder(folder.path); });
    summary.append(remove);
    if (folder.error || !items.length) {
      const status = document.createElement("p");
      status.className = folder.error ? "folder-status error-text" : "folder-status";
      status.textContent = folder.error ? errorText(folder.error) : t("Tidak ada aset di folder ini.");
      node.append(status);
    }
    appendTree(node, folder.path, items);
    list.append(node);
  }
  const extra = [...assets.values()].filter((asset) => !owner(asset.path));
  if (extra.length) {
    const node = disclosure("extra", t("File tambahan"), extra.length);
    appendTree(node, "", extra);
    list.append(node);
  }
  list.scrollTop = scroll;
  el("list-empty").hidden = assets.size > 0 || folders.size > 0;
  el("asset-count").textContent = String(assets.size);
  el("selected-count").textContent = t("{0} dipilih", selected.size);
  const all = el<HTMLInputElement>("select-all");
  all.checked = assets.size > 0 && selected.size === assets.size;
  all.indeterminate = selected.size > 0 && selected.size < assets.size;
  all.disabled = busy || aiBusy || assets.size === 0;
}

function renderKeywords() {
  const value = current()?.draft.keywords ?? [];
  const list = el("keyword-list");
  list.replaceChildren();
  el("keyword-count").textContent = t("{0} keyword", value.length);
  value.forEach((keyword, index) => {
    const item = document.createElement("li");
    const number = document.createElement("span");
    number.className = "keyword-number";
    number.textContent = String(index + 1);
    const text = document.createElement("span");
    text.className = "keyword-text";
    text.textContent = keyword;
    item.append(number, text);
    for (const [direction, label, step] of [
      ["up", t("Naikkan"), -1],
      ["down", t("Turunkan"), 1],
    ] as const) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "keyword-move";
      button.title = `${label} ${keyword}`;
      button.setAttribute("aria-label", button.title);
      button.append(icon(direction));
      button.disabled =
        busy || aiBusy || index + step < 0 || index + step >= value.length;
      button.addEventListener("click", () => {
        const asset = current();
        if (!asset) return;
        [asset.draft.keywords[index], asset.draft.keywords[index + step]] = [
          asset.draft.keywords[index + step],
          asset.draft.keywords[index],
        ];
        el<HTMLTextAreaElement>("keywords").value =
          asset.draft.keywords.join(", ");
        render(false);
      });
      item.append(button);
    }
    list.append(item);
  });
}

function render(updateFields = false) {
  const asset = current();
  renderList();
  el<HTMLFieldSetElement>("editor-fields").disabled =
    busy || aiBusy || !asset || !!asset.error;
  for (const id of ["add-files", "add-folder", "empty-add", "choose-output"])
    el<HTMLButtonElement>(id).disabled = busy || aiBusy;
  el<HTMLButtonElement>("reload").disabled = busy || aiBusy || selected.size === 0;
  el<HTMLSelectElement>("save-mode").disabled = !native || busy || aiBusy || settingsBusy;
  el<HTMLButtonElement>("save-settings").disabled = !native || busy || aiBusy || settingsBusy || !preferences;
  el<HTMLButtonElement>("clear-preview-cache").disabled = !native || previewCacheBusy;
  el<HTMLButtonElement>("choose-output").disabled = !native || busy || aiBusy || settingsBusy;
  el<HTMLTextAreaElement>("additional-context").disabled = busy || aiBusy || !asset || !!asset.error;
  el<HTMLButtonElement>("open-settings").disabled = !native || busy || aiBusy || settingsBusy;
  const hasAsset = !!asset && !asset.error;
  el<HTMLButtonElement>("generate-title-description").disabled =
    !native || busy || aiBusy || !hasAsset;
  el<HTMLButtonElement>("generate-keywords").disabled =
    !native || busy || aiBusy || !hasAsset;
  el<HTMLSelectElement>("description-length").disabled =
    !native || busy || aiBusy || !hasAsset;
  el<HTMLInputElement>("keyword-target-count").disabled =
    !native || busy || aiBusy || !hasAsset;
  el<HTMLButtonElement>("generate-title-description").innerHTML = aiBusy
    ? '<span class="loading-dot" aria-hidden="true"></span>' + t("Generating…")
    : '<svg class="icon" aria-hidden="true"><use href="#i-spark" /></svg>' + t("Generate Title + Description");
  el<HTMLButtonElement>("generate-keywords").innerHTML = aiBusy
    ? '<span class="loading-dot" aria-hidden="true"></span>' + t("Generating…")
    : '<svg class="icon" aria-hidden="true"><use href="#i-tag" /></svg>' + t("Generate Keywords");
  const batch = selected.size > 1;
  el("batch-notice").hidden = !batch;
  el("batch-title").textContent = t("{0} aset dipilih", selected.size);
  document.querySelectorAll<HTMLElement>(".batch-toggle").forEach((label) => {
    label.hidden = !batch;
  });
  el("apply-batch").hidden = !batch;
  el<HTMLButtonElement>("apply-batch").disabled = ![
    "title",
    "description",
    "keywords",
  ].some((field) => el<HTMLInputElement>(`apply-${field}`).checked) || busy || aiBusy;
  if (updateFields) {
    el<HTMLTextAreaElement>("additional-context").value = asset?.additionalContext ?? "";
    el<HTMLInputElement>("title").value = asset?.draft.title ?? "";
    el<HTMLTextAreaElement>("description").value =
      asset?.draft.description ?? "";
    el<HTMLTextAreaElement>("keywords").value =
      asset?.draft.keywords.join(", ") ?? "";
  }
  el("title-count").textContent =
    t("{0} karakter", [...(asset?.draft.title ?? "")].length);
  el("description-count").textContent =
    t("{0} karakter", [...(asset?.draft.description ?? "")].length);
  el("draft-status").textContent = !asset
    ? t("Belum ada aset")
    : asset.error || asset.saveError
      ? t("Perlu perhatian")
      : dirty(asset)
        ? t("Belum disimpan")
        : t("Siap");
  el("draft-status").classList.toggle("modified-text", !!asset && dirty(asset));
  el("editor-help").textContent = asset
    ? asset.name
    : t("Pilih sebuah aset. Metadata yang sudah tertanam akan ditampilkan di sini.");
  const error = asset?.error || asset?.saveError;
  el("file-error").hidden = !error;
  el("file-error").textContent = error
    ? t("{0} Muat ulang file jika sumber telah berubah.", errorText(error))
    : "";
  const saved = asset?.savedCopy || asset?.backupPath;
  el("save-result").hidden = !saved;
  el("save-result").textContent = asset?.savedCopy
    ? t("Salinan: {0}", asset.savedCopy)
    : asset?.backupPath
      ? t("Backup: {0}", asset.backupPath)
      : "";
  renderKeywords();
  const mode = preferences?.saveMode ?? "copy";
  el("save-mode-summary").textContent = mode === "copy" ? t("Simpan salinan") : t("Ganti sumber + backup");
  el("save-destination-summary").textContent = mode === "copy"
    ? t("{0}/[tipe file]", preferences?.outputDir ?? t("Atur tujuan melalui Settings"))
    : t("Di direktori sumber");
  el("save-destination-summary").title = preferences?.outputDir ?? "";
  el("save-hint").textContent =
    mode === "copy"
      ? t("Salinan baru menjaga file sumber tetap utuh.")
      : t("Backup .bak dibuat sebelum sumber diganti.");
  const pending = targets().length;
  el<HTMLButtonElement>("save").disabled =
    busy || aiBusy || settingsBusy || !preferences || pending === 0;
  el("save-label").textContent = busy
    ? t("Memproses…")
    : mode === "copy"
      ? t("Simpan metadata")
      : t("Simpan + backup");
  if (!busy) el("progress-text").textContent = t("{0} perubahan dipilih", pending);
  el("progress").hidden = !busy;
}

async function loadPreview() {
  const generation = ++previewGeneration;
  const asset = current();
  const image = el<HTMLImageElement>("preview-image");
  image.hidden = true;
  if (previewUrl) {
    URL.revokeObjectURL(previewUrl);
    previewUrl = null;
  }
  el("preview-name").textContent = asset?.name ?? t("Ruang preview");
  el("preview-detail").textContent = asset
    ? `${asset.format.toUpperCase()} · ${sizeText(asset.size)}`
    : t("Pilih aset untuk melihat isinya");
  el("preview-format").textContent = asset?.format.toUpperCase() ?? t("Preview").toUpperCase();
  el("preview-empty").hidden = false;
  el("empty-add").hidden = !!asset;
  el("format-list").hidden = !!asset;
  el("preview-caption").textContent = asset
    ? asset.path
    : t("Aset tetap berada di perangkat Anda.");
  el("preview-state").textContent = asset
    ? t("Membaca preview…")
    : t("Menunggu aset");
  el("empty-title").textContent = asset
    ? t("Membaca preview…")
    : t("Semua aset, siap diberi cerita.");
  el("empty-description").textContent = asset
    ? t("Metadata dan preview diproses secara terpisah.")
    : t("Tambahkan aset untuk menyunting title, description,\ndan keywords tanpa mengubah konten media.");
  if (!asset) return;
  el("empty-icon").setAttribute("href", "#i-image");
  if (asset.error) {
    el("empty-title").textContent = t("Preview tidak tersedia");
    el("empty-description").textContent = errorText(asset.error);
    el("preview-state").textContent = t("File tidak dapat dibaca");
    return;
  }
  try {
    const bytes = await invoke<ArrayBuffer>("read_preview", {
      path: asset.path,
    });
    if (generation !== previewGeneration) return;
    previewUrl = URL.createObjectURL(
      new Blob([bytes], {
        type: "image/png",
      }),
    );
    image.onload = () => {
      if (generation === previewGeneration) {
        image.hidden = false;
        el("preview-empty").hidden = true;
        el("preview-state").textContent = t("Preview");
      }
    };
    image.onerror = () => {
      if (generation === previewGeneration) {
        el("empty-title").textContent = t("Preview tidak dapat ditampilkan");
        el("empty-description").textContent = t("Metadata tetap dapat disunting.");
        el("preview-state").textContent = t("Tanpa preview");
      }
    };
    image.alt = t("Preview {0}", asset.name);
    image.src = previewUrl;
  } catch (error) {
    if (generation !== previewGeneration) return;
    el("empty-title").textContent = t("Preview tidak tersedia");
    el("empty-description").textContent = errorText(error);
    el("preview-state").textContent = t("Tanpa preview");
  }
}

async function importAssets(paths: string[], reload = false) {
  if (busy || aiBusy || !paths.length) return;
  busy = true;
  el("progress-text").textContent = t("Membaca metadata…");
  el<HTMLProgressElement>("progress").removeAttribute("value");
  render();
  try {
    const report = await invoke<Inspection>("inspect_assets", { paths, refresh: reload });
    const incoming = report.assets;
    if (!reload) {
      report.folders.forEach((folder) => folders.set(folder.path, folder));
      report.files.forEach((path) => explicitFiles.add(path));
      if (preferences) preferences.folders = [...folders.keys()];
    }
    for (const asset of incoming) {
      if (!assets.has(asset.path) || reload)
        assets.set(asset.path, {
          ...asset,
          draft: copy(asset.metadata),
          additionalContext: assets.get(asset.path)?.additionalContext ?? "",
          lastSavedCopy: null,
          savedCopy: null,
          backupPath: null,
          saveError: null,
        });
    }
    if (!active && incoming.length) {
      active = incoming[0].path;
      selected.add(active);
    }
    message(
      incoming.length
        ? t("{0} aset dibaca{1}", incoming.length, incoming.some((a) => a.error) ? t(". Beberapa file perlu diperiksa.") : ".")
        : t("Tidak ada format yang didukung dalam pilihan tersebut."),
    );
  } catch (error) {
    message(t("Aset tidak dapat dibaca: {0}", errorText(error)), true);
  } finally {
    busy = false;
    render(true);
    void loadPreview();
  }
}

async function addFiles(folder = false) {
  if (!requireDesktop() || busy || aiBusy) return;
  try {
    const paths = await open({
      directory: folder,
      multiple: !folder,
      title: folder ? t("Pilih folder aset") : t("Pilih aset"),
      filters: folder
        ? undefined
        : [
            {
              name: t("Aset microstock"),
              extensions: ["jpg", "jpeg", "png", "svg", "eps", "mp4", "mov"],
            },
          ],
    });
    if (paths) await importAssets(Array.isArray(paths) ? paths : [paths]);
  } catch (error) {
    message(errorText(error), true);
  }
}

async function chooseOutput() {
  if (!requireDesktop() || busy || aiBusy) return;
  try {
    const path = await open({
      directory: true,
      multiple: false,
      title: t("Pilih folder keluaran"),
    });
    if (typeof path === "string") {
      outputDir = path;
      el<HTMLInputElement>("output-directory").value = path;
    }
  } catch (error) {
    message(errorText(error), true);
  }
}

async function save() {
  if (!requireDesktop() || busy || aiBusy || !targets().length) return;
  if (!preferences) return;
  const mode = preferences.saveMode;
  const items = targets().map((asset) => {
    const patch: MetadataPatch = {};
    if (asset.draft.title !== asset.metadata.title)
      patch.title = asset.draft.title;
    if (asset.draft.description !== asset.metadata.description)
      patch.description = asset.draft.description;
    if (
      JSON.stringify(asset.draft.keywords) !==
      JSON.stringify(asset.metadata.keywords)
    )
      patch.keywords = asset.draft.keywords;
    return { path: asset.path, patch };
  });
  busy = true;
  el("progress-text").textContent = t("0 / {0} disimpan", items.length);
  el<HTMLProgressElement>("progress").max = items.length;
  el<HTMLProgressElement>("progress").value = 0;
  render();
  try {
    const results = await invoke<SaveResult[]>("save_metadata", {
      request: { items },
    });
    for (const result of results) {
      const asset = assets.get(result.path)!;
      asset.saveError = result.error;
      if (result.metadata) {
        if (mode === "copy") asset.lastSavedCopy = copy(result.metadata);
        else {
          asset.metadata = copy(result.metadata);
          asset.lastSavedCopy = null;
        }
        asset.draft = copy(result.metadata);
        asset.savedCopy = mode === "copy" ? result.outputPath : null;
        asset.backupPath = result.backupPath;
      }
    }
    const failed = results.filter((r) => r.error).length;
    message(
      t("{0} aset tersimpan{1}", results.length - failed, failed ? t(", {0} gagal. Lihat detail pada daftar aset.", failed) : "."),
      failed > 0,
    );
  } catch (error) {
    message(t("Penyimpanan gagal: {0}", errorText(error)), true);
  } finally {
    busy = false;
    render(true);
  }
}

function currentProvider(): ProviderSetting | undefined {
  return aiSettings?.providerSettings.find(
    (provider) => provider.providerName === selectedProvider,
  );
}

function updateAiSetupStatus() {
  if (!aiSettings) return;
  const active = aiSettings.providerSettings.find(
    (provider) => provider.providerName === aiSettings?.activeProvider,
  );
  const hasApiKey = statusForProvider(aiSettings.activeProvider);
  const ready = !!active?.endpointUrl.trim() && !!active.modelName.trim() &&
    (aiSettings.activeProvider === "Ollama" || hasApiKey);
  el("ai-status").classList.remove("is-error");
  el("ai-status").textContent = ready
    ? t("{0} aktif. Hasil generate masuk ke draft aset.", providerLabels[aiSettings.activeProvider] ?? aiSettings.activeProvider)
    : aiSettings.activeProvider === "Ollama"
      ? t("Lengkapi endpoint dan model Ollama melalui Settings untuk mulai.")
      : t("Tambahkan API key provider melalui Settings untuk mulai.");
}

function updateRuntimeLabel() {
  if (!aiSettings) return;
  const provider = aiSettings.providerSettings.find(
    (setting) => setting.providerName === aiSettings?.activeProvider,
  );
  const ready = !!provider?.endpointUrl.trim() && !!provider.modelName.trim() &&
    (aiSettings.activeProvider === "Ollama" || statusForProvider(aiSettings.activeProvider));
  el("runtime-label").textContent = !ready
    ? t("Lokal · AI opsional")
    : aiSettings.activeProvider === "Ollama"
      ? t("Lokal · Ollama")
      : `AI · ${providerLabels[aiSettings.activeProvider] ?? aiSettings.activeProvider}`;
}

function statusForProvider(providerName: string) {
  return aiSettings?.keyStatus.find((status) => status.providerName === providerName)
    ?.hasApiKey ?? false;
}

function setSettingsStatus(text: string, state: "error" | "success" | "neutral" = "neutral") {
  const node = el("settings-status");
  node.textContent = text;
  node.classList.toggle("is-error", state === "error");
  node.classList.toggle("is-success", state === "success");
}

function renderModelOptions() {
  const datalist = el("model-options");
  datalist.replaceChildren();
  const provider = currentProvider();
  const options = discoveredModels.get(selectedProvider) ?? currentModelCatalog[selectedProvider] ?? [];
  const existing = provider?.modelName.trim();
  const entries = [...options];
  if (existing && !entries.some((option) => option.modelName === existing)) {
    entries.unshift({ modelName: existing, label: existing, supportsVision: null });
  }
  for (const option of entries) {
    const item = document.createElement("option");
    item.value = option.modelName;
    item.label = `${option.label}${option.supportsVision === true ? t(" · vision") : ""}`;
    datalist.append(item);
  }
}

function renderVisionHelp() {
  const provider = currentProvider();
  const mode = el<HTMLSelectElement>("vision-mode").value;
  const model = el<HTMLInputElement>("provider-model").value.trim();
  const discovered = (discoveredModels.get(selectedProvider) ?? []).find((item) => item.modelName === model);
  const fallback = (currentModelCatalog[selectedProvider] ?? []).find((item) => item.modelName === model);
  const supports = discovered?.supportsVision ?? fallback?.supportsVision ?? null;
  const text = mode === "yes"
    ? t("Preview raster akan dikirim untuk model ini.")
    : mode === "no"
      ? t("Preview tidak akan dikirim, meskipun model mendukung gambar.")
      : supports === true
        ? t("Model ini mendukung gambar; preview raster dapat dikirim.")
        : t("Jika kemampuan model tidak diketahui, preview tidak dikirim.");
  el("vision-help").textContent = provider ? text : t("Pilih provider untuk mengatur input gambar.");
}

function renderProviderSettings(resetForm = false) {
  const select = el<HTMLSelectElement>("provider-select");
  if (!select.options.length) {
    for (const providerName of providerNames) {
      const option = document.createElement("option");
      option.value = providerName;
      option.textContent = providerName === "Ollama" ? t("Ollama (lokal)") : providerName === "Custom" ? t("Custom / OpenAI-compatible") : providerLabels[providerName] ?? providerName;
      select.append(option);
    }
  }
  for (const option of select.options) {
    if (option.value === "Ollama") option.textContent = t("Ollama (lokal)");
    if (option.value === "Custom") option.textContent = t("Custom / OpenAI-compatible");
  }
  select.value = selectedProvider;
  const provider = currentProvider();
  if (!provider) return;
  if (resetForm) {
    el<HTMLInputElement>("provider-endpoint").value = provider.endpointUrl;
    el<HTMLInputElement>("provider-model").value = provider.modelName;
    el<HTMLSelectElement>("vision-mode").value = provider.visionOverride === true
      ? "yes"
      : provider.visionOverride === false
        ? "no"
        : "auto";
  }
  const keyInput = el<HTMLInputElement>("provider-api-key");
  if (resetForm) {
    keyInput.value = "";
    keyInput.type = "password";
    clearSelectedApiKey = false;
    hasSelectedApiKey = statusForProvider(selectedProvider);
  }
  el("api-key-state").textContent = selectedProvider === "Ollama"
    ? t("API key tidak diperlukan untuk Ollama lokal.")
    : clearSelectedApiKey
      ? t("API key tersimpan akan dihapus saat pengaturan disimpan.")
      : hasSelectedApiKey
        ? t("API key tersimpan di credential store OS. Isi hanya untuk menggantinya.")
        : t("API key belum disimpan.");
  const clearKey = el<HTMLButtonElement>("clear-api-key");
  clearKey.hidden = !hasSelectedApiKey;
  clearKey.textContent = clearSelectedApiKey ? t("Batalkan penghapusan") : t("Hapus key tersimpan");
  const warning = el("keyring-warning");
  warning.textContent = aiSettings?.keyringError ? errorText(aiSettings.keyringError) : "";
  warning.hidden = !aiSettings?.keyringError;
  renderModelOptions();
  renderVisionHelp();
  for (const id of ["provider-select", "provider-endpoint", "provider-api-key", "provider-model", "vision-mode", "toggle-api-key", "clear-api-key", "refresh-models", "test-provider", "save-settings", "language", "theme", "save-mode", "choose-output"])
    el<HTMLInputElement | HTMLSelectElement | HTMLButtonElement>(id).disabled = settingsBusy || (id === "save-settings" && !preferences);
}

async function loadAiSettings() {
  const loaded = await invoke<AiSettings>("load_ai_settings");
  aiSettings = loaded;
  selectedProvider = loaded.activeProvider;
  el<HTMLSelectElement>("description-length").value = loaded.descriptionLength;
  el<HTMLInputElement>("keyword-target-count").value = String(loaded.keywordCount);
  updateAiSetupStatus();
  updateRuntimeLabel();
  renderProviderSettings(true);
  render();
}

function providerDraft(): ProviderDraft {
  const visionMode = el<HTMLSelectElement>("vision-mode").value;
  const key = el<HTMLInputElement>("provider-api-key").value.trim();
  return {
    providerName: selectedProvider,
    endpointUrl: el<HTMLInputElement>("provider-endpoint").value.trim(),
    modelName: el<HTMLInputElement>("provider-model").value.trim(),
    visionOverride: visionMode === "auto" ? null : visionMode === "yes",
    apiKey: key || null,
  };
}

function setSettingsBusy(value: boolean) {
  settingsBusy = value;
  renderProviderSettings();
  el<HTMLButtonElement>("settings-back").disabled = value;
  render();
}

async function saveSettings() {
  if (!requireDesktop() || !preferences || settingsBusy) return;
  const draft = providerDraft();
  // Validate before writing either settings file. Saving never contacts the provider.
  if (draft.endpointUrl) {
    try {
      const url = new URL(draft.endpointUrl);
      if (!["http:", "https:"].includes(url.protocol) || url.username || url.password || url.search || url.hash ||
        (url.protocol === "http:" && !["localhost", "127.0.0.1", "[::1]"].includes(url.hostname))) throw new Error();
    } catch {
      setSettingsStatus(t("URL endpoint tidak valid."), "error");
      return;
    }
  }
  const language = el<HTMLSelectElement>("language").value as Language;
  const theme = el<HTMLSelectElement>("theme").value as Theme;
  const previousOutput = preferences.outputDir;
  const providerSettings = aiSettings?.providerSettings.map((provider) => provider.providerName === selectedProvider
    ? { ...provider, endpointUrl: draft.endpointUrl, modelName: draft.modelName, visionOverride: draft.visionOverride }
    : provider);
  const providerChanged = !!aiSettings && (JSON.stringify(providerSettings) !== JSON.stringify(aiSettings.providerSettings)
    || selectedProvider !== aiSettings.activeProvider || !!draft.apiKey || clearSelectedApiKey);
  setSettingsBusy(true);
  setSettingsStatus(t("Menyimpan pengaturan…"));
  let appSaved = false;
  try {
    preferences = await invoke<AppPreferences>("save_app_preferences", {
      saveMode: el<HTMLSelectElement>("save-mode").value as SaveMode,
      outputDir, language, theme,
    });
    appSaved = true;
    if (previousOutput !== preferences.outputDir) {
      assets.forEach((asset) => { asset.lastSavedCopy = null; asset.savedCopy = null; });
    }
    applyAppearance(preferences.language, preferences.theme);
    if (providerChanged && aiSettings) {
      aiSettings = await invoke<AiSettings>("save_ai_settings", {
        providerSettings, selectedProvider, apiKey: draft.apiKey, clearApiKey: clearSelectedApiKey,
        descriptionLength: aiSettings.descriptionLength, keywordCount: aiSettings.keywordCount,
      });
      renderProviderSettings(true);
    }
    setSettingsStatus(t("Pengaturan tersimpan."), "success");
  } catch (error) {
    // Refresh the saved state without replacing the user's unsaved form fields/key.
    try { aiSettings = await invoke<AiSettings>("load_ai_settings"); } catch { /* Keep last known state. */ }
    setSettingsStatus(t(appSaved ? "Pengaturan aplikasi tersimpan. Pengaturan AI gagal: {0}" : "Pengaturan tidak tersimpan: {0}", errorText(error)), "error");
  } finally {
    updateAiSetupStatus();
    updateRuntimeLabel();
    setSettingsBusy(false);
  }
}

async function testProviderConnection() {
  if (!requireDesktop() || settingsBusy) return;
  const draft = providerDraft();
  setSettingsBusy(true);
  setSettingsStatus(t("Menguji koneksi provider…"));
  try {
    await invoke("test_ai_connection", { draft });
    setSettingsStatus(t("Koneksi berhasil. Provider mengembalikan respons JSON yang valid."), "success");
  } catch (error) {
    setSettingsStatus(errorText(error), "error");
  } finally {
    setSettingsBusy(false);
  }
}

async function refreshProviderModels() {
  if (!requireDesktop() || settingsBusy) return;
  const draft = providerDraft();
  setSettingsBusy(true);
  setSettingsStatus(t("Memuat daftar model…"));
  try {
    const models = await invoke<AiModelOption[]>("list_ai_models", { draft });
    discoveredModels.set(selectedProvider, models);
    renderModelOptions();
    renderVisionHelp();
    setSettingsStatus(t("{0} model tersedia. Pilih model atau masukkan ID secara manual.", models.length), "success");
  } catch (error) {
    setSettingsStatus(t("{0} Model saat ini tetap dapat digunakan atau diisi manual.", errorText(error)), "error");
  } finally {
    setSettingsBusy(false);
  }
}

async function saveGeneratorPreferences() {
  if (!native) return;
  const descriptionLength = el<HTMLSelectElement>("description-length").value as AiSettings["descriptionLength"];
  const keywordCount = Number(el<HTMLInputElement>("keyword-target-count").value);
  if (!Number.isInteger(keywordCount) || keywordCount < 10 || keywordCount > 50) {
    el<HTMLInputElement>("keyword-target-count").value = String(aiSettings?.keywordCount ?? 40);
    el("ai-status").textContent = t("Jumlah keywords harus antara 10 dan 50.");
    el("ai-status").classList.add("is-error");
    return;
  }
  if (aiSettings) aiSettings = { ...aiSettings, descriptionLength, keywordCount };
  pendingPreferenceWrites += 1;
  const write = preferenceWriteQueue.then(async () => {
    aiSettings = await invoke<AiSettings>("save_generator_preferences", {
      descriptionLength,
      keywordCount,
    });
  });
  preferenceWriteQueue = write.catch(() => {});
  try {
    await write;
    el("ai-status").classList.remove("is-error");
  } catch (error) {
    el("ai-status").textContent = t("Preferensi tidak tersimpan: {0}", errorText(error));
    el("ai-status").classList.add("is-error");
  } finally {
    pendingPreferenceWrites -= 1;
  }
}

function generationRequest(asset: AssetDraft): AiGenerationRequest {
  return {
    path: asset.path,
    assetName: asset.name,
    additionalContext: asset.additionalContext,
    metadata: copy(asset.draft),
    descriptionLength: el<HTMLSelectElement>("description-length").value as AiSettings["descriptionLength"],
    keywordCount: Number(el<HTMLInputElement>("keyword-target-count").value),
  };
}

async function generateTitleDescription() {
  if (!requireDesktop() || busy || aiBusy) return;
  const asset = current();
  if (!asset || asset.error) return;
  aiBusy = true;
  el("ai-status").textContent = t("Membuat title dan description untuk draft aset aktif…");
  el("ai-status").classList.remove("is-error");
  render();
  try {
    const result = await invoke<GeneratedTitleDescription>("generate_title_description", {
      request: generationRequest(asset),
    });
    asset.draft.title = result.title;
    asset.draft.description = result.description;
    asset.saveError = null;
    el<HTMLInputElement>("title").value = result.title;
    el<HTMLTextAreaElement>("description").value = result.description;
    el("ai-status").textContent = t("Title dan description masuk ke draft ({0} · {1}). Tinjau sebelum menyimpan.", result.providerName, result.modelName);
  } catch (error) {
    el("ai-status").textContent = t("Generate gagal: {0}", errorText(error));
    el("ai-status").classList.add("is-error");
  } finally {
    aiBusy = false;
    render(false);
  }
}

async function generateKeywords() {
  if (!requireDesktop() || busy || aiBusy) return;
  const asset = current();
  if (!asset || asset.error) return;
  aiBusy = true;
  el("ai-status").textContent = t("Membuat keywords untuk draft aset aktif…");
  el("ai-status").classList.remove("is-error");
  render();
  try {
    const result = await invoke<GeneratedKeywords>("generate_keywords", {
      request: generationRequest(asset),
    });
    asset.draft.keywords = result.keywords;
    asset.saveError = null;
    el<HTMLTextAreaElement>("keywords").value = result.keywords.join(", ");
    el("ai-status").textContent = t("{0} keywords masuk ke draft ({1} · {2}). Tinjau sebelum menyimpan.", result.keywords.length, result.providerName, result.modelName);
  } catch (error) {
    el("ai-status").textContent = t("Generate gagal: {0}", errorText(error));
    el("ai-status").classList.add("is-error");
  } finally {
    aiBusy = false;
    render(false);
  }
}

async function openSettings() {
  if (!requireDesktop() || busy || aiBusy) return;
  settingsOpen = true;
  el("app").classList.add("settings-open");
  el("settings-view").hidden = false;
  previewCacheBytes = null;
  previewCacheStatus = null;
  renderPreviewCacheSize();
  renderPreviewCacheStatus();
  void refreshPreviewCacheSize();
  setSettingsStatus(t("Memuat pengaturan lokal…"));
  setSettingsBusy(true);
  try {
    await preferenceWriteQueue;
    if (!settingsOpen) return;
    await loadAppPreferences();
    await loadAiSettings();
    setSettingsStatus("");
  } catch (error) {
    setSettingsStatus(t("Pengaturan tidak dapat dimuat: {0}", errorText(error)), "error");
  } finally {
    setSettingsBusy(false);
  }
  if (settingsOpen) {
    el("settings-view").scrollTop = 0;
    el<HTMLButtonElement>("settings-back").focus({ preventScroll: true });
  }
}

function renderPreviewCacheSize() {
  el("preview-cache-size").textContent = previewCacheBytes === null
    ? t("Memuat ukuran cache…")
    : sizeText(previewCacheBytes);
}

function renderPreviewCacheStatus() {
  const node = el("preview-cache-status");
  node.textContent = previewCacheStatus
    ? t(previewCacheStatus.key as Parameters<typeof t>[0], ...previewCacheStatus.params)
    : "";
  node.classList.toggle("is-error", previewCacheStatus?.state === "error");
  node.classList.toggle("is-success", previewCacheStatus?.state === "success");
}

function setPreviewCacheStatus(
  key: string,
  state: "error" | "success" | "neutral" = "neutral",
  ...params: unknown[]
) {
  previewCacheStatus = { key, params, state };
  renderPreviewCacheStatus();
}

async function refreshPreviewCacheSize() {
  try {
    previewCacheBytes = await invoke<number>("get_preview_cache_size");
    renderPreviewCacheSize();
  } catch (error) {
    setPreviewCacheStatus("Ukuran cache preview tidak dapat dimuat: {0}", "error", errorText(error));
  }
}

async function clearPreviewCache() {
  if (!requireDesktop() || previewCacheBusy) return;
  previewCacheBusy = true;
  const button = el<HTMLButtonElement>("clear-preview-cache");
  button.disabled = true;
  setPreviewCacheStatus("Menghapus cache preview…");
  try {
    const clearedBytes = await invoke<number>("clear_preview_cache");
    previewCacheBytes = 0;
    renderPreviewCacheSize();
    setPreviewCacheStatus("Cache preview dihapus ({0}).", "success", sizeText(clearedBytes));
  } catch (error) {
    setPreviewCacheStatus("Cache preview tidak dapat dihapus: {0}", "error", errorText(error));
  } finally {
    previewCacheBusy = false;
    button.disabled = !native;
  }
}

function closeSettings() {
  if (settingsBusy) return;
  settingsOpen = false;
  el("settings-view").hidden = true;
  el("app").classList.remove("settings-open");
  el<HTMLInputElement>("provider-api-key").value = "";
  el<HTMLInputElement>("provider-api-key").type = "password";
  clearSelectedApiKey = false;
  render();
  el<HTMLButtonElement>("open-settings").focus();
}

el("add-files").addEventListener("click", () => void addFiles());
el("empty-add").addEventListener("click", () => void addFiles());
el("add-folder").addEventListener("click", () => void addFiles(true));
el("open-settings").addEventListener("click", () => void openSettings());
el<HTMLButtonElement>("clear-preview-cache").addEventListener("click", () => void clearPreviewCache());
el("settings-back").addEventListener("click", closeSettings);
el<HTMLSelectElement>("provider-select").addEventListener("change", () => {
  selectedProvider = el<HTMLSelectElement>("provider-select").value;
  setSettingsStatus("");
  renderProviderSettings(true);
});
el<HTMLInputElement>("provider-api-key").addEventListener("input", () => {
  clearSelectedApiKey = false;
  renderProviderSettings();
});
el<HTMLButtonElement>("toggle-api-key").addEventListener("click", () => {
  const input = el<HTMLInputElement>("provider-api-key");
  const show = input.type === "password";
  input.type = show ? "text" : "password";
  const label = show ? t("Sembunyikan API key") : t("Tampilkan API key");
  el<HTMLButtonElement>("toggle-api-key").setAttribute("aria-label", label);
  el<HTMLButtonElement>("toggle-api-key").title = label;
  input.focus();
});
el<HTMLButtonElement>("clear-api-key").addEventListener("click", () => {
  if (!hasSelectedApiKey) return;
  clearSelectedApiKey = !clearSelectedApiKey;
  if (clearSelectedApiKey) el<HTMLInputElement>("provider-api-key").value = "";
  renderProviderSettings();
});
el<HTMLInputElement>("provider-model").addEventListener("input", renderVisionHelp);
el<HTMLSelectElement>("vision-mode").addEventListener("change", renderVisionHelp);
el("refresh-models").addEventListener("click", () => void refreshProviderModels());
el("test-provider").addEventListener("click", () => void testProviderConnection());
el("save-settings").addEventListener("click", () => void saveSettings());
el<HTMLSelectElement>("description-length").addEventListener("change", () => void saveGeneratorPreferences());
el<HTMLInputElement>("keyword-target-count").addEventListener("change", () => void saveGeneratorPreferences());
el("generate-title-description").addEventListener("click", () => void generateTitleDescription());
el("generate-keywords").addEventListener("click", () => void generateKeywords());
el("choose-output").addEventListener("click", () => void chooseOutput());
el("save").addEventListener("click", () => void save());
el("save-mode").addEventListener("change", renderStorageFields);
el("metadata-form").addEventListener("submit", (event) => {
  event.preventDefault();
  void save();
});
el("select-all").addEventListener("change", () => {
  if (busy || aiBusy) return;
  selected.clear();
  if (el<HTMLInputElement>("select-all").checked)
    assets.forEach((asset) => selected.add(asset.path));
  render();
});
el("reload").addEventListener("click", async () => {
  if (!requireDesktop() || busy || aiBusy) return;
  if (
    [...selected].some((path) => dirty(assets.get(path)!)) &&
    !(await confirm(
      t("Draft pada file terpilih akan diganti dengan metadata dari disk. Lanjutkan?"),
      {
        title: t("Muat ulang aset"),
        kind: "warning",
        okLabel: t("Muat ulang"),
        cancelLabel: t("Kembali"),
      },
    ))
  )
    return;
  await importAssets([...selected], true);
});
for (const field of ["title", "description", "keywords"] as const) {
  el(field).addEventListener("input", () => {
    const asset = current();
    if (!asset || busy) return;
    const value = el<HTMLInputElement | HTMLTextAreaElement>(field).value;
    if (field === "keywords") asset.draft.keywords = keywords(value);
    else asset.draft[field] = value;
    asset.saveError = null;
    render(false);
  });
  el(`apply-${field}`).addEventListener("change", () => render(false));
}
el("apply-batch").addEventListener("click", () => {
  if (aiBusy || busy) return;
  const asset = current();
  if (!asset) return;
  for (const path of selected) {
    const target = assets.get(path)!;
    if (target.error) continue;
    for (const field of ["title", "description", "keywords"] as const) {
      if (!el<HTMLInputElement>(`apply-${field}`).checked) continue;
      if (field === "keywords")
        target.draft.keywords = [...asset.draft.keywords];
      else target.draft[field] = asset.draft[field];
    }
    target.saveError = null;
  }
  message(
    t("Field terpilih diterapkan ke {0} aset. Simpan untuk menanamkan metadata.", selected.size),
  );
  render(false);
});
document.addEventListener("keydown", (event) => {
  if (settingsOpen && event.key === "Escape") {
    event.preventDefault();
    closeSettings();
    return;
  }
  if (event.ctrlKey || event.metaKey) {
    if (!settingsOpen && event.key.toLowerCase() === "o") {
      event.preventDefault();
      void addFiles();
    }
    if (!settingsOpen && event.key.toLowerCase() === "s") {
      event.preventDefault();
      void save();
    }
  }
});

function applyAppearance(language: Language, theme: Theme) {
  applyTranslations(language, theme);
  renderPreviewCacheSize();
  renderPreviewCacheStatus();
  el("message").hidden = true;
  render();
  renderProviderSettings();
  updateAiSetupStatus();
  updateRuntimeLabel();
  void loadPreview();
}

async function loadAppPreferences() {
  preferences = await invoke<AppPreferences>("load_app_preferences");
  outputDir = preferences.outputDir;
  el<HTMLInputElement>("output-directory").value = outputDir;
  el<HTMLSelectElement>("save-mode").value = preferences.saveMode;
  el<HTMLSelectElement>("language").value = preferences.language;
  el<HTMLSelectElement>("theme").value = preferences.theme;
  applyAppearance(preferences.language, preferences.theme);
  renderStorageFields();
  render();
}

function renderStorageFields() {
  el("output-setting").hidden = el<HTMLSelectElement>("save-mode").value !== "copy";
}


async function removeFolder(path: string) {
  if (busy || aiBusy || settingsBusy) return;
  const removed = [...assets.values()].filter((asset) => inside(asset.path, path)
    && !explicitFiles.has(asset.path)
    && ![...folders.keys()].some((root) => root !== path && inside(asset.path, root)));
  if (removed.some((asset) => dirty(asset) || asset.additionalContext.trim()) && !(await confirm(
    t("Draft dan konteks aset dalam folder ini akan dilepas dari ruang kerja. File di disk tetap ada. Lanjutkan?"),
    { title: t("Keluarkan folder"), kind: "warning", okLabel: t("Keluarkan"), cancelLabel: t("Kembali") },
  ))) return;
  busy = true;
  render();
  try {
    preferences = await invoke<AppPreferences>("remove_asset_folder", { path });
    folders.delete(path);
    for (const asset of removed) { assets.delete(asset.path); selected.delete(asset.path); }
    if (active && !assets.has(active)) active = selected.values().next().value ?? assets.keys().next().value ?? null;
    if (active && !selected.size) selected.add(active);
  } catch (error) {
    message(t("Folder tidak dapat dikeluarkan: {0}", errorText(error)), true);
  } finally {
    busy = false;
    render(true);
    void loadPreview();
  }
}

el("additional-context").addEventListener("input", () => {
  const asset = current();
  if (asset && !busy && !aiBusy) asset.additionalContext = el<HTMLTextAreaElement>("additional-context").value;
});

async function initializeDesktop() {
  try {
    await loadAppPreferences();
    preferences?.folders.forEach((path) => folders.set(path, { path, error: null }));
    if (folders.size) await importAssets([...folders.keys()]);
  } catch (error) {
    message(t("Ruang kerja tidak dapat dipulihkan: {0}", errorText(error)), true);
  }
  try {
    await loadAiSettings();
  } catch (error) {
    el("ai-status").textContent = t("Pengaturan AI tidak dapat dimuat: {0}", errorText(error));
    el("ai-status").classList.add("is-error");
  }
  await listen<Progress>("save_progress", ({ payload }) => {
    el("progress-text").textContent =
      t("{0} / {1} disimpan", payload.completed, payload.total);
    el<HTMLProgressElement>("progress").value = payload.completed;
  });
  const window = getCurrentWindow();
  await window.onDragDropEvent(({ payload }) => {
    if (payload.type === "drop") void importAssets(payload.paths);
  });
  await window.onCloseRequested(async (event) => {
    if (busy || aiBusy || settingsBusy || pendingPreferenceWrites > 0) {
      event.preventDefault();
      message(t("Tunggu hingga pemrosesan selesai sebelum menutup aplikasi."));
      return;
    }
    if ([...assets.values()].some((asset) => dirty(asset) || asset.additionalContext.trim())) {
      event.preventDefault();
      if (
        await confirm(t("Ada perubahan yang belum disimpan. Tutup aplikasi?"), {
          title: t("Perubahan belum disimpan"),
          kind: "warning",
          okLabel: t("Tutup"),
          cancelLabel: t("Kembali"),
        })
      )
        await window.destroy();
    }
  });
}

setupPanelResize();
applyTranslations("id", "light");

if (native)
  void initializeDesktop().catch((error) =>
    message(t("Integrasi desktop gagal: {0}", errorText(error)), true),
  );
else el("runtime-label").textContent = t("Pratinjau browser");
render(true);

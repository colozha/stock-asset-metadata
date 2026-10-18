import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, confirm } from "@tauri-apps/plugin-dialog";
import type {
  Asset,
  AssetDraft,
  Metadata,
  MetadataPatch,
  Progress,
  SaveMode,
  SaveResult,
} from "./types";
import "./styles.css";

const el = <T extends HTMLElement = HTMLElement>(id: string) =>
  document.getElementById(id) as T;
const assets = new Map<string, AssetDraft>();
const selected = new Set<string>();
const native = isTauri();
let active: string | null = null;
let busy = false;
let outputDir: string | null = null;
let previewUrl: string | null = null;
let previewGeneration = 0;
let messageTimer: ReturnType<typeof setTimeout>;

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
    el<HTMLSelectElement>("save-mode").value === "copy"
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
  size < 1024 * 1024
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
  message("Buka aplikasi desktop untuk mengakses dan menyimpan file lokal.");
  return false;
}

function renderList() {
  const list = el("asset-list");
  const scroll = list.scrollTop;
  list.replaceChildren();
  for (const asset of assets.values()) {
    const row = document.createElement("div");
    row.className = `asset-row${active === asset.path ? " active" : ""}`;
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.checked = selected.has(asset.path);
    checkbox.disabled = busy;
    checkbox.setAttribute("aria-label", `Pilih ${asset.name}`);
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
    button.disabled = busy;
    button.title = asset.path;
    button.setAttribute("aria-pressed", String(active === asset.path));
    const name = document.createElement("strong");
    name.textContent = asset.name;
    const detail = document.createElement("span");
    detail.textContent = `${asset.format === "jpeg" ? "JPG" : asset.format.toUpperCase()} · ${sizeText(asset.size)}`;
    const status = document.createElement("small");
    const hasError = asset.error || asset.saveError;
    status.textContent = hasError
      ? "Gagal · lihat detail"
      : dirty(asset)
        ? "Belum disimpan"
        : asset.savedCopy
          ? "Salinan tersimpan"
          : asset.backupPath
            ? "Tersimpan + backup"
            : "Siap disunting";
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
    list.append(row);
  }
  list.scrollTop = scroll;
  el("list-empty").hidden = assets.size > 0;
  el("asset-count").textContent = String(assets.size);
  el("selected-count").textContent = `${selected.size} dipilih`;
  const all = el<HTMLInputElement>("select-all");
  all.checked = assets.size > 0 && selected.size === assets.size;
  all.indeterminate = selected.size > 0 && selected.size < assets.size;
  all.disabled = busy || assets.size === 0;
}

function renderKeywords() {
  const value = current()?.draft.keywords ?? [];
  const list = el("keyword-list");
  list.replaceChildren();
  el("keyword-count").textContent = `${value.length} keyword`;
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
      ["up", "Naikkan", -1],
      ["down", "Turunkan", 1],
    ] as const) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "keyword-move";
      button.title = `${label} ${keyword}`;
      button.setAttribute("aria-label", button.title);
      button.append(icon(direction));
      button.disabled =
        busy || index + step < 0 || index + step >= value.length;
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
    busy || !asset || !!asset.error;
  for (const id of ["add-files", "add-folder", "empty-add", "choose-output"])
    el<HTMLButtonElement>(id).disabled = busy;
  el<HTMLButtonElement>("reload").disabled = busy || selected.size === 0;
  el<HTMLSelectElement>("save-mode").disabled = busy;
  const batch = selected.size > 1;
  el("batch-notice").hidden = !batch;
  el("batch-title").textContent = `${selected.size} aset dipilih`;
  document.querySelectorAll<HTMLElement>(".batch-toggle").forEach((label) => {
    label.hidden = !batch;
  });
  el("apply-batch").hidden = !batch;
  el<HTMLButtonElement>("apply-batch").disabled = ![
    "title",
    "description",
    "keywords",
  ].some((field) => el<HTMLInputElement>(`apply-${field}`).checked);
  if (updateFields) {
    el<HTMLInputElement>("title").value = asset?.draft.title ?? "";
    el<HTMLTextAreaElement>("description").value =
      asset?.draft.description ?? "";
    el<HTMLTextAreaElement>("keywords").value =
      asset?.draft.keywords.join(", ") ?? "";
  }
  el("title-count").textContent =
    `${[...(asset?.draft.title ?? "")].length} karakter`;
  el("description-count").textContent =
    `${[...(asset?.draft.description ?? "")].length} karakter`;
  el("draft-status").textContent = !asset
    ? "Belum ada aset"
    : asset.error || asset.saveError
      ? "Perlu perhatian"
      : dirty(asset)
        ? "Belum disimpan"
        : "Siap";
  el("draft-status").classList.toggle("modified-text", !!asset && dirty(asset));
  el("editor-help").textContent = asset
    ? asset.name
    : "Pilih sebuah aset. Metadata yang sudah tertanam akan ditampilkan di sini.";
  const error = asset?.error || asset?.saveError;
  el("file-error").hidden = !error;
  el("file-error").textContent = error
    ? `${error} Muat ulang file jika sumber telah berubah.`
    : "";
  const saved = asset?.savedCopy || asset?.backupPath;
  el("save-result").hidden = !saved;
  el("save-result").textContent = asset?.savedCopy
    ? `Salinan: ${asset.savedCopy}`
    : asset?.backupPath
      ? `Backup: ${asset.backupPath}`
      : "";
  renderKeywords();
  const mode = el<HTMLSelectElement>("save-mode").value as SaveMode;
  el("choose-output").hidden = mode !== "copy";
  el("output-label").textContent =
    outputDir?.split("/").filter(Boolean).pop() ?? "Pilih folder keluaran";
  el("choose-output").title = outputDir ?? "Pilih folder keluaran";
  el("save-hint").textContent =
    mode === "copy"
      ? "Salinan baru menjaga file sumber tetap utuh."
      : "Backup .bak dibuat sebelum sumber diganti.";
  const pending = targets().length;
  el<HTMLButtonElement>("save").disabled =
    busy || pending === 0 || (mode === "copy" && !outputDir);
  el("save-label").textContent = busy
    ? "Memproses…"
    : mode === "copy"
      ? "Simpan metadata"
      : "Simpan + backup";
  if (!busy) el("progress-text").textContent = `${pending} perubahan dipilih`;
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
  el("preview-name").textContent = asset?.name ?? "Ruang preview";
  el("preview-detail").textContent = asset
    ? `${asset.format.toUpperCase()} · ${sizeText(asset.size)}`
    : "Pilih aset untuk melihat isinya";
  el("preview-format").textContent = asset?.format.toUpperCase() ?? "PREVIEW";
  el("preview-empty").hidden = false;
  el("empty-add").hidden = !!asset;
  el("format-list").hidden = !!asset;
  el("preview-caption").textContent = asset
    ? asset.path
    : "Aset tetap berada di perangkat Anda.";
  el("preview-state").textContent = asset
    ? "Membaca preview…"
    : "Menunggu aset";
  el("empty-title").textContent = asset
    ? "Membaca preview…"
    : "Semua aset, siap diberi cerita.";
  el("empty-description").textContent = asset
    ? "Metadata dan preview diproses secara terpisah."
    : "Tambahkan aset untuk menyunting title, description,\ndan keywords tanpa mengubah konten media.";
  if (!asset) return;
  const video = ["mp4", "mov"].includes(asset.format);
  el("empty-icon").setAttribute("href", video ? "#i-video" : "#i-image");
  if (video || asset.error) {
    el("empty-title").textContent = video
      ? "Metadata untuk setiap frame."
      : "Preview tidak tersedia";
    el("empty-description").textContent = video
      ? "Preview video belum tersedia.\nTitle, description, dan keywords tetap dapat disunting."
      : (asset.error ?? "");
    el("preview-state").textContent = video
      ? "Informasi video"
      : "File tidak dapat dibaca";
    return;
  }
  try {
    const bytes = await invoke<ArrayBuffer>("read_preview", {
      path: asset.path,
    });
    if (generation !== previewGeneration) return;
    previewUrl = URL.createObjectURL(
      new Blob([bytes], {
        type: asset.format === "svg" ? "image/svg+xml" : "image/png",
      }),
    );
    image.onload = () => {
      if (generation === previewGeneration) {
        image.hidden = false;
        el("preview-empty").hidden = true;
        el("preview-state").textContent = "Preview";
      }
    };
    image.onerror = () => {
      if (generation === previewGeneration) {
        el("empty-title").textContent = "Preview tidak dapat ditampilkan";
        el("empty-description").textContent = "Metadata tetap dapat disunting.";
        el("preview-state").textContent = "Tanpa preview";
      }
    };
    image.alt = `Preview ${asset.name}`;
    image.src = previewUrl;
  } catch (error) {
    if (generation !== previewGeneration) return;
    el("empty-title").textContent = "Preview tidak tersedia";
    el("empty-description").textContent = String(error);
    el("preview-state").textContent = "Tanpa preview";
  }
}

async function importAssets(paths: string[], reload = false) {
  if (busy || !paths.length) return;
  busy = true;
  el("progress-text").textContent = "Membaca metadata…";
  el<HTMLProgressElement>("progress").removeAttribute("value");
  render();
  try {
    const incoming = await invoke<Asset[]>("inspect_assets", { paths, refresh: reload });
    for (const asset of incoming) {
      if (!assets.has(asset.path) || reload)
        assets.set(asset.path, {
          ...asset,
          draft: copy(asset.metadata),
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
        ? `${incoming.length} aset dibaca${incoming.some((a) => a.error) ? ". Beberapa file perlu diperiksa." : "."}`
        : "Tidak ada format yang didukung dalam pilihan tersebut.",
    );
  } catch (error) {
    message(`Aset tidak dapat dibaca: ${error}`, true);
  } finally {
    busy = false;
    render(true);
    void loadPreview();
  }
}

async function addFiles(folder = false) {
  if (!requireDesktop() || busy) return;
  try {
    const paths = await open({
      directory: folder,
      multiple: !folder,
      title: folder ? "Pilih folder aset" : "Pilih aset",
      filters: folder
        ? undefined
        : [
            {
              name: "Aset microstock",
              extensions: ["jpg", "jpeg", "png", "svg", "eps", "mp4", "mov"],
            },
          ],
    });
    if (paths) await importAssets(Array.isArray(paths) ? paths : [paths]);
  } catch (error) {
    message(String(error), true);
  }
}

async function chooseOutput() {
  if (!requireDesktop() || busy) return;
  try {
    const path = await open({
      directory: true,
      multiple: false,
      title: "Pilih folder keluaran",
    });
    if (typeof path === "string") {
      outputDir = path;
      render();
    }
  } catch (error) {
    message(String(error), true);
  }
}

async function save() {
  if (!requireDesktop() || busy || !targets().length) return;
  const mode = el<HTMLSelectElement>("save-mode").value as SaveMode;
  if (mode === "copy" && !outputDir) {
    await chooseOutput();
    if (!outputDir) return;
  }
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
  el("progress-text").textContent = `0 / ${items.length} disimpan`;
  el<HTMLProgressElement>("progress").max = items.length;
  el<HTMLProgressElement>("progress").value = 0;
  render();
  try {
    const results = await invoke<SaveResult[]>("save_metadata", {
      request: { items, mode, outputDir },
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
      `${results.length - failed} aset tersimpan${failed ? `, ${failed} gagal. Lihat detail pada daftar aset.` : "."}`,
      failed > 0,
    );
  } catch (error) {
    message(`Penyimpanan gagal: ${error}`, true);
  } finally {
    busy = false;
    render(true);
  }
}

el("add-files").addEventListener("click", () => void addFiles());
el("empty-add").addEventListener("click", () => void addFiles());
el("add-folder").addEventListener("click", () => void addFiles(true));
el("choose-output").addEventListener("click", () => void chooseOutput());
el("save").addEventListener("click", () => void save());
el("save-mode").addEventListener("change", () => render());
el("metadata-form").addEventListener("submit", (event) => {
  event.preventDefault();
  void save();
});
el("select-all").addEventListener("change", () => {
  selected.clear();
  if (el<HTMLInputElement>("select-all").checked)
    assets.forEach((asset) => selected.add(asset.path));
  render();
});
el("reload").addEventListener("click", async () => {
  if (!requireDesktop()) return;
  if (
    [...selected].some((path) => dirty(assets.get(path)!)) &&
    !(await confirm(
      "Draft pada file terpilih akan diganti dengan metadata dari disk. Lanjutkan?",
      {
        title: "Muat ulang aset",
        kind: "warning",
        okLabel: "Muat ulang",
        cancelLabel: "Kembali",
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
    `Field terpilih diterapkan ke ${selected.size} aset. Simpan untuk menanamkan metadata.`,
  );
  render(false);
});
document.addEventListener("keydown", (event) => {
  if (event.ctrlKey || event.metaKey) {
    if (event.key.toLowerCase() === "o") {
      event.preventDefault();
      void addFiles();
    }
    if (event.key.toLowerCase() === "s") {
      event.preventDefault();
      void save();
    }
  }
});

async function initializeDesktop() {
  await listen<Progress>("save_progress", ({ payload }) => {
    el("progress-text").textContent =
      `${payload.completed} / ${payload.total} disimpan`;
    el<HTMLProgressElement>("progress").value = payload.completed;
  });
  const window = getCurrentWindow();
  await window.onDragDropEvent(({ payload }) => {
    if (payload.type === "drop") void importAssets(payload.paths);
  });
  await window.onCloseRequested(async (event) => {
    if (busy) {
      event.preventDefault();
      message("Tunggu hingga pemrosesan selesai sebelum menutup aplikasi.");
      return;
    }
    if ([...assets.values()].some(dirty)) {
      event.preventDefault();
      if (
        await confirm("Ada perubahan yang belum disimpan. Tutup aplikasi?", {
          title: "Perubahan belum disimpan",
          kind: "warning",
          okLabel: "Tutup",
          cancelLabel: "Kembali",
        })
      )
        await window.destroy();
    }
  });
}

if (native)
  void initializeDesktop().catch((error) =>
    message(`Integrasi desktop gagal: ${error}`, true),
  );
else el("runtime-label").textContent = "Pratinjau browser";
render(true);

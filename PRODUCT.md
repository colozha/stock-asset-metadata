# Stock Asset Metadata

<!-- impeccable:product-schema 1 -->

## Platform
web

Desktop Linux application using Tauri's webview; Fedora 44 is the initial verified target.

## Stack
Confirmed from the approved plan: Tauri 2, Rust, Vite, vanilla TypeScript, CSS, and Adobe XMP Toolkit bindings. SVG uses a small XML adapter. No application server or database is required.

## Users
Microstock contributors preparing existing photos, vectors, and footage for submission.

## Product Purpose
Read and embed title, description, and ordered keywords in JPG/JPEG, PNG, SVG, EPS, MP4, and MOV while protecting original media. Optional AI generation fills the active asset draft for review before saving.

## Operating Context
Metadata editing uses local files. Import files or recursively scan folders, skipping symbolic links. Folder roots persist locally and are rescanned at startup; direct file imports remain session-only. The sidebar groups assets by folder hierarchy and file format. Edit one asset or explicitly apply selected fields to several assets. Metadata drafts and optional per-asset AI context live in memory until the application closes. AI is optional; only an explicit generation, connection test, or model refresh contacts the configured provider.

## Capabilities and Constraints
- Resize both side panels with visible pointer and keyboard splitters; widths persist locally and clamp to the desktop viewport.
- One Settings save applies storage, language (ID/EN), theme (light/dark), and changed AI configuration. Incomplete AI configuration remains optional; generated metadata stays English.
- Save copies by default to the system Documents directory under stock-assets/{format}, grouping JPEG with JPG; fall back to ~/Documents when the system has no Documents configuration. Save mode and output root persist through Settings; optionally replace originals after creating versioned backups.
- Remove folder references without deleting disk files; missing folders retain a visible error until restored or removed.
- Browse Your assets and read-only Output through exclusive sidebar accordions, with independent filename search and persisted sorting inside folder/format groups. Refresh rescans imported roots, direct files, and the output directory while preserving metadata drafts and source-conflict protection. Output also refreshes after successful saves and output-directory changes.
- Work sequentially in a background worker, reporting results per file.
- Show lightweight image previews, embedded EPS TIFF previews, and video file information.
- SVG support is UTF-8. Unsupported or malformed files report errors without modifying sources.
- Offer optional Shutterstock CSV export before saving changed MP4/MOV videos, with one category per batch. Write one CSV per successful video output folder using final filenames and saved metadata; preserve existing CSVs with numbered suffixes. Other agency exports and video playback remain outside this version.
- Explicit Pixabay image search replaces the central preview during keyword research. Selected references persist across result pages; frequency-ranked tags are individually selected and appended to the current editable asset draft without reordering existing keywords. Output remains read-only.
- Pixabay requests send only query/filter/page data through Rust. Responses are cached separately for 24 hours, thumbnails are temporary, and source attribution links open through the OS. Each user supplies a personal Pixabay API key through Settings, committed by the shared Save settings action and stored in the OS credential store under a separate account. Missing keys disable new searches; builds do not embed Pixabay keys.
- Local readback does not establish agency ingestion or an Illustrator open/save roundtrip.
- Provider endpoint, model, active provider, and generator preferences are stored locally. API keys use the operating system credential store. Explicit AI requests send the prompt, optional per-asset context, and, for supported raster vision models, an image preview to the configured provider.

## Brand Commitments
Name: Stock Asset Metadata. Follow the user's supplied desktop-editor reference: square panels, soft cool colors, thin borders, muted purple selection, coral primary action. Indonesian and English interfaces, with Indonesian and light as the defaults. The light theme preserves the established palette; dark uses the same visual hierarchy.

## Product Principles
- Keep the application and dependency graph small.
- Prefer existing format handlers over binary writers.
- Never silently discard fields or reorder keywords.
- Validate completed output before replacing a source.
- Show actual capabilities and errors, with no fabricated results.

## Accessibility & Inclusion
Keyboard-accessible controls, visible focus, readable text contrast, explicit labels, and status text alongside color.

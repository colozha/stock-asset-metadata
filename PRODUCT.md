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
Read and embed title, description, and ordered keywords in JPG/JPEG, PNG, SVG, EPS, MP4, and MOV while protecting original media.

## Operating Context
Offline, local files. Import files or the direct children of a folder. Edit one asset or explicitly apply selected fields to several assets. Drafts live in memory until the application closes.

## Capabilities and Constraints
- Save copies by default; optionally replace originals after creating versioned backups.
- Work sequentially in a background worker, reporting results per file.
- Show lightweight image previews, embedded EPS TIFF previews, and video file information.
- SVG support is UTF-8. Unsupported or malformed files report errors without modifying sources.
- CSV agency exports and video playback are outside the first version.
- Local readback does not establish agency ingestion or an Illustrator open/save roundtrip.

## Brand Commitments
Name: Stock Asset Metadata. Follow the user's supplied desktop-editor reference: square panels, soft cool colors, thin borders, muted purple selection, coral primary action. Indonesian interface with familiar metadata field names.

## Product Principles
- Keep the application and dependency graph small.
- Prefer existing format handlers over binary writers.
- Never silently discard fields or reorder keywords.
- Validate completed output before replacing a source.
- Show actual capabilities and errors, with no fabricated results.

## Accessibility & Inclusion
Keyboard-accessible controls, visible focus, readable text contrast, explicit labels, and status text alongside color.

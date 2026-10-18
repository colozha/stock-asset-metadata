# Editor surface

Mode: Operate. Audience: microstock contributors editing local assets. Main action: safely save title, description, and ordered keywords for selected files. Follow DESIGN.md and the supplied reference. Build directly in code. Display real imported assets only. Inspect the complete editor at 1440×900 and 1024×700, including long names, batch fields, unavailable preview, and write errors. JPEG, PNG, EPS, and video save operations must not pass through a rendering or encoding pipeline. A desktop browser preview cannot replace native IPC verification.


The editor retains its three-column desktop relationship: persisted asset tree, real-media preview, and metadata form. Imported folder roots disclose recursive subfolders before format groups; JPEG appears under JPG. Keep disclosure summaries keyboard accessible, distinguish active assets from the checked save batch, and make root removal explicitly remove the workspace entry without deleting files.

Optional additional context belongs to the active asset and informs both AI generation actions. Keep it beside the generation preferences with a visible label and helper text.

Settings is an independently scrolling surface with appearance (ID/EN, light/dark), storage, and AI provider sections. One sticky Save settings action commits all sections; language/theme apply on save, and Back discards unsaved settings. AI is optional; incomplete configuration is saved without activating it. Partial save failures identify the completed and failed sections and retain unsaved AI inputs. Save mode and copy-output root are edited there. The default copy destination is the operating system Documents directory under `stock-assets/{format}`, with `~/Documents` as fallback; JPEG uses `jpg`. The editor footer shows a read-only mode/destination summary and the save action. Preserve the incumbent cool square panels, system typography, purple selection/focus, and coral save styling.

Use locally embedded official Lucide SVG paths with the established icon sizes and stroke weight, round caps and joins. All selects share a Lucide chevron, suppressed native appearance, explicit height/line height, and zero vertical padding to keep labels readable in WebKitGTK.

## Resizable panels and localization

Both desktop side panels have 7px vertical separators with a visible grip, purple hover/focus/drag state, and `col-resize` cursor. Pointer capture keeps dragging reliable outside the separator. Keyboard arrows adjust widths by 10px (Shift: 50px); Home/End select limits and double-click resets defaults. Widths persist in localStorage; minimums are 200px tree, 300px metadata, and 240px preview, clamped when the window shrinks. The browser fallback below 900px hides separators and keeps metadata below the preview.

Every application message has ID/EN entries in `src/messages.ts`; static copy uses `data-i18n`, dynamic copy uses `t()`, and application-owned IPC errors carry `{ key, params, detail }` for `errorText()`. OS/library/provider diagnostics remain literal. Metadata drafts, context, filenames, paths, provider/model identifiers, and generated English metadata do not change with the UI language. Light preserves the original palette; dark uses root CSS tokens and `color-scheme`. Legacy preferences default to Indonesian/light.

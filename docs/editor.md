# Editor surface

Mode: Operate. Audience: microstock contributors editing local assets. Main action: safely save title, description, and ordered keywords for selected files. Follow DESIGN.md and the supplied reference. Build directly in code. Display real imported assets only. Inspect the complete editor at 1440×900 and 1024×700, including long names, batch fields, unavailable preview, and write errors. JPEG, PNG, EPS, and video save operations must not pass through a rendering or encoding pipeline. A desktop browser preview cannot replace native IPC verification.


The editor retains its three-column desktop relationship: persisted asset tree, real-media preview, and metadata form. Imported folder roots disclose recursive subfolders before format groups; JPEG appears under JPG. Keep disclosure summaries keyboard accessible, distinguish active assets from the checked save batch, and make root removal explicitly remove the workspace entry without deleting files.

The sidebar has exclusive Your assets and Output accordions. The expanded section fills the available height; its tree scrolls while both headers remain accessible. Reuse the filename search, folder/format tree, and sorting controls with separate section state. Select all affects visible Your assets results and retains hidden selections with an explicit count. Output reads the configured output folder, shows its path, and shares previews and read-only metadata; it has no selection, AI, batch, or save actions. Returning to Your assets restores its active draft and selection. Refresh rescans both libraries without discarding drafts or renewing their conflict snapshots. Unavailable output has an explicit error and is never created by browsing.

Search starts hidden in both sections. A separate search icon follows the count in each header; it opens the section if collapsed, reveals search, and focuses the input. Clicking it again in the active section closes search and clears its filter. The inline clear button clears only the query. Visibility and query are independent per section during the session and reset at startup. Keep the search button separate from the accordion button, with localized tooltips, accessible names, and expanded/control attributes; preserve the 52px header and established theme tokens.

Optional additional context belongs to the active asset and informs both AI generation actions. Keep it beside the generation preferences with a visible label and helper text.

Settings is an independently scrolling surface with appearance (ID/EN, light/dark), storage, and AI provider sections. One sticky Save settings action commits all sections; language/theme apply on save, and Back discards unsaved settings. AI is optional; incomplete configuration is saved without activating it. Partial save failures identify the completed and failed sections and retain unsaved AI inputs. Save mode and copy-output root are edited there. The default copy destination is the operating system Documents directory under `stock-assets/{format}`, with `~/Documents` as fallback; JPEG uses `jpg`. The editor footer shows a read-only mode/destination summary and the save action. Preserve the incumbent cool square panels, system typography, purple selection/focus, and coral save styling.

Use locally embedded official Lucide SVG paths with the established icon sizes and stroke weight, round caps and joins. All selects share a Lucide chevron, suppressed native appearance, explicit height/line height, and zero vertical padding to keep labels readable in WebKitGTK.

## Resizable panels and localization

Both desktop side panels have 7px vertical separators with a visible grip, purple hover/focus/drag state, and `col-resize` cursor. Pointer capture keeps dragging reliable outside the separator. Keyboard arrows adjust widths by 10px (Shift: 50px); Home/End select limits and double-click resets defaults. Widths persist in localStorage; minimums are 200px tree, 300px metadata, and 240px preview, clamped when the window shrinks. The browser fallback below 900px hides separators and keeps metadata below the preview.

Every application message has ID/EN entries in `src/messages.ts`; static copy uses `data-i18n`, dynamic copy uses `t()`, and application-owned IPC errors carry `{ key, params, detail }` for `errorText()`. OS/library/provider diagnostics remain literal. Metadata drafts, context, filenames, paths, provider/model identifiers, and generated English metadata do not change with the UI language. Light preserves the original palette; dark uses root CSS tokens and `color-scheme`. Legacy preferences default to Indonesian/light.

## Pixabay keyword research

Mode: Operate, extending the incumbent editor. The Keywords action switches only
the central panel between local preview and reference search. Query/filter edits
wait for Enter/Search; search controls and pagination remain outside the scrolling
grid. Cards use small, uncropped thumbnails, native checkboxes, contributor text,
and a separate source link. The grid collapses to one column at the central
panel's 240px minimum. The side-panel resize contract remains unchanged.

Selected images persist across pages and local-asset changes. Suggestions start
unchecked; labels combine coverage counts, high/medium/low text, and theme tokens.
The add action identifies the active target and appends to its draft. Existing
keyword text/order, metadata fields, and additional context are preserved. A new
submitted query/filter resets references; Back to preview clears the research
session. Settings and Output suspend it; Output offers no add action. Language
and theme changes preserve references and retranslate status. Delayed responses
cannot replace a newer query or reopen a closed session.

Keep reference cards square and flat, with the existing thin borders and purple
selected/focus treatments. Search, pagination, and adding tags use secondary
controls; the coral action remains Save metadata. Attribution stays visible in
the panel footer, and each card's source link is separate from its checkbox.
Thumbnail failures retain the reference label and source link.

Coverage labels use the approved light/dark `relevance-high-*`,
`relevance-medium-*`, and `relevance-low-*` background/ink pairs only on the count
badge. High means at least 75% of selected unique images, medium 40% to below
75%, and low below 40%; every label includes the count/total and localized level.
A single selected image gives each of its tags 1/1 coverage. Suggestions rank by
frequency with stable first-seen ties; this describes consistency across chosen
references. Reference selection, suggestion selection, and the local save batch
remain separate controls. Adding selected suggestions skips duplicates, reports
the result, and requires the existing Save metadata action to write the draft.

The Pixabay Settings section follows storage and reuses the AI password-row,
show/hide and pending-delete patterns. One Save settings action commits all
sections and its status remains visible beside that action; blank input preserves the saved key, Back cancels replacement/deletion,
and successful saving clears and masks the input. The stored key is never read
back to the renderer. App preferences, AI and Pixabay report partial save results
independently while retaining failed inputs. Saving does not make a Pixabay HTTP
request. Missing credentials expose Set Pixabay key, opening Settings with focus
on its input. Deleting the key disables new searches but keeps existing reference
selection, suggestions and metadata drafts. Personal credentials use the OS store;
no build-time key or shared-key fallback is supported.

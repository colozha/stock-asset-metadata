---
name: Stock Asset Metadata
description: Local multimedia metadata workbench with cool square panels.
colors:
  bg: "#eaf1f3"
  panel: "#f7fafb"
  canvas: "#cbdde1"
  border: "#a7bacb"
  text: "#2f3f54"
  muted: "#506379"
  purple: "#7b68a4"
  coral: "#d06d6d"
  white: "#ffffff"
  primary-ink: "#271d25"
  coral-hover: "#bb565b"
  primary-disabled: "#e5bcbc"
  primary-disabled-ink: "#6e444b"
  secondary-hover: "#e1eaf0"
  secondary-active: "#d6e2eb"
  selected-surface: "#e2dced"
  selected-border: "#b0a2ca"
  preview-muted: "#475f72"
  keyword-surface: "#f8fafb"
  keyword-border: "#bdcbd6"
  field-disabled: "#f5f8fa"
  modified: "#6f528e"
  error: "#913838"
  saved: "#36564a"
typography:
  body:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: "13px"
    fontWeight: 400
    lineHeight: "normal"
  brand:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: "14px"
    fontWeight: 750
    letterSpacing: "0.045em"
  title:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: "13px"
    fontWeight: 700
  label:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: "12px"
    fontWeight: 650
  helper:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.75
  detail:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: "10px"
    fontWeight: 400
  badge:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: "9px"
    fontWeight: 400
    letterSpacing: "0.07em"
  headline:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: "20px"
    fontWeight: 550
    lineHeight: 1.4
    letterSpacing: "-0.02em"
  headline-wide:
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: "23px"
    fontWeight: 550
    lineHeight: 1.4
    letterSpacing: "-0.02em"
rounded:
  square: "0px"
  control: "1px"
spacing:
  unit: "4px"
  label: "8px"
  gap: "10px"
  inset: "12px"
  panel: "16px"
  compact-preview: "20px"
  preview: "30px"
  wide-preview: "42px"
components:
  button-primary:
    backgroundColor: "{colors.coral}"
    textColor: "{colors.primary-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "9px 14px"
  button-primary-hover:
    backgroundColor: "{colors.coral-hover}"
    textColor: "{colors.white}"
  button-primary-disabled:
    backgroundColor: "{colors.primary-disabled}"
    textColor: "{colors.primary-disabled-ink}"
  button-secondary:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.text}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "7px 12px"
  button-secondary-hover:
    backgroundColor: "{colors.secondary-hover}"
  button-secondary-active:
    backgroundColor: "{colors.secondary-active}"
  field:
    backgroundColor: "{colors.white}"
    textColor: "{colors.text}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "10px 11px"
    width: "100%"
  field-disabled:
    backgroundColor: "{colors.field-disabled}"
    textColor: "{colors.muted}"
  asset-row:
    textColor: "{colors.text}"
    rounded: "{rounded.square}"
    padding: "9px 8px"
  asset-row-active:
    backgroundColor: "{colors.selected-surface}"
  keyword-row:
    backgroundColor: "{colors.keyword-surface}"
    textColor: "{colors.text}"
    rounded: "{rounded.square}"
    padding: "2px 5px 2px 7px"
---

# Design System: Stock Asset Metadata

## Overview

**Creative North Star: "Cool square panels"**

A local multimedia metadata workbench with restrained blue-gray materials, thin rules, muted purple selection, and a coral save action. The user's supplied desktop-editor reference is the visual authority.

Density supports repeated editing: compact controls and labels, clear panel boundaries, and space around the asset itself. State changes use text, modest color shifts, and visible focus. All application copy supports Indonesian and English. User metadata stays unchanged; generated metadata stays English.

**Key Characteristics:**
- Cool surfaces and square panel geometry.
- Compact system typography and one-pixel rules.
- Purple for selection and focus; coral for saving.
- Quiet, explicit feedback for drafts, saves, and errors.

## Colors

The palette is cool and subdued, with limited warm emphasis. Frontmatter values are the canonical extracted tokens; component snippets retain incidental border and surface values from the source.

### Primary

- **Coral** (`coral`): the save action, with dark `primary-ink` at rest. Hover uses `coral-hover` with white text. Disabled saving uses its own opaque muted pair.

### Secondary

- **Muted Purple** (`purple`): focus outlines, checkbox accents, carets, and progress. The active asset uses the pale `selected-surface` with `selected-border`; modified draft text uses `modified`.

### Neutral

- **Blue-gray Ground** (`bg`), **Cool Panel** (`panel`), and **Preview Canvas** (`canvas`): the application surround, toolbar/save surfaces, and central media stage.
- **Panel Rule** (`border`): structural borders and default control outlines.
- **Ink** (`text`), **Secondary Ink** (`muted`), and **Preview Secondary Ink** (`preview-muted`): body text, supporting labels/keyword ordinals, and helper text on the darker preview canvas.
- **White** (`white`) and **Disabled Field** (`field-disabled`): editable and unavailable field surfaces.
- **Secondary Hover / Active**: understated button state fills. **Keyword Surface / Border**: the ordered keyword rows.
- **Error** (`error`) and **Saved** (`saved`): explicit feedback text, always accompanied by a written result or next action.

**The Coral Save Rule.** Reserve the coral fill for the save action; use cool controls for import, navigation, and editing.

## Typography

Use the operating system sans stack throughout. There is no external font or separate display face.

- **Body:** the base control and input role (`body`).
- **Brand:** the compact application identifier (`brand`); its tracking belongs to the identifier, not form copy.
- **Title:** panel headings (`title`). **Label:** field names and the save action (`label`).
- **Helper / Detail:** explanatory copy and compact metadata (`helper`, `detail`). Counts and keyword ordinals use tabular numerals.
- **Badge:** short uppercase format names (`badge`).
- **Headline:** the central empty-state message (`headline`), switching to `headline-wide` at the wide-preview breakpoint. It is a modest workbench message, not a marketing hero.

Description and keyword textareas use a readable line height (1.65). Long error strings and keyword phrases wrap; asset names and short preview identifiers truncate with ellipsis.

**The Working Type Rule.** Separate working roles through size, weight, and spacing within the same system font stack.

## Layout

The native Linux window starts at (1440×900) and has a minimum size of (1024×700). The desktop shell fills the viewport, with outer padding (12px), a toolbar row (61px), a flexible workspace, and a save row (86px), separated by gaps (10px). Toolbar and save controls remain visible while the asset tree and metadata form scroll independently. Settings replaces the workspace with an independently scrolling, centered panel (maximum width 760px); the editor footer is hidden there.

The default workspace is three content columns: asset tree (240px), flexible preview, metadata editor (340px), with 7px separators between them. Side panels resize independently through pointer drag and keyboard arrows (Shift: 50px, otherwise 10px; Home/End: limits). Purple hover/focus/drag indicators and a centered grip expose the interaction. Double-click resets default widths; localStorage preserves custom widths. Minimum widths are 200px for the tree, 300px for metadata, and 240px for the preview; the layout clamps to the available window. At widths up to (1180px), side columns become (210px) and (310px); the preview keeps the remaining space. Metadata insets reduce from (16px) to (14px). The preview stage uses (30px) padding by default, (42px) from (1200px), and (20px) in the compact desktop layout.

The base spacing unit is (4px), with observed optical adjustments for compact control gaps and borders. Preserve actual component padding rather than forcing every measurement to a multiple of four. Panel headings align on a common height (52px).

Below (900px), the browser inspection fallback moves metadata beneath a two-column list/preview and wraps the toolbar/footer. This is not a native mobile shipment requirement; the desktop window minimum remains binding.

## Elevation & Depth

Depth comes from cool surface changes, thin rules, and selected fills. The shipped interface uses no shadows or decorative gradients. Feedback messages are flat bordered surfaces. Controls transition only background and border color (140ms, ease); there is no entrance animation or movement. Reduced-motion preference removes transitions.

**The Flat Panel Rule.** Define regions with cool surface tones and one-pixel rules; do not add shadows.

## Shapes

Panels, asset rows, keyword rows, badges, and notices are square. Form controls use the minimally softened control radius. The user's approved geometry stays within (0–2px); the implemented reusable radii are the frontmatter's square and control values. Borders are (1px).

Icons use locally embedded official Lucide SVG paths with round caps and joins, normally (18px) and (1.6px) stroke. Reorder icons reduce to (14px). Keep the square panel geometry independent of the icon stroke treatment; use actual SVG paths rather than text glyphs or icon fonts.

## Components

### Buttons

Secondary controls use cool panel fill, the shared thin border, and compact padding. Hover darkens the fill and border; press uses the secondary active fill. Disabled secondary buttons reduce opacity (0.5).

The primary save button uses coral with dark text, a stronger coral border, and a minimum height (37px). Hover switches to darker coral with white text. Its disabled state has explicit colors at full opacity. All actionable controls share the purple focus outline (2px), offset by (2px).

### Inputs / Fields

Text fields and textareas use white fill, the common border, and compact inset padding. Purple marks carets and keyboard focus. Fieldsets become visibly cool and muted when editing is unavailable. Textareas resize vertically and have a minimum height (70px). Labels remain explicit; placeholders provide examples rather than replacing labels.

All selects use a local Lucide SVG chevron, `appearance: none`, explicit height and line height, and zero vertical padding to prevent native WebKit text clipping. Settings selects use height (37px) and line height (35px); compact AI preferences use height (32px) and line height (30px). Preserve this shared treatment across the editor and Settings.

The optional additional-context textarea sits with the AI generation preferences. Its explicit label and helper text identify context for both generation actions on the active asset.

### Navigation

Asset navigation is a scrollable, recursive folder tree with keyboard-accessible disclosure summaries. Imported roots persist across restarts; subfolders precede format groups, with JPEG grouped as JPG. Each root has a separate remove control that removes it from the workspace without deleting files. Leaf assets retain square rows with an independent save-batch checkbox and an asset button. The active row has a pale purple fill and border; the asset button itself stays transparent. A filename, format/size detail, and written draft/save/error state establish the row hierarchy. Long filenames truncate. The active asset and checked save batch remain distinct concepts.

### Settings / Save Summary

Settings places appearance controls (ID/EN language and light/dark theme) before storage and AI provider controls, using the same cool bordered panels, explicit labels, and restrained spacing. Save mode and the copy-output root are edited in its storage section. One sticky Save settings action commits all sections. Language and theme apply only on save; Back discards unsaved settings. AI remains optional and incomplete provider configuration is not activated. Partial persistence failures identify the saved and failed sections and preserve unsaved inputs. The editor footer presents the saved mode and destination as a read-only summary beside the save action. Long destination paths must remain readable through wrapping or their full-path affordance.

### Ordered Keywords

Keywords are numbered rows with separate up/down SVG buttons. Ordinals use secondary ink and tabular numerals; the phrase wraps. First/last reorder controls disable when they cannot move. Preserve phrase boundaries and keyword order.

### Badges / Containers

Format badges are small square outlined labels. They are informational, not interactive chips. Batch notices are pale purple bordered panels with a compact title and explanation. Metadata errors and save results use flat tinted blocks, written status, and wrapping paths; color alone never carries the result.

## Do's and Don'ts

### Do:

- **Do** preserve the cool square-panel identity and thin structural rules.
- **Do** reserve purple for selection/focus and coral fill for the save action.
- **Do** keep the toolbar, asset navigation, central preview, metadata form, and save controls in their established desktop relationship.
- **Do** retain explicit labels, visible keyboard focus, readable supporting text, and written status.
- **Do** use real imported media, preserve keyword order, and keep metadata editing available when only the preview fails.

### Don't:

- **Don't** add decorative shadows, gradients, pill panels, external fonts, or entrance animation.
- **Don't** use white text on the resting light coral action; white belongs to its darker hover state.
- **Don't** make preview helper text or keyword ordinals lighter than their documented ink roles.
- **Don't** restore native select styling or vertical padding that clips select labels.
- **Don't** infer a mobile product contract from the browser inspection fallback.

## Theme and localization

Light retains the established palette. Dark uses cool charcoal panels, pale blue-gray ink, purple focus/selection, and coral save actions. All component colors use CSS tokens, including browser controls, selection, scrollbars, disabled states, errors, and previews. `data-theme` and `color-scheme` on the document root choose the palette; explicit light is the default rather than the OS preference.

Use the typed ID/EN message dictionary for static and dynamic application copy, accessibility text, dialogs, and backend errors. Language changes rerender labels without changing metadata drafts or keyword order. Source filenames, provider/model identifiers, external diagnostics, and metadata content remain literal.

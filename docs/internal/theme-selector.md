# Visual theme selector

`argvus theme-selector` dispatches to the private GTK 4 helper shipped by `argvus`.
`argvus theme-switch` without an ID delegates before acquiring the switch lock.
Explicit IDs continue through the existing config transaction. No schema or theme
manifest changes are needed. Control Panel transient services and the Hyprland
shortcut keep using the public command; no Control Center screen was migrated.

The catalog reuses theme-core discovery, registry palette defaults and manifest
validation, retaining each installed theme's system or user origin. Stable IDs
preserve selection across search and category filters: Dark and Light contain
system themes of that category, while Custom contains all user-installed themes.
The canonical `/layout/variant` determines control and preview corner geometry.
Sticky is square; Spaced is rounded. A worker observes changes while the selector
is open. Search controls use the same responsive width as the central preview.
The selector uses scoped monospace styling consistent with ARGVUS utilities.
Only the current theme and unique neighbors are loaded. A worker drains obsolete
requests; the UI discards images outside the current neighborhood. Thumbnails
are decoded off the GTK thread with the workspace image library. JXL conversion
uses bounded `djxl` execution; derived PNGs are cached by path, size and mtime in
`$XDG_CACHE_HOME/argvus/theme-selector/v2`, with up to 128 cached images. GTK paints
premultiplied native ARGB surfaces, without requiring desktop image loader plugins.
A missing or corrupt preview falls back to the wallpaper and a palette illustration.
No preview writes canonical configuration or generates a theme build.
The carousel receives focus on open and reactivation. `/` toggles focus between
search and carousel without clearing its query; arrows edit the query when search
is focused and navigate the carousel otherwise.

Application is asynchronous and serialized by the existing backend. The selector
stays open on error and rereads effective state. It closes after success. The
existing Control Panel transient service isolates the operation from panel restarts.

## Validation checkpoint

The selector, config, theme-core and dispatcher Rust tests passed, together with
workspace check, scoped Clippy, repository packaging checks and catalog validation.
The appearance integration suite passed 49 of 50 tests. Its existing
`test_theme_switch_resets_spacing_to_mode_defaults` failure also reproduces with
`/usr/bin/argvus` installed before this change: expected bottom-bar geometry
`top = 2, right = 5, bottom = 0, left = 4`, observed
`top = 0, right = 5, bottom = 3, left = 4`. This unrelated spacing behavior is
outside the selector change.

A Hyprland 0.57.2 session with one 1920×1080 output at scale 1 verified overlay
opening, keyboard navigation and search, Escape, Portuguese and RTL translations, single instance, duplicate
confirmation suppression, simulated application failure and retry success.
The backend transaction itself is covered by isolated integration tests; session
service restarts, multiple physical outputs and fractional scaling still require
hardware validation. No installed package or user appearance state was replaced.

Release builds of the CLI and selector completed. A staged install verified the
private-helper dispatcher and all catalogs at their runtime paths. The full
release `argvus theme-switch` route opened and closed the staged selector without
changing appearance state. The package recipes include the helper in their normal
build/install lists and declare `libjxl`; no `.pkg.tar.zst` was generated or installed.

## Application feedback

The preview overlay follows the central card geometry. Applying displays a dim
layer, a GTK spinner and the existing localized application label. The existing
busy state blocks repeated confirmation and navigation. Failure stops the spinner,
hides the layer and restores controls; success closes the window.
Search focus uses the effective appearance accent, falling back to the active
manifest palette. The dropdown owns one visible border on its internal button.

Scoped check, Clippy and five selector tests passed for this refinement. A real
Hyprland session verified the centered busy overlay and simulated failure recovery
and success, without applying a desktop theme. Multiple outputs and fractional
scaling still need hardware validation.

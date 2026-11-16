# Eight layout presets

`argvus layout-mode-menu` dispatches to the GTK helper `layout-mode-selector`.
The helper shares catalog, bounded wallpaper decoding, theme styles and monitor
discovery with the theme selector. No Rofi process is involved.

The picker uses six previews: Sticky and Spaced in the left column, with four
window and taskbar combinations in a right-hand grid. The public typed model is
`argvus_config_core::layout_preset::LayoutPreset`. Canonical values remain
`/layout/variant`, `/hyprland/window_mode` and `/layout/taskbar/position`.
No new configuration schema or public command was added. `float` remains a
legacy alias for Spaced. Array positions are UI indices, not persistent IDs.

The coordinator `layout_preset::apply` holds its own cross-process preset lock,
then calls geometry, taskbar position and window mode backends in that order.
Geometry resets the bar to Top, so applying position afterwards is essential.
The existing per-domain locks remain in place; the outer coordinator never holds
the theme lock while launching a theme transaction. The shell geometry backend
now propagates a failed theme transaction instead of writing a success marker.

The final canonical tuple is reread and compared with the requested preset.
Window-mode runtime status distinguishes Applied from Pending. A saved preference
without a matching compositor state stays open and is labelled Saved/Pending.
Partial failure leaves changes visible and allows retry, without rolling back
window placement. During apply, the spinner overlays the last focused submode preview.

Illustrations reuse the current wallpaper without flipping it. Tile windows are
side by side; Floating windows overlap and include the opposite application bar.
The four right-hand cards each show a unique window and taskbar combination;
the left cards follow the selected combination. These are illustrative
layouts, not live application screenshots.

The package installs the helper without new dependencies. Documentation and
appearance strings are synchronized in every existing locale. Tests cover tuple
round trips, legacy names, all eight application orders, failure short circuiting,
runtime pending status, distinct preview pixels and wallpaper orientation. UI
tests cover separate vertical mode navigation, the non-wrapping horizontal
submode sequence, four complete window/taskbar combinations, inactive-mode opacity,
focus-only Tab movement, and output sizing.

Validation used isolated configuration and mocked runtime commands for all eight
coordinator combinations, including real theme reapplication and final Bottom
position. A real Wayland session displayed the two-column UI with isolated XDG
state and a mock compositor endpoint; keyboard selection, pending status,
failure/retry and cancellation were exercised without changing the user's saved
layout. Unit tests cover navigation, focus, opacity and sizing. These simulations
do not replace hardware validation of multiple physical outputs and fractional
scaling.

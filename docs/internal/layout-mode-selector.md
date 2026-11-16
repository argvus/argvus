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
then prepares geometry and the final taskbar edge before rendering once, followed
by window-mode application. An unchanged variant skips theme reapplication; a
position-only change uses the taskbar backend. A failed render is retried even
if the canonical variant was already saved.
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
tests cover separate vertical mode navigation, spatial navigation within the focused
submode grid, four complete window/taskbar combinations, inactive-mode opacity,
focus-only Tab movement, and output sizing.

Validation used isolated configuration and mocked runtime commands for all eight
coordinator combinations, including real theme reapplication and final Bottom
position. A real Wayland session displayed the two-column UI with isolated XDG
state and a mock compositor endpoint; keyboard selection, pending status,
failure/retry and cancellation were exercised without changing the user's saved
layout. Unit tests cover navigation, focus, opacity and sizing. These simulations
do not replace hardware validation of multiple physical outputs and fractional
scaling.

Main previews retain a 16:9 aspect ratio; submode previews are 75% of their
size and vertically centered. All unselected cards use 40% opacity. The apply
spinner is centered at 20 logical pixels without expanding to fill its cover.

Initial focus is on the selected main mode. Tab and Shift+Tab toggle groups
without changing selection. Arrows stay within the focused group, stopping at
grid boundaries; Up/Down select the main mode, while all four arrows navigate
the 2×2 submode grid. Enter applies and Escape closes.

Selector-initiated application propagates `ARGVUS_SELECTOR_APPLY=1` to the
runtime controller, which leaves progress to the selector instead of starting
a session splash. The coordinator skips geometry/theme reapplication when the
variant is unchanged and skips unchanged taskbar position writes in that case.

Performance: sessionctl batches required surface restarts in one systemd
transaction. Theme consumers reload once after the final files and accent build
are ready. Existing content-based builds and unchanged-file writes remain in
use. `session.log` records `theme.prepare`, `theme.render`, `theme.project` and
`theme.runtime` timings, alongside the existing session timings.

Measured with debug binaries and isolated state (desktop commands mocked):
three surface restart calls became one; seven-run median orchestration time
changed from 42.1 ms to 35.6 ms. Theme application with the real session planner
and mocked desktop commands used two Hyprland theme hooks before and one after;
three samples were 4.383/4.163/4.202 s before and 4.115/4.006/4.006 s after.
These are integration measurements, not production desktop latency guarantees.
The default-accent finishing step also reuses the transaction's existing build;
custom accents retain their separate render. Source edits still invalidate the
existing content-based build cache.

The coordinator keeps `data/internal/layout-preset.pending` until runtime
application is confirmed. A retry replays geometry/runtime work even if the
canonical tuple and theme build were already saved by the failed attempt.

A repeat with the final debug build measured 4.164/3.912/3.968 s before and
3.926/4.192/4.010 s after under the same mocked runtime. End-to-end timing did
not show a stable improvement above run-to-run variation; the verified gains
are fewer hooks/restart requests and skipping full theme work for submode-only
changes. Measure installed hardware latency through the recorded phase timings.

Validation for this change: 269 configuration tests and 11 selector tests,
Clippy with warnings denied, session validation (13 orchestration tests),
targeted appearance integration tests, and all eight preset tuples using
isolated state and mocked compositor/systemd commands. A real Wayland selector
remained mapped during a delayed simulated failure, displayed inline progress,
and accepted Escape after the failure. No real layout change was applied.

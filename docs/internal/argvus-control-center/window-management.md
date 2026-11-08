# Floating window controls and recovery bar

The implementation extends the global `/hyprland/window_mode` preference. There
is no second mode setting. `argvus-config::windows` implements the in-process
`argvus windows` command, its session service, the secondary Waybar supervisor,
and directional taskbar positioning.

## Ownership and transitions

- Persistent preference: canonical config store. Position is
  `/layout/taskbar/position`; the secondary position is always its opposite.
- Session state: an atomic JSON registry under `$XDG_RUNTIME_DIR/argvus`, scoped
  by a hash of the full compositor instance signature. Each record contains the
  address, PID, initial class, workspace, monitor coordinates, geometry,
  fullscreen/client state and pin state. Close events remove records explicitly
  to prevent fast address reuse; reconciliation handles missed events.
- Minimize parks a window (or its group) on `special:argvus-minimized`, after
  recording its origin. It requires a current frontend heartbeat and a mapped
  recovery bar on the target monitor. Restore returns to the original workspace;
  named workspaces use Hyprland's `name:` selector. Missing outputs use a remaining
  monitor, with translated/clamped geometry that stays inside its usable area.
- Tile freezes new minimizations and restores all records before projecting or
  applying the new mode. Recovery remains accessible on failure. A later Floating
  selection enables minimization again. Configuration reloads reconcile the
  state even when the change did not pass through the interactive command.

## Runtime interfaces

`argvus windows service` serializes requests over an owner-only Unix socket and
subscribes to Hyprland events. `list --json` returns the current snapshot;
`watch --json` streams changes. Snapshots carry windows, translated context-menu
labels, mode, primary position, readiness and pending recovery. CLI actions use
validated window addresses; workspace names are escaped as Lua bytes, never
passed through a shell. Visible failures show a translated notification and
retain technical details in logs.

`argvus windows bar` supervises its own Waybar child, dormant in Tile unless
recovery is pending. `argvus-windows.service` and `argvus-window-bar.service`
belong to the session target; their process groups are stopped with the session.
The bar runner recreates only its own surface for mode/position/geometry changes.
`position top|bottom` projects the canonical geometry and restarts the principal
bar when necessary; `--persist` skips the runtime restart. The legacy spacing
script delegates position changes and resets, while keeping its other geometry
operations. The Control Center and directional shortcuts use the Rust route.

The native `hyprland/argvus-windows` module is compiled into argvus-waybar using
its existing pinned upstream source and patches. A worker reads the snapshot
stream; GTK work stays on the main thread. It creates one button per window,
filters by the current workspace of each output (or its open special workspace),
collapses hidden group tabs and minimized groups, keeps minimized windows
associated with their original workspace/output, and sends actions
as argument arrays. Unknown or broken application icons fall back to a symbol
without invoking GTK's fatal missing-image path.

## Decorations and packaging

argvus-hyprland now ships the hyprbars plugin on x86_64, pinned to Hyprland 0.57.0.
The upstream revision is `d2427d4d15b36df40468f5a58bbd67cdba19835d`; its archive
checksum is fixed in both PKGBUILDs. The small downstream patch captures the
clicked window address before command execution and exposes button clearing for
idempotent theme refresh. The upstream BSD-3-Clause license is shipped with the
package, alongside the existing project license.

Floating enables three uniform title-bar buttons and double-click maximization;
Tile and fullscreen hide the additional bars. Existing application-drawn
controls are intentionally retained. The `window-controls` consumer derives title colors from the active theme.
The secondary bar imports the effective primary stylesheet and adds only
window-list rules; it shares the primary taskbar theme consumer. Its height and
margins derive from the effective primary JSONC, with vertical edges exchanged. `argvus windows decorations`
refreshes decorations after a theme build; no user-home plugin compilation or
hyprpm installation is involved.

## Validation checkpoint

Implemented and verified:

- Scoped configuration/CLI/settings/appearance Rust tests and strict scoped
  Clippy; Rust formatting. Release dispatcher builds include the service.
- Hyprbars compilation and loading against installed 0.57.0 headers, and native
  Waybar compilation with all existing ARGVUS patches plus the new module.
- Real pointer clicks on minimize, maximize, close and the recovery-bar entry
  inside an isolated nested compositor, including correct window targeting.
- Real-window minimization/restoration, geometry/maximization, service restart,
  groups (including fullscreen) and return to Tile with pending minimizations.
  Removed-monitor geometry is additionally covered by a pure Rust regression.
- Startup directly in Floating, then return to Tile, preserves application
  dialogs that opened floating. The policy remembers their observed origin
  across transient decoration/layout refreshes; Lua regressions cover pending
  application rules and transient floating-state changes.
- Opt-in regression `tests/test_window_controls.py` in argvus-hyprland launches
  its own compositor/services/Waybar and removes its process groups afterward.
  Provide `ARGVUS_TEST_ARGVUS`, `ARGVUS_TEST_HYPRBARS`, `ARGVUS_TEST_WAYBAR`, and a
  parent Wayland display. The ordinary test run skips it without built artifacts.
- Theme rendering across official themes/layouts, i18n parity/validation,
  package metadata and staged payloads, unit syntax and documentation references.

A follow-up nested multi-output readiness test could not configure a usable
headless output (reported size 0x0). The backend correctly refused to hide its
windows while that output had no recovery surface. Multi-output frontend hotplug
therefore remains unverified in that environment.

Installation into the active desktop, package archive generation, physical
monitor hotplug and a real logout/login remain outside the validation performed
here. The nested tests exercise virtual monitors rather than physical hardware.

## Spaced floating bounds

The service reads effective compositor gaps and border/title dimensions, then
clamps floating windows to logical monitor bounds after accounting for both
bars, output scaling and rotation. True fullscreen is excluded. ARGVUS maximize
in Spaced fits this rectangle without compositor fullscreen; original geometry
is saved atomically in a separate `maximized.json` runtime registry for recovery
across service restarts. Preparing Tile restores that geometry before tiling.
Targeted window operations temporarily enable `cursor.no_warps` in the same IPC
evaluation and restore its prior value even on dispatcher errors. This also
covers implicit focus changes during minimize/restore and fullscreen restoration.

The secondary bar's reserved inner margin is subtracted from its effective outer
gap, just as the existing primary-bar adapter does. Otherwise an 18 px bar
margin plus an 18 px gap produces a 36 px space on that edge. The regression
checks the visible window-to-bar distance on both edges, with the primary bar
at either the top or bottom.

The Control Center entry point is now Desktop & Input > Hyprland > Windows >
Mode. Rules and Spaces are sibling pages; all use the same shared mode backend
and preserve the Windows parent when entered through search or CLI aliases.

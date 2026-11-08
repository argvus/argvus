# Global window policy

The canonical `/hyprland/window_mode` preference belongs to `argvus-config`.
The settings page and GTK popup call the same backend. `window-mode set`
serializes save, Lua projection and compositor IPC under an advisory lock;
ordinary config-store mutations retain their existing lock. Generated state is
`data/generated/hypr/window-mode.lua`; the Lua module owns runtime transitions.

The selector lives at Desktop & Input > Hyprland > Windows > Mode, alongside
Rules and Spaces. The legacy `settings/window-rules` search route and
`window-rules` CLI alias still open application Rules, with the new breadcrumb.
Global mode search opens its dedicated page and matches tile, floating and their
Portuguese terms. Floating decorations use the packaged hyprbars plugin.

The Lua module captures tiled origins after normal application rules. Runtime
records are scoped by compositor signature, window address, PID and initial
class. Close removes a record; reload loads and reconciles surviving records.
Pinned windows keep their explicit pin when returning to Tile. Layer-shell
surfaces are outside this policy. Fullscreen transitions are delegated to the
compositor's floating dispatcher.

`status --json` exposes `preferred`, nullable `applied`, `available`, `pending`
and nullable technical `error`. Interfaces translate presentation and keep raw
errors in diagnostics. A failed application must never be displayed as success.
The taskbar's `--status` adapter emits Waybar text, tooltip and CSS class.

## Validation checkpoint

- Scoped Cargo tests: 601 passed (configuration, settings, CLI and its integration
  tests); the Control Center consumer compiles.
- Scoped Clippy with `--all-targets --no-deps -- -D warnings` and formatting pass.
  The initial dependency-inclusive Clippy run found the pre-existing
  `too_many_arguments` warning in `argvus-tui::page::shell_with_status`.
- The popup builds in release. Master-package shell/metadata checks pass; no
  package installation or package archive generation was performed.
- Locale catalogs validate in en-US and pt-BR. The appearance tests render all
  official themes in both layouts and preserve the new utility-group module.
- Lua lifecycle regression runs with simulated windows in Hyprland's embedded
  runtime using `python3 -m unittest discover -s tests` in argvus-hyprland.
- An isolated nested Hyprland 0.57.0 session passed real-client Tile/Floating
  transitions, natural float rules, new-window conversion, configuration reload,
  maximization/fullscreen preservation, grouped-window restoration, normal/special
  workspace ownership, two virtual outputs, explicit pin preservation, backend
  status and popup keyboard selection/dismissal. The active
  desktop's windows and persistent configuration were not used as test fixtures.
- Physical multi-monitor changes and logout/login have not been exercised.
  Exact tiling split-tree restoration remains outside this feature. The additional window controls and recovery bar are described in [Window management](window-management.md).

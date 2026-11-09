---
title: ARGVUS Configuration
description: The logical JSON configuration contract used by ARGVUS.
---

ARGVUS keeps managed user preferences in modular JSON files under `$XDG_CONFIG_HOME/argvus/config/` (normally `~/.config/argvus/config/`). They form the canonical logical model for preferences managed by ARGVUS, and `argvus-config` is the only component that writes files below `data/generated/`. Native files and generated files remain compatibility projections or application-specific overrides.

## User guide: Quick start

For most users, configuration happens through the ARGVUS Control Center or this command-line tool:

```sh
# View your configuration location and validate it
argvus config path
argvus config validate

# Get a specific value
argvus config get /appearance/theme
argvus config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus config apply
```

For administrators and advanced configuration, continue reading below.

The user root contains only the canonical document and managed data:

```text
~/.config/argvus/
├── config.json
└── data/
```

The file is written atomically, kept private to the user, and protected by `.config.lock`, outside the `data/` tree that can be swapped during projection. A previous version is stored at `data/backups/config.json.bak`.

## Command line

The `argvus config` subcommand is provided in-process by the `argvus` package:

```sh
argvus config path
argvus config validate
argvus config ensure
argvus config migrate
argvus config status
argvus config explain /layout/gaps_out
argvus config get /appearance/theme --effective --raw
argvus config set /appearance/accent '"#61AFEF"'
argvus config patch appearance-patch.json
argvus config audio volume +5
argvus config audio mute toggle
argvus config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus config apply-theme universe-spaced --variant spaced --accent '#EEEEEE' --gtk-mode dark
argvus config theme-manifest universe-spaced
argvus config unset /appearance/accent
argvus config apply
argvus config rebuild
argvus config project
argvus config project --force
```

`ensure` materializes a new `config.json` with the schema defaults, or fills
missing fields without replacing explicit values. It fails on malformed JSON.
`migrate` imports existing legacy appearance state without replacing values
already present in `config.json`. `argvus sessionctl` runs `recover`, `ensure`,
`migrate` and `project` before applying a reload after configuration changes:

```sh
argvus sessionctl apply-config
```

`apply` and `rebuild` are aliases for rebuilding every projection from
`config.json`; `project` is the historical spelling and accepts `--force`.
`apply` also delegates the post-commit runtime reload to `argvus sessionctl`.
Set `ARGVUS_NO_RUNTIME=1` for preparation or isolated tests.
Generation happens in staging and the `data/` tree is swapped only after all
projectors succeed. Therefore deleting `data/generated/` and running
`argvus config apply` is a reproducible rebuild without publishing a partial
sequence of Hyprland or Taskbar outputs. The command records derived
section hashes and the runtime decision in
`$XDG_STATE_HOME/argvus/config-projection.json` (normally
`~/.local/state/argvus/config-projection.json`). Neither the manifest nor
generated files are sources of truth. `apply` performs the post-commit reload
through `argvus sessionctl`; `project` remains projection-only for compatibility.

Every persistent mutation uses the same rule: if the canonical document changed,
the CLI invokes one centralized `argvus sessionctl reload` after commit. The
reload result is recorded in `data/internal/last-reload.json`.

`argvus sessionctl reload` is therefore an *apply* step, not a projection step.
When the incoming manifest reports pending work it restores the targeted
consumers described by the plan and does not call `argvus config project` again;
reprojecting would rebuild an identical tree and discard the plan the mutation
just produced. It falls back to migration and projection only when no committed
plan exists, for example during session startup.

### Reload targets

`argvus config session-plan` answers the reload with the targets of the
sections that changed, and `argvus sessionctl` runs only those. The splash
covers a reload only when a target recreates a surface: a Waybar bar or the
Control Panel moving back on screen shifts every window. Everything else
reloads in place.

| Change | Targets | Splash |
| --- | --- | --- |
| Theme, highlight color or GTK mode | `hyprland`, `consumers`, `wallpaper`, `polkit` | no |
| Wallpaper only | `wallpaper` (hyprpaper IPC; the service restarts only for swaybg) | no |
| `effects` (blur, animations, transparency, shadow) | `hyprland`, `consumers` | no |
| `layout` of windows (gaps, borders) | `hyprland` | no |
| `layout.taskbar` (position, margins) | `hyprland`, `taskbar`, `widget_telemetry` | yes |
| `fonts` | `consumers` | no |
| `control_panel` (switch, cards) | `control_panel` | yes |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | no |
| `power` | `hypridle` | no |

`consumers` runs `argvus config theme-runtime consumers` (the bar stylesheets,
the Control Panel over IPC, Hyprland colours, Dunst's theme reload, btop and
kitty). Dunst defers its theme reload while Do Not Disturb is active.
`wallpaper` runs `argvus config theme-runtime wallpaper`, which shows the
effective `/appearance/wallpaper`. The `layout.taskbar` hash is kept
apart from the window layout in the projection manifest, so a gap change does
not restart the bars. The fallback `full` plan, used when no committed plan
exists, restarts every surface under the splash.

Each effective configuration has an operational generation. It is stored in
`data/internal/generation.json` and in the projection manifest under
`$XDG_STATE_HOME/argvus/config-projection.json`; it is not part of the dotfiles
configuration. Reapplying an unchanged effective configuration preserves the
generation. `status` reports whether the manifest hash, generation metadata and
all generated files agree; `apply` repairs a stale or incomplete projection.

`explain /path` reads the resolver directly and prints the effective value,
source, theme and variant. It does not parse generated files or create a second
precedence rule.

Theme and effects projection are implemented in `argvus-config` itself.
The theme transaction (`argvus theme-switch`, which runs `theme-apply`), `argvus config accent` and `argvus effects-toggle` do not project:
they validate, lock, commit through this CLI and then
reconcile external consumers that cannot read `config.json`. `argvus effects-toggle`
is a thin delegate with no output of its own. The appearance helpers are never
called by the projector, and they must never write below `data/generated/`.

### Projection surfaces

`project_effective` renders every ARGVUS-owned consumer in one staging tree
before publication:

| Surface | Output |
| --- | --- |
| Hyprland | `generated/hypr/`, including theme profiles, `spaces-effective.conf`, `borders-effective.conf`, `hypridle.conf` and `monitors.lua` |
| Layout/effects | `generated/effects/`, `generated/spaces.conf`, `generated/borders.conf` |
| Waybar | `generated/waybar/`, including `argvus-taskbar.css` and `argvus-widget-telemetry.{jsonc,css}` |
| Widget telemetry themes | `generated/widget-telemetry/` and `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| Notifications | `generated/notifications/` and `dunstrc` |
| Lock screen | `generated/hypr/hyprlock.conf` and the Hyprtoolkit theme profiles |
| GTK | `generated/gtk/` |
| Yazi | `generated/yazi/` |
| Superfile | `generated/superfile/` |
| Terminal and Qt profiles | `generated/terminal/`, `generated/qt6ct/` |
| Fonts | `generated/fonts.conf` plus managed font blocks in the Waybar stylesheets |
| Removable devices | `generated/removable-devices/theme.css` |
| Wallpaper | `generated/hypr/hyprpaper.conf` and `data/hypr/hyprpaper.conf` |
| Control Panel and defaults | `data/control-panel/`, `data/control-center/defaults.json` |
| Calendar, input, keyboard, power | calendar TOML, input fragment, keyboard state, keep-awake marker |

The theme layer is also mirrored outside the generated root:
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`,
`data/qt6ct/`, `data/hypr/hyprtoolkit.conf` and
`data/hypr/application-style.conf`. Those copies are replaced from the packaged
defaults when the appearance section changes, and the projector then rewrites
only its delimited managed blocks — the font block in the Waybar stylesheets and
the `ARGVUS_TELEMETRY_*` blocks in the telemetry profile — so text outside the
markers does not survive a theme switch. `fonts.targets` and
`control_panel.widget_telemetry_blocks` remain canonical: the Control Center
persists them through `argvus-config` and never writes these files itself.

Widget telemetry block selection is canonical at
`/control_panel/widget_telemetry_blocks`, an array of the seven block names
(`system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network`, `keys`). The
legacy `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` file is read only as a
migration fallback and is never written. `argvus widget-telemetry-toggle blocks
set` and `blocks all` patch the canonical array in a single atomic write, and
`apply-state` assumes the selection was already persisted by its caller.

Hyprland spacing and border preferences follow the same boundary. The
argvus config spaces and argvus config borders commands (and their historical
compatibility wrappers) update /layout; Rust generates
data/generated/spaces-effective.conf and data/generated/borders-effective.conf
and owns validation and live hyprctl application. .spaces and .borders are
migration-only state and are never the source of truth.

Accent and GTK mode state use the same Rust boundary. Use `argvus-config
accent COLOR` or `argvus config accent --theme-default` for accent ownership,
and `argvus config mode toggle|set dark|light` for GTK mode. The historical
accent and mode scripts are compatibility wrappers; `.accent-color`,
`.accent-custom` and `.gtk-mode` are migration/projection files only.

Logical ARGVUS paths are also resolved by `argvus config paths`. The legacy
`paths.sh` file only exposes compatibility functions; native overrides, user
data, generated data and system fallback are resolved by the Rust path model.

`audio` changes the default output sink and records the result in `/audio` in one step: `argvus config audio volume <N|+N|-N> [--limit PERCENT]` and `argvus config audio mute <on|off|toggle>`. It writes the store without reloading the configuration, because PipeWire already has the value. The Waybar volume module, the Hyprland media keys and the Control Panel use it, so `argvus config project`, which projects `/audio` onto the sink, never restores an older volume. A volume above 100% (the Hyprland keys allow up to 150%) is stored as 100.

`patch` applies a JSON object whose keys are JSON pointers in one locked
read-modify-write operation. `apply-theme` performs the equivalent atomic
theme transaction and replaces the managed appearance sections from the
theme's optional appearance profile or packaged defaults.
Mutating commands build a `NextUserConfig`, resolve an `EffectiveConfig`, stage
all projectors, and only then publish the canonical JSON. An operational
`.config-transaction.json` marker allows `argvus config recover` to reproject
the previous configuration after a crash between data and JSON publication.
Theme defaults are declared by each theme's manifest
(`themes/<theme>/theme.toml`): the default accent (`palette.accent`), the GTK color scheme
and the wallpaper. CSS, Lua, Rasi and Qt files are rendered from it into the theme build.

Changing the theme, including Light/Dark, resets the highlight color to the theme's default,
even when the user chose a custom color: the custom color is kept only until the next theme
change. Use `argvus config accent` after the theme change to set it again. Applying a
theme with `argvus config theme-apply` writes `/appearance/accent` and `/appearance/gtk_mode`
from the manifest, and `argvus config accent` writes the chosen color with `accent_custom`
set, in one command.

`apply-theme` takes the variant explicitly as `--variant sticky|spaced` and
resolves the requested name through the manifest, so a request can never
resolve to the assets of the other variant. `theme-manifest THEME` is the same
resolution exposed read-only: it prints the manifest entry together with the
`asset_id` and `layout_variant` it resolved to, which is what a caller needs to
address packaged assets. Sticky is the unsuffixed id and only Spaced carries a
`-spaced` suffix, so `dracula-sticky` and `argvus-dark-dracula-sticky` are
accepted and normalize to `dracula`.

With `--raw`, string values are printed without JSON quotes, while objects and arrays remain JSON values. Theme and layout dot-files are no longer written by the projection helper; `config.json` remains the only persistent source for those preferences. If projection fails, the session reload stops before restarting desktop surfaces so an older partial theme is not presented as current.

## Export and import

The same logical model is used for profile exchange:

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```

Use `--scope desktop` for the complete managed desktop model. Missing fields use ARGVUS defaults; generated output should not be edited directly.

Control Center theme export and import exchange a theme and its visual profile: the archive holds a schema 2 theme directory (`<id>/theme.toml`, optional `appearance.json` and `effects.conf`, `overrides/`, the wallpaper) that installs under `$XDG_DATA_HOME/argvus/themes/`. The profile holds appearance, layout geometry, visual effects, fonts and panel presentation. Audio, keyboard, network and other non-visual preferences remain outside themes. Use the `appearance` scope above to move the complete appearance configuration, and the `desktop` scope for a complete configuration backup.

The current schema groups preferences under `appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar`, and `removable_devices`. `default_apps` is the only persistent source for preferred applications, and `keyboard_shortcuts` stores stable English, locale-independent keys with string values or `null` for disabled bindings.

The Control Center reads the canonical appearance values when it opens an Appearance page and writes changes through `argvus-config`. The existing native state files remain projections for the appearance scripts and older installations.

The Control Panel uses `control_panel.cards` for card order and visibility, and `control_panel.enabled` for the panel master switch. Its `control-panel/cards.json` and `state/control-panel` files are kept as compatibility projections. Changes made in the Control Panel are therefore part of the `desktop` export scope.

Font preferences use `fonts.targets` for the family, style, and size of the taskbar, telemetry, Control Panel, system, applications, terminal, and browser targets. `fonts.rendering` stores antialiasing, hinting, subpixel order, and DPI preferences. `fonts.conf`, the fontconfig rule, desktop settings and the managed font blocks inside the Waybar stylesheets are projections: the Control Center persists `/fonts/targets/*` through `argvus-config`, and the projector rewrites only its delimited block.

Hyprland preferences use `keyboard_shortcuts` for shortcuts, `hyprland.input` for mouse and touchpad settings, and `hyprland.keyboard` for XKB layout, variant, and options. A manifest binding ID maps to a stable `config_key` (for example `window.close` → `close_window` and `window.drag_mouse` → `drag_window__floating_window_only`). Display preferences use `displays.monitors`, `displays.workspaces`, and `displays.primary_monitor`. Native TOML files and generated Lua files under `data/generated/` remain projections consumed by Hyprland.

Every `effects` switch defaults to enabled (animations, blur, widget telemetry and each transparency surface), on virtual machines and low-power sessions too; window shadow (`shadow_enabled`) and wobble (`wobble_enabled`) default to off. Effect preferences use `effects` for the Animations master switch (`animations`, applied to `hyprland.lua` from the projected `hypr/state.lua`, independent of the Effects switch, which is `blur_global_enabled` plus the `transparency_<surface>_enabled` of `taskbar`, `control-panel`, `widget-telemetry`, `terminal` and `launchers`, and reads as enabled when any of them is; `argvus effects-toggle animations|effects` switch them), the global blur switch (`blur_global_enabled`), the Hyprland blur parameters (`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`), the window shadow (`shadow_enabled`, `shadow_sharp`, `shadow_color`, `shadow_range`, `shadow_render_power`, `shadow_offset_x`, `shadow_offset_y`, `shadow_scale`), the Animations values (`animation_<leaf>_speed`, `_curve`, `_style`, `_popin` and `_enabled` for each animation, plus `animation_bezier_x1` to `animation_bezier_y2` and `animation_spring_mass`, `animation_spring_stiffness` and `animation_spring_dampening` for the custom curves) and per-surface transparency values. Effects are theme-owned, not variant-owned: Sticky and Spaced use the same `<theme>.conf` effects projection while their layout projections differ. Surfaces currently include `taskbar`, `control-panel`, and `widget-telemetry`; transparency values are percentages from `0` to `100` (the Control Center and launcher surfaces default to `15`, the others to `50`), blur parameters are numbers within the ranges declared in `schema.json`, and enabled states are booleans. Animations values are validated by `argvus-config` too: speeds are numbers from `1` to `10` (decimals allowed), pop-in percentages are whole numbers from `1` to `100`, curves must be names from the Animations page, styles must be names from the Animations page and accepted by the animation's own leaf (for example `popin` is refused for `animation_workspace_in_style`, `gnomed` is accepted only for window opening and closing, and the slide variants are refused on the window and layer leaves), the Bézier Y points stay between `-1` and `2`, the range Hyprland 0.56.2 parses, and the other curve parameters stay within their documented ranges. The `layers` and `switching` leaves no longer exist, so configs saved before their removal lose the `animation_layers_enabled`, `animation_layers_speed`, `animation_layers_curve` and `animation_switching_*` keys when they load. Styles outside the current set are dropped and Bézier Y points outside `-1` to `2` are clamped, so the packaged defaults apply again to whatever was removed.

Power preferences use `power.screen_off_minutes`, `power.lock_minutes`, `power.keep_awake`, and the lid/power-button policies. The projected forms are `data/generated/hypr/hypridle.conf` and the marker `data/power/keep-awake`; `.keep-awake` is a migration input only. `session.language` is retired: existing values are ignored, new writes are rejected, and no language compatibility file is projected. Set `LANG` in the user `locale.conf` instead. Calendar preferences use `calendar` and are projected to the taskbar-calendar user TOML. Removable-device preferences use `removable_devices`; the removable-devices application treats this section as a user override over packaged defaults.

`migrate` imports the old root files and sections once, preserving an existing canonical value. Legacy `defaults.json`, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, dot-files, `generated/` and component directories are moved below `data/`; repeated reloads do not recreate the old root layout. Selecting the current theme again is idempotent: if canonical state and projections already agree, the projection plan reports no changed sections and the session skips splash and service restarts.

Power follows the same projection contract: Control Center writes `/power` in `config.json`, `argvus config project` renders `data/hypr/hypridle.conf` atomically, and only then may `argvus sessionctl` restart `argvus-hypridle.service`. Removing that file is recoverable with `argvus config project`; a value of `0` omits the corresponding timeout and means Never.

Lock-screen appearance is projected by Rust from the canonical theme, accent,
effects, wallpaper and font state into `data/hypr/hyprlock.conf`. The historical
`hyprlock-theme.sh` entrypoint is compatibility-only and delegates to
`argvus config lock apply`; `--invalidate` only invalidates the derived blurred
wallpaper cache.

Power timeout changes use `/power/lock_minutes` and
`/power/screen_off_minutes` as their source of truth. `argvus config project`
reuses the existing Hypridle renderer, and the `idle-timeout.sh` entrypoint is
only a compatibility adapter; service restart remains delegated to
`argvus sessionctl`.

Notification appearance is derived from the canonical theme, accent and
effects into `data/notifications/dunstrc`. The historical notifications
`theme.sh` file only exposes compatibility functions that call
`argvus config notifications`; it does not parse or mutate Dunst configuration.

Hardware-oriented adapters such as display discovery and Bluetooth control are
now imported by the Rust display domain: `nwg-displays` is an external importer,
while `/displays` and `data/generated/hypr/monitors.lua` remain canonical and
derived state respectively. The display tools change the layout with
`argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` and `monitor-remove`; the display commands are `argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply`. The Hyprland file is projected from `/displays`; no shell script writes it any more. Wallpaper selection, taskbar utility-group mode,
and default-app lookup also use `argvus-config` APIs; their historical Shell
entrypoints are UI or compatibility wrappers. Keep Awake is a persistent
`/power/keep_awake` preference projected to the `data/power/keep-awake` marker,
while lock-DPMS is session runtime state under
`$XDG_CACHE_HOME/argvus/lock-dpms`. Keyboard layout is now owned by the Rust
`argvus-keyboard-layout` helper: the configured order comes from the generated
Hyprland input projection or locale fallback, while the active layout remains
session runtime state discovered from Hyprland. The cycle entrypoint and the
keyboard-layout daemon are compatibility wrappers only; the user service runs
the Rust daemon directly and `argvus sessionctl keyboard-layout cycle` is the
stable command for callers.

Bluetooth runtime state follows the external-adapter boundary but is not part
of `argvus-config`: `argvus bluetooth` owns typed parsing and
validation around `bluetoothctl`, bounded retries, system-service checks and
RFKill status. `argvus bluetoothctl` is the entrypoint. Adapter power, paired devices and connected devices
remain BlueZ runtime state rather than canonical JSON state; hardware-live
Bluetooth validation is environment-dependent.
## Runtime identity and reconciliation

Use `argvus config status --verbose` to inspect the exact binary, build
identity, configuration root, generated root, asset root, manifest source and
active generation. It reports `synchronized`, `stale`, `incomplete`, `legacy`
and `pending-transaction` projection states.

`argvus config --version` reports package version, build revision when supplied
by the build environment, and profile. Development tests must inject their
binary and assets explicitly; they must not silently use `argvus config`
or `~/.config/argvus`. `apply` safely normalizes legacy combined identifiers
such as `argvus-dark-spaced` into separate theme and variant values before
rebuilding projections. Packaged asset identity is not recorded in a manifest in this release.

## Night Light

Open **Desktop & Input > Displays > Night Light** or run `argvus control-center displays night-light`. Configure temperature, manual activation and fixed local schedules through `argvus config display night-light`. See [Night Light](/docs/user-guide/night-light/) for commands, state and runtime requirements.

### Wobble settings (Hyprland 0.57+)

`effects.wobble_enabled`, `wobble_stiffness`, `wobble_damping`, `wobble_mass`,
`wobble_intensity` and `wobble_mesh` project directly into `hypr/state.lua`.
Use `argvus effects-toggle wobble-settings get` to read them, or
`argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8`
to save and request a reload. `enabled=disabled` turns the effect off.

The Control Center shares the Animations draft: `animations-settings get` also
returns `wobble_*` entries, and `animations-settings set` accepts these full names
alongside its existing animation arguments. This persists the whole draft in one
validated patch and requests one reload. For example:

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```

The global `enabled` controls animations; `wobble_enabled` is its independent
preference. Disabling global animations suppresses wobble in Lua without erasing
that preference. Invalid values reject the entire patch. Missing keys receive
the ARGVUS defaults without replacing existing choices. See [Appearance](/docs/user-guide/appearance/#window-wobble)
for bounds and defaults.


### User-created keyboard shortcuts

`/keyboard_shortcuts` retains its existing string-or-null values for official manifest actions. Custom global keyboard shortcuts use a separate `/hyprland/custom_shortcuts` object, stored in `config/hyprland.json`. Its stable `custom_` IDs are independent of descriptions and locales. Each record has `description`, `keys`, `enabled` and a typed `action`:

```json
{
  "custom_editor": {
    "description": "Open my editor",
    "keys": "SUPER + CTRL + E",
    "enabled": true,
    "action": { "type": "command", "command": "editor --new-window" }
  },
  "custom_float": {
    "description": "Toggle floating",
    "keys": "SUPER + CTRL + F",
    "enabled": false,
    "action": {
      "type": "dispatcher",
      "dispatcher": "window.float",
      "arguments": { "action": "toggle" }
    }
  }
}
```

The example is the value of `/hyprland/custom_shortcuts`, not a complete configuration document. Dispatcher names come from the supported Lua dispatcher factories; arguments are an optional JSON object or `null`, never evaluated code. Custom shortcuts have no submap, mouse binding or advanced flags. The projection writes `data/generated/hypr/custom-shortcuts.lua` as an escaped data table, including an empty table when the last record is removed. Both full projection and `project --only keyboard_shortcuts` generate it. Hyprland registers enabled entries with `hl.bind`; optional file and individual registration failures are contained and logged.

The Control Center creates/edits records and disables confirmed conflicts using one `argvus config patch` transaction. It saves before applying through the existing config service. A failed apply does not undo a successful save. Restoring official defaults preserves custom records and is refused if it would collide with an active custom shortcut. User descriptions remain literal text in all locales and the effective generated cheatsheet.

## Global window mode

Open **Control Center → Desktop & Input → Hyprland → Windows → Mode** and select **Tile Windows (Default)** or **Floating Windows**. The same selector is available from the window-mode icon in the taskbar utility group.

The preference applies to every monitor and workspace and is remembered after logout. Floating converts existing application windows and newly opened windows. Tile restores only windows that this policy converted; dialogs and other windows that were already floating remain floating. A window explicitly pinned while Floating is active also remains floating to preserve its pin. The tiled layout is recalculated; its exact previous split tree is not restored.

`SUPER + SHIFT + Space` keeps its normal per-window behavior in Tile. In Floating it does not switch a window back to tiling. Panels, the taskbar and notifications are layer-shell surfaces and are not affected. Fullscreen and maximization continue to use Hyprland's normal behavior.

The canonical preference is `/hyprland/window_mode` in `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. Its generated projection is `data/generated/hypr/window-mode.lua`; do not edit it. The per-window restoration registry lives under `$XDG_RUNTIME_DIR/argvus`, scoped to the compositor instance, and survives configuration reloads. It is not reused in a new session.

If application is pending or fails, the interface distinguishes this from the saved preference. Select the option again to retry. Without a compositor, the preference is saved for the next login.

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```

## Window buttons and application bar

Floating mode adds an ARGVUS title bar with **Minimize**, **Maximize / Restore**, and **Close** to normal application windows. Double-clicking the title bar toggles maximization. The additional decoration is hidden in Tile and fullscreen. Applications that draw their own controls may display both bars; uniform ARGVUS controls are intentional.

A second Waybar appears on every monitor, on the edge opposite the main taskbar. It lists only the windows belonging to that monitor's current workspace, including minimized windows from that workspace. An open special workspace takes precedence. Pinned windows remain visible; hidden group tabs do not create extra entries, and a minimized group has one recovery entry. Tabs inside an application are not enumerated separately. Minimized entries retain their original monitor/workspace association. Click an inactive window to activate it, an active window to minimize it, or a minimized window to restore it; its context menu offers restore, minimize, maximize/restore and close. Unknown application icons fall back to a window symbol. Activating or restoring an entry preserves the pointer position. The secondary bar uses the effective primary CSS (colors, fonts, border and rounding), with its height and mirrored margins on the opposite edge.

`SUPER + Alt + Up` places the main bar at the top and the window bar at the bottom. `SUPER + Alt + Down` does the reverse. Repeating a direction leaves the positions unchanged. Tile has only the main bar. Returning to Tile restores all minimized windows before removing the window bar, and then preserves the original tiled/floating policy.

Minimization uses the reserved `special:argvus-minimized` workspace. Do not use that workspace manually. The `argvus windows service` process records identity, original workspace/monitor, geometry, fullscreen/maximization and pin state in a compositor-instance-specific runtime registry under `$XDG_RUNTIME_DIR/argvus`. Records survive a service restart or config reload, but do not cross compositor sessions. Closing a window removes its record. If a monitor disappears, restoration uses an available output. A missing or failed recovery bar prevents new minimizations; failed restoration retains the record and recovery bar.

The session owns `argvus-windows.service` and `argvus-window-bar.service`. The latter keeps its Waybar child dormant in Tile. The hyprbars plugin is bundled by `argvus-hyprland`, built for the Hyprland 0.57.0 ABI on x86_64; it is not compiled in the user's home directory. Rebuild the plugin with a compositor ABI update.

Commands: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all`, and `minimize|restore|activate|maximize|close ADDRESS`. The window addresses come from the list. `position top --persist` stores/projects the preference without restarting the main bar. Configuration and runtime error details are written to the journal; visible action failures also show a translated notification.

In **Spaced + Floating**, windows respect outer gaps and the areas reserved by both bars. The bounds include the border and title, which sits inside the same window border. Maximizing through ARGVUS controls fills this area with margins; restoring recovers the previous geometry, including after a service restart. Fullscreen still uses the whole output. Floating windows may overlap; tiling inner gaps do not separate them.

Inner margins reserved by the bars are not counted twice in window outer gaps. With symmetric spacing, the visible window-to-bar distance is equal above and below, including after swapping the bars.

`SUPER + Alt + Space` toggles the global Tile/Floating mode, using the same preference as the taskbar and Control Center. The equivalent command is `argvus config window-mode toggle`. Concurrent invocations are serialized; toggling uses the saved preference even when application is pending. The shortcut can be changed or disabled in Keyboard Shortcuts; `SUPER + Shift + Space` keeps its individual-window action.

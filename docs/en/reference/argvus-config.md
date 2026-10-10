---
title: ARGVUS Configuration
description: The logical JSON configuration contract used by ARGVUS.
---

ARGVUS keeps managed user preferences in `$XDG_CONFIG_HOME/argvus/config.json` (normally `~/.config/argvus/config.json`). This document is the canonical logical model for preferences managed by ARGVUS, and `argvus-config` is the only component that writes files below `data/generated/`. Native files and generated files remain compatibility projections or application-specific overrides.

## User guide: Quick start

For most users, configuration happens through the ARGVUS Control Center or this command-line tool:

```sh
# View your configuration location and validate it
argvus-config path
argvus-config validate

# Get a specific value
argvus-config get /appearance/theme
argvus-config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus-config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus-config apply
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

The `argvus-config` command is installed by the `argvus-config` package:

```sh
argvus-config path
argvus-config validate
argvus-config ensure
argvus-config migrate
argvus-config status
argvus-config explain /layout/gaps_out
argvus-config get /appearance/theme --effective --raw
argvus-config set /appearance/accent '"#61AFEF"'
argvus-config patch appearance-patch.json
argvus-config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus-config apply-theme universe-float --variant float --accent '#EEEEEE' --gtk-mode dark
argvus-config theme-manifest universe-float
argvus-config unset /appearance/accent
argvus-config apply
argvus-config rebuild
argvus-config project
argvus-config project --force
```

`ensure` materializes a new `config.json` with the schema defaults, or fills
missing fields without replacing explicit values. It fails on malformed JSON.
`migrate` imports existing legacy appearance state without replacing values
already present in `config.json`. `argvus-sessionctl` runs `recover`, `ensure`,
`migrate` and `project` before applying a reload after configuration changes:

```sh
argvus-sessionctl apply-config
```

`apply` and `rebuild` are aliases for rebuilding every projection from
`config.json`; `project` is the historical spelling and accepts `--force`.
`apply` also delegates the post-commit runtime reload to `argvus-sessionctl`.
Set `ARGVUS_NO_RUNTIME=1` for preparation or isolated tests.
Generation happens in staging and the `data/` tree is swapped only after all
projectors succeed. Therefore deleting `data/generated/` and running
`argvus-config apply` is a reproducible rebuild without publishing a partial
sequence of Hyprland or Taskbar outputs. The command records derived
section hashes and the runtime decision in
`$XDG_STATE_HOME/argvus/config-projection.json` (normally
`~/.local/state/argvus/config-projection.json`). Neither the manifest nor
generated files are sources of truth. `apply` performs the post-commit reload
through `argvus-sessionctl`; `project` remains projection-only for compatibility.

Every persistent mutation uses the same rule: if the canonical document changed,
the CLI invokes one centralized `argvus-sessionctl reload` after commit. The
reload result is recorded in `data/internal/last-reload.json`.

`argvus-sessionctl reload` is therefore an *apply* step, not a projection step.
When the incoming manifest reports pending work it restores the targeted
consumers described by the plan and does not call `argvus-config project` again;
reprojecting would rebuild an identical tree and discard the plan the mutation
just produced. It falls back to migration and projection only when no committed
plan exists, for example during session startup.

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
`theme-switch.sh`, `accent-switch.sh` and `effects-toggle.sh` do not project:
they validate, lock, show the splash, commit through this CLI and then
reconcile external consumers that cannot read `config.json`. `effects-toggle.sh`
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
migration fallback and is never written. `argvus-widget-telemetry-toggle blocks
set` and `blocks all` patch the canonical array in a single atomic write, and
`apply-state` assumes the selection was already persisted by its caller.

Hyprland spacing and border preferences follow the same boundary. The
argvus-config spaces and argvus-config borders commands (and their historical
compatibility wrappers) update /layout; Rust generates
data/generated/spaces-effective.conf and data/generated/borders-effective.conf
and owns validation and live hyprctl application. .spaces and .borders are
migration-only state and are never the source of truth.

Accent and GTK mode state use the same Rust boundary. Use `argvus-config
accent COLOR` or `argvus-config accent --theme-default` for accent ownership,
and `argvus-config mode toggle|set dark|light` for GTK mode. The historical
accent and mode scripts are compatibility wrappers; `.accent-color`,
`.accent-custom` and `.gtk-mode` are migration/projection files only.

Logical ARGVUS paths are also resolved by `argvus-config paths`. The legacy
`paths.sh` file only exposes compatibility functions; native overrides, user
data, generated data and system fallback are resolved by the Rust path model.

`patch` applies a JSON object whose keys are JSON pointers in one locked
read-modify-write operation. `apply-theme` performs the equivalent atomic
theme transaction and preserves `accent_custom` and `wallpaper_custom` values.
Mutating commands build a `NextUserConfig`, resolve an `EffectiveConfig`, stage
all projectors, and only then publish the canonical JSON. An operational
`.config-transaction.json` marker allows `argvus-config recover` to reproject
the previous configuration after a crash between data and JSON publication.
Theme defaults are declared by the packaged
`appearance/config/theme-defaults.json` manifest; CSS, Lua, Rasi and Qt files
are projections of that state.

`apply-theme` takes the variant explicitly as `--variant sticky|float` and
resolves the requested name through the manifest, so a request can never
resolve to the assets of the other variant. `theme-manifest THEME` is the same
resolution exposed read-only: it prints the manifest entry together with the
`asset_id` and `layout_variant` it resolved to, which is what a caller needs to
address packaged assets. Sticky is the unsuffixed id and only Float carries a
`-float` suffix, so `dracula-sticky` and `argvus-dark-dracula-sticky` are
accepted and normalize to `dracula`.

With `--raw`, string values are printed without JSON quotes, while objects and arrays remain JSON values. Theme and layout dot-files are no longer written by the projection helper; `config.json` remains the only persistent source for those preferences. If projection fails, the session reload stops before restarting desktop surfaces so an older partial theme is not presented as current.

## Export and import

The same logical model is used for profile exchange:

```sh
argvus-config export --scope appearance --output appearance.json
argvus-config import --scope appearance appearance.json
```

Use `--scope desktop` for the complete managed desktop model. Missing fields use ARGVUS defaults; generated output should not be edited directly.

Control Center theme profiles use the `appearance` scope inside their archive. New profiles include the canonical JSON together with legacy native files for compatibility; applying a profile imports that JSON and regenerates its projections. The command-line `desktop` scope remains the option for a complete configuration backup.

The current schema groups preferences under `appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar`, and `removable_devices`. `default_apps` is the only persistent source for preferred applications, and `keyboard_shortcuts` stores stable English, locale-independent keys with string values or `null` for disabled bindings.

The Control Center reads the canonical appearance values when it opens an Appearance page and writes changes through `argvus-config`. The existing native state files remain projections for the appearance scripts and older installations.

The Control Panel uses `control_panel.cards` for card order and visibility, and `control_panel.enabled` for the panel master switch. Its `control-panel/cards.json` and `state/control-panel` files are kept as compatibility projections. Changes made in the Control Panel therefore remain available to Control Center export/import profiles.

Font preferences use `fonts.targets` for the family, style, and size of the taskbar, telemetry, Control Panel, system, applications, terminal, and browser targets. `fonts.rendering` stores antialiasing, hinting, subpixel order, and DPI preferences. `fonts.conf`, the fontconfig rule, desktop settings and the managed font blocks inside the Waybar stylesheets are projections: the Control Center persists `/fonts/targets/*` through `argvus-config`, and the projector rewrites only its delimited block.

Hyprland preferences use `keyboard_shortcuts` for shortcuts, `hyprland.input` for mouse and touchpad settings, and `hyprland.keyboard` for XKB layout, variant, and options. A manifest binding ID maps to a stable `config_key` (for example `window.close` → `close_window` and `window.drag_mouse` → `drag_window__floating_window_only`). Display preferences use `displays.monitors`, `displays.workspaces`, and `displays.primary_monitor`. Native TOML files and generated Lua files under `data/generated/` remain projections consumed by Hyprland.

Effect preferences use `effects` for global animation state, the global blur switch (`blur_global_enabled`), the Hyprland blur parameters (`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`), the Animations values (`animation_<leaf>_speed`, `_curve`, `_style`, `_popin` and `_enabled` for each animation, plus `animation_bezier_x1` to `animation_bezier_y2` and `animation_spring_mass`, `animation_spring_stiffness` and `animation_spring_dampening` for the custom curves) and per-surface transparency values. Effects are theme-owned, not variant-owned: Sticky and Float use the same `<theme>.conf` effects projection while their layout projections differ. Surfaces currently include `taskbar`, `control-panel`, and `widget-telemetry`; transparency values are percentages from `0` to `100`, blur parameters are numbers within the ranges declared in `schema.json`, and enabled states are booleans. Animations values are validated by `argvus-config` too: speeds are numbers from `1` to `10` (decimals allowed), pop-in percentages are whole numbers from `1` to `100`, curves must be names from the Animations page, styles must be names from the Animations page and accepted by the animation's own leaf (for example `popin` is refused for `animation_switching_style`, and `gnomed` is accepted only for window opening and closing), and the curve parameters stay within their documented ranges.

Power preferences use `power.screen_off_minutes`, `power.lock_minutes`, `power.keep_awake`, and the lid/power-button policies. The projected forms are `data/generated/hypr/hypridle.conf` and the marker `data/power/keep-awake`; `.keep-awake` is a migration input only. `session.language` is the canonical interface language and the legacy `language` file remains a compatibility projection. Calendar preferences use `calendar` and are projected to the taskbar-calendar user TOML. Removable-device preferences use `removable_devices`; the removable-devices application treats this section as a user override over packaged defaults.

`migrate` imports the old root files and sections once, preserving an existing canonical value. Legacy `defaults.json`, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, dot-files, `generated/` and component directories are moved below `data/`; repeated reloads do not recreate the old root layout. Selecting the current theme again is idempotent: if canonical state and projections already agree, the projection plan reports no changed sections and the session skips splash and service restarts.

Power follows the same projection contract: Control Center writes `/power` in `config.json`, `argvus-config project` renders `data/hypr/hypridle.conf` atomically, and only then may `argvus-sessionctl` restart `argvus-hypridle.service`. Removing that file is recoverable with `argvus-config project`; a value of `0` omits the corresponding timeout and means Never.

Lock-screen appearance is projected by Rust from the canonical theme, accent,
effects, wallpaper and font state into `data/hypr/hyprlock.conf`. The historical
`hyprlock-theme.sh` entrypoint is compatibility-only and delegates to
`argvus-config lock apply`; `--invalidate` only invalidates the derived blurred
wallpaper cache.

Power timeout changes use `/power/lock_minutes` and
`/power/screen_off_minutes` as their source of truth. `argvus-config project`
reuses the existing Hypridle renderer, and the `idle-timeout.sh` entrypoint is
only a compatibility adapter; service restart remains delegated to
`argvus-sessionctl`.

Notification appearance is derived from the canonical theme, accent and
effects into `data/notifications/dunstrc`. The historical notifications
`theme.sh` file only exposes compatibility functions that call
`argvus-config notifications`; it does not parse or mutate Dunst configuration.

Hardware-oriented adapters such as display discovery and Bluetooth control are
now imported by the Rust display domain: `nwg-displays` is an external importer,
while `/displays` and `data/generated/hypr/monitors.lua` remain canonical and
derived state respectively. Wallpaper selection, taskbar utility-group mode,
and default-app lookup also use `argvus-config` APIs; their historical Shell
entrypoints are UI or compatibility wrappers. Keep Awake is a persistent
`/power/keep_awake` preference projected to the `data/power/keep-awake` marker,
while lock-DPMS is session runtime state under
`$XDG_CACHE_HOME/argvus/lock-dpms`. Keyboard layout is now owned by the Rust
`argvus-keyboard-layout` helper: the configured order comes from the generated
Hyprland input projection or locale fallback, while the active layout remains
session runtime state discovered from Hyprland. The cycle entrypoint and the
keyboard-layout daemon are compatibility wrappers only; the user service runs
the Rust daemon directly and `argvus-sessionctl keyboard-layout cycle` is the
stable command for callers.

Bluetooth runtime state follows the external-adapter boundary but is not part
of `argvus-config`: `/usr/bin/argvus-bluetooth` owns typed parsing and
validation around `bluetoothctl`, bounded retries, system-service checks and
RFKill status. `bluetooth-control.sh` and `argvus-bluetoothctl` remain
compatibility entrypoints. Adapter power, paired devices and connected devices
remain BlueZ runtime state rather than canonical JSON state; hardware-live
Bluetooth validation is environment-dependent.
## Runtime identity and reconciliation

Use `argvus-config status --verbose` to inspect the exact binary, build
identity, configuration root, generated root, asset root, manifest source and
active generation. It reports `synchronized`, `stale`, `incomplete`, `legacy`
and `pending-transaction` projection states.

`argvus-config --version` reports package version, build revision when supplied
by the build environment, and profile. Development tests must inject their
binary and assets explicitly; they must not silently use `/usr/bin/argvus-config`
or `~/.config/argvus`. `apply` safely normalizes legacy combined identifiers
such as `argvus-dark-float` into separate theme and variant values before
rebuilding projections. The installed asset manifest is
`/usr/share/argvus/version.json`.

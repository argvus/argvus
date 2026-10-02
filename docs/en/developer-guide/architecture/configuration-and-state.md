---
title: Configuration and state
description: Shared state, overrides and generated runtime files.
---

The normal user root is `$XDG_CONFIG_HOME/argvus` (usually `~/.config/argvus`). Managed logical preferences are stored in `config.json`. Theme, accent, wallpaper, spacing and border preferences are canonical there; the dot-files remain derived compatibility projections for current Hyprland Lua and shell consumers while those consumers migrate to `data/`.

`argvus-config` is the only writer below `generated/`. Every ARGVUS-owned
consumer file is a projection of `config.json`, including Hyprland, GTK, Qt6ct,
Waybar, Quickshell, Rofi, Dunst, Hyprlock, the Waybar taskbar and widget-telemetry
profiles, the Yazi and Superfile configuration trees, the terminal and Qt profile
trees, the font managed blocks and the removable-devices stylesheet. Components
resolve explicit user overrides, then generated output, then package defaults.
Normal theme/reload projection never promotes either generated or packaged files
into a user override. Documentation and tools should identify the logical source
instead of asking users to edit generated output.

`argvus-appearance` therefore owns theme and accent orchestration, while
`argvus-config` owns canonical state and structured projection.
`theme-switch.sh` and `accent-switch.sh` validate, lock and commit through
`argvus-config` when available, then reconcile compatibility consumers that
cannot read `config.json` directly. Those adapters are
the native Qt6ct file, GTK `settings.ini` and `gsettings` hints, the terminal and
system-monitor applications, snappy-switcher, foot, superfile's own config
directory and the pre-authentication greeter. A helper that needs a new adapter
must add it there, never inside `generated/`.

The runtime ownership is intentionally split by responsibility: Rust owns the
canonical store, schema validation, migrations, path model, locks, atomic
writes, projection hashes and structured renderers. Shell remains only as
small compatibility entrypoints or adapters for external consumers such as
`hyprctl` and asset-owned theme scripts. Python is reserved for tests, audits
and development helpers; it is not part of the critical desktop runtime.

`argvus-config apply` is the rebuild boundary (`project` remains the
compatibility spelling). It regenerates missing outputs even when the canonical
document is unchanged, renders projectors in staging, and atomically swaps the
`data/` tree only after every projection succeeds. The session lifecycle remains owned by
`argvus-sessionctl`, which consumes that manifest to plan targeted reloads.

When a caller has already committed a generation — `apply-theme`, an accent
change, a `set`/`patch`, or a widget-telemetry block change — `reload` **applies**
that generation instead of projecting it again. It reads the plan from the
manifest, restores the targeted consumers and only projects when the manifest is
absent or has no pending work. Reprojecting at that point would rebuild an
identical tree and discard the plan the mutation just produced.

The manifest also records the operational generation, effective-configuration
hash, changed domains and generated file list. `data/internal/generation.json`
is a disposable local projection marker and is never a user preference.
`argvus-config status` compares these records with the resolved configuration
and reports missing or extra generated files.

Mutations follow the same engine boundary: `UserConfig` is cloned into a
`NextUserConfig`, validated, resolved into `EffectiveConfig`, projected, and
only then persisted. `.config-transaction.json` is operational recovery state;
`argvus-config recover` deterministically reprojects the previous canonical
document after a crash between publishing `data/` and `config.json`. Reload is
requested only after the commit, and a reload failure does not roll back a
coherent persisted configuration.

## Resolution model

```text
config.json logical model
    ↓
ARGVUS native compatibility state and per-app override
    ↓
ARGVUS generated runtime file
    ↓
/usr/share/argvus packaged default
    ↓
upstream default
```

The exact candidates vary by component, but the principle is consistent: a user override wins, generated output is derived state, and packaged files remain read-only inputs. The session exports `ARGVUS_CONFIG_HOME` and `ARGVUS_SYSTEM_CONFIG` so helpers and user services resolve the same roots.

Reload recovery follows `recover → ensure → migrate → apply → runtime reload`,
and the projection step is skipped when the incoming manifest already describes a
committed generation. `ensure`
materializes `config.json` from schema defaults only when the profile is
missing, and fills absent fields without replacing explicit `false`, `0`, or
custom values. An existing malformed document remains an error rather than
being silently replaced.

Persistent mutations compare the resolved old and new configurations through
`ChangedDomains`. After commit, one reload request is sent through
`argvus-sessionctl`; reload failure is reported and recorded without rolling
back the committed configuration. Session startup performs recovery before
bootstrap materialization and projection, so consumers do not start against an
abandoned transaction.

| State | Purpose |
| --- | --- |
| `config.json` | Canonical managed logical preferences; the single source for everything below `generated/`. |
| `.active-theme`, `.accent-color`, `.accent-custom` | Legacy migration inputs; not runtime sources and not regenerated. |
| `.wallpaper-custom` | Legacy migration input for `appearance.wallpaper`; not a runtime source. |
| `.spaces`, `.borders` | Legacy layout migration inputs; canonical values are stored under `layout` in `config.json`. |
| `fonts.conf` | Compatibility projection of canonical font selections. |
| `generated/effects` | Compatibility projection of canonical effects; disposable. |
| `$XDG_STATE_HOME/argvus/config-projection.json` | Projection manifest: changed sections, affected consumers and whether a reload is required. Read by `reload`; never a source of truth. |
| `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` | Legacy telemetry block preferences. Read as a migration fallback only; canonical values live at `/control_panel/widget_telemetry_blocks` and the file is no longer written. |

A property is canonical when it is read from `config.json` and projected from
there. A property is derived when it is projected but also consumable as a
migration input. Anything a component still writes at runtime — a cache, a native
config file under `$XDG_CONFIG_HOME/<app>`, a greeter state file — is an adapter
target and must not be read back as configuration.

Projection covers consumers for Hyprland, GTK/Qt, Waybar, Quickshell, terminals, notifications, lock screen and native applications. Hyprland reads active and inactive border colors from the selected theme; an explicit `appearance.accent_custom=true` is the only case that overrides the active/group border with the user accent. Theme switching reapplies these values through the compositor reload so it cannot retain the previous theme's colors. If the user configuration is recreated, effects default to animations enabled. Deleting `generated/` and running `argvus-config apply` recreates it byte-for-byte from `config.json`; editing generated files directly is not durable.
Runtime diagnosis must identify the complete execution tuple: binary path,
version/revision, asset root, config root, data root, generated root, internal
root and projection manifest. `status --verbose` exposes this tuple and
compares the published generation/hash with the current resolved config.
Development and installed-package tests are separate modes; neither may fall
back silently to an unrelated executable or the developer's home. `apply` is
also the reconciliation boundary for legacy combined theme identifiers: a
value such as `argvus-dark-float` becomes `theme=argvus-dark` and
`variant=float` before resolution. `/usr/share/argvus/version.json` describes
packaged asset identity and is diagnostic metadata, not a source of truth.

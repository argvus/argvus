# argvus-control-center

`argvus-control-center` is the keyboard-first terminal control center for ARGVUS.
It is one full-screen TUI built in the `argvus` workspace (`crates/argvus-control-center*`)
and opened with `argvus control-center`. It runs inside the ARGVUS terminal profile and
does not introduce a second configuration system: it reads and writes the canonical
ARGVUS configuration through `argvus config`.

The home screen opens Default Apps or About; Fonts is under Appearance > Desktop Style. Settings retain the existing
list/search workflows and ARGVUS backends. About retains its distinct five-tab
layout, responsive logo, system information, links, credits, donation page, and
full copyright text.

## Requirements

- Rust 1.95 or newer
- a terminal supported by Crossterm, including Foot on Wayland
- `fontconfig` (`fc-list`) for installed-font discovery
- the ARGVUS settings resources under `/etc/argvus/control-center`
- `xdg-open` for About links
- when building from source, the `argvus` workspace at the repository root
  (`cargo build -p argvus-control-center`)

The settings backends may invoke the same user-level tools as the GTK settings
application, including `gsettings`, `xdg-mime`, and ARGVUS refresh helpers. No
operation uses `sh -c` or requests root privileges.

## Build and test

```bash
cargo build --release --workspace
cargo test --workspace
```

The full validation used by the project is:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release --workspace
```

Install into a staging root or the configured prefix with:

```bash
make install DESTDIR=/tmp/argvus-control-center-package
```

## Run

```bash
argvus control-center
argvus control-center apps
argvus control-center fonts
argvus control-center about
argvus control-center about credits
```

`--help` and `--version` run without entering the alternate screen. Direct routes
are suitable for ARGVUS keyboard shortcuts such as `SUPER+ALT+S`.

## Controls

- `Up` / `Down` or `j` / `k`: navigate lists and About content
- `Right` or `Enter`: open the selected home/settings item
- `Left` / `Right` or `h` / `l`: change About tabs
- `Tab` / `Shift+Tab`: change About tabs
- `PageUp` / `PageDown`, `Home`, `End`: scroll About and long settings lists
- `/`: search application and font lists
- `+` / `-`: change a pending font size
- `Enter`: apply a setting or open the selected About link
- `Esc`: cancel search, close a popup, or return to the previous level
- `?`: contextual keyboard help
- `q`: quit from every page, including search mode

The terminal uses raw mode, the alternate screen, and a hidden cursor. A shared
RAII guard plus panic hook restores all three on normal exit and unwind.

## Architecture

This repository is a Cargo workspace with deliberate boundaries:

- `argvus-control-center`: binary, Home, CLI, global route and event composition
- `argvus-tui`: reusable terminal lifecycle and shared chrome/widgets
- `argvus-theme`: global ARGVUS theme consumer and semantic TUI theme
- `argvus-i18n`: shared language detection and translations (the `argvus i18n` command)
- `argvus-control-center-settings`: default apps and fonts domain
- `argvus-control-center-about`: tab documents, system data, logo and links

The Phase 1 infrastructure in `argvus-control-center-core` is shared by future
domains and deliberately has no TUI dependency. It provides centralized
capability detection, a structured `ProcessRunner` with timeouts, cooperative
background jobs, validated privileged-operation requests, terminal-output
sanitization, and a `SearchRegistry` prepared for global search (`id`, category,
title, keywords, and route).

`argvus-tui::components` provides reusable status/result and confirmation-dialog
models/widgets. They consume the existing semantic ARGVUS `Theme`; they do not
define a second palette or language system. Existing settings confirmation and
status flows remain intact until a later, separately validated migration.

The Home lists Firewall under its Security card, next to SSH keys and GPG keys;
the `network firewall` route is kept and opens that same page. No future-domain
screen is registered until its backend and view are implemented.

### Safety and backend boundaries

The infrastructure never invokes a shell and never concatenates user input into
a command. External tools are selected by domain backends, with separated
arguments, captured output, and explicit timeouts. Privileged requests carry a
validated domain/action pair and arguments; implementations must map those
actions to polkit/D-Bus or a narrowly scoped ARGVUS helper, never an arbitrary
command line. Jobs run outside the render loop, and views consume cached state
and job results at entry, refresh, or action completion.

Optional dependencies are capabilities, not requirements: absent NetworkManager,
BlueZ, PipeWire, smartmontools, Plymouth, reflector, power-management tools, or
bootloader tools must produce an unavailable state, never a panic. Future domains
will retain separate model, detection, backend, action, error, and view modules.
Firewall belongs to the Security card of the Home, not to the Network domain.

### Phase 3 domains

Phase 3 adds the following domain crates and routes:

```text
Rede
├── Status
├── Interfaces
├── Wi-Fi
├── Ethernet
├── VPN
├── DNS
└── Proxy

Áudio
├── Saída
├── Entrada
└── Dispositivos

Bluetooth
├── Estado
├── Dispositivos
└── Parear
```

Network uses structured, argument-separated `nmcli` calls when NetworkManager
is available and falls back to `/etc/resolv.conf` and environment inspection for
read-only DNS/proxy information. Wi-Fi, VPN, and connection actions stay behind
the backend; secrets are passed only as process arguments, are masked in the UI,
and are never logged. Audio uses `wpctl` with a PipeWire/WirePlumber capability
check. Bluetooth uses one-shot `bluetoothctl` BlueZ commands; an interactive
BlueZ pairing agent and D-Bus event subscriptions remain future work, so PIN/
passkey pairing is not claimed as fully supported.

All snapshots and scans are loaded by `JobManager`; no network, audio, or
Bluetooth probe runs during rendering. Optional tools are detected centrally by
`Capabilities`. See [docs/PHASE3.md](docs/PHASE3.md) for backends, security,
refresh policy, and current limitations.

### Boot

The `argvus-control-center-boot` crate provides Summary, Kernel, Bootloader,
Initramfs and Plymouth routes. It uses conservative evidence from firmware/sysfs,
loader entries, GRUB defaults, `/usr/lib/modules`, mkinitcpio configuration and
Plymouth theme directories. Parsing is done as data; no configuration file is
sourced. Boot state is cached through `JobManager` and refreshed with `r`.

Supported mutations are deliberately narrow: systemd-boot default entry and
timeout, GRUB timeout/kernel command line/regeneration, mkinitcpio regeneration,
and Plymouth theme application followed by initramfs regeneration. They use
typed `system-settings boot` actions, fixed target paths, validation in both
the TUI and helper, non-overwriting backups and atomic writes. Plymouth reports
partial success when the theme changes but initramfs regeneration fails. Secure
Boot remains informational and ambiguous/unsupported layouts leave mutations
unavailable.

### Phase 2 domains

The implemented Phase 2 tree is:

```text
Hardware
├── Resumo
├── CPU
├── GPU
├── Memória
├── Energia
└── Dispositivos

Serviços
├── Sistema
├── Usuário
├── Falhos
└── Logs
```

`argvus-control-center-hardware` reads `/proc`, DMI, sysfs, `/sys/class/drm`,
power-supply and optional probes. It falls back from sysfs to `lspci`, and
does not require `glxinfo`, `vulkaninfo`, `powerprofilesctl`, or a battery.
Hardware snapshots are collected by a background job on entry and with `r`;
rendering only consumes the cached snapshot. Governor changes are validated and
sent through the existing `system-settings` privilege boundary.

`argvus-control-center-services` uses `systemctl --output=json` for system and
user service lists, and `journalctl --output=json` for logs. Unit names and
actions are validated before execution. System actions use the existing
polkit/pkexec system-settings helper; user actions use the user systemd manager.
Logs are sanitized before rendering, limited to 200 entries, and support unit,
priority (`0`-`7`) and current/previous boot (`b`) refresh filters. Follow mode
is intentionally deferred until streaming jobs are needed.

Rendering consumes cached state. Application discovery, fontconfig, system probes,
theme files, and logo decoding occur at startup or explicit update points, never
inside per-frame drawing code. See [docs/MIGRATION.md](docs/MIGRATION.md) for the
source-to-workspace map.

### Storage and diagnostics

The current tree also exposes read-only Storage and Diagnostics routes. Storage
uses structured `lsblk`/`findmnt` discovery, bounded filesystem usage probes,
swap inspection and optional SMART JSON. It never formats, mounts, repairs,
partitions, unlocks, or changes storage. Diagnostics produces objective checks
with severity and evidence from available snapshots; it does not apply
automatic fixes. Package diagnostics intentionally respect the incomplete
Recovery 5.2 state and do not claim package transactions, AUR review or mirror
editing are complete. See [docs/RECOVERY6.md](docs/RECOVERY6.md).

## ARGVUS themes

ARGVUS remains the source of truth. `argvus-theme` reads the active name from
`$XDG_CONFIG_HOME/argvus/data/.active-theme` (with the legacy root copy kept as a
read-only fallback for profiles that have not migrated yet), follows the shared
CSS resources and
imports in `/etc/argvus-settings`, and consumes the generated ARGVUS theme cache
used by other integrated applications. `ARGVUS_SETTINGS_RESOURCE_DIR` and the
normal XDG/ARGVUS config overrides are respected for development and user setups.

The resolver maps the global `argvus_*` palette to one semantic `Theme` used by
Home, Settings, About, tabs, links, selections, borders, and status messages. A
central ARGVUS Dark fallback keeps the application readable when a file or color is
missing. Official themes therefore require no palettes compiled into this binary;
reopening the application after a global theme change loads the new theme, in the
same consumer-oriented model used for Superfile and other ARGVUS terminal tools.

## Languages

All application areas share the `argvus-i18n` crate. It respects `ARGVUS_LANG`, then the
standard locale variables, and currently preserves the Portuguese and English
coverage of the source TUIs. Menu labels, tabs, breadcrumbs, controls, help,
feedback, errors, and About content all pass through that layer.

## Logo and terminal graphics

The About system tab preserves the existing renderer. It loads and rasterizes the
ARGVUS logo (`ARGVUS-logo.svg`) from `/usr/share/argvus/svg/` (provided by the
`argvus-branding` package) once, uses Kitty/Sixel/iTerm2 when detected, then falls
back to colored Unicode half blocks or ASCII. Resizing only rebuilds the cached
target when dimensions change. Set `ARGVUS_ABOUT_IMAGE=graphics`, `halfblocks`, or
`ascii` to force a backend for diagnostics.

## Screenshots

Place release screenshots in `docs/screenshots/`:

- `control-center-home.png`
- `default-apps.png`
- `fonts-search.png`
- `about-system.png`
- `about-credits.png`

## Phase 5 package inspection

The `argvus-control-center-packages` crate adds:

```text
Pacotes
├── Buscar
├── Instalados
├── Atualizações
├── Órfãos
├── Cache
├── AUR
├── Histórico
├── Downgrade
└── Mirrors
```

The package pages use asynchronous structured pacman queries and the shared
selection/modal/job infrastructure. Search, installed packages, updates,
orphans, cache inspection, history, AUR helper discovery and mirror inspection
are available in the TUI. Mutating package operations are routed through the
typed `system-settings package` boundary: install/reinstall/remove always use
full `-Syu` semantics, upgrades never use `-Sy`, the pacman lock is never
removed, and AUR builds run as the normal user. Cache cleanup uses paccache
when available, while mirror generation uses reflector output validation before
the existing backup/atomic-replace policy.

The package backend does not implement a dependency solver, signature
verification, or AUR build system itself; pacman and the selected AUR helper
remain authoritative. Interactive provider/conflict decisions and complete
transaction previews remain intentionally conservative limitations.
ARGVUS Control Center never performs partial upgrades.

## Recovery 1 — UX/UI and navigation

The Control Center domains use the same shared page shell as the canonical
Settings UI: outer border, ARGVUS header, breadcrumb, semantic theme colors,
selected-row treatment and contextual footer. `argvus-tui::page` provides this
shell and the bounded keyboard selection primitive. Esc and Left return one
level, and domain-home Back returns to global Home. Background probes remain
outside rendering. See [docs/RECOVERY1.md](docs/RECOVERY1.md).

## Snapshot

Open **Control Center > System Administration > Snapshot**, search for
`Snapper`, or run `argvus control-center snapshot`. The `snapshot` Cargo feature
is included in `full` (the default), and is independent of `storage`.

This page manages only an existing Snapper configuration whose `SUBVOLUME` is
`/` and whose `FSTYPE` is `btrfs`. It checks the mount containing `/` with
`findmnt`; a configuration named `root` pointing elsewhere is not accepted.
Multiple matching configurations require a selection. The `snapper` Arch package
is optional and must be installed manually. Snapper must support `--jsonout` and
explicit active/default columns. No configuration is created automatically.

On a non-Btrfs root the page displays **This filesystem is not Btrfs** and offers
no snapshot operations. Detection failure, missing Snapper, missing root
configuration, permission denial and backend errors are separate states.
If normal-user queries are denied, press **a — Authorize** to request Polkit
access. Administrative actions run in the restricted `system-settings` helper,
not in an elevated TUI; authentication may be requested again on later queries.
An existing graphical Polkit agent is required for authentication.

Keys: **↑/↓** select, **Enter** details/configuration, **c** create, **d** delete,
**r** refresh, **Esc** back, **?** help. Creation opens an optional description
(up to 256 printable characters); an empty description uses “ARGVUS manual
snapshot”. Both creation and deletion require confirmation, initially focused on
Cancel. Deletion applies to one selected snapshot and rejects snapshot 0, the
currently active snapshot and the default snapshot. The helper rechecks these
conditions before invoking Snapper. Lists refresh after operations; an interrupted
operation is never retried automatically. Inspect the refreshed list first.

Manual snapshots are `single`, read-only by Snapper's default, with no cleanup
algorithm assigned by this page. Existing Snapper timers and external operations
continue to apply. Snapshots are not full-system images: separate subvolumes,
including a separate `/home`, are not included. This version provides **no
rollback, restore, automatic cleanup, installation or Snapper configuration**.

### Manual validation

- On an ext4/xfs root, open the page in English and Portuguese. In Portuguese,
  verify the exact message **Este sistema de arquivos não é Btrfs** and verify
  that c/d do not open an operation.
- In a disposable Btrfs VM, test Snapper absent, Snapper without a root
  configuration, and a configuration named `root` pointing to another subvolume.
- With an existing root configuration, list snapshots, open details, create a
  snapshot with spaces/special characters, cancel a deletion, then delete only
  the newly created snapshot after confirming its number and description.
- Check denied/cancelled authentication, protected active/default snapshots,
  refresh, keyboard navigation, resize and both languages. Do not test deletion
  against snapshots needed for recovery.

Only the `argvus` package changes. Rebuild from `de/argvus` with `make build` to
produce the `.pkg.tar.zst` package; installing it is a separate operator action.

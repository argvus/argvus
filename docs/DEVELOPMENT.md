# Development

This repository is the `argvus` master package. It is the Rust workspace of the
`argvus` command (`crates/`), the Control Center, the configuration model and the
translations, and it also builds the greeter (`crates/argvus-greeter`, with its
handoff helper `crates/argvus-greeter-handoff`).
Every component package depends on `argvus`. Session, Hyprland and theme files live
in the component repositories listed in [README.md](../README.md).

Most changes here are dispatcher routes, packaging metadata or install and
validation tooling. Still, test the full session behavior in a real ARGVUS
session when possible, not only syntax.

## Workflow

The ecosystem workflow for building and managing ARGVUS is maintained in
[github.com/argvus/workflow](https://github.com/argvus/workflow). Use it as the
reference for the shared development process. This document only covers what is
specific to this repository.

## Repository layout

```text
Cargo.toml               workspace root; [workspace.dependencies] for every crate
crates/                  one crate per command or library (`argvus-cli` is the dispatcher; `argvus-greeter` keeps its README and greetd files in `crates/argvus-greeter/` (`packaging/greetd/`))
resources/locales/       shared translation catalogs (en-US, pt-BR)
src/                     data payload installed by the packages (themes, defaults, docs)
packaging/arch/remote/   PKGBUILD for tagged releases (pkgbase `argvus`, three split packages)
packaging/arch/local/    PKGBUILD for working-tree builds (same split packages)
packaging/arch/common/   functions.sh shared by both PKGBUILDs
tools/sh/                pkgbuild_local.sh, validate.sh, sync-skeleton.sh, install helpers
docs/                    documentation in en and pt-br; contributor files (CONTRIBUTING, SECURITY, CODE_OF_CONDUCT, AUTHORS, FLOWCHART)
.github/workflows/       ci.yml (validation on push and pull request to main)
                         release.yml (Arch packages for tags v*)
```

## Local setup

Install the dispatcher for user-level testing:

```sh
tools/sh/install.sh --user --restart
```

`--user` installs into `~/.local` by default (`--prefix <dir>` changes it).
`--restart` reloads the ARGVUS runtime after the install when it is available.

Install system-like paths for package testing:

```sh
tools/sh/install.sh --system
```

Use `--dry-run` with either mode to print the actions without changing files.

`install.sh` only installs the `argvus` command from this checkout. A complete
desktop comes from the packages (`sudo pacman -S argvus`), so test runtime
changes in the owning module repository.

## Build and validation

| Target | What it does |
|--------|--------------|
| `make build` (`make package`) | Runs `tools/sh/pkgbuild_local.sh`, writing archives to `build/artifacts/` and the package (`argvus`) to `build/dist/`. |
| `make install` | Installs the built packages with `sudo pacman -U build/dist/argvus*.zst --overwrite="*"`. |
| `make validate` | Checks the shared-file drift (`sync-skeleton.sh --check`) and the workspace version, then runs `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test --locked` for the workspace (the greeter included), then `tools/sh/validate.sh`. |
| `make set-permissions` | Makes the shell scripts executable. |
| `make clean` | Removes `build/`. |

`make help` also lists `make uninstall`, which has no target in this repository.

`tools/sh/validate.sh` requires `bash`, `makepkg` and `shellcheck`. It runs
`shellcheck` and `bash -n` on the shell scripts, verifies that the remote and local
PKGBUILDs have the same metadata, runs `makepkg --printsrcinfo` for both, and
checks `git diff --check`.

Common checks before a change is ready:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
make validate
(cd packaging/arch/remote && makepkg -p PKGBUILD --printsrcinfo >/dev/null)
```

For Hyprland changes, test inside a real ARGVUS session when possible. Check that
the session starts from a clean user, the taskbar appears, the Quickshell sidebar
toggles, theme switching creates only intentional user overrides, and package
defaults remain read-only.

## Packaging metadata

`packaging/arch/remote/PKGBUILD` and `packaging/arch/local/PKGBUILD` share the same
metadata (`pkgname`, `pkgver`, `pkgrel`, `pkgdesc`, `arch`, `license`,
`depends`, `makedepends`, `options`). Keep them identical when you edit
dependencies. `validate.sh` fails when they differ.

Only the source input differs: the remote PKGBUILD downloads the GitHub tag archive,
and the local PKGBUILD uses the archive generated from the working tree.

The `depends` array is the list of ARGVUS components and third-party packages
that define the desktop. Optional themes and applications belong in
`optdepends`.

## Configuration model

Package installation owns `/usr/share/argvus`. User configuration under
`$XDG_CONFIG_HOME/<app>` is an optional complete-application override, not a
startup requirement. Generated Argvus runtime config belongs under
`$XDG_CONFIG_HOME/argvus/data/generated` so theme changes can be rebuilt from the
current packaged defaults after upgrades.

Runtime scripts should source `/usr/share/argvus/session/sh/bootstrap.sh` unless
a user-copied override explicitly replaces that script. Keep Hyprland's Lua theme
loader aligned with the preference directory used by runtime scripts:
`$XDG_CONFIG_HOME/argvus/data`.

Do not make package install scripts write directly to `$HOME`. User-level
application config should be created only by explicit customization flows such
as `argvus --setup --copy <app>`. Theme tools should write small preference files
under `$XDG_CONFIG_HOME/argvus/data` and generated runtime config under
`$XDG_CONFIG_HOME/argvus/data/generated`, not native `$XDG_CONFIG_HOME/<app>` trees.

Hyprland user Lua overrides live under `$XDG_CONFIG_HOME/argvus/data/hypr`.
Supported files are loaded after packaged defaults in this order:
`monitors.lua`, `rules.lua`, `bindings.lua`, `user.lua`. Missing files must be
ignored.

The removable-devices helper is the `argvus removable-devices` route, provided by the
`argvus` package. Its system defaults belong under
`/usr/share/argvus/removable-devices/config` (shipped by `argvus`), and its system-wide
JSON config lives under `/etc/argvus/removable-devices/`.

`argvus-wallpapers` and `argvus-fonts` own wallpapers and bundled fonts. Desktop
configs should reference `/usr/share/backgrounds/argvus` and system fonts rather
than copying those assets into `~/.config`.

Kitty launch commands must pass the resolved Argvus `kitty.conf`, because Kitty
does not consume `/usr/share/argvus/terminal/config/kitty.conf` through
`XDG_CONFIG_DIRS`.

Use the `argvus --spf`, `argvus --yazi` and `argvus --system-monitor` routes for
bundled TUI apps that do not consume `/usr/share/argvus` directly. Each route
keeps `$XDG_CONFIG_HOME/<app>` as a native user override and otherwise points the
app at the Argvus config tree.

Yazi themes are shipped only as native flavors under
`argvus-app-profiles/src/usr/share/argvus/app-profiles/config/yazi/flavors/<theme>.yazi/flavor.toml`.
Theme switching writes `theme.toml` with the active Argvus flavor and fills
missing packaged flavors into `$XDG_CONFIG_HOME/argvus/data/yazi` when needed.

Btop themes must use the native `theme[key]="value"` syntax. Runtime refresh
copies the active packaged theme into `$XDG_CONFIG_HOME/argvus/data/btop` so older
materialized Argvus themes are repaired after package upgrades.

Foot themes are selected by rewriting the `include` in `foot/foot.ini` to the
active `foot/themes/<theme>/theme.ini`. When `foot` is the default terminal,
Hyprland should launch it with `foot -c <resolved foot.ini>` so the Argvus tree
is used even when the app does not read `$XDG_CONFIG_HOME/argvus/data` by itself.

Hyprlock lockscreen wallpaper caches belong under `$XDG_CACHE_HOME/argvus/hypr`;
do not use legacy `~/.cache/hypr` paths.

Session ownership: the `argvus` package owns `/usr/bin/argvus`. `argvus-session`
owns `/usr/lib/argvus/commands/session`, `/usr/lib/argvus/commands/start`, `/usr/lib/argvus/commands/tty`,
the user systemd units, and the Wayland session entry
`/usr/share/wayland-sessions/argvus.desktop`. The greeter that `greetd` starts is
the login greeter that this repository installs as part of the `argvus` package.

## Release flow

The Arch Linux packages are built from `packaging/arch/remote/PKGBUILD` in this
repository, which produces the single `argvus` package. The release
workflow runs when a `v*` tag is pushed. It sets `pkgver` in the remote PKGBUILD from the
tag, builds the packages, signs them and publishes each package and its `.sig` to
`argvus/packages`:

```text
public/arch/x86_64/argvus-<version>-1-x86_64.pkg.tar.zst(.sig)
```

The workflow also updates the `argvus` repository database files with
`repo-add -R` and signs the database artifacts.

The binary package is also uploaded as a temporary GitHub Actions artifact for
one day. It is not published through GitHub Releases.

Because `argvus` is an environment metapackage whose runtime dependencies
include Argvus components and packages that are not guaranteed to exist in the
base Arch runner, the workflow builds with `makepkg --nodeps`. Dependency
metadata remains declared in the `PKGBUILD`; the published repository is the
source of truth for installing those packages together.

Required repository secrets:

```text
PACKAGES_REPO_TOKEN
GPG_PRIVATE_KEY
GPG_KEY_ID
GPG_PASSPHRASE
```

Create an annotated git tag and push it to the remote:

```sh
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3
```

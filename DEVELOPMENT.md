# Development

This repository contains the packaged Argvus desktop defaults. Most changes are configuration changes, so development should focus on testing the full session behavior rather than only checking file syntax.

## Repository layout

```text
bin/       command entrypoints; the argvus package installs only argvus-setup
config/    packaged desktop defaults installed under /usr/share/argvus
packaging/ Arch Linux package metadata
tools/sh/  local development install and validation helpers
```

## Local setup

Clone `argvus-storage` next to this repository:

```sh
git clone https://github.com/argvus/argvus-storage ../argvus-storage
```

Install for user-level testing:

```sh
tools/sh/install.sh --user --restart
```

Install system-like paths for package testing:

```sh
tools/sh/install.sh --system
```

Use `ARGVUS_STORAGE_DIR=/path/to/argvus-storage` when the storage checkout is not next to this repository.

## Common checks

```sh
bash -n bin/argvus-setup
bash -n tools/sh/install.sh tools/sh/uninstall.sh
find config tools/sh -type f -name '*.sh' -print0 | xargs -0 -r sh -n
makepkg --printsrcinfo -p packaging/arch/PKGBUILD
```

For Waybar/storage integration: check the `argvus-storage` repository for
updated tests and run Waybar manually as needed.

For Hyprland changes, test inside a real Argvus session when possible. Check that the session starts from a clean user, the Waybar appears, the Quickshell sidebar toggles, theme switching creates only intentional user overrides and package defaults remain read-only.

## Configuration model

Package installation owns `/usr/share/argvus`. User configuration under
`$XDG_CONFIG_HOME/<app>` is an optional complete-application override, not a
startup requirement. Generated Argvus runtime config belongs under
`$XDG_STATE_HOME/argvus/config` so theme changes can be rebuilt from current
packaged defaults after upgrades.
Runtime scripts should source `/usr/share/argvus/scripts/argvus/bootstrap.sh` unless
a user-copied override explicitly replaces that script.
Keep Hyprland's Lua theme loader aligned with the preference directory used by
runtime scripts: `$XDG_CONFIG_HOME/argvus`.

Do not make package install scripts write directly to `$HOME`. User-level
application config should be created only by explicit customization flows such
as `argvus-setup --copy <app>`. Theme tools should write small preference files
under `$XDG_CONFIG_HOME/argvus` and generated runtime config under
`$XDG_STATE_HOME/argvus/config`, not native `$XDG_CONFIG_HOME/<app>` trees.

Hyprland user Lua overrides live under `$XDG_CONFIG_HOME/argvus/hypr`.
Supported files are loaded after packaged defaults in this order:
`monitors.lua`, `rules.lua`, `bindings.lua`, `user.lua`. Missing files must be
ignored.

`argvus-storage` is packaged separately. Its system defaults belong under `/etc/argvus-storage`, not under `config/argvus-storage`.

`argvus-appearance` owns shared wallpapers and bundled fonts. Desktop configs
should reference `/usr/share/backgrounds/argvus` and system fonts rather than
copying those assets into `~/.config`.
Kitty launch commands must pass the resolved Argvus `kitty.conf`, because Kitty
does not consume `/usr/share/argvus/kitty/kitty.conf` through `XDG_CONFIG_DIRS`.
Hyprlock lockscreen wallpaper caches belong under `$XDG_CACHE_HOME/argvus/hypr`;
do not use legacy `~/.cache/hypr` paths.

The `argvus` package owns `/usr/bin/argvus-setup`. `argvus-session` owns
`/usr/bin/argvus-session`, `/usr/bin/argvus-start`, `/usr/bin/argvus-tty` and
the Wayland display-manager entry under `/usr/share/wayland-sessions`.

## Release flow

The Arch Linux package is built from `packaging/arch/PKGBUILD` in this
repository. The release workflow runs when a `v*` tag is pushed, updates
`pkgver` from the tag, builds the package, signs it and publishes both files to
`argvus/packages`:

```text
public/arch/x86_64/argvus-<version>-1-x86_64.pkg.tar.zst
public/arch/x86_64/argvus-<version>-1-x86_64.pkg.tar.zst.sig
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
GPG_PASSPHRASE
```

Create an annotated git tag and push it to the remote:

```sh
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3
```

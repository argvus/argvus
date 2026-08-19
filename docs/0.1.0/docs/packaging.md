---
title: Packaging
description: Arch package layout, local builds and the publishing flow.
slug: 0.1.0/docs/packaging
---

The ARGVUS project is organized as several repositories under the `argvus` GitHub organization:

* [argvus](https://github.com/argvus/argvus): desktop configuration (config, bin/argvus-setup, share).
* [argvus-session](https://github.com/argvus/argvus-session): session launchers and the Wayland session entry.
* [argvus-appearance](https://github.com/argvus/argvus-appearance): shared wallpapers and bundled fonts.
* [argvus-storage](https://github.com/argvus/argvus-storage): Rust program powering the removable-storage Waybar module.
* [argvus-calendar](https://github.com/argvus/argvus-calendar): Rust program powering the Waybar calendar popup.
* [argvus-greeter](https://github.com/argvus/argvus-greeter): greetd greeter integration.
* [argvus-pkgbuild](https://github.com/argvus/argvus-pkgbuild): Arch PKGBUILDs for the desktop and its component packages.
* [packages](https://github.com/argvus/packages): binary package repository served at `https://argvus.github.io/packages/`.
* [site-src](https://github.com/argvus/site-src): this site's source (Astro).
* [argvus.github.io](https://github.com/argvus/argvus.github.io): the published site.

The Arch manifests live under `arch/` in [argvus-pkgbuild](https://github.com/argvus/argvus-pkgbuild): `arch/argvus/PKGBUILD`, `arch/argvus-session/PKGBUILD`, `arch/argvus-appearance/PKGBUILD`, `arch/argvus-storage/PKGBUILD`, `arch/argvus-calendar/PKGBUILD` and `arch/argvus-greeter/PKGBUILD`. Vendored dependencies (`pwvucontrol`, `snappy-switcher` and `wlogout`) live under [vendors/](https://github.com/argvus/argvus-pkgbuild/tree/main/vendors).

## ARGVUS Packaging

ARGVUS is packaged as an Arch Linux Hyprland spin split into focused component packages.

Package installation is system-only: it installs read-only defaults under `/usr/share/`, commands under `/usr/bin/`, and the display-manager session entry under `/usr/share/wayland-sessions/`. It never writes to `$HOME` during installation.

## Installed layout

The `argvus` package installs:

```text
/usr/bin/argvus-setup
/usr/share/argvus/
```

`argvus-session` installs the session entrypoints:

```text
/usr/bin/argvus-session
/usr/bin/argvus-start
/usr/bin/argvus-tty
/usr/share/wayland-sessions/argvus.desktop
```

`argvus-appearance` installs the shared assets:

```text
/usr/share/backgrounds/argvus/
/usr/share/fonts/
```

`argvus-storage` is packaged separately. It installs `/usr/bin/argvus-storage`, default configuration under `/etc/argvus-storage/`, and its license under `/usr/share/licenses/argvus-storage/`. See the [Removable Storage](/0.1.0/docs/storage/) page for usage and configuration.

`argvus-calendar` is packaged separately. It installs `/usr/bin/argvus-calendar`, defaults under `/etc/argvus-calendar/` and the user systemd unit at `/usr/lib/systemd/user/argvus-calendar.service`. See the [Calendar](/0.1.0/docs/calendar/) page for usage and configuration.

The `argvus` package depends on `argvus-appearance`, `argvus-calendar`, `argvus-greeter`, `argvus-session` and `argvus-storage`, so installing ARGVUS resolves them from the same repository.

## User configuration

ARGVUS does not copy dotfiles into `$HOME` before the desktop can start. Runtime entrypoints read complete user overrides from `$XDG_CONFIG_HOME/<app>`, generated runtime config from `$XDG_STATE_HOME/argvus/config/<app>`, and packaged defaults from `/usr/share/argvus/<app>`.

`$XDG_STATE_HOME` falls back to `~/.local/state` when it is unset. Generated runtime config is rebuilt from current package defaults plus ARGVUS preferences, so package upgrades can update `/usr/share/argvus` without overwriting user files.

`argvus-setup` is an optional helper. It copies `/usr/share/argvus/<app>` to `$XDG_CONFIG_HOME/<app>` on demand (`--copy <app>` or `--copy-all`), replacing an existing destination only with `--force`, which backs up the current directory as `$XDG_CONFIG_HOME/<app>.bak-<timestamp>`. Once copied, files are user-owned overrides that package upgrades never replace.

`argvus-tty` provides TTY login support without a display manager. It registers a logind session, starts D-Bus, sets environment variables, and launches Hyprland when the user runs `argvus-tty` manually after logging in on a TTY. The package does not install a `profile.d` auto-start script; an opt-in profile is shipped under `/usr/share/argvus/argvus/profile` and automatic TTY startup is an explicit user choice.

Some runtime dependencies that previously required the AUR, including `pwvucontrol`, `snappy-switcher`, and `wlogout`, are vendored and published from the same repository as ARGVUS:

```sh
sudo pacman -Syu argvus
```

## Package versioning

The component packages pin their source repositories by tag:

```sh
ENV_VER=0.1.0            # arch/argvus/PKGBUILD
pkgver=0.1.4              # arch/argvus-storage/PKGBUILD
pkgver=0.2.2              # arch/argvus-calendar/PKGBUILD
```

Bump the package version with `make version <X.Y.Z>` from the argvus-pkgbuild repository, keep `ENV_VER` aligned with the `argvus` tag, and bump the component PKGBUILDs when their repositories publish new tags.

## GitHub Actions

The `build-and-publish.yml` workflow in the argvus-pkgbuild repository runs on package changes and on manual dispatch. It builds every discovered Arch `.pkg.tar.zst`, including `argvus` and its component packages, signs the package files and pacman databases with the ARGVUS GPG key, generates `argvus.db` and `argvus.files`, and publishes the package files to the `main` branch of [packages](https://github.com/argvus/packages).

The packages repository layout:

```text
packages
  public/arch/argvus.conf
  public/arch/argvus.gpg
  public/arch/x86_64/argvus.db
  public/arch/x86_64/argvus.db.sig
  public/arch/x86_64/*.pkg.tar.zst
  public/arch/x86_64/*.pkg.tar.zst.sig
  public/debian/
  public/rpm/
```

The site deploy workflow (in the [site-src](https://github.com/argvus/site-src) repository) builds the static site with Astro and mirrors `dist/` to [argvus.github.io](https://github.com/argvus/argvus.github.io). It does not store binary packages.

Publishing the built packages to the packages repository requires `PACKAGES_REPO_TOKEN` (a fine-grained personal access token with `contents: write` on the packages repository) and `GPG_PRIVATE_KEY` (the ASCII-armored private key used only inside GitHub Actions). `GPG_PASSPHRASE` is optional when the private key is protected by a passphrase.

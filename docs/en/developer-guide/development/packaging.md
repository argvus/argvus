---
title: Packaging
description: Build and inspect ARGVUS Arch packages.
---

Each component owns its Arch packaging under its repository. Inspect `packaging/arch/`, the package functions and the installed file list before documenting a payload. Package defaults under `/usr/share/argvus` and administrator configuration under `/etc/argvus` are not interchangeable.

For runtime verification, compare package contents with `pacman -Ql <package>` and inspect active user overrides separately.

The workspace Makefile treats each desktop repository as an independent Arch package. `make build` builds installable projects that have a `build` target; `make build-selected` builds the selected development subset; `make collect` copies package archives into `builds/`; and `make install` installs the collected `argvus-*.pkg.tar.zst` files with pacman. These are developer workflow commands, not prerequisites for users installing from the ARGVUS package repository.

Do not infer the runtime payload from the repository name alone. A package may own binaries, user services, `/usr/share/argvus` defaults, `/etc/argvus` administrator configuration or assets. Confirm the actual payload with the repository's `PKGBUILD` and `pacman -Ql` after installation.

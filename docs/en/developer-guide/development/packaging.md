---
title: Packaging
description: Build and inspect ARGVUS Arch packages.
---

Each component owns its Arch packaging under its repository. Inspect `packaging/archlinux/`, the package functions and the installed file list before documenting a payload. Package defaults under `/usr/share/argvus` and administrator configuration under `/etc/argvus` are not interchangeable.

For runtime verification, compare package contents with `pacman -Ql <package>` and inspect active user overrides separately.

Local builds keep the Cargo objects in `build/cargo-target`, outside the package source tree, so a repeated `make build` recompiles only what changed. `make clean` removes that directory, and the next build starts cold.

Do not infer the runtime payload from the repository name alone. A package may own binaries, user services, `/usr/share/argvus` defaults, `/etc/argvus` administrator configuration or assets. Confirm the actual payload with the repository's `PKGBUILD` and `pacman -Ql` after installation.

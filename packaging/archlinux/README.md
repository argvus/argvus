# Arch packaging layout

The package payload is `packaging/archlinux/src/` (`usr/` and `etc/`, mirroring the installed root).

This package has separate local and remote PKGBUILDs (`pkgbuilds/local` and `pkgbuilds/remote`). The metadata and package payload are shared; only the source input
differs.

| Path | Use | Source |
| --- | --- | --- |
| `pkgbuilds/local/PKGBUILD` | `make build` from the working tree | generated local archive |
| `pkgbuilds/remote/PKGBUILD` | tagged release | GitHub tag archive |
| `pkgbuilds/common/functions.sh` | shared source normalization and payload logic | — |

One PKGBUILD (`pkgname=argvus`) produces one package with the command, its shared data,
the GTK 4 helpers and the login greeter, with `argvus.install` as its install hook.
`common/functions.sh` builds the root workspace (the greeter included, with the `greeter`
profile) and installs each part through `arch_package_argvus`, `arch_package_argvus_gtk`
and `arch_package_argvus_greeter`.

The local builder writes archives to `build/artifacts/` and packages to
`build/dist/`. Run `make validate` before building.

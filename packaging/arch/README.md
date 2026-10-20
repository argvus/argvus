# Arch packaging layout

This package follows the ARGVUS packaging skeleton with separate local and CI
PKGBUILDs. The metadata and package payload are shared; only the source input
differs.

| Path | Use | Source |
| --- | --- | --- |
| `local/PKGBUILD` | `make build` from the working tree | generated local archive |
| `remote/PKGBUILD` | tagged release | GitHub tag archive |
| `common/functions.sh` | shared source normalization and payload logic | — |

One PKGBUILD (`pkgname=argvus`) produces one package with the command, its shared data,
the GTK 4 helpers and the login greeter, with `argvus.install` as its install hook.
`common/functions.sh` builds the root workspace (the greeter included, with the `greeter`
profile) and installs each part through `arch_package_argvus`, `arch_package_argvus_gtk`
and `arch_package_argvus_greeter`.

The local builder writes archives to `build/artifacts/` and packages to
`build/dist/`. Run `make validate` before building.

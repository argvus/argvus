# Arch packaging layout

This package follows the ARGVUS packaging skeleton with separate local and CI
PKGBUILDs. The metadata and package payload are shared; only the source input
differs.

| Path | Use | Source |
| --- | --- | --- |
| `local/PKGBUILD` | `make build` from the working tree | generated local archive |
| `ci/PKGBUILD` | tagged release | GitHub tag archive |
| `common/functions.sh` | shared source normalization and payload logic | — |

The local builder writes archives to `build/artifacts/` and packages to
`build/dist/`. Run `make validate` before building.

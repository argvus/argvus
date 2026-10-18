# Phase 5 package architecture

`argvus-control-center-packages` is the package-domain boundary. It reuses the
shared `Capabilities`, `ProcessRunner`, `JobManager` and terminal sanitizer.
The first implementation uses pacman structured queries (`-Q --print-format`,
`-Qu`, and `-Ss`) instead of manipulating the pacman database. Package names
are validated before being passed as arguments and the database lock is
reported rather than removed.

The domain provides models/parsers for installed packages, repository search,
package details, updates, orphans, cache entries, pacman history, AUR helper
search and mirrorlists. The TUI exposes navigable lists, details,
confirmations and background results for package actions, cache cleanup, full
upgrades, local-cache downgrade and reflector mirror generation.

System mutations use the existing `system-settings` helper with a restricted
`package` area. The helper validates package names, cached package roots,
mirror content and cache policies again before invoking pacman/paccache or
performing mirrorlist replacement. The pacman database lock is reported and
never removed. No UI path can request an arbitrary root command.

Installation, reinstall and system upgrade use full-system `-Syu` semantics;
there is no `-Sy package` path. Pacman remains authoritative for dependency
resolution and signature checking. AUR builds remain user-owned and separate
from privileged package installation. Reflector writes are generated before
validation, confirmation, backup and atomic replacement.

# Git theme installation

`argvus theme install URL [--ref BRANCH_OR_TAG] [--replace]` imports exactly one
schema-2 skeleton theme. `argvus-theme-core::git_install` obtains a bare shallow
clone and exports HEAD with `git archive`; repository scripts, hooks and checkout
filters are not run. Only regular files and directories are extracted. Assets
are resolved within the exported source tree and rewritten relative to the theme.
`package::install_user_theme` validates both layouts and atomically publishes the
result in the XDG user theme directory, restoring the previous version if needed.

Discovery is the shared source for CLI and Control Center. Custom themes refresh
on page entry and through `r`. Installation never applies a theme; replacing an
active theme requires explicitly reapplying it. `package::is_active` checks the
current build, with the legacy active marker as fallback. CLI and UI refuse to
remove an active theme. User removal leaves generated builds intact.

Runtime dependencies are Git and, for SSH URLs, OpenSSH. Branches and tags use
`--ref`; default installs follow the remote default branch. Submodules and Git
LFS are unsupported. Official IDs already installed cannot be replaced.

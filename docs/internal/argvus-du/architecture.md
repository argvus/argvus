# Disk Usage Analyzer architecture

`argvus-du` is a private executable in the `argvus` Cargo workspace. The public
route is `argvus du`; the dispatcher forwards arguments to
`/usr/lib/argvus/commands/argvus-du`. The desktop entry uses that public route.

The helper opens its dedicated Kitty window with the packaged ARGVUS Kitty
policy, then starts its own TUI. The TUI embeds the real ncdu process through a
PTY, uses `argvus-tui::TerminalView` to parse and render its VT screen, and uses
`argvus-tui::TerminalGuard` to restore terminal state. Shared `argvus-theme` and
`argvus-i18n` provide the active palette and locale catalogs. Superfile selection
uses an isolated hotkey file in the user's cache; it does not modify the
configured hotkeys.

The workspace package build installs the private executable and payload files:
the `.desktop` entry, Kitty policy and `du` locale catalogs. `argvus-du` replaces
the former standalone package during upgrade. `ncdu` comes from the required
`argvus-devtools` dependency, and Kitty is provided by `argvus-desktop`.

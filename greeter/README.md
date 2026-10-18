# Argvus Greeter

Argvus Greeter is a keyboard-first Ratatui terminal frontend for
[greetd](https://git.sr.ht/~kennylevinsen/greetd) built for the Argvus Desktop
Environment.

It is designed for Wayland and for Argvus' Hyprland-based desktop, while still
discovering installed Wayland sessions instead of hardcoding a single session.
The packaged launcher opens it through `argvus tui-terminal` using Kitty's
graphics protocol, preserving avatars while remaining navigable with arrows,
Enter and Tab. Its colors and semantic controls follow the active ARGVUS theme
for the currently selected account; the greeter does not render a wallpaper.
Because the greeter runs before authentication as the isolated `greeter` user,
`argvus-appearance` publishes only the validated theme name to
`/var/lib/argvus/greeter/themes/<uid>`; private user configuration remains
unreadable to the login process.
The launcher also requests a Greeter-only Kitty profile, which keeps the
normal Kitty tab configuration unchanged while disabling tabs during login.

## Screenshot

Screenshot placeholder.

## Architecture

```text
systemd
   -> greetd
   -> minimal Wayland compositor session
   -> argvus tui-terminal
   -> argvus-greeter
   -> Wayland layer-shell handoff spinner
   -> selected Wayland desktop session
```

The greeter does not authenticate users itself. It uses the greetd IPC socket
from `GREETD_SOCK`; greetd then delegates authentication to PAM.
Diagnostic logs are written to the runtime greeter directory instead of the
interactive terminal, so backend messages cannot corrupt the TUI layout.

## Requirements

- Rust 1.95 or newer
- `argvus tui-terminal` and Kitty graphics support
- greetd
- A Wayland compositor suitable for running the greeter, such as Hyprland
- `argvus-appearance` for the official ARGVUS TUI themes
- systemd/logind for the power menu

## Building

```sh
cargo build --release
```

The binary is produced at `target/release/argvus-greeter`.

## Installation

Expected package locations:

- `/usr/lib/argvus/commands/argvus-greeter`
- `/etc/argvus/greeter.toml`
- `/usr/share/doc/argvus-greeter/`
- `/usr/share/backgrounds/argvus/`
- `/usr/share/wayland-sessions/argvus.desktop`

Arch packaging can install this repository's `packaging/greetd` examples as
documentation or adapt them into package defaults.

Arch packaging is owned by this repository through
`packaging/arch/ci/PKGBUILD`. Tag pushes build and publish signed
`.pkg.tar.zst` packages to the shared
`argvus/packages` repository.

## greetd Configuration

See:

- `packaging/greetd/config.toml`
- `packaging/greetd/hyprland-argvus-greeter.lua`

The example starts a dedicated minimal Hyprland instance for the greeter
through `/usr/lib/argvus/commands/argvus-greeter-session` and does not start the user's normal Argvus
desktop configuration before authentication. The launcher clears VT1 and
redirects compositor output to the greeter user's state directory, or to a
per-UID fallback under `/tmp` when greetd does not provide a writable `HOME`, so
boot or terminal logs do not remain visible under the login screen.
The greeter compositor still sets `XDG_CURRENT_DESKTOP=Hyprland` to satisfy
Hyprland's startup checks, while `XDG_SESSION_DESKTOP=argvus-greeter` identifies
the login environment. When authentication succeeds, the greeter starts the
normal `--spinner-only` layer-shell splash and waits for its first frame before
asking greetd to start the selected session. It keeps the terminal and minimal
compositor alive until greetd terminates the greeter session, so terminal
restoration cannot uncover a blank compositor. The user-session loading helper
then starts the same spinner as soon as the new Wayland socket appears and
stops it through the existing Hyprland readiness bridge.

When installed from the Arch package, apply the Argvus greetd configuration
with:

```sh
sudo argvus greeter-setup --enable
```

Use `--now` instead of `--enable` to restart `greetd.service` immediately.
The helper backs up an existing `/etc/greetd/config.toml` before replacing it.
If `/etc/greetd/config.toml` still points to `Hyprland --config ...conf`, run
the helper again so greetd uses the packaged `/usr/lib/argvus/commands/argvus-greeter-session` + Lua
configuration. Older direct commands can leave terminal output visible while
the graphical greeter starts.

The package synchronizes existing `.active-theme` files during installation.
To refresh the public pre-login theme projections manually without changing
greetd configuration, run:

```sh
sudo argvus greeter-setup --sync-themes
```

## Configuration

System configuration is read from `/etc/argvus/greeter.toml`.

```toml
[appearance]
show_clock = true
show_date = true

[session]
default = "argvus"
```

If the file does not exist, safe defaults are used.

## User Avatars

The greeter shows the avatar of the account selected on the login screen. The
avatar belongs to the system account, not to the greeter configuration; there
is no avatar entry in `greeter.toml` and no way to change an avatar from the
login screen.

### How it is resolved

For each user discovered in `/etc/passwd`, Argvus Greeter resolves the avatar
in this order:

1. `$HOME/.face` — written and validated by
   [`argvus-accounts`](https://github.com/argvus/argvus-accounts), the official
   account-metadata source of the Argvus desktop (regular PNG file, 256x256,
   mode `0644`, owned by the user);
2. `/var/lib/AccountsService/icons/<username>` — freedesktop
   [AccountsService](https://www.freedesktop.org/wiki/Software/AccountsService/)
   convention, kept for interoperability with GNOME/KDE and other display
   managers;
3. The `Icon=` path declared in `/var/lib/AccountsService/users/<username>`.

If none of these sources yields an existing, readable, regular image file —
or if the file cannot be decoded at render time (corrupted or unsupported
format) — the built-in Argvus default avatar (`assets/avatar-default.svg`) is
shown instead. No network access is ever performed.

Security details of the lookup:

- `.face` is checked with `lstat`: only real regular files qualify and
  symlinks are never followed, so a stale or hostile link cannot redirect
  the greeter to arbitrary paths;
- usernames are sanitized before being used in any path construction;
- reading `$HOME/.face` requires the user's home directory to be traversable
  by the `greeter` user (the Arch Linux default is `0755`). Hardened setups
  that restrict home directories should relax traversal for the greeter or
  keep using the AccountsService locations.

### Setting an avatar for a user

The preferred way inside Argvus is the account manager:

```sh
argvus accounts self avatar ~/Pictures/avatar.png   # own avatar, no privileges
sudo argvus accounts avatar alice photo.png         # another user, admin
```

As an administrator, the AccountsService locations also work and stay
compatible with other desktop environments:

```sh
sudo install -Dm644 photo.png /var/lib/AccountsService/icons/alice
```

PNG and JPEG images work out of the box on Arch Linux; SVG also works when
librsvg's gdk-pixbuf loader is installed. A custom location can be registered
through the AccountsService user file:

```ini
# /var/lib/AccountsService/users/alice
[User]
Icon=/var/lib/AccountsService/icons/alice
SystemAccount=false
```

Desktop environments that integrate with AccountsService (GNOME, KDE Plasma)
write both locations automatically when an avatar is set in their settings
panels.

Because the image path is re-read from disk every time the selected user
changes:

- changing a user's photo takes effect on the next login screen selection
  without rebuilding or restarting the greeter;
- selecting another user in the dropdown immediately switches name and avatar;
- users without a photo always receive the Argvus default avatar.

The login screen is strictly read-only with respect to avatars: there is no
button, menu, or flow to upload, choose, or remove them. Avatar lifecycle
(set, replace, remove) belongs exclusively to `argvus-accounts`.

## Development

Useful checks:

```sh
cargo build --release --locked
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```

The greeter expects `GREETD_SOCK` to be set by greetd for real authentication.
Without greetd it can still be compiled and inspected, but login cannot proceed.

## Security Notes

- Passwords and other PAM responses are never logged.
- Authentication is handled only through greetd IPC.
- The greeter never reads `/etc/shadow`.
- Session commands come from standard Wayland `.desktop` files and are parsed
  into argv vectors rather than passed through a shell.
- Power operations use systemd/logind over D-Bus.
- User discovery intentionally filters obvious service accounts and avoids
  depending on user home directories.

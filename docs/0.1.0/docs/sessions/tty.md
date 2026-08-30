---
title: TTY
description: Start ARGVUS directly from a TTY without a display manager.
slug: 0.1.0/docs/sessions/tty
---

ARGVUS can be started directly from a bare TTY without GDM or any other display manager. The package does not start Hyprland automatically after TTY login; the user decides when to start the session.

## How it works

After logging in on the TTY with your username and password, run:

```sh
argvus-tty
```

`argvus-tty` is provided by the `argvus-session` package and handles everything a display manager would normally do before starting Hyprland:

* Registers the session with logind (`loginctl open-session`)
* Starts the D-Bus session bus
* Sets the environment variables (XDG, Qt, Electron, Wayland)
* Launches the ARGVUS session (`argvus-session`)

On exit, the script cleans up (D-Bus, logind) and returns to the TTY.

## Setup

To use without a display manager, disable GDM if it is active and reboot:

```sh
sudo systemctl disable gdm
sudo reboot
```

Log in with your username and password at the TTY login screen. Then start the environment manually:

```sh
argvus-tty
```

For diagnostics:

```sh
argvus-tty --status
```

## Optional auto-start

The package does not configure TTY autologin and does not start Hyprland automatically by default. If you want the environment to start automatically after you log in manually on the TTY, add your own rule to `~/.bash_profile` or `~/.zprofile`:

```sh
# ~/.bash_profile
if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ] && [ "${XDG_VTNR:-}" = "1" ]; then
  exec argvus-tty
fi
```

There is also an opt-in profile shipped at `/usr/share/argvus/argvus/profile`. Copy it to `$XDG_CONFIG_HOME/argvus/profile` with `argvus-setup --copy argvus` and source it:

```sh
# ~/.bash_profile
[ -f "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/profile" ] && . "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/profile"
```

The profile auto-starts `argvus-tty` on the configured VT (default `1`) when no display manager is running. To disable it, export `ARGVUS_TTY_DISABLE=1`; to use a different VT, export `ARGVUS_TTY_VT=2`. It also skips auto-start when greetd owns the login flow.

## Logs

TTY session logs are stored at:

```sh
cat ~/.local/state/argvus/tty.log
cat ~/.local/state/argvus/session.log
```

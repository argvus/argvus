# FLOWCHART — How ARGVUS works

This document explains how ARGVUS is wired at runtime: where configuration lives,
how the desktop starts, what happens on the first login, and how the user changes
the theme, accent colour, spacing and other behaviour.

> **Summary in one sentence:** ARGVUS does not copy the packaged defaults into the
> home directory. The desktop reads the base from `/usr/share/argvus`, and
> `~/.config/argvus/data` only keeps the user's changes and state, created on demand.

---

## 1. Configuration locations

| Location | Role | Written by |
| --- | --- | --- |
| `/usr/share/argvus` | Packaged defaults (read-only base). | pacman (install and upgrade) |
| `~/.config/argvus/data/` | User layer: active theme, accent, spacing, borders, and the app configs the user or the runtime changed. | `argvus` commands and session scripts |
| `~/.config/argvus/data/generated/` | Derived runtime configuration, rebuilt from the defaults and the user's choices. | runtime |
| `~/.config/argvus/data/state/` | User state (the legacy `~/.local/state/argvus` is still read). | runtime |
| `~/.config/argvus/data/control-center/` | Control Center preferences, such as `defaults.json`. | Control Center |
| `~/.config/<app>/` | Manual per-app override. ARGVUS does not touch it. | user |
| `~/.cache/argvus/` | Disposable cache: session stamps, logs, daemon state. | runtime |

The data directory is `$XDG_CONFIG_HOME/argvus/data` by default and can be
overridden with `ARGVUS_DATA_HOME`.

### Read precedence

```text
~/.config/<app>/...                   manual override (wins)
  -> ~/.config/argvus/...             ARGVUS user layer
  -> ~/.config/argvus/data/generated  generated projection
  -> /usr/share/argvus/...            packaged default
```

The first file that exists wins. At each step the lookup also tries the legacy
name of the file: for example, `taskbar/config/...` is found as `waybar/...`.

### Why it is not a copy

ARGVUS does **not** run `cp -R /usr/share/argvus ~/.config/argvus` at install time.
A file is copied into the user layer only when something has to write to it
(theme, accent, spacing, or a manual `argvus --setup --copy`). Files that are
never changed keep being read from `/usr/share/argvus`.

---

## 2. Login and session start

```text
greetd
   │
   ▼
argvus-greeter                 login screen, run inside a minimal Hyprland
   │                           (argvus-greeter-session)
   │  user logs in
   ▼
start-hyprland -- --config <hyprland.lua>
   │
   ▼
hyprland.lua
   │  • reads the active theme and accent from the user layer
   │  • registers the keybindings
   ▼
argvus-session (systemd user session)
   ├── argvus-session-prepare.service   sessionctl prepare (section 3)
   ├── argvus-session-loading.service   loading overlay
   ├── argvus-taskbar.service           taskbar (waybar)
   ├── argvus-wallpaper.service         wallpaper
   ├── argvus-hypridle.service          idle and lock timeouts
   ├── argvus-dunst.service             notifications
   ├── argvus-keyboard-layout.service   keyboard layout
   └── argvus-session-reload.service    reload-desktop --login
```

`argvus-session` prepares the environment before Hyprland starts: its bootstrap
prepends the packaged project config directories to `XDG_CONFIG_DIRS`, so the
packaged files are found even when no user file exists.

### Key point

- **No `~/.config/argvus`?** The desktop still starts. Everything is read from
  `/usr/share/argvus`.
- **First login:** `session-prepare` finds no theme stamp and materializes the
  default theme (section 3).

---

## 3. What `session-prepare` does

`argvus-session-prepare.service` runs `sessionctl prepare`. The same logic is
available as `argvus config session-prepare`. The steps, in order:

1. Update the XDG user directories (`xdg-user-dirs-update`).
2. Migrate the legacy monitor file, if any (`displayctl --migrate-nwg`).
3. **Startup theme.** Runs `argvus theme-switch <active theme>` with
   `ARGVUS_NO_RUNTIME=1`, which only writes the theme files. The active theme
   is the one in `.active-theme`, or the packaged default `argvus-dark` when none
   was chosen. The step runs only when the materialization stamp
   (`~/.cache/argvus/session/theme-startup.stamp`) is missing, does not match the
   theme's fingerprint, or the theme files are not materialized. It does not
   commit the theme again, so a custom wallpaper is not reset on every login.
   If an accent is set, `argvus config accent --startup` runs afterwards, so the
   user's accent wins over the theme's own accent. If the theme was reapplied,
   Hyprland is reloaded through its IPC.
4. Set GTK, icon, cursor and font values with `gsettings`.
5. Sync the user files and run `spaces-switch.sh` (when `.active-theme` or
   `.spaces` exists) and `borders-switch.sh` (when `.active-theme` or `.borders`
   exists).
6. Apply the display layout (`argvus config display`).
7. Resolve the terminal editor (`get-default terminal_editor`).
8. Export the session environment (`QT_QPA_PLATFORM=wayland`,
   `QT_QPA_PLATFORMTHEME=qt6ct`, `EDITOR`, `VISUAL`, `PATH`) to Hyprland
   (`hyprctl setenv`) and to the systemd user environment
   (`sessionctl import-environment`).

The wallpaper, the taskbar, notifications and the other daemons are started by
their own units in section 2, not by `session-prepare`.

---

## 4. Mutable configs and why they are materialized

Some configs must be **rewritten** at runtime to reflect the active theme, the
accent colour or the spacing. A script such as `sed -i` needs a real, editable file.

The mutable lookup (`paths_config` in shell scripts, which sets
`ARGVUS_MUTABLE_CONFIG=1`, and `config_path` in `user_copy.rs`) works like this:

```text
paths_config("<app>/<file>")
   │
   ├── script or docs path (sh/, docs/)  → read from the live location, never copied
   │
   └── any other file
          │  the user copy already exists? → use it
          │  otherwise create it from the first source that has the file:
          │     1. the user-layer copy
          │     2. the legacy flat copy
          │     3. the generated projection (moved, not copied)
          │     4. the packaged file in /usr/share/argvus
          ▼
   the script edits the user copy and the owning service is reloaded
```

Waybar stylesheets are the exception: the packaged stylesheet is copied, because
the user copy is rewritten by every theme switch.

This is what lets the **Highlight color** control spread the accent colour across
the desktop without the user editing anything by hand.

> The precedence chain exists for this reason: the runtime always reads the most
> specific file and writes only to the user layer (`~/.config/argvus/`), never to
> `/usr/share/argvus`.

---

## 5. Theme, accent and spacing

### Theme (`SUPER + SHIFT + T`)

1. The keybinding runs `argvus theme-switch`.
2. The chosen theme is stored in `~/.config/argvus/data/.active-theme`.
3. The switch writes the theme's files into the user layer and applies them to the
   affected apps.
4. On the next login, the startup theme step (section 3) re-materializes the files
   if they are missing or outdated.

### Accent colour (**Highlight color** control)

1. The colour is stored in `~/.config/argvus/data/.accent-color`.
2. `argvus config accent` rewrites the accent in the user copies of the affected
   apps and applies it.

### Spacing (`.spaces`)

1. The value is stored in `~/.config/argvus/data/.spaces`.
2. `spaces-switch.sh --apply-static` computes the effective geometry and applies
   the Waybar margins.

Borders follow the same pattern with `.borders` and `borders-switch.sh`.

---

## 6. Default applications

Default applications per category (terminal, editor, browser and so on) are chosen
in the Control Center and stored in
`~/.config/argvus/data/control-center/defaults.json`.

`get-default.sh <category>` resolves the value with this precedence:

```text
~/.config/argvus/data/control-center/defaults.json   (user)
  -> $XDG_STATE_HOME/argvus/defaults.json             (legacy, read during migration)
  -> /usr/share/argvus/defaults.json                  (packaged)
```

If no value is set, the packaged default for that category is used.

---

## 7. `argvus --setup` (optional)

`argvus --setup` is **not required** for the desktop to work. It exists for users
who want to edit packaged files by hand:

```sh
argvus --setup --copy <app>        # copy the packaged files of <app> into the user layer
argvus --setup --copy-all          # copy every app
argvus --setup --copy-all --force  # overwrite existing copies (backups are created)
```

Once copied, a file belongs to the user, and package upgrades do not overwrite it.

---

## 8. Example lifecycle (first boot to a theme change)

```text
1. Install:     /usr/share/argvus  <-- package (nothing in ~/.config/argvus yet)
2. First login: sessionctl prepare
                  • no theme stamp → theme-switch materializes the default theme
                  • gsettings sets GTK, icon and cursor; wallpaper, taskbar and
                    the other units start
3. Use:         every file is read from ~/.config/argvus (wins) or /usr/share/argvus
4. Highlight color control → argvus config accent rewrites the accent in the user layer
5. Reboot:      hyprland.lua reads the theme and accent from the user layer → same session
```

---

## 9. Golden rules

- **`/usr/share/argvus`** is read-only (packaged base).
- **`~/.config/argvus/data`** holds the user's changes and state.
- **`~/.config/<app>`** is a free manual override (optional).
- **`~/.cache/argvus`** is disposable.
- Packaged **`scripts/`** and **`docs/`** are never copied into the user layer and
  never overwritten by the runtime.
- Terminal UI apps with their own config search should be started with the
  `argvus` subcommands, which respect `~/.config/<app>` as the native override:
  `argvus --spf` (superfile), `argvus --yazi` (Yazi, which uses the native flavors
  it manages), and `argvus --system-monitor` (btop).

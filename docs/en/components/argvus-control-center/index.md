---
title: Control Center
description: Configure the persistent parts of your ARGVUS desktop.
---

The Control Center is ARGVUS's keyboard-first settings application. Use it when you want to configure the desktop rather than perform a one-time action.

## Control Center and Control Panel

| Use… | For… |
| --- | --- |
| **Control Center** | Themes, wallpapers, effects, layout, taskbar behavior, fonts, default applications, keyboard and input settings, language, region and system configuration. |
| **Control Panel** | Current status and frequent actions such as volume, brightness, network, notifications, power and session controls. |

The Control Panel can expose quick controls for a setting that is configured permanently in Control Center. For example, use the panel for a quick appearance or display action, and Control Center when you want to change the desktop configuration behind it. See Control Panel and [Where to configure things](/docs/user-guide/where-to-configure/).

## Main areas

The current Control Center home provides these areas:

The home is a catalog of destinations grouped in six cards. Each destination opens its page directly, and the search finds it under any card.

- **Appearance** — customize the desktop: Desktop Style (theme, appearance mode, accent color, wallpapers, icons and fonts), Application Style (terminal and launcher transparency), Taskbar, Control Panel and Widget Telemetry.
- **Desktop & Input** — configure the workspace: Hyprland (window rules, window spaces, animations, blur, borders and the installed version), Displays, Keyboard & Shortcuts and Mouse & Touchpad.
- **Preferences** — personal and regional settings: Language & Region (the language, time zone, formats and keyboard layout hub), Date & Time, Default Applications, Power and Session.
- **Hardware & Connectivity** — devices and connections: Hardware Overview, Network & Wi-Fi, Bluetooth, Audio and Storage.
- **System Administration** — manage the operating system: Packages, Services, Boot, Hostname, Users & Groups, Firewall and Notifications (Do Not Disturb).
- **Settings** — the Control Center itself, listed after the cards: Language (the interface language) and Style (decorative icons and transparency). Press `s` to open it.
- **Tools & Diagnostics** — developer and maintenance tools: Projects, Snippets (text saved for `argvus snippets`, typed with `SUPER + ALT + N`), SSH keys (key pairs in `~/.ssh` managed by `argvus keys`), GPG keys (the keyring managed by `argvus gpg`; gpg-agent uses pinentry-gtk), Dev Services, Diagnostics and About ARGVUS.

The exact provider pages can depend on installed packages and permissions. Do not assume that a missing optional domain is a broken Control Center.

When the grid has more entries than fit on screen, a scrollbar appears along the right edge to show there is more below.

## Finding a setting

The home screen registers settings by title, category, identifier and keywords. Search is accent-insensitive and accepts practical terms such as `theme`, `wallpaper`, `shortcut`, `mouse`, `monitor`, `network`, `user` and `DPI`. Exact title matches are ranked first, followed by a title prefix, any word of the title or category that contains the typed text, and finally keyword matches — so typing any part of a word (not only its start) can still find the destination.

The visible menu is capability-aware. A route can be absent when its provider is not installed, when the hardware is not available, or when the current session cannot provide the required capability. Search is therefore also a useful way to discover what this installation can configure.

The main settings tree currently includes:

| Area | Current destinations |
| --- | --- |
| Appearance | Desktop Style: theme, appearance mode, accent color, wallpapers, icons and fonts; Application Style: terminal and launcher transparency; Taskbar; Control Panel; Widget Telemetry |
| Desktop & Input | Hyprland (window rules, window spaces, animations, blur, borders), Displays, Keyboard & Shortcuts, Mouse & Touchpad |
| Preferences | Language & Region (language, time zone, formats, system locales and keyboard layout), Date & Time, Default Applications, Power, Session |
| Hardware & Connectivity | Hardware Overview (summary, CPU, GPU, memory, devices), Network & Wi-Fi, Bluetooth, Audio, Storage |
| System Administration | Packages, Services, Boot, Hostname, Users & Groups, Firewall, Notifications |
| Settings | Language (interface language), Style (icons and transparency) |
| Tools & Diagnostics | Projects, Snippets, SSH keys, GPG keys, Dev Services, Diagnostics, About ARGVUS |

Some of these are full pages and some are subpages reached through search or a domain summary. The exact list is determined by the installed ARGVUS providers.

Each page shows its path in the header, for example `Control Center > Hardware & Connectivity > Audio`. The path starts with the Home card that lists the page, so the same page reads the same from the Home, from search and from the CLI. The root and the card names are translated: the Portuguese interface shows `Central de Controle > Hardware e conectividade > Áudio`.

## What each area is for

### Appearance

Appearance is the main customization area. Its Desktop group holds the theme (with its Sticky or Float mode), the appearance mode, the accent color, the wallpapers, the icon theme and the Fonts page, and its Application Style group holds the terminal and launcher transparency. Taskbar, Control Panel and Widget Telemetry are listed directly under Appearance on the home and reuse Appearance's shared transparency page for their own surfaces. Blur is one global setting, under Hyprland > Blur, and it applies to every ARGVUS surface that has blur enabled. That page edits its Enable switch and the Hyprland blur values (Size, Passes, Brightness, Noise, Contrast, Vibrancy and Vibrancy darkness) as a draft, and nothing changes until **Apply** is selected. Animations works the same way under Hyprland > Animations: its Enable switch, the Custom curves (Bézier, Spring, Linear and Ease-out) and the five animation categories form one draft that is applied together with **Apply**. Bézier and Spring have editable parameters (the Bézier points and the spring mass, stiffness and dampening); Linear and Ease-out are fixed curves, so their page only lists their control points and cannot be edited. The style choices follow what Hyprland 0.56.2 draws: Opening and Closing offer slide, popin and gnomed, layer entrance and exit offer slide, popin and fade, and the workspace leaves offer slide, fade, slidefade, slidevert and slidefadevert. These controls coordinate with the appearance and session components instead of changing only the Control Center window.

### Taskbar

Taskbar configures the panel's position, the spacing around it and its windows, its height and border, the utility icon group, and the date/time blocks, plus its own transparency. It is listed under **Appearance** on the home.

**Height** chooses 24, 28 (the default), 32 or 36 pixels. **Border** has an **Enable** switch and a **Thickness** of 1 to 3 pixels, with 1 as the default. With the border off, the taskbar window, the right group and the MPRIS player have no border. These changes are a draft until **Apply** is selected. Switching the theme restores the theme's height and border.

The **Icons** page lists the Audio Player view and each utility widget (Network, Power profile, Keyboard layout, Memory usage, CPU usage, CPU Temperature and GPU Temperature) first, as checks with their own icon. The **Launcher** and **Utilities** submenus come last.

### Control Panel

Control Panel configures the visibility and order of its quick-action cards, plus its own sessions and transparency.

The page opens and accepts input as soon as the card settings are read. The cards that depend on hardware (Bluetooth and Brightness) are checked in the background: until that check finishes they keep the availability last seen in this window, and the list updates on its own when the result arrives.

### Widget Telemetry

Widget Telemetry configures the system-information widget's sessions and transparency.

### Fonts

Fonts is reached from **Appearance > Desktop Style > Fonts**. It lets you select a family and size for the taskbar, telemetry/system information, Control Panel, ARGVUS system interface, applications, terminal and browser. It also exposes font rendering settings such as antialiasing, hinting, subpixel mode and DPI. The default primary family is IBM Plex Mono; individual targets can be changed independently and restored from the Fonts page.

### Default applications

Default applications selects installed programs for roles such as terminal, file manager, text editor, terminal editor, browser, image viewer, PDF viewer, video player, audio player, archive tool and launcher. The selection is saved by ARGVUS and may update standard XDG associations.

### Snippets

Snippets is the page for the text that `argvus snippets` types with `SUPER + ALT + N`. It is in **Tools & Diagnostics → Snippets** (search `snippets` or run `argvus control-center snippets`).

- **New snippet** — type a **Name** and a **Content**, then **Save snippet**. A name that already exists replaces that snippet's content.
- **Saved snippets** — listed with their position (`1  email`) and the content on one line. Enter on the content edits it. **Type** types the snippet, and **Remove** asks for confirmation.
- **Open picker** — opens the same picker as `SUPER + ALT + N`.

Typing starts 3 seconds after you press Enter, because the Control Center itself has the focus at that moment. Click the field that should receive the text within that time. The page only edits one-line content; a multi-line snippet can be created in a terminal with `argvus snippets add`.

The page reads the entries from `argvus-config` and writes them through `argvus snippets`, so the launcher keeps the validation. When `argvus snippets` is not installed, the page says so.

### SSH keys

SSH keys is the page for the key pairs that `argvus keys` manages. It is in **Tools & Diagnostics → SSH keys** (search `ssh keys` or run `argvus control-center ssh-keys`).

The page has four parts, from top to bottom:

```
Search        filters the key pairs by name or comment
Generate      name, type and the Generate key pair action
Keys          one block per key pair; a scrollbar appears when they do not fit
Actions       Refresh and Unload all keys from agent
```

#### How to use it

1. Open **Control Center → Tools & Diagnostics → SSH keys**. From the Home, you can also search for `ssh keys`.
2. **Find a key:** press `/`, then type to show only the pairs whose name or comment matches. `Enter` ends the search and runs the highlighted row, as in the other lists, so check the row before pressing it; `Esc` clears the search.
3. **Create a pair:** in **Generate**, select **Name**, press `Enter`, type the name and press `Enter` again. Select **Type** and press `Enter` to switch between `ed25519` and `rsa`. Select **Generate key pair** and press `Enter`. The new pair appears under **Keys** at once, without a passphrase.
4. **Copy the public key:** in the pair's block, select **Copy public key** and press `Enter`. The contents of the `.pub` file go to the Wayland clipboard, ready to paste into a server's `authorized_keys` or a Git host. The private key is never copied. This needs `wl-clipboard` (`wl-copy`); without it, the page says the key was not copied.
5. **Load a pair into the agent:** in the pair's block, select **Load into agent** and press `Enter`. A key without a passphrase is loaded at once, with no window. A key with a passphrase opens an ARGVUS terminal, where OpenSSH asks for it; close the terminal with `Enter` when it says the key is loaded. The page checks the agent every few seconds while it is open, so the row changes to **Unload from agent** by itself. To update at once, use **Refresh** in **Actions**.
6. **Remove a key from the agent:** in the pair's block, select **Unload from agent**. To remove every key, use **Unload all keys from agent** in **Actions**. The key files stay on disk.
7. **Delete a pair:** in the pair's block, select **Delete** and press `Enter`. The confirmation focuses **Cancel**, so a stray `Enter` does nothing. Press `y` to confirm, or `n`/`Esc` to cancel. A key that is loaded in the agent cannot be deleted; unload it first.

Moving around: `↑`/`↓` or `j`/`k` move through the list, `Enter` activates a row, `Esc` goes back and `?` shows help. `Tab` moves the focus into **Actions**, where `←`/`→` choose a button and `Enter` runs it. `↑`/`↓` bring the focus back to the list.

Passphrases are never typed into the Control Center and never passed as command arguments. The page reads the list with `argvus keys list` and writes only through `argvus keys`. When `argvus-keys` is not installed, the page says so.

#### Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| `Could not open a connection to your authentication agent` after **Load into agent** | The terminal did not receive the agent socket. Hyprland keeps the environment it started with, and the agent may have started later. Update `argvus-launcher`, which falls back to `$XDG_RUNTIME_DIR/ssh-agent.socket`, and check `systemctl --user status argvus-ssh-agent.service`. |
| **Agent** shows `Agent unavailable` | The launcher cannot reach the agent. Check the service as above. |
| The page says `argvus-keys is not installed` | Update the `argvus` package: `argvus keys` is a subcommand of it. |
| **Delete** shows an error | The key is loaded in the agent, and it is kept. Unload it first, then delete it again. |

### GPG keys

GPG keys is the page for the keyring that `argvus gpg` manages. It is in **Tools & Diagnostics → GPG keys** (search `gpg keys` or run `argvus control-center gpg-keys`).

The page has these sections, from top to bottom:

```
Pinentry         the pinentry gpg-agent uses, for information only: pinentry-gtk
Generate key ▾   full name, e-mail, comment, algorithm, key strength, expiration date,
                 passphrase, and the Generate key and Cancel actions (expanded at first)
Import ▾         Import public key… and Import private key… (expanded at first)
Keys ›           the keyring, on its own page, with the search at the top
```

`▾` marks an expanded section and `▸` a collapsed one. `Enter` or `Space` expands or collapses it.

#### How to use it

1. Open **Control Center → Tools & Diagnostics → GPG keys**. From the Home, you can also search for `gpg keys`.
2. **Pinentry** is fixed to `pinentry-gtk`. The page only shows it. The first time a key is generated, ARGVUS writes that pinentry into `gpg-agent.conf`, between its own marker lines, and keeps your other lines. If you already wrote a `pinentry-program` line outside those markers, generation stops with an error so that you can remove that line.
3. **Create a key:** in **Generate key ▾**, type **Full name** and **E-mail**. Both are required, and **Generate key** stays disabled until both have text. **Comment** is optional. The user ID becomes `Name (comment) <e-mail>`.
   - **Algorithm** switches between **RSA** and **DSA ElGamal**. A DSA key gets an ElGamal encryption subkey; an RSA key gets an RSA one.
   - **Key strength** cycles through the sizes of the algorithm: RSA 1024, 2048 and 4096; DSA 1024, 2048 and 3072. gpg does not create DSA keys above 3072 bits, so 4096 is not offered for DSA.
   - **Expiration date** opens its own page. Type the date in the layout of your language: `DD/MM/YYYY` in pt-BR, `YYYY/MM/DD` in English. The date must be after today. Turn on **Never expires** to make the key permanent; the date field is then disabled and not used.
   - **Passphrase** opens its own page. **Protect with a passphrase** is on by default; type the **Passphrase** and then **Confirm passphrase**. The two must match. Turning protection off clears both fields, and the page asks for confirmation before it generates an unprotected key.
4. Select **Generate key** and press `Enter`. The key gets a signing key and an encryption subkey. The passphrase is sent to the launcher on its standard input, never as a command argument. If generation fails, the other fields stay, but the passphrases are cleared and must be typed again.
5. **Cancel** clears every field of the form and returns the dates and sizes to their defaults.
6. **Import:** in **Import ▾**, choose **Import public key…** or **Import private key…**. The file manager opens so that you can choose the file. A private key asks for confirmation first, and its passphrase is asked by gpg when the key is used. If the file manager is a graphical one, it cannot report the file to the page; the page then opens a field where you type the path.
7. **Keys ›** opens the keyring. Each key shows its identity, fingerprint, type, expiry and whether its private key is in the keyring. Each key has these actions:
   - **Edit key ›** opens the key's own page (described below).
   - **Export public key…** asks for a file path, proposed as `~/<fingerprint>.asc`. The file is created with mode `600`. An existing file is never replaced.
   - **Export private key…** appears only for keys with a private part. It asks for confirmation, then for a file path, proposed as `~/<fingerprint>-private.asc`. The file is created with mode `600`.
   - **Copy public key** and **Copy fingerprint** copy to the Wayland clipboard with `wl-copy`.
   - **Delete key…** asks for confirmation, with the focus on Cancel. It removes the key and its private part.
8. **Find a key:** on the **Keys** page, the search bar at the top filters the keys. Type to narrow the list; the search matches the identity (which includes the comment) and the fingerprint. `Esc` clears the search.
9. **Edit key ›** shows the key's details, then these actions:
   - **Expiration date ›** changes the expiry of the key and of each of its subkeys. Use **Never expires** to remove the expiry. **Apply** saves the change.
   - **Change passphrase** asks the old and the new passphrase in the pinentry window.
   - **Sign key with ›** certifies this key with one of your other keys that has a private part. The signer's passphrase is asked. The page asks for confirmation first.
   - **Revoke key ›** lists the reasons: no reason, key compromised, key superseded, key no longer used. Each one asks for confirmation. A revoked key cannot be used any more.
   - **Trust** sets how far you trust the key: not set, marginal, full or ultimate.
   - **Publish to keyserver…** sends the public key to the keyserver set in `gpg.conf`, after confirmation. **Fetch from keyserver** downloads the key from that keyserver.

Moving around: `↑`/`↓` or `j`/`k` move through the list, `Enter` activates a row, `Esc` goes back and `?` shows help.

Passphrases are typed only into the masked fields of the passphrase page, or into the pinentry window. The page sends the generation passphrase to `argvus gpg` on standard input, and the launcher hands it to gpg on a file descriptor, so it never appears in the process list, in a command argument or in the logs. Key changes run in the background; while one runs, the others wait. The page reads the keyring with `argvus gpg list` and writes only through `argvus gpg`. When `argvus gpg` is not installed, the page says so.

Not in the page yet: searching the keyserver for a key by name or e-mail. Use **Fetch from keyserver** with a known fingerprint.

#### Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| The page says `argvus gpg is not available` | The installed `argvus-launcher` predates the GPG command. Run `argvus gpg list` in a terminal: `unknown command` means the package must be rebuilt and installed. Keys also need `gnupg`. |
| **Generate key** stays disabled | Type both **Full name** and **E-mail**. |
| A status line says `argvus-gpg: pinentry-gtk is not installed` | Install the `pinentry` package, which provides `pinentry-gtk`. |
| Generation stops with a `pinentry-program` error | `gpg-agent.conf` sets a pinentry outside the ARGVUS block. Remove that line; ARGVUS then writes its own. |
| **Pinentry** shows `Not set` | gpg-agent uses its built-in default until the first key is generated from the page. |
| The page says the installed `argvus-launcher` is older than the Control Center | Reinstall `argvus-launcher`: run `make build` and then `make install` in its repository, and reopen the page. The key actions stay off until the versions match. |
| The file manager opens, but the page asks for a path | The file manager is a graphical one and cannot report the chosen file. Type the path in the field. |
| **Copy public key** says it could not copy | `wl-copy` is not installed or the Wayland session is not available. Install `wl-clipboard`. |
| **Edit key ›** says the key is no longer in the keyring | The keyring changed since the page opened. Go back to **Keys ›** and refresh. |
| A status line shows `argvus-gpg: ...` | The launcher refused the action, and the text gives the reason, such as an invalid expiry, a file that already exists, or a missing `--yes`. |
| An action waits and nothing happens | The pinentry window is hidden or was not opened. Check the session exposes `WAYLAND_DISPLAY` or `DISPLAY`. |
| The import is refused | The file contains no key of the kind chosen. Public keys go through **Import public key…**; private keys through **Import private key…**. |

### Language & Region

This card lists Language (the regional locale), Time Zone, Date & Time, System Locales and Keyboard Layout, and each one opens directly from the home. The keyboard pages also hold the variant and console keymap. The home search also exposes keyboard shortcuts and mouse/touchpad settings as direct settings routes.

### Hardware and displays

When the corresponding capabilities are installed, the Control Center can show hardware information such as CPU, GPU, memory, power and devices. The input page configures mouse and touchpad behavior; the display area handles monitor resolution, refresh rate, scale, position, orientation, primary display, VRR and HDR where supported by the running session. Display changes apply immediately. After a change that can leave the screen unusable (resolution or refresh rate, position, mirroring, disabling a monitor, color depth) or after **Apply**, Control Center asks whether to keep it, with the focus on **Revert**: press `y` to keep it; `n`, `Esc` or `Enter` revert it, and without an answer within 15 seconds the previous configuration comes back automatically. Removing the saved configuration of a disconnected monitor and deleting a profile ask for confirmation; profiles open their own page with **Apply profile**, **Rename** and **Delete profile**. The layout is saved through `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` and `monitor-remove`, so the canonical `/displays` section of `config.json` is the only source of the monitor layout. Only the profile catalog stays in `display/config.toml`.

### Connectivity and audio

The network area exposes the installed network provider's status and connection pages, including Wi-Fi, Ethernet, VPN, DNS and proxy routes when available. The firewall is listed under **System Administration**. Bluetooth is conditional on Bluetooth support. The audio area is provided when its optional capability is enabled and integrates with the installed audio service.

The **Network & Wi-Fi** home and its **Status** page fill in as each NetworkManager query answers, instead of after the slowest one: the device list first, then the radio, the connected interfaces' addresses and DNS, the saved connections and the Wi-Fi networks NetworkManager already knows (a new scan only runs from the Wi-Fi page with `r`). Until the device list arrives the rows say *Loading...*, and the Wi-Fi switch is only offered when the computer has a Wi-Fi card. Starting an action (the Wi-Fi switch, connecting, DNS) while a page is refreshing replaces the refresh, and the page refreshes again when the action ends.

### Power and session

Power provides the system power controls and policy pages available to the installation, including the Keep Awake toggle. When enabled, Keep Awake prevents the ARGVUS idle policy from starting automatic screen-lock and display-power timers. Session exposes ARGVUS session status, components, autostart, diagnostics and logs. On **Power**, the **Actions** bar under the list holds **Suspend**, **Hibernate** and **Refresh** (`r` refreshes too): `Tab` or `←/→` move to it, `Enter` runs the focused button, `↑`/`↓` bring the focus back to the list, and `Esc` goes back. Suspend and Hibernate ask for confirmation first. These pages may require system permissions and should be distinguished from the quick actions in the Control Panel.

### System tools

The system section can expose boot information, packages, services, storage, diagnostics, user/group administration and system information. These are administrative or diagnostic tools; a page may be read-only or require authorization depending on the operation.

**Dev Services** reports three development-facing sources in one read-only page: the current user's systemd services, the TCP ports currently listening (`ss`), and the running containers from any installed container runtime (Podman, Docker). It only appears on the Home grid when at least one of these is available on the system — a machine with no systemd user session, no Podman and no Docker does not show the entry. `r` refreshes the three sources independently, so a slow or failing one (for example a container runtime that stopped responding) does not block the others from showing their data.

On the **Packages** lists that filter while typing (Install / Official, Install / AUR, Installed / Official, Installed / Orphans and Updates / Official), letters go to the filter, shown in the **Search** box fixed at the top of the page; the packages are listed under it in the **Results** block (**Updates** on the Updates page), and an empty list says why (for example **No results** or **No matching items**). Use the arrow keys to move and `/` to type a new search. `Enter` on a package opens its page, with **Install** (or **Reinstall**) and, in its **Danger zone**, **Remove**. On **Orphans**, `Space` marks packages (`[x]`) and **Remove marked** removes them together. On **Updates**, the **Actions** bar under the list holds **Upgrade all** and **Refresh database**: `←`/`→` or `Tab` move to its buttons, `Enter` runs the focused one, and `↑`/`↓` (or typing) bring the focus back to the list; **Upgrade all** is dimmed and skipped while no update is listed, and an empty list says **No pending updates**. Since `←`/`→` belong to the bar there, `Esc` goes back. The breadcrumb and the global search name each Packages list by its Home row: **Install Official**, **Install AUR**, **Installed Official**, **Installed Orphans** and **Updates Official**. The cache cleanups are in the Danger zone of the Cache page, and **Mirrors → Configure mirrors** sets the reflector options before **Generate preview**. Every package operation shows its plan and asks for confirmation (`y` confirms, `n` or `Esc` cancels) before it runs, and its output is shown while it runs.

### Users and groups

When the account provider is available, open **Control Center → System Administration → Users & Groups → Users** or search for **users**. The interface can list normal and system accounts, create a user, edit account metadata, manage supplementary and primary groups, change or lock a password, unlock it, require a password change at the next login, set or remove an avatar, and enable automatic login.

User creation asks for a username, full name, shell, supplementary groups and an optional password confirmation. A blank password creates the account with its password locked; it does not silently create a usable password. A user page groups the account fields (full name, shell, primary and supplementary groups, followed by **Save changes**), the **Automatic login** toggle, the **Password** actions (change, lock, unlock, require a change at the next login) and the **Avatar** actions. Username, UID / GID and home directory are shown for information only. The **Danger zone** at the end holds the separate destructive actions to delete the account while keeping its home directory or to delete the home directory as well. Read the confirmation text carefully before choosing the latter. Account operations are privileged and may open the system authorization prompt. Avatars can be consumed by the greeter through the account's standard face image integration.

**Automatic login** starts the selected account's session at the next boot without showing the login screen, the same convention GNOME, LightDM and SDDM use for this feature: no password is stored anywhere. Enabling it for one account disables it for any previously configured one (only one account can auto-login at a time). It applies once, at the next boot; after a manual logout the normal login screen returns for the rest of that session. The toggle is applied immediately, with a confirmation prompt, and is only available to administrators.

Groups have their own list and edit pages, including member management and group deletion. The Control Center does not replace the system's account policy: protected system accounts and operations requiring authorization remain subject to the operating system's restrictions.

### Control Center settings

**Settings** is the last entry of the home. Its **Language** page sets the interface language, and its **Style** page controls the decorative icons and the Control Center transparency. Style does not hold blur: blur is a global Hyprland setting, under Hyprland > Blur. The Style page changes the settings application's presentation; it is not a switch that disables ARGVUS services or removes settings providers. Saving the icons can require authorization, while the visual change is applied to the current application. Esc on Style returns to the Settings page, and Esc there returns to the home.

## Search and optional capabilities

The home screen has a searchable registry of routes. Search terms include user-facing concepts such as appearance, themes, wallpapers, fonts, keyboard shortcuts, mouse, touchpad, displays, network and session. Optional providers are included only when their feature is compiled and their runtime capability is detected, so the visible menu is intentionally capability-aware.

While the Home search is active, printable keys are entered into the query, including `j` and `k`; use the `↑` and `↓` arrow keys to move through matching routes. Press `Enter` to open the selected route, `Backspace` to edit the query, and `Esc` to clear the query or leave search mode.

## Appearance workflow

Open **Appearance** to see the integrated visual controls. Its groups are:

- **Desktop** — **Theme** (choose a bundled family and its Sticky or Float mode, or manage custom theme profiles), **Appearance Mode** (Sticky or Float, independent of the theme), **Accent Color** (a bundled accent or a valid six-digit RGB color), **Wallpaper** (a bundled wallpaper or a custom file) **Icons** (an installed icon theme: search the list with `/`, choose one with Space or Enter, then select **Apply**; nothing changes before Apply) and **Fonts** (the font targets and font rendering settings, opened on the Fonts page).
- **Application Style** — the **Terminal** and **Launcher** transparency.

The Appearance card holds the surfaces that use these settings: **Widget Telemetry** (enable the telemetry surface, select its available blocks and reorder them with `Shift+Up`/`Shift+Down` on a focused block) and **Control Panel** (enable, disable and reorder panel cards with `Shift+Up`/`Shift+Down` on a focused card, applied immediately through the same primitive the real panel's drag-and-drop uses).

These actions update the logical ARGVUS state and apply the affected runtime configuration. See [Appearance](/docs/user-guide/appearance/), [Themes](/docs/user-guide/appearance/) and [Windows and layout](/docs/argvus-hyprland/windows-and-layout/).

## Keyboard use

The settings application supports keyboard navigation. Every page is a single list: use the arrow keys or `j`/`k` to move, `Enter` to open or run the selected row, `Space` to toggle, `Esc` to go back, `/` to search lists and `?` for help; the footer shows the keys that apply to the selected row. Actions are rows of the list, not buttons, and `Tab` does not move to them, except on the pages with an **Actions** bar under the list (Default Apps, Time Zone, Keyboard Layout, System Locales and SSH Keys). Rows that only show information are skipped. While a text field, a search or a list filter is active, `q` and `?` are typed as text instead of quitting or opening the help. On **Default Apps**, **Restore Defaults** is the button in the **Actions** bar under the list: `Tab` or `←/→` focus it and `Enter` asks for the reset. **Reset defaults** on the other pages and **Restore all shortcuts** are rows in a **Danger zone** at the end of their pages; `r` asks for the same reset, and on the shortcuts list `r` restores the selected shortcut after a confirmation. On **Language & Region → Keyboard Layout**, `/` focuses the **Search** box at the top of the page, and `Esc` clears it; `Enter` sets the default layout (marked with `●`) and `Space` adds or removes a layout (`[x]`); these keys only change a draft, and the **Apply** button in the **Actions** bar under the list writes the layouts and the default to Hyprland in one step. From the list, `Tab` or `←/→` moves the focus to that bar while **Apply** is enabled (the footer shows `←/→ Actions` then); `←/→` steps through its buttons and `↑/↓` return to the list. Leaving the page with unapplied changes asks for confirmation first. On **Language & Region → Time Zone** the layout is the same: `/` focuses the **Search** box at the top, `Enter` picks a zone in a draft (marked with `●`), and the **Apply** button in the **Actions** bar under the list writes it to the system with `timedatectl`; `Tab`, `←/→` and `↑/↓` work as on Keyboard Layout. Leaving with a zone that was not applied asks for confirmation first. On **System Locales** the layout is the same as Time Zone: `/` focuses the **Search** box at the top, `Enter` or `Space` marks or unmarks a locale (`[x]`) in a draft, and the **Apply** button in the **Actions** bar under the list writes the selection to `/etc/locale.gen` and runs `locale-gen`. Because that regenerates the system locales, **Apply** asks for confirmation first; `Tab`, `←/→` and `↑/↓` work as on the other two. On **Desktop & Input → Keyboard Shortcuts** the search box also sits at the top, with `/` focusing it, but the page has no **Actions** bar: its list keeps the shortcut sections and the **Danger zone**, and the footer lists `r Restore default` only on the list itself, not on **Edit shortcut**.

Pages that collect changes before saving them (a user, a group, account creation, password change and firewall configuration) end that block with a **Save** or **Apply** row that stays dimmed until something changed. If going back would discard unsaved changes, Control Center asks first. Every confirmation shows **Confirm** and **Cancel** as two rows with the focus on **Cancel**: `Enter` runs the focused row, `y` confirms and `n` or `Esc` cancel.

In **Appearance**, every page is a single list: the footer shows only the keys that apply to the selected row, and `Space` also opens or selects the row. Pages that collect changes before applying them (Taskbar, Widget Telemetry, Control Panel and the effect sliders) end with an **Apply** row, which stays dimmed until something changed. Selecting **Apply** shows the loading screen at once and keeps it until the last change of the page is applied, so no change appears before it. On percentage rows, `←`/`→` (or `+`/`-`) change the value in steps of 5 and `Enter` lets you type it; use `Esc` to go back. If going back would discard changes that were not applied, Control Center asks first. Deleting a custom theme (`d`) or replacing an imported one asks for confirmation; the focus starts on **Cancel**, `y` confirms and `n` or `Esc` cancel.

## Persistence and reset

Settings pages apply changes through their owning provider and save the supported user state. Appearance changes are stored in the canonical configuration, and `argvus-config` is the only component that writes the derived consumer files under `~/.config/argvus/data/generated/`; those generated files are not the place to make a permanent edit. Input and keybinding pages have their own persisted state and reset actions. Other system changes may require permissions or a service reload.

There is no global reset for every ARGVUS setting. Restore a change from the page that owns it, or use the relevant feature's documented recovery procedure.

Reset support is deliberately per domain:

- **Appearance** provides a reset for the highlight color to the active theme default. It does not provide a single global appearance reset; restore other appearance values from their own controls.
- **Fonts** can restore all font settings, a target, or an individual setting.
- **Default applications** can restore all defaults, a category, or an individual selector.
- **Keyboard shortcuts** can restore one binding or all bindings and then reload the generated session bindings.
- **Window rules** is in **Desktop & Input → Hyprland → Window Rules** (search `window rules` or run `argvus control-center window-rules`). Each rule has a **Workspace** row (`Enter` or `←/→` steps through 1 to 10) and a **Window classes** row that opens a text field with comma-separated regular expressions. **Add rule** creates `rule-N` with workspace 1 and no classes. Each rule's **Remove rule** sits in the **Danger zone** and asks for confirmation. Every change is written to `hyprland.window_rules` in `argvus config` and reloads the configuration immediately; invalid class patterns are rejected and shown as an error.

- **Projects** is in **Tools & Diagnostics → Projects** (search `projects` or run `argvus control-center projects`). **Add project folder** and **Add root folder** sit at the top of the page and open a text field with the path; the launcher checks that it is a directory and shows the error otherwise, including when the folder would push the effective project count past 9 (`SUPER + ALT + 1..9` cannot address more). Each configured folder, marked as a root (its subfolders are projects) or a single project, gets its own block below the form with a **Remove** row that asks for confirmation. The page calls `argvus-projects`, so changes are written by the launcher and apply the next time `SUPER + O` or `SUPER + ALT + 1..9` runs. If `argvus-projects` is not installed, the page says so.
- **Other system pages** expose reset, apply, delete or restore actions only when the underlying provider supports them.

Removing a user preference can make a provider fall back to its packaged/default state, but deleting files by hand is not a general recovery procedure. Prefer the page's reset action or the documented feature-specific command.

## CLI entry points

The helper is `argvus-control-center`, run through `argvus control-center`. It can open focused areas directly, for example:

```sh
argvus control-center appearance themes
argvus control-center appearance wallpapers
argvus control-center input
argvus control-center keybindings
```

Run `argvus control-center --help` for the routes available in the installed version. The command-line route is a shortcut into the same settings application, not a separate configuration system.

Useful direct routes include `apps`, `window-rules`, `projects`, `snippets`, `ssh-keys`, `gpg-keys`, `firewall`, `fonts`, `locale`, `input`, `keybindings`, `language`, `config`, `system`, `hardware`, `services`, `dev-services`, `network`, `audio`, `bluetooth`, `boot`, `packages`, `storage`, `diagnostics`, `power`, `session`, `displays` and `appearance`. Appearance also accepts focused routes such as `themes`, `wallpapers`, `icons`, `accents`, `effects`, `spaces`, `taskbar` and `widget-telemetry`; display accepts `resolution`, `refresh`, `scale`, `position`, `orientation`, `primary`, `vrr` and `hdr`. Use the installed `--help` output when scripting because availability still depends on the installed build.

## Related

- [First configuration](/docs/getting-started/)
- Control Panel
- [Appearance](/docs/user-guide/appearance/)
- [Keyboard shortcuts](/docs/argvus-hyprland/keyboard-shortcuts/)
- [Mouse and touchpad](/docs/argvus-hyprland/input/)

---
title: Networking and Bluetooth
description: Manage NetworkManager and Bluetooth through ARGVUS.
---

The `argvus network` subcommand reports and controls NetworkManager, and `argvus bluetooth` reports and controls Bluetooth adapter state and availability. Both run in-process inside `argvus`; `networkctl` and `bluetoothctl` remain as aliases. The Control Panel and Waybar call these commands.

Use `--help` on either command for the installed subcommands. Network and Bluetooth UI integration is separate from optional advanced managers such as Blueman.

The Control Center separates network status from configuration pages: **Network → Status** summarizes the current state, while interfaces, Wi-Fi, Ethernet, VPN, DNS, proxy and firewall are separate routes when their providers are available. **Bluetooth → State**, **Devices** and **Pair** cover adapter state and device actions. These pages perform provider-backed system actions; the taskbar and Control Panel expose the quick status/action surface.

Network and Bluetooth changes may require authorization and may apply immediately through NetworkManager or the Bluetooth service. They are not the same as changing the ARGVUS taskbar indicator.

The Control Center sends Wi-Fi passwords to nmcli through standard input with `--ask`; passwords are excluded from process arguments and redacted from returned command errors.

## Command-line network management

Use `argvus networkctl` to query and control networking without the graphical interface:

```sh
# Show network status
argvus networkctl status

# Toggle networking on/off
argvus networkctl enable
argvus networkctl disable

# Wi-Fi control
argvus networkctl wifi on
argvus networkctl wifi off

# Open the network manager
argvus networkctl manager
```

## Command-line Bluetooth management

Use `argvus bluetoothctl` for the Bluetooth adapter:

```sh
# Show the adapter state (key=value lines)
argvus bluetoothctl status

# Exit 0 when an adapter is available, 1 otherwise
argvus bluetoothctl available

# Turn the adapter on or off, or toggle it
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Open the Bluetooth manager (Blueman)
argvus bluetoothctl manager
```

When the kernel lists no controller in `/sys/class/bluetooth`, `status` answers `status=no-controller` immediately, without asking `bluetoothctl` (which would otherwise wait for BlueZ when the service is stopped). The taskbar module and the Control Panel cards read the same status.

Use `--help` on either command to see additional options and subcommands specific to your installed version.

While the Wi-Fi password dialog is open, scan updates preserve the selected SSID and security type. If that network disappears or its security changes, confirmation stops with an error and requires a new selection.

Manual DNS sends IPv4 and IPv6 servers to their respective NetworkManager settings before updating the resolver file. Service-managed resolver symlinks are preserved by refusing file replacement. A successful return to automatic DNS restores and removes the backup, so a later manual cycle saves the current resolver state.

Closing the network profile editor waits while its current job finishes, so saving or deleting a profile cannot be interrupted by discarding the editor. Background refreshes remain cancellable.

Wi-Fi connection requests share the network action guard: a running action or profile-editor job completes before another mutation can start. Progress and completion messages follow the selected language.

Bluetooth removal confirmation keeps the original device address even when the device list refreshes. Battery percentages from bluetoothctl are recognized, and device detail queries run with bounded concurrency and individual timeouts.

Wi-Fi password, manual DNS and network search fields accept bracketed paste. Pasted controls and line breaks are omitted. Unsupported Ctrl-only or Alt-only character shortcuts do not insert letters. Editors accept printable AltGr events encoded as Ctrl+Alt; Ctrl+S saves only without Alt. The shared text editor keeps vertical navigation in visual columns and aligns horizontal scrolling for wide characters.

---
title: Networking and Bluetooth
description: Manage NetworkManager and Bluetooth through ARGVUS.
---

The `argvus network` subcommand reports and controls NetworkManager, and `argvus bluetooth` reports and controls Bluetooth adapter state and availability. Both run in-process inside `argvus`; `networkctl` and `bluetoothctl` remain as aliases. The Control Panel and Waybar call these commands.

Use `--help` on either command for the installed subcommands. Network and Bluetooth UI integration is separate from optional advanced managers such as Blueman.

The Control Center separates network status from configuration pages: **Network → Status** summarizes the current state, while interfaces, Wi-Fi, Ethernet, VPN, DNS, proxy and firewall are separate routes when their providers are available. **Bluetooth → State**, **Devices** and **Pair** cover adapter state and device actions. These pages perform provider-backed system actions; the taskbar and Control Panel expose the quick status/action surface.

Network and Bluetooth changes may require authorization and may apply immediately through NetworkManager or the Bluetooth service. They are not the same as changing the ARGVUS taskbar indicator.

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

Use `argvus bluetoothctl` for Bluetooth device and adapter control:

```sh
# Show Bluetooth adapter state
argvus bluetoothctl status

# Toggle Bluetooth on/off
argvus bluetoothctl enable
argvus bluetoothctl disable

# List paired devices
argvus bluetoothctl list

# Connect/disconnect devices
argvus bluetoothctl connect <device-name-or-address>
argvus bluetoothctl disconnect <device-name-or-address>

# Trust or untrust a device
argvus bluetoothctl trust <device-address>
argvus bluetoothctl untrust <device-address>
```

Use `--help` on either command to see additional options and subcommands specific to your installed version.

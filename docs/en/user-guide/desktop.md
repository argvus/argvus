---
title: Desktop
description: Understand the ARGVUS desktop surfaces.
---

The desktop shell is composed of Hyprland, the patched Waybar taskbar, a Quickshell Control Panel and optional telemetry widgets. Hyprland owns windows and workspaces; the taskbar communicates workspace/window and system status; the Control Panel provides quick actions; notifications and applications appear alongside these surfaces.

Configuration lives in the [Control Center](/docs/components/argvus-control-center/), while frequent actions live in the Control Panel. Start with [First configuration](/docs/user-guide/getting-started/) if this is your first session.

- Control Panel
- [Taskbar](/docs/argvus-taskbar/taskbar/)
- [Widgets](/docs/argvus-taskbar/widgets/)
- [Keyboard shortcuts](/docs/argvus-hyprland/keyboard-shortcuts/)
- [Hyprland overrides](/docs/argvus-hyprland/hyprland-overrides/)
- [Windows and layout](/docs/argvus-hyprland/windows-and-layout/)

The session starts and stops these surfaces; there is no monolithic `argvus-shell` service.

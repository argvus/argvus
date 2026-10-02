---
title: Desktop
description: Understand the ARGVUS desktop surfaces.
---

The desktop shell is composed of Hyprland, the patched Waybar taskbar, a Quickshell Control Panel and optional telemetry widgets. Hyprland owns windows and workspaces; the taskbar communicates workspace/window and system status; the Control Panel provides quick actions; notifications and applications appear alongside these surfaces.

Configuration lives in the [Control Center](./control-center/), while frequent actions live in the [Control Panel](./desktop/control-panel/). Start with [First configuration](/docs/user-guide/getting-started/) if this is your first session.

- [Control Panel](./desktop/control-panel/)
- [Taskbar](./desktop/taskbar/)
- [Widgets](./desktop/widgets/)
- [Keyboard shortcuts](./desktop/keyboard-shortcuts/)
- [Hyprland overrides](./desktop/hyprland-overrides/)
- [Windows and layout](./desktop/windows-and-layout/)

The session starts and stops these surfaces; there is no monolithic `argvus-shell` service.

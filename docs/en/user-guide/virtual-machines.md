---
title: Virtual machines
description: Run ARGVUS with virtual graphics hardware.
---

`argvus-start` detects virtualization and exports compatibility settings for affected sessions, including software-rendering and cursor/modifier fallbacks when required by the detected environment.

If a virtual machine still fails to render, inspect the session environment and logs before adding an override. See [graphics troubleshooting](./troubleshooting/graphics-and-virtual-machines/).

When virtualization is detected, `argvus-start` exports compatibility variables for the session, including software-rendering, software-cursor and modifier fallbacks. This makes the normal login path the first thing to try; do not add a permanent override just because the desktop is running in a VM.

If detection is not sufficient, use a user Hyprland override as a diagnostic fallback:

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```

Add this only when the VM graphics stack requires it:

```lua
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```

Log out and back in after changing the session environment. Confirm the active value with `echo $LIBGL_ALWAYS_SOFTWARE`; software rendering can make the desktop usable but may reduce performance.

---
title: Requirements
description: System requirements and prerequisites for ARGVUS.
---

ARGVUS targets Arch Linux and requires a Wayland-capable graphics stack, Hyprland and the runtime dependencies pulled by the packages. A display manager is optional: ARGVUS supports a graphical login flow through greetd and a manual TTY flow.

Before installation, make sure that:

- the system is fully updated with `pacman`;
- a working kernel and graphics driver are installed;
- the user can run a Wayland session;
- networking is available when the package repository is configured.

Virtual machines may need the compatibility settings described in [Virtual machines](../virtual-machines/).

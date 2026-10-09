---
title: Frequently asked questions
description: Common questions about ARGVUS, its architecture and daily use.
---

This page answers common questions about what ARGVUS is, how it is distributed and where to configure it. For detailed procedures, follow the links in each answer.

## What is ARGVUS?

ARGVUS is a modular desktop environment for Wayland and Arch Linux, built around Hyprland. It coordinates the graphical session, shell surfaces, settings applications, appearance, desktop services and integrated tools through a collection of packages.

It is more than a Hyprland configuration or a collection of personal dotfiles: it provides a supported session entry point, shared configuration and appearance state, and defined ownership for components such as the taskbar, Control Center, notifications, networking and displays. See the [introduction](/docs/introduction/) for an overview of how the pieces fit together.

## Is ARGVUS free?

Yes. ARGVUS is free to download and use, and its core project packages are distributed as free software under the [GNU General Public License version 3](https://www.gnu.org/licenses/gpl-3.0.html). This gives users the freedom to run, study, modify and redistribute the covered software under the license terms.

ARGVUS is distributed as signed Arch Linux packages. The software itself has no purchase price, but downloading packages may still involve normal internet or hosting costs. Components and applications provided by third parties can have their own licenses.

## Is ARGVUS a Linux distribution?

No. ARGVUS is the desktop environment layer installed on top of Arch Linux. Arch Linux provides the operating system, kernel, package manager and base packages; Hyprland provides Wayland composition; ARGVUS coordinates the desktop session and its integrated components.

See [installation](/docs/getting-started/installation/) for the supported installation model and [What ARGVUS is—and is not](/docs/introduction/#what-argvus-isand-is-not) for the architectural distinction.

## Why a desktop environment instead of a distribution?

Keeping ARGVUS as a desktop environment gives users more control over the rest of their system. A distribution normally chooses and maintains a complete baseline, including the kernel, package selection, repositories, defaults and system policies. ARGVUS focuses on the desktop experience while leaving those broader decisions to the user and the underlying Arch Linux installation.

This model also keeps the project modular: session management, Hyprland configuration, appearance, taskbar, settings and providers can be developed and packaged separately while working together as one desktop.

## Which systems are supported?

The current supported base is Arch Linux. ARGVUS is packaged for Arch Linux and depends on the Wayland, Hyprland and system components described in the [installation requirements](/docs/getting-started/installation/requirements/).

Other distributions may provide some of the same upstream components, but that does not make the complete ARGVUS package set or session integration supported there.

## How do I install ARGVUS?

Follow the [installation guide](/docs/getting-started/installation/). It explains the requirements, how to configure the signed ARGVUS package repository, how to install the package set and how to start the first session.

ARGVUS is delivered as a coordinated set of Arch packages. The `argvus` package is the main installation target and pulls in the desktop components it coordinates.

## How do I change ARGVUS settings?

Use **Control Center** for persistent desktop configuration, including themes, wallpapers, effects, layout, fonts, keyboard, input, language, region and default applications. The most efficient ways to open it are the `SUPER + Alt + C` shortcut, or the launcher with `SUPER + D`: type **ARGVUS Control Center**. You can also type `argvus` to list the ARGVUS applications.

As an additional option, you can open it from a terminal with:

```sh
argvus --control-center
```

You can also open a focused area with `argvus-control-center`, for example `argvus-control-center appearance themes`. Run `argvus-control-center --help` to see the routes available in the installed version.

Use **Control Panel** for current status and frequent actions such as volume, brightness, network, notifications, power and session controls. It is not a second configuration database. See [Where to configure things](/docs/user-guide/where-to-configure/) and [Control Center](/docs/argvus-control-center/) for the distinction.

## Where are ARGVUS settings stored?

User-owned configuration is stored below `$XDG_CONFIG_HOME/argvus` (normally `~/.config/argvus`). The canonical logical configuration is maintained there, while the derived consumer files for Hyprland, Waybar and the telemetry widget, GTK, Qt6ct, Quickshell, Dunst, Hyprlock, Yazi, Superfile, terminals, fonts, effects, calendars, input, keyboard, power and removable devices live under `~/.config/argvus/data/generated/`. That tree is written only by `argvus-config` and rebuilt from the canonical configuration; `argvus-appearance` commits appearance changes and then reconciles only the external consumers that cannot read the canonical configuration on their own. Rofi is the exception: `argvus-launcher` packages its configuration and themes, and the user's edited copies live under `~/.config/argvus/data/rofi/`.

Do not edit generated files as a permanent configuration method. Use Control Center or the documented native override path for the component you want to customize. See [configuration files](/docs/reference/configuration-files/) and [configuration and state](/docs/developer-guide/architecture/configuration-and-state/).

## What should I do if a change is not applied?

First confirm that the change was saved or applied in the owning page. Then check whether the affected component needs a session or service reload. A committed change is projected into the consumer files by `argvus-config`, but projecting them does not guarantee that every running consumer has reloaded them. `argvus-sessionctl reload` applies an already-committed generation from the projection manifest instead of projecting again, and skips the graphical reload when the canonical state did not change.

Use the relevant [troubleshooting guide](/docs/user-guide/troubleshooting/) for installation, session startup, graphics, displays, Hyprland, themes and wallpapers. When reporting a problem, include the component, the setting changed, the command or page used and the output of the relevant status or diagnostic command.

## Where can I learn more?

- [First configuration](/docs/user-guide/getting-started/) for the recommended first-session flow.
- [User guide](/docs/user-guide/) for feature-specific instructions.
- [Reference](/docs/reference/) for commands, paths, services and configuration contracts.
- [Developer guide](/docs/developer-guide/) for architecture, package ownership and contribution workflows.

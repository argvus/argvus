---
title: Projects
description: The project picker and the projects configuration, through argvus projects.
---

`argvus projects` lists and opens your projects. It is an in-process command of the `argvus`
package, and `SUPER + O` opens the picker.

## Commands

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position, full path or unique name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```

## What is a project

A project is a directory listed in the configuration (`/projects/paths`), or a subdirectory of
a root listed in `/projects/roots`. At most nine projects are listed: the numbers of the
`SUPER + CTRL + N` shortcuts. Opening a project focuses its Hyprland workspace. The first time a
project opens, ARGVUS starts a terminal and the editor set in `/projects/editor` inside it.

The configuration is read and written through `argvus config`, so the Control Center and the
shell see the same list.

## Project workspaces and the taskbar

Projects use ordinary named Hyprland workspaces, not special workspaces. `SUPER + O` shows the project number, directory and direct `SUPER + CTRL + N` shortcut. Both entry points focus the same workspace. Only open workspaces appear in the taskbar, with a `Project: name` label, active state, and a tooltip containing the full path and shortcut. Click the workspace button to return to it. The taskbar follows the workspace's monitor.

The canonical directory path determines a stable workspace identity; changing the list order changes the shortcut, not the workspace. Projects with the same directory name remain separate, and symbolic links to the same directory are deduplicated. Terminal and editor launches explicitly target the project's workspace. Editors that reuse an existing process/window may retain that application's own workspace behavior.

Workspaces opened before the update keep their existing windows and native names. Opening a project after the update uses its new path-derived identity; close old project windows when convenient. No automatic association or movement is attempted because an old name does not identify the directory reliably. With duplicate directory names, use the picker, a number or the full canonical path rather than an ambiguous name.

Opening a project regenerates `$XDG_RUNTIME_DIR/argvus/project-workspaces.json` atomically from the configured project list and translations. This is session metadata for workspace labels and tooltips, not configuration; do not edit it. Reopen a project after changing the list or language to refresh its labels. Update both `argvus` and `argvus-waybar`, plus the `argvus-taskbar` configuration, for this integration.

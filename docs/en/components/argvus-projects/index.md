---
title: Projects
description: The project picker and the projects configuration, through argvus projects.
---

`argvus projects` lists and opens your projects. It is an in-process command of the `argvus`
package, and `SUPER + O` opens the picker.

## Commands

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position or name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```

## What is a project

A project is a directory listed in the configuration (`/projects/paths`), or a subdirectory of
a root listed in `/projects/roots`. At most nine projects are listed: the numbers of the
`SUPER + ALT + N` shortcuts. Opening a project focuses its Hyprland workspace. The first time a
project opens, ARGVUS starts a terminal and the editor set in `/projects/editor` inside it.

The configuration is read and written through `argvus config`, so the Control Center and the
shell see the same list.

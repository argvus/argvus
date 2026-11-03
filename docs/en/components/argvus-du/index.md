---
title: ARGVUS Disk Usage Analyzer
description: Browse disk usage interactively with ncdu in a themed Kitty window.
---

Open **ARGVUS Disk Usage Analyzer** from the application launcher or run:

```sh
argvus du
```

The analyzer opens a dedicated, centered Kitty window and embeds the real ncdu interface inside its ARGVUS frame. It starts in `$HOME`. Edit the directory path or choose **Browse** to open the configured file manager. With Superfile, navigate to a directory and press **Space** to select it; `q` or `Esc` cancels. Other configured file managers open normally, but may not return a selected directory, so enter or paste the path in the analyzer.

Use Tab or the arrow keys to move between the directory, options and actions. The graph, percentage, dark color scheme and directories-first ordering are enabled by default. The remaining scan and display options can be toggled before selecting **Analyze**. In Results, use ncdu's keyboard controls; `q` returns to the form and `Esc` stops the scan. Press `d` to delete the selected file or directory. ncdu asks for confirmation; deletion is permanent and uses the current user's permissions. No elevated privileges are requested.

The outer interface follows the active ARGVUS theme and locale. Kitty and ncdu are required; the `argvus-devtools` package provides ncdu. If Kitty or ncdu is missing, the application reports the missing dependency.

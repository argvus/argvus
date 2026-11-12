---
title: एकीकरण मानचित्र
description: ARGVUS रनटाइम में क्रॉस-प्रोजेक्ट संबंध।
---

```text
greeter or TTY
  -> argvus-session -> argvus start -> Hyprland
  -> argvus-session.target
     -> taskbar / control panel / notifications / wallpaper / idle / clipboard

Control Center -> argvus config (canonical state) -> argvus sessionctl reload
argvus config   -> the only writer of data/generated/
                   Hyprland / GTK / Qt6ct / Waybar / Quickshell / Dunst /
                   Hyprlock / Yazi / Superfile / terminal / fonts / removable-devices
theme-switch / config accent -> argvus config (commit) -> external adapters
                   Qt6ct file / GTK settings.ini + gsettings / terminals /
                   system-monitor / foot / superfile / greeter
Waybar          -> popups via immutable root coordinates; calendar resolves real horizontal Taskbar geometry
```

फ़ीचर पृष्ठ उपयोगकर्ता के व्यवहार का वर्णन करते हैं; यह पृष्ठ कोड बदलते समय उपयोग की जाने वाली सीमाओं को रिकॉर्ड करता है।

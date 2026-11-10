---
title: Schalenoberflächen
description: Taskleiste, Systemsteuerung und Telemetrie-Eigentum.
---
`argvus-waybar` stellt das gepatchte `/usr/bin/waybar` bereit. `argvus-taskbar` besitzt Taskleistenaktionen und UX; die Taskleisten-JSONC und CSS, die es bereitstellt, sind Projektionen von `config.json`, und `argvus-config` schreibt die aktive Ebene unter `data/generated/waybar/` sowie eine gespiegelte Kopie unter `data/waybar/`. `argvus-control-panel` besitzt Quickshell-Karten. `argvus-widget-telemetry` besitzt die optionale Telemetrie-Leiste und deren Blocksemantik; sein JSONC und CSS werden bei einer Erscheinungsänderung von den verpackten Standardeinstellungen zu `data/waybar/` geschrieben, wonach `argvus-config` nur die abgegrenzten `ARGVUS_TELEMETRY_*`- und Schriftblocke überschreibt. `argvus-session` besitzt ihre Benutzerdienste.

Es gibt kein aktuelles `argvus-shell`-Paket oder `argvus-shell.service` in diesem Laufzeitmodell.

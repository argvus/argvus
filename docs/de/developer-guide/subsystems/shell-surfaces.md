---
title: Schalenoberflächen
description: Taskleiste, Systemsteuerung und Telemetrie-Eigentum.
---
`argvus-panels` stellt das gepatchte `Quickshell Taskbar` bereit. `argvus-panels` besitzt Taskleistenaktionen und UX; die Taskleisten-JSONC und CSS, die es bereitstellt, sind Projektionen von `config.json`, und `argvus-config` schreibt die aktive Ebene unter `data/generated/taskbar/` sowie eine gespiegelte Kopie unter `data/taskbar/`. `argvus-panels` besitzt Quickshell-Karten. `argvus-panels` provides the optional Telemetry surface, while the `argvus widget-telemetry` command in `argvus` retains block-control semantics; sein JSONC und CSS werden bei einer Erscheinungsänderung von den verpackten Standardeinstellungen zu `data/taskbar/` geschrieben, wonach `argvus-config` nur die abgegrenzten `ARGVUS_TELEMETRY_*`- und Schriftblocke überschreibt. `argvus-session` besitzt ihre Benutzerdienste.

Es gibt kein aktuelles `argvus-shell`-Paket oder `argvus-shell.service` in diesem Laufzeitmodell.

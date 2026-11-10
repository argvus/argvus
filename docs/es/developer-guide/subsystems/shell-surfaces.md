---
title: Superficies de la cáscara
description: Barra de tareas, panel de control y propiedad de la telemetría.
---
`argvus-waybar` proporciona el `/usr/bin/waybar` parcheado. `argvus-taskbar` posee las acciones de la barra de tareas y la UX; el JSONC y CSS de la barra de tareas que proporciona son proyecciones de `config.json`, y `argvus-config` escribe la capa activa bajo `data/generated/waybar/` más una copia reflejada bajo `data/waybar/`. `argvus-control-panel` posee las tarjetas de Quickshell. `argvus-widget-telemetry` posee la barra de telemetría opcional y su semántica de bloques; su JSONC y CSS se escriben en `data/waybar/` desde los valores predeterminados del paquete cuando hay un cambio de apariencia, después de lo cual `argvus-config` reescribe solo los bloques delimitados `ARGVUS_TELEMETRY_*` y de fuentes. `argvus-session` posee sus servicios de usuario.

No hay ningún paquete `argvus-shell` o `argvus-shell.service` en este modelo de ejecución.

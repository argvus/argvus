---
title: Ubicaciones de archivos
description: Archivos y directorios importantes de ARGVUS instalados.
---

- `/usr/bin/argvus*` — comandos públicos.
- `/usr/share/argvus/hyprland/` — configuración de Hyprland y asistentes.
- `/usr/share/argvus/control-panel/` — QML del Panel de Control y scripts.
- `/usr/share/argvus/taskbar/` — configuración de la barra de tareas y scripts.
- `/usr/share/argvus/lock/` — configuración de Hyprlock.
- `/usr/share/argvus/portal/` — archivos de integración del portal.
- `/usr/share/backgrounds/argvus/` — recursos de fondos de pantalla.
- `/etc/argvus/` — configuración del proveedor del sistema.
- `/etc/argvus/greeter.toml` — configuración del saludo.
- `/etc/argvus/removable-devices/config.json` — valores predeterminados del sistema para almacenamiento extraíble.
- `$XDG_CONFIG_HOME/argvus/config.json` — preferencias portátiles canónicas.
- `$XDG_CONFIG_HOME/argvus/data/` — datos de componentes gestionados, copias de seguridad, bloqueos y salida generada.
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — anulaciones completas opcionales de la barra de tareas Waybar nativa.
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — gestionó las copias de la barra de tareas ARGVUS.  
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — generó perfiles de Waybar, incluyendo `argvus-taskbar.css` y `argvus-widget-telemetry.{jsonc,css}`; estos tienen prioridad sobre las copias anteriores.  
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — también se refleja bajo `data/waybar/`, que se reemplaza a partir del valor predeterminado empaquetado cuando cambia la apariencia y luego se reescriben sus bloques `ARGVUS_TELEMETRY_*` y de fuente.  
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — árbol de configuración proyectado de Yazi, con un fallback consciente de variantes para temas que no incluyen ninguna versión de Yazi.  
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — conjunto de temas proyectado de Superfile.  
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — perfiles de terminal proyectados.
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — paleta Qt6ct proyectada.
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — archivos de tema GTK proyectados.
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — copias de usuario de la configuración de Rofi, los temas y el modo. ARGVUS crea cada copia cuando cambia el archivo por primera vez; los archivos sin cambios se leen desde `/usr/share/argvus/launcher/config/`.
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — hoja de estilos proyectada para dispositivos extraíbles.
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — configuraciones de ARGVUS Focus, historial de sesiones y punto de control de la sesión actual.
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — estado efímero de ARGVUS Focus consumido por la barra de tareas; se elimina cuando Focus se cierra.
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — Configuraciones del reloj ARGVUS y relojes mundiales, alarmas y punto de control del temporizador/cronómetro. 
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — colores de ARGVUS Focus y ARGVUS Clock (consumidor de temas `time`). 
- `/usr/share/argvus/sounds/` — sonidos de ARGVUS Focus y ARGVUS Clock (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`). 
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — construcciones de temas (`builds/<id>-<variant>-<hash>/`), escritas por `argvus theme render` y por cada cambio de tema y de acento, y el enlace simbólico `current` a la construcción del tema activo. 
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — sobrescrituras de usuario de Hyprland Lua y proyecciones nativas de Hyprland.
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — configuración completa nativa opcional de Hyprland seleccionada por `argvus start`; no es una superposición incremental. 
- `$XDG_STATE_HOME/argvus/config-projection.json` — manifiesto de proyección: generación, hash de configuración efectiva, secciones cambiadas y trabajo pendiente en tiempo de ejecución. Leído por `argvus sessionctl reload`; nunca es una fuente de verdad. 
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — preferencias de bloque de telemetría legado. Solo entrada para migración; los valores canónicos se encuentran en `config.json` y este archivo ya no se escribe. 
- `$XDG_STATE_HOME/argvus/session.log` — registro de sesión gráfica (normalmente `~/.local/state/argvus/session.log`). 
- `/usr/share/wayland-sessions/argvus.desktop` — entrada de sesión gráfica. 
- `/usr/share/plymouth/themes/argvus/` — activos empaquetados de pantalla de inicio. 

Las cargas exactas son específicas del paquete; consulte la referencia de componentes para desarrolladores y el manifiesto del paquete relevante.

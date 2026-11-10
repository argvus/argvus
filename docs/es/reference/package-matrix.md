---
title: Matriz de paquetes
description: Responsabilidades del paquete ARGVUS.
---

| Paquete | Responsabilidad |
| --- | --- |
| `argvus` | Paquete principal: despachador, comandos en proceso (configuración, traducciones, claves, proyectos, terminal, monitor del sistema, red, Bluetooth, telemetría, terminal TUI, configuración), los datos compartidos y los catálogos i18n. Cada paquete de componentes depende de él. |
| `argvus` (helpers) | Ayudantes en el mismo paquete: el Centro de Control y el Analizador de Uso de Disco (aplicaciones de terminal), además de las aplicaciones GTK 4 para el calendario de la barra de tareas, dispositivos extraíbles, la superposición de carga, ARGVUS Focus y ARGVUS Clock. |
| `argvus` (greeter) | Saludo de inicio de sesión para greetd, incluido en el mismo paquete y construido a partir de `crates/argvus-greeter`. |
| `argvus-desktop` | Perfil de escritorio y de aplicaciones cotidianas requerido por el paquete principal `argvus`. No depende de `argvus`, por lo que la dependencia sigue siendo unidireccional. |
| `argvus-devtools`, `argvus-pentest` | Metapaquetes independientes opcionales para herramientas de desarrollo y auditoría de seguridad autorizada. El paquete maestro no instala estos perfiles. |
| `argvus-session` | Puntos de entrada de sesión y ciclo de vida del usuario. |
| `argvus-hyprland` | Configuración de Hyprland. |
| `argvus-control-panel` | Panel de control Quickshell. |
| `argvus-taskbar`, `argvus-waybar` | Configuración de Waybar y binario parcheado. |
| `argvus-branding` | Recursos de logotipo y marca textual. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Estado de apariencia y recursos visuales. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | Proveedores de escritorio. |
| `argvus-lock`, `argvus-boot-splash` | Pantalla de bloqueo e inicio. |
| `argvus-firewall` | Cortafuegos, con sus propios privilegios. |
| `argvus-launcher` | Configuración del lanzador y puntos de entrada. |
| `argvus-app-profiles` | Perfiles solo de datos para aplicaciones de terminal incluidas. |
| `argvus-portal` | Portal XDG y valores predeterminados de Wayland/DBus. |
| `argvus-theme-*` | Un paquete opcional por familia de temas oficial. |
| `argvus-game-snake`, `argvus-games` | Opcional. Snake es un proyecto independiente; `argvus-games` agrupa los juegos opcionales. |

## Paquetes reemplazados

`argvus` declara `replaces`, `conflicts` y `provides` para los paquetes que absorbió, por lo que una actualización los elimina y sus archivos no permanecen como huérfanos:

`argvus-config`, `argvus-i18n` (con `argvus-language-en-us` y `argvus-language-pt-br`), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` y `argvus-du`. `argvus` también reemplaza a `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices`, y los antiguos paquetes divididos `argvus-gtk` y `argvus-greeter`, los cuales todavía proporciona para que los paquetes que dependen de ellos sigan resolviendo. El gestor de cuentas (`argvus accounts`) se construye a partir de `crates/argvus-accounts` y se instala como `/usr/lib/argvus/commands/accounts`, con su regla de polkit. El Analizador de Uso de Disco se construye a partir de `crates/argvus-du` y se instala como `/usr/lib/argvus/commands/argvus-du`.

`/usr/bin/argvus-terminal` permanece instalado como un lanzador de compatibilidad para `argvus terminal`. El terminal predeterminado de Hyprland, el catálogo del Centro de Control y las aplicaciones predeterminadas guardadas aún nombran ese comando.

Los paquetes eliminados `argvus-pwvucontrol` y `argvus-snappy-switcher` no son proporcionados por ningún paquete.

No se requiere un orden de instalación entre los paquetes de componentes ARGVUS: `argvus` es su paquete principal, y cada componente depende de él. Instalar `argvus` también instala el perfil `argvus-desktop`; `argvus-devtools` y `argvus-pentest` siguen siendo opcionales. La presencia de un paquete en esta tabla no significa que posea toda la función: por ejemplo, la barra de tareas combina `argvus-waybar`, `argvus-taskbar`, `argvus-session` y proveedores de dominio.

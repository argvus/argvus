---
title: Roles del repositorio
description: Responsabilidades de los repositorios de origen de ARGVUS después de la consolidación del espacio de trabajo.
---
Los repositorios están agrupados por responsabilidad en lugar de por página de navegación:

- workspace: `argvus` (despachador, comandos en proceso, ayudantes de GTK 4, crates compartidos, el greeter en `crates/argvus-greeter` y los datos de los comandos);
- sesión e integración: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash`, y el greeter (`argvus-greeter`, construido a partir de `argvus/crates/argvus-greeter`);
- UI fuera del workspace: Panel de Control, barra de tareas (`argvus-taskbar`), lanzador y Waybar;
- proveedores: display, energía, firewall, notificaciones;
- assets y empaquetado: apariencia, fondos de pantalla, tipografías, íconos, perfiles.

## Consolidado en el workspace

Estos repositorios fueron fusionados en `de/argvus`. Su documentación ahora se encuentra bajo `docs/` del workspace, y sus repositorios se mantienen solo por historial:

| Repositorio anterior | Ahora | Documentación |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [línea de comando](/docs/reference/command-line/), [archivos de configuración](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [Componente de internacionalización](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | línea de comando |
| `argvus-terminal` | `argvus terminal` | [Componente de terminal](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [Componente de monitor del sistema](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [Componente de red](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | línea de comando |
| `argvus-control-center` | `argvus control-center` (aplicación de terminal) | [Componente del Centro de Control](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (helper de GTK 4) | [Componente de calendario de la barra de tareas](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (helper de GTK 4) | [Componente de dispositivos extraíbles](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (helper `splash`) | línea de comando |
| `argvus-tui-terminal` | `argvus tui-terminal` (crate en el espacio de trabajo) | línea de comando |
| `argvus-tui`, `argvus-theme` | Crates compartidos `crates/argvus-tui` y `crates/argvus-theme` | notas del desarrollador |
| `argvus-greeter` | Fuente en `crates/argvus-greeter`, distribuido en el paquete `argvus` | [componente greeter](/docs/components/argvus-greeter/) |
| `argvus-themes` | Retirado. Cada tema oficial es su propio paquete `argvus-theme-*`. | [instalando temas](/docs/user-guide/appearance/) |

Las notas del desarrollador de los componentes consolidados están en `docs/internal/<repository>/`.

El lenguaje no es el límite de propiedad: cada componente mantiene sus ayudas de ejecución con su paquete. La única regla estricta que atraviesa los paquetes: solo `argvus-config` escribe debajo de `data/generated/`. Un componente que necesita alcanzar una aplicación que no puede leer `config.json` agrega un adaptador en su propia ayuda y permite que el orquestador lo llame después del commit canónico; nunca escribe un archivo generado. Las interfaces de línea de comandos públicas pertenecen a `/usr/bin`; las ayudas internas pertenecen a `/usr/lib/argvus/<component>` o al directorio de comandos cuando se necesitan en tiempo de ejecución; los datos independientes de la arquitectura permanecen en `/usr/share/argvus/<component>`.

Consulta la [matriz de paquetes](/docs/reference/package-matrix/)] para el límite de paquetes instalados.

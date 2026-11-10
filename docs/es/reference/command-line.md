---
title: Línea de comandos
description: El comando argvus, sus subcomandos, las banderas mantenidas para las combinaciones de teclas y dónde se ejecuta cada uno.
---
`/usr/bin/argvus` es el punto de entrada público de ARGVUS. Es un programa en Rust creado a partir del crate `argvus-cli`. Cada subcomando se ejecuta dentro de ese programa, ejecuta un archivo auxiliar desde el directorio de comandos, o ejecuta un script empaquetado.

Ejecute `argvus --help` para la lista instalada con su versión. Los subcomandos se escriben sin el `--` inicial. Las formas `--` se conservan solo cuando un atajo de teclado o una entrada de escritorio las utiliza.

## Dónde se ejecuta cada comando

- **En proceso:** el comando es una biblioteca del espacio de trabajo y se ejecuta dentro de `argvus`. No existe un ejecutable separado. Estos son `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`, `tui-terminal`, `sensors` y `setup`.
- **Ayudante:** un ejecutable separado en `/usr/lib/argvus/commands/`, instalado por el paquete `argvus`. Estas son `control-center` (aplicación terminal), `argvus-du` (lanzada con `argvus du`) y las aplicaciones GTK 4 `taskbar-calendar`, `removable-devices`, `focus` y `clock`.
- **Ejecutable privado:** un ejecutable que otro paquete instala en `/usr/lib/argvus/commands/`. `argvus` solo lo inicia. Estos incluyen `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` y los puntos de entrada `greeter-*`.
- **Script:** un script de shell empaquetado que se ejecuta con `/bin/sh`. Estos son los comandos del script de Hyprland que se enumeran a continuación.

`ARGVUS_COMMANDS_DIR` cambia el directorio de comandos, para pruebas y comprobaciones `DESTDIR`. Un ejecutable privado que no se encuentra allí se busca en `PATH`.

## Subcomandos

| Comando | Carreras | Notas |
| --- | --- | --- |
| `argvus config ...` | En proceso | Modelo de configuración y herramientas. También se accede a través de `theme-switch` y `effects-toggle`. |
| `argvus theme ...` | En proceso | `apply ID` es el cambio de tema (el mismo que `theme-switch`). Construcciones de temas: `list` los temas instalados, `check DIRECTORY` un tema contra las plantillas instaladas, `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`. Véase [Apariencia y temas](/docs/developer-guide/subsystems/appearance-and-themes/). |
| `argvus i18n ...` | En proceso | Traducciones: `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`. |
| `argvus keys ...` | En proceso | Administrador de claves SSH. `gpg` es un alias para los subcomandos de GPG. |
| `argvus gpg ...` | En proceso | Claves GPG: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | En proceso | Selector de proyecto. |
| `argvus terminal ...` | En proceso | El perfil de terminal ARGVUS. |
| `argvus system-monitor ...` | En proceso | El monitor del sistema btop; `--gpu` selecciona solo cajas GPU, y `--panel [--cpu|--mem|--gpu]` lo integra en la shell TUI de ARGVUS. |
| `argvus du` | Ayudante (aplicación de terminal) | Abre el Analizador de Uso de Disco de ARGVUS en una ventana Kitty dedicada y ejecuta ncdu. |
| `argvus network ...` | En proceso | Estado y controles de NetworkManager (`networkctl` se mantiene como alias). |
| `argvus bluetooth ...` | En proceso | Estado y controles de Bluetooth (`bluetoothctl` se mantiene como alias). |
| `argvus widget-telemetry block ...` | En proceso | El bloque de telemetría de Waybar. Este es el camino crítico de la barra. |
| `argvus widget-telemetry toggle ...` | En proceso | Activa y desactiva los bloques de telemetría, y su orden. |
| `argvus game-snake` | Ejecutable privado | El juego de Snake en el terminal, del proyecto independiente `argvus-game-snake`, un paquete opcional. Fuera de un terminal, abre uno. |
| `argvus control-center [page]` | Ayudante (aplicación de terminal) | Abre el Centro de Control, o enfoca la ventana que ya está abierta. |
| `argvus welcome [--if-needed]` | Despachador → Centro de Control | Abre la página de bienvenida integrada en el Centro de Control. `--if-needed` se abre una vez, a menos que los registros de progreso por usuario indiquen que la bienvenida ya fue mostrada, completada o descartada. |
| `argvus taskbar-calendar [args]` | Aplicación GTK 4 | Por defecto a `toggle`. |
| `argvus removable-devices [args]` | Aplicación GTK 4 | Por defecto a `menu`. |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Aplicación GTK 4 | Ventana emergente ARGVUS Focus. Una instancia por sesión. |
| `argvus focus status --waybar [--follow]` | En proceso | Emite el indicador de la barra de tareas de Focus como JSON de Waybar; `--follow` transmite cambios sin cargar GTK. |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Aplicación GTK 4 | Ventana emergente del reloj ARGVUS. Una instancia por sesión; `--background` mantiene las alarmas activas sin una ventana emergente. |
| `argvus loading-theme ...` | `splash` empaquetado en `/usr/lib/argvus/loading-theme/splash` | La superposición de recargas directas de sesión; un cambio de tema no lo usa. |
| `argvus session`, `start`, `tty`, `sessionctl` | Ejecutables privados | Puntos de entrada de sesión y el controlador de sesiones. |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | Ejecutables privados | Puntos de entrada del saludo (greeter). |
| `argvus accounts`, `firewall` | Ejecutables privados | Herramientas de cuentas y firewall. |
| `argvus launcher`, `snippets`, `ssh`, `git` | Ejecutables privados | Herramientas de lanzamiento y selección. |
| `argvus tui-terminal` | En proceso | Envoltorio de terminal para aplicaciones TUI. |
| `argvus sensors` | En proceso | Visión general de temperatura en vivo de `sensors -j`. |
| `app-profiles-yazi-keymap` | Ejecutable privado | Asistente de mapa de teclas. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | Ejecutables privados | Notificaciones, acciones de pantalla y de la barra de tareas. |
| `argvus setup [options]` | En proceso | Copia los valores predeterminados empaquetados en la configuración del usuario. Ver más abajo. |

Los argumentos se reenvían sin cambios, incluidos los espacios y los valores vacíos. La entrada estándar, la salida, el error, las señales y el estado de salida pertenecen al comando que se ejecuta. Un nombre desconocido falla con una sugerencia de ejecutar `argvus --help`. Un comando que se ejecuta en proceso rechaza un argumento que no es UTF-8 válido (estado de salida 2) en lugar de pasar un valor alterado; los comandos que ejecutan un programa separado reciben los bytes sin cambios.

## Banderas y alias

Estos nombres existen para que las combinaciones de teclas, las entradas de escritorio y los scripts antiguos sigan funcionando. Prefiera la forma de subcomando en código nuevo.

| Bandera o alias | Equivalente |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (con páginas `about` y `apps`) |
| `--displays`, `--monitors` página | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | el administrador de archivos (`spf`) |
| `--yazi`, `--yazy` | el explorador de archivos (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | inicia `tty` (no se envían argumentos) |
| `--version`, `-V`, `version` | impresiones `ARGVUS <version>` |
| `help`, `--help`, `-h` | imprime el uso |
| `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

El Centro de control acepta indicadores de páginas heredadas como primer argumento y los asigna a nombres de páginas. Ejemplos: `--apps` se convierte en `apps`, `--keyboard-layout` se convierte en `locale` y `--hostname` se convierte en `system`.

## Comandos de script de Hyprland

Estos ejecutan scripts empaquetados y son usados por la configuración de Hyprland:

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`.

Un script en la configuración del usuario sobrescribe el empaquetado. El orden de búsqueda es el mismo que `_config_path` en `hyprland.lua`: la copia de la configuración del usuario, luego la copia de los datos ARGVUS, luego la copia del estado, luego la copia generada, y solo entonces el archivo empaquetado bajo `/usr/share/argvus`.

## Ejemplos

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```
## Configuración

`argvus setup` copia los valores predeterminados empaquetados de `/usr/share/argvus/<app>` en `~/.config/argvus/data/<app>`, para que puedas personalizarlos. Nunca edita los archivos empaquetados.

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```
Un destino que ya existe se reporta y se deja sin cambios, a menos que se proporcione `--force`. Con `--force`, el directorio existente se renombra a `<name>.bak-<timestamp>` antes de la copia.

La aplicación `argvus` es especial: copia el contenido de su directorio empaquetado en la raíz de configuración. Debido a que la raíz de configuración ya existe cuando se ejecuta la configuración, `--copy argvus` necesita `--force` en el código actual.

La configuración valida cada opción antes de cambiar cualquier cosa. Se rechazan los nombres de aplicaciones no válidos (vacíos, que comiencen con `.` o que contengan `/`).

## Sesión y recarga

`SUPER + SHIFT + R` ejecuta `argvus sessionctl reload`. El controlador de sesión aplica la configuración generada y reinicia únicamente los componentes que controla. No copia `/usr/share/argvus` en el directorio de inicio y no reemplaza un archivo JSON malformado.

## Luz nocturna

Abre **Escritorio y Entrada > Pantallas > Luz Nocturna** o ejecuta `argvus control-center displays night-light`. Configura la temperatura, la activación manual y los horarios locales fijos a través de `argvus config display night-light`. Consulta [Luz Nocturna](/docs/user-guide/night-light/) para comandos, estado y requisitos de ejecución.

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` ejecuta un comando interactivo dentro del marco de terminal compartido ARGVUS. Ctrl+E cierra el marco; su llamador suministra el pie de página localizado. Úsalo dentro de una terminal o como el comando pasado a `argvus tui-terminal --term kitty -- ...`.

`argvus tui-terminal --term kitty --profile ai-agents` utiliza una configuración dedicada de Kitty con pestañas ocultas y solo accesos directos al portapapeles. El llamador suministra el comando y el título; la colocación de la ventana se maneja mediante la regla de Hyprland para la clase `argvus-ai`.

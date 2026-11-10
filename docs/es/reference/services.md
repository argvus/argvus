---
title: Servicios
description: Servicios systemd y de usuario de ARGVUS.
---
El usuario principal objetivo es `argvus-session.target`. Los servicios comunes para usuarios incluyen `argvus-taskbar.service`, `argvus-control-panel.service`, `argvus-widget-telemetry.service`, `argvus-hyprland.service`, `argvus-dunst.service`, `argvus-wallpaper.service`, `argvus-hypridle.service`, servicios del portapapeles y `argvus-session-loading.service`.

El firewall es un servicio del sistema llamado `argvus-firewall.service`. El paquete de calendario instala `argvus-taskbar-calendar.service` como una unidad de usuario, y `argvus-clock.service` (deshabilitado por defecto) inicia ARGVUS Clock oculto después del inicio de sesión cuando **Iniciar con la sesión** está activado.

Inspecciona las unidades activas con:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```
## Unidades propiedad de la sesión

El paquete de sesión instala estas unidades de usuario bajo `/usr/lib/systemd/user` (la carga útil fuente está bajo el árbol de configuración compartida del paquete de sesión):

| Unidad | Rol |
| --- | --- |
| `argvus-session.target` | Límite de vida para la sesión gráfica ARGVUS. |
| `argvus-session-prepare.service` | Preparación de inicio de una sola vez y configuración generada. |
| `argvus-control-panel.service` | Panel de control Quickshell. |
| `argvus-taskbar.service` | Barra de tareas principal de ARGVUS. |
| `argvus-widget-telemetry.service` | Superficie opcional de telemetría/información del sistema. |
| `argvus-hyprland.service` | Unidad de una sola vez bajo demanda que ejecuta `hyprctl reload`. |
| `argvus-config.service` | Proyecta la configuración canónica (`argvus config project`); la recarga ejecuta `argvus sessionctl reload-config-transaction`. |
| `argvus-session-reload.service` | Recarga de inicio de sesión (`argvus sessionctl reload-desktop --login`): recarga Hyprland solo cuando la proyección lo afectó. |
| `argvus-ssh-agent.service` | Agente SSH para la sesión. |
| `argvus-secret-service.service` | Inicia el servicio secreto a través de `argvus sessionctl start-secret-service`. |
| `argvus-wallpaper.service` | Backend de fondo de pantalla activo. |
| `argvus-dunst.service` | Daemon de notificaciones Dunst. |
| `argvus-hypridle.service` | Política de inactividad y bloqueo. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | Observadores del historial del portapapeles. |
| `argvus-keyboard-layout.service` | Notificaciones de distribución del teclado. |
| `argvus-polkit.service` | Integración con el agente PolicyKit. |
| `argvus-session-loading.service` | Superposición de carga de la sesión desde la entrega del compositor hasta que inician los componentes del escritorio. |

`argvus-blueman-applet.service` también se envía mediante el paquete de sesión. Algunas unidades son opcionales o se inician de manera condicional; no todas están activas en cada sesión.

Para inspeccionar la definición efectiva de una unidad en lugar de solo su verificación de origen:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```
El `argvus-firewall.service` a nivel de sistema pertenece a `argvus-firewall`. El paquete de calendario envía por separado `argvus-taskbar-calendar.service` como una unidad de usuario. `argvus-clock.service` es una unidad de usuario del paquete `argvus`, `WantedBy=graphical-session.target`, habilitado desde la configuración del reloj ARGVUS.

## Comandos de gestión de sesiones

El comando `argvus sessionctl` gestiona el ciclo de vida de la sesión y puede reiniciar componentes específicos sin cerrar sesión:

### Ciclo de vida de la sesión

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```
### Reinicios específicos de componentes

Reinicia servicios individuales sin afectar toda la sesión:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```
### Sincronización del entorno

Importar variables de entorno a la sesión:

```sh
argvus sessionctl import-environment
```
Esto sincroniza las variables de Wayland, Hyprland, Qt, cursor y DBus entre el shell y los servicios de usuario de systemd.

## Referencia de unidades de servicio

Se pueden gestionar unidades de usuario adicionales directamente con `systemctl --user`:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```
Los registros de la inicialización de la sesión inicial se almacenan en:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```

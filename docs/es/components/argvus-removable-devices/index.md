---
title: Dispositivos extraíbles
description: Montar y gestionar almacenamiento extraíble desde ARGVUS.
---
`argvus-removable-devices` supervisa UDisks2 y proporciona el módulo de almacenamiento de la barra de tareas ARGVUS y su menú de dispositivos. El menú también está disponible como una interfaz separada al estilo Rofi, por lo que las mismas acciones del dispositivo se pueden usar desde la barra de tareas o desde un flujo de trabajo del lanzador.

Las acciones disponibles son `open`, `mount`, `unmount`, `eject`, `poweroff`, `lock`, `unlock` y `copy`. Los comandos `list`, `watch` y `menu` exponen las interfaces correspondientes de estado, flujo de la barra de tareas y menú para integraciones.

## Menú de la barra de tareas

Con `"mode": "gui"` (el predeterminado del sistema), un clic en el ícono de almacenamiento de la barra de tareas abre un menú GTK adjunto a la barra de tareas ARGVUS, como el popup del calendario: debajo de la barra cuando la barra de tareas está en la parte superior de la pantalla, y por encima de ella cuando la barra de tareas está en la parte inferior. Cada fila de dispositivo tiene un submenú de acciones con solo las acciones que permite su estado.
- **Puntero:** pasar el puntero o hacer clic en una fila de dispositivo abre su submenú. El submenú permanece abierto mientras el puntero cruza los bordes del menú o sale de los menús, y sigue otra fila solo después de que el puntero se detiene allí brevemente, por lo que un camino diagonal hacia el submenú no lo cierra. Un clic fuera de ambos menús cierra la ventana emergente.
- **Teclado:** `Up`/`Down` selecciona una fila, `Right`, `Enter` o `Space` abre el submenú (o ejecuta **Actualizar**/**Salir**), `Enter` o `Space` ejecutan la acción seleccionada, `Left` o `Escape` vuelve a la lista de dispositivos, y `Escape` cierra allí la ventana emergente.

**Copiar ruta** coloca el punto de montaje en el portapapeles a través de `copy_command` (por defecto `wl-copy`, desde `wl-clipboard`), por lo que la ruta todavía está allí después de que el menú se cierre; una notificación lo confirma cuando `show_notification` está activado. **Bloquear** se ofrece para cada volumen encriptado desbloqueado y lo desmonta primero cuando es necesario.

La acción predeterminada del administrador de archivos sigue al administrador de archivos seleccionado en el Centro de Control ARGVUS. Esto mantiene las acciones de los dispositivos extraíbles consistentes con los valores predeterminados de las aplicaciones del usuario.

La configuración del sistema es `/etc/argvus/removable-devices/config.json`. La proyección por usuario administrada por ARGVUS es `~/.config/argvus/data/removable-devices/config.json`, con un `theme.css` correspondiente junto a ella. Esa hoja de estilos también se proyecta a `~/.config/argvus/data/generated/removable-devices/theme.css` desde el tema canónico, el acento y el modo, con un recurso de reserva de familia/variante para temas que no incluyen un activo dedicado para dispositivos extraíbles, por lo que un cambio de tema mantiene el módulo consistente sin ningún paso manual. Las rutas de componentes heredados son entradas de migración; la configuración posterior del usuario anula los valores predeterminados del sistema.

Las configuraciones útiles incluyen si el módulo está oculto cuando no hay ningún dispositivo presente (`hide_when_empty`), si se muestran nombres y capacidades, el orden de clasificación de los dispositivos, el comportamiento de las notificaciones, el backend del menú y los iconos de estado. Su hoja de estilos se genera a partir del tema activo ARGVUS (`removable-devices/theme.css` de la compilación del tema) y se copia al `theme.css` del panel cada vez que hay un cambio de tema o acento.

La interfaz utiliza el idioma del usuario de `LANG` en `$XDG_CONFIG_HOME/locale.conf`, siendo el inglés el predeterminado para un usuario no configurado. Las variables de localización heredadas no seleccionan el idioma de la interfaz.

## Catálogos de idiomas compartidos

La interfaz utiliza los catálogos compartidos `argvus-i18n` para los diez idiomas. Los textos se almacenan en `resources/locales/<locale>/removable-devices.json` en el espacio de trabajo maestro y se instalan mediante `argvus`. El inglés es el de respaldo; los nuevos idiomas ya no requieren un diccionario bilingüe local.

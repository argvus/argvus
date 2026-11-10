---
title: Solución de problemas de temas y fondos de pantalla
description: Diagnosticar la propagación del tema y los archivos de apariencia generados.
---
Verifique la sección canónica `appearance` en `$XDG_CONFIG_HOME/argvus/config.json`, luego inspeccione los archivos generados bajo `$XDG_CONFIG_HOME/argvus/data/generated`. Los archivos puntuales heredados son solo insumos de migración.

`argvus-config` es el único escritor debajo de `data/generated/`. Para recuperar un tema o fondo de pantalla roto, reconstruya toda la generación desde el documento canónico y aplique el resultado en lugar de editar directamente los consumidores generados:

```sh
argvus config validate
argvus config apply
argvus sessionctl reload
```
`argvus theme-switch` (que ejecuta `argvus config theme-apply`) y `argvus config accent` son orquestadores: validan, bloquean, muestran la pantalla de presentación, confirman a través de `argvus-config` y luego reconcilian a los consumidores externos que no pueden leer `config.json`. Nunca escriben `data/generated/`, y la transacción de tema no escribe los archivos GTK, de almacenamiento o `mode.css` a través de scripts separados. Usa `argvus config accent --apply` para volver a ejecutar solo la reconciliación de acentos.

Los cambios de tema oficiales aplican sus valores de apariencia guardados y actualizan los consumidores en tiempo de ejecución afectados. Aplicar un tema personalizado reinicia la barra de tareas, recarga Hyprland, el fondo de pantalla y los servicios del Panel de Control. Cambiar Sticky o Spaced vuelve a aplicar el espaciado, los bordes y los márgenes de la barra de tareas de ese modo, luego reinicia los servicios de la barra de tareas, telemetría y recarga de Hyprland. Los colores de notificación de Dunst son la excepción mientras No Molestar esté activo: su recarga se pospone hasta que se apaga el modo DND. Si otra superficie mantiene los colores antiguos, verifica que la compilación del tema esté activa y que sea el tema que elegiste, luego ejecuta las recargas nuevamente:

```sh
cat "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/data/generated/theme/current/build.toml"
argvus config theme-runtime consumers
```
Cada archivo temático proviene de la compilación: un tema que no está instalado (no hay `theme.toml` bajo `/usr/share/argvus/themes/` o `~/.local/share/argvus/themes/`) o que no se renderiza es rechazado, y el tema anterior permanece. `argvus theme check DIRECTORY` muestra la plantilla y la línea de cada error de renderizado. Una barra Waybar recarga sus colores solo con `reload_style_on_change` en su JSONC. Si un interruptor parece atascado, verifica si todavía está en ejecución; se registra a sí mismo en `theme-switch.active`, junto a su bloqueo:

```sh
cat "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/theme-switch.active"
ps -ef | grep -E 'theme-switch|theme-apply'
```
Un PID en el marcador que todavía está vivo significa que el cambio no ha terminado. Un cambio espera como máximo 60 segundos por el bloqueo del tema y por la proyección, y 5 segundos por los comandos de recarga; un marcador cuyo proceso ha salido está obsoleto y el siguiente cambio lo ignora. Incluya la salida de los comandos anteriores al informar del problema.

Si solo el saludo previo al inicio de sesión es blanco o usa el tema de respaldo, compare la fuente privada con su proyección pública:

```sh
uid=$(id -u)
argvus config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```
Repara la proyección sin cambiar la configuración greetd:

```sh
sudo argvus greeter-setup --sync-themes
```
Ese archivo se publica para preautenticación por el orquestador de apariencia y
vive fuera de `data/generated/`, por lo que un simple `argvus config apply` no
lo actualiza.

Si el saludador informa `Invalid color name: 'null'` en una caché de Kitty, actualiza el
paquete `argvus terminal` y regenera la caché abriendo el saludador de nuevo.
Un acento canónico ausente es un valor predeterminado, no un color de Kitty; el lanzador de terminal actual
rechaza JSON `null` y valores heredados inválidos antes de escribir
`active_tab_foreground`.

Si el terminal predeterminado deja de abrirse después de varios cambios de tema, inspecciona la caché derivada de Kitty en lugar de editarla manualmente:

```sh
argvus config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus terminal --apply
argvus terminal --print-config
```
El comando anterior lee la preferencia canónica. `.active-theme` es solo una proyección de compatibilidad. La caché se regenera mediante `argvus terminal` y se sincroniza con la aplicación del tema. La ausencia o el desactualizado `/usr/lib/argvus/loading-theme/splash` indica una actualización incompleta del paquete; reconstruya e instale los paquetes coordinados antes de juzgar un resultado de sesión en vivo.

Si los bordes de la ventana mantienen el color del tema anterior, no edite manualmente el archivo Lua de Hyprland. Los cambios de tema y acento se proyectan desde `config.json`, luego `argvus sessionctl reload` recarga Hyprland y reaplica los colores de borde activos, inactivos y de grupo a lo largo del ciclo de vida de la sesión:

```text
config.json → projection plan → Hyprland reload → hyprctl reapply of active/inactive/group borders
```
Las *preferencias* de borde se cambian a través de `argvus config borders` (o su delegado `borders-switch.sh`) en el Centro de Control y el Panel de Control; recargar reaplica los colores comprometidos a lo largo del ciclo de vida de la sesión y no los vuelve a derivar.

Inspeccione el plan y los valores efectivos del compositor:

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus sessionctl reload
```
Si `hyprctl` no puede alcanzar el socket actual de Hyprland, la proyección de borde se informa como fallida y los componentes dependientes no se consideran recargados con éxito. Revise el diario de la sesión de usuario antes de reintentar.

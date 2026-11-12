---
title: Configuración de ARGVUS
description: El contrato de configuración JSON lógico utilizado por ARGVUS.
---
ARGVUS mantiene las preferencias de usuario gestionadas en archivos JSON modulares bajo `$XDG_CONFIG_HOME/argvus/config/` (normalmente `~/.config/argvus/config/`). Forman el modelo lógico canónico para las preferencias gestionadas por ARGVUS, y `argvus-config` es el único componente que escribe archivos bajo `data/generated/`. Los archivos nativos y los archivos generados permanecen como proyecciones de compatibilidad o sobreescrituras específicas de la aplicación.

## Guía del usuario: Inicio rápido

Para la mayoría de los usuarios, la configuración ocurre a través del Centro de Control de ARGVUS o esta herramienta de línea de comandos:

```sh
# View your configuration location and validate it
argvus config path
argvus config validate

# Get a specific value
argvus config get /appearance/theme
argvus config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus config apply
```
Para administradores y configuración avanzada, continúe leyendo a continuación.

El usuario root contiene únicamente el documento canónico y los datos gestionados:

```text
~/.config/argvus/
├── config.json
└── data/
```
El archivo se escribe de manera atómica, se mantiene privado para el usuario y está protegido por `.config.lock`, fuera del árbol `data/` que puede ser intercambiado durante la proyección. Una versión anterior se almacena en `data/backups/config.json.bak`.

## Línea de comandos

El subcomando `argvus config` se proporciona en proceso mediante el paquete `argvus`:

```sh
argvus config path
argvus config validate
argvus config ensure
argvus config migrate
argvus config status
argvus config explain /layout/gaps_out
argvus config get /appearance/theme --effective --raw
argvus config set /appearance/accent '"#61AFEF"'
argvus config patch appearance-patch.json
argvus config audio volume +5
argvus config audio mute toggle
argvus config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus config apply-theme universe-spaced --variant spaced --accent '#EEEEEE' --gtk-mode dark
argvus config theme-manifest universe-spaced
argvus config unset /appearance/accent
argvus config apply
argvus config rebuild
argvus config project
argvus config project --force
```
`ensure` materializa un nuevo `config.json` con los valores predeterminados del esquema, o completa los campos faltantes sin reemplazar los valores explícitos. Falla con JSON malformado. `migrate` importa el estado de apariencia heredado existente sin reemplazar los valores ya presentes en `config.json`. `argvus sessionctl` ejecuta `recover`, `ensure`, `migrate` y `project` antes de aplicar una recarga después de los cambios de configuración:

```sh
argvus sessionctl apply-config
```
`apply` y `rebuild` son alias para reconstruir cada proyección desde `config.json`; `project` es la ortografía histórica y acepta `--force`. `apply` también delega la recarga en tiempo de ejecución posterior al commit a `argvus sessionctl`. Configure `ARGVUS_NO_RUNTIME=1` para preparación o pruebas aisladas. La generación ocurre en staging y el árbol `data/` se intercambia solo después de que todos los proyectores tengan éxito. Por lo tanto, eliminar `data/generated/` y ejecutar `argvus config apply` es una reconstrucción reproducible sin publicar una secuencia parcial de salidas de Hyprland o Taskbar. El comando registra hashes de secciones derivadas y la decisión en tiempo de ejecución en `$XDG_STATE_HOME/argvus/config-projection.json` (normalmente `~/.local/state/argvus/config-projection.json`). Ni el manifiesto ni los archivos generados son fuentes de verdad. `apply` realiza la recarga posterior al commit a través de `argvus sessionctl`; `project` permanece solo para proyección por compatibilidad.

Cada mutación persistente utiliza la misma regla: si el documento canónico cambió, la CLI invoca un `argvus sessionctl reload` centralizado después del commit. El resultado de la recarga se registra en `data/internal/last-reload.json`.

`argvus sessionctl reload` es por lo tanto un paso de *aplicación*, no un paso de proyección. Cuando el manifiesto entrante informa trabajo pendiente, restaura los consumidores objetivo descritos por el plan y no llama a `argvus config project` nuevamente; reproyectar reconstruiría un árbol idéntico y descartaría el plan que la mutación acaba de producir. Solo se recurre a la migración y proyección cuando no existe un plan comprometido, por ejemplo, durante el inicio de la sesión.

### Recargar objetivos

`argvus config session-plan` responde a la recarga con los objetivos de las secciones que cambiaron, y `argvus sessionctl` ejecuta solo esos. El splash cubre una recarga solo cuando un objetivo recrea una superficie: una barra de Waybar o el Panel de Control moviéndose de nuevo en la pantalla desplaza cada ventana. Todo lo demás se recarga en su lugar.

| Cambio | Objetivos | Pantalla de inicio |
| --- | --- | --- |
| Tema, color destacado o modo GTK | `hyprland`, `consumers`, `wallpaper`, `polkit` | no |
| Solo fondo de pantalla | `wallpaper` (IPC de hyprpaper; el servicio se reinicia solo para swaybg) | no |
| `effects` (desenfoque, animaciones, transparencia, sombra) | `hyprland`, `consumers` | no |
| `layout` de ventanas (espacios, bordes) | `hyprland` | no |
| `layout.taskbar` (posición, márgenes) | `hyprland`, `taskbar`, `widget_telemetry` | sí |
| `fonts` | `consumers` | no |
| `control_panel` (interruptor, tarjetas) | `control_panel` | sí |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | no |
| `power` | `hypridle` | no |

`consumers` ejecuta `argvus config theme-runtime consumers` (las hojas de estilo de la barra, el Panel de Control sobre IPC, los colores de Hyprland, la recarga del tema de Dunst, btop y kitty). Dunst aplaza la recarga de su tema mientras el modo No Molestar esté activo. `wallpaper` ejecuta `argvus config theme-runtime wallpaper`, que muestra el `/appearance/wallpaper` efectivo. El hash `layout.taskbar` se mantiene aparte del diseño de la ventana en el manifiesto de proyección, por lo que un cambio de brecha no reinicia las barras. El plan de respaldo `full`, usado cuando no existe un plan comprometido, reinicia cada superficie bajo la pantalla de inicio.

Cada configuración efectiva tiene una generación operativa. Se almacena en `data/internal/generation.json` y en el manifiesto de proyección bajo `$XDG_STATE_HOME/argvus/config-projection.json`; no forma parte de la configuración de archivos ocultos. Reaplicar una configuración efectiva sin cambios preserva la generación. `status` informa si el hash del manifiesto, los metadatos de generación y todos los archivos generados coinciden; `apply` repara una proyección obsoleta o incompleta.

`explain /path` lee el resolutor directamente e imprime el valor efectivo, la fuente, el tema y la variante. No analiza archivos generados ni crea una segunda regla de precedencia.

La proyección de temas y efectos se implementa en `argvus-config` por sí misma. La transacción de temas (`argvus theme-switch`, que ejecuta `theme-apply`), `argvus config accent` y `argvus effects-toggle` no proyectan: verifican, bloquean, confirman a través de este CLI y luego reconcilian los consumidores externos que no pueden leer `config.json`. `argvus effects-toggle` es un delegado ligero sin salida propia. Los ayudantes de apariencia nunca son llamados por el proyector, y nunca deben escribir por debajo de `data/generated/`.

### Superficies de proyección

`project_effective` genera cada consumidor propiedad de ARGVUS en un árbol de preparación antes de la publicación:

| Superficie | Salida |
| --- | --- |
| Hyprland | `generated/hypr/`, incluyendo perfiles de temas, `spaces-effective.conf`, `borders-effective.conf`, `hypridle.conf` y `monitors.lua` |
| Diseño/efectos | `generated/effects/`, `generated/spaces.conf`, `generated/borders.conf` |
| Waybar | `generated/waybar/`, incluyendo `argvus-taskbar.css` y `argvus-widget-telemetry.{jsonc,css}` |
| Temas de telemetría de widgets | `generated/widget-telemetry/` y `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| Notificaciones | `generated/notifications/` y `dunstrc` |
| Pantalla de bloqueo | `generated/hypr/hyprlock.conf` y los perfiles de tema de Hyprtoolkit |
| GTK | `generated/gtk/` |
| Yazi | `generated/yazi/` |
| Superarchivo | `generated/superfile/` |
| Terminal y perfiles Qt | `generated/terminal/`, `generated/qt6ct/` |
| Fuentes | `generated/fonts.conf` más los bloques de fuentes gestionados en las hojas de estilo de Waybar |
| Dispositivos extraíbles | `generated/removable-devices/theme.css` |
| Fondo de pantalla | `generated/hypr/hyprpaper.conf` y `data/hypr/hyprpaper.conf` |
| Panel de control y valores predeterminados | `data/control-panel/`, `data/control-center/defaults.json` |
| Calendario, entrada, teclado, energía | calendario TOML, fragmento de entrada, estado del teclado, marcador de mantener activo |

La capa de tema también se refleja fuera de la raíz generada:
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`,
`data/qt6ct/`, `data/hypr/hyprtoolkit.conf` y
`data/hypr/application-style.conf`. Esas copias se reemplazan a partir de los
valores predeterminados empaquetados cuando cambia la sección de apariencia, y el proyector entonces reescribe
solo sus bloques gestionados delimitados: el bloque de fuente en las hojas de estilo de Waybar y
los bloques `ARGVUS_TELEMETRY_*` en el perfil de telemetría; así, el texto fuera de los
marcadores no sobrevive a un cambio de tema. `fonts.targets` y
`control_panel.widget_telemetry_blocks` permanecen canónicos: el Centro de Control
los conserva a través de `argvus-config` y nunca escribe estos archivos por sí mismo.

La selección de bloques de telemetría de widgets es canónica en `/control_panel/widget_telemetry_blocks`, un arreglo de los siete nombres de bloque (`system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network`, `keys`). El archivo heredado `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` es de solo lectura como respaldo de migración y nunca se escribe. ` and ` and blocks all parchea el arreglo canónico en una sola escritura atómica, y `apply-state` asume que la selección ya fue persistida por su llamador.

Las preferencias de espaciado y borde de Hyprland siguen el mismo límite. Los comandos argvus config spaces y argvus config borders (y sus envoltorios de compatibilidad histórica) actualizan /layout; Rust genera data/generated/spaces-effective.conf y data/generated/borders-effective.conf y tiene la propiedad de la validación y la aplicación en vivo de hyprctl. .spaces y .borders son estado exclusivo para migración y nunca son la fuente de la verdad.

El estado de acento y el modo GTK utilizan el mismo límite de Rust. Use ` or ` or argvus config accent --theme-default para la propiedad del acento, y `argvus config mode toggle|set dark|light` para el modo GTK. Los scripts históricos de acento y modo son envoltorios de compatibilidad; `.accent-color`, `.accent-custom` y `.gtk-mode` son solo archivos de migración/proyección.

Las rutas lógicas de ARGVUS también se resuelven mediante `argvus config paths`. El archivo heredado `paths.sh` solo expone funciones de compatibilidad; las anulaciones nativas, los datos del usuario, los datos generados y el recurso alternativo del sistema se resuelven mediante el modelo de rutas de Rust.

`audio` cambia el dispositivo de salida predeterminado y registra el resultado en `/audio` en un solo paso: `argvus config audio volume <N|+N|-N> [--limit PERCENT]` y `argvus config audio mute <on|off|toggle>`. Escribe la configuración sin recargarla, porque PipeWire ya tiene el valor. El módulo de volumen de Waybar, las teclas de medios de Hyprland y el Panel de Control lo utilizan, por lo que `argvus config project`, que proyecta `/audio` sobre el dispositivo de salida, nunca restaura un volumen anterior. Un volumen superior al 100% (las teclas de Hyprland permiten hasta 150%) se almacena como 100.

`patch` aplica un objeto JSON cuyos claves son punteros JSON en una sola operación bloqueada de lectura-modificación-escritura. `apply-theme` realiza la transacción atómica equivalente de tema y reemplaza las secciones de apariencia gestionadas desde el perfil de apariencia opcional del tema o los valores predeterminados empaquetados. Los comandos de mutación construyen un `NextUserConfig`, resuelven un `EffectiveConfig`, preparan todos los proyectores, y solo entonces publican el JSON canónico. Un marcador operativo `.config-transaction.json` permite que `argvus config recover` reproyecte la configuración anterior después de un fallo entre los datos y la publicación del JSON. Los valores predeterminados del tema son declarados por el manifiesto de cada tema (`themes/<theme>/theme.toml`): el acento predeterminado (`palette.accent`), el esquema de color GTK y el fondo de pantalla. Los archivos CSS, Lua, Rasi y Qt se generan a partir de ellos en la compilación del tema.

Cambiar el tema, incluyendo Claro/Oscuro, restablece el color de resaltado al valor predeterminado del tema, incluso cuando el usuario eligió un color personalizado: el color personalizado se mantiene solo hasta el próximo cambio de tema. Use `argvus config accent` después del cambio de tema para configurarlo nuevamente. Aplicar un tema con `argvus config theme-apply` escribe `/appearance/accent` y `/appearance/gtk_mode` desde el manifiesto, y `argvus config accent` escribe el color elegido con `accent_custom` configurado, en un solo comando.

`apply-theme` toma la variante explícitamente como `--variant sticky|spaced` y resuelve el nombre solicitado a través del manifiesto, por lo que una solicitud nunca puede resolverse a los recursos de la otra variante. `theme-manifest THEME` es la misma resolución expuesta en solo lectura: imprime la entrada del manifiesto junto con `asset_id` y `layout_variant` a los que se resolvió, que es lo que un llamador necesita para acceder a los recursos empaquetados. Sticky es el id sin sufijo y solo Spaced lleva un sufijo `-spaced`, por lo que `dracula-sticky` y `argvus-dark-dracula-sticky` son aceptados y se normalizan a `dracula`.

Con `--raw`, los valores de cadena se imprimen sin comillas JSON, mientras que los objetos y arreglos permanecen como valores JSON. Los archivos puntuales de tema y diseño ya no son escritos por el asistente de proyección; `config.json` sigue siendo la única fuente persistente para esas preferencias. Si la proyección falla, la recarga de la sesión se detiene antes de reiniciar las superficies de escritorio, de modo que no se presenta un tema parcial antiguo como actual.

## Exportar e importar

Se utiliza el mismo modelo lógico para el intercambio de perfiles:

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```
Utilice `--scope desktop` para el modelo completo de escritorio gestionado. Los campos faltantes usan los valores predeterminados de ARGVUS; la salida generada no debe editarse directamente.

La exportación e importación de temas del Centro de Control intercambia un tema y su perfil visual: el archivo contiene un directorio de tema de esquema 2 (`<id>/theme.toml`, opcional `appearance.json` y `effects.conf`, `overrides/`, el fondo de pantalla) que se instala bajo `$XDG_DATA_HOME/argvus/themes/`. El perfil contiene apariencia, geometría de diseño, efectos visuales, fuentes y presentación del panel. Las preferencias de audio, teclado, red y otras no visuales permanecen fuera de los temas. Utilice el alcance `appearance` anterior para mover la configuración completa de apariencia, y el alcance `desktop` para una copia de seguridad completa de la configuración.

El esquema actual agrupa las preferencias bajo `appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar` y `removable_devices`. `default_apps` es la única fuente persistente para aplicaciones preferidas, y `keyboard_shortcuts` almacena claves estables en inglés, independientes de la configuración regional, con valores de cadena o `null` para enlaces deshabilitados.

El Centro de Control lee los valores de apariencia canónicos cuando se abre una página de Apariencia y escribe los cambios a través de `argvus-config`. Los archivos de estado nativos existentes siguen siendo proyecciones para los scripts de apariencia y para instalaciones más antiguas.

El Panel de Control utiliza `control_panel.cards` para el orden y la visibilidad de las tarjetas, y `control_panel.enabled` para el interruptor maestro del panel. Sus archivos `control-panel/cards.json` y `state/control-panel` se mantienen como proyecciones de compatibilidad. Por lo tanto, los cambios realizados en el Panel de Control son parte del alcance de exportación `desktop`.

Las preferencias de fuente utilizan `fonts.targets` para la familia, estilo y tamaño de la barra de tareas, telemetría, Panel de Control, sistema, aplicaciones, terminal y objetivos del navegador. `fonts.rendering` almacena preferencia de antialiasing, hinting, orden de subpíxeles y DPI. `fonts.conf`, la regla de fontconfig, la configuración del escritorio y los bloques de fuentes gestionados dentro de las hojas de estilo de Waybar son proyecciones: el Centro de Control persiste `/fonts/targets/*` a través de `argvus-config`, y el proyector reescribe solo su bloque delimitado.

Las preferencias de Hyprland usan `keyboard_shortcuts` para atajos, `hyprland.input` para la configuración del ratón y del panel táctil, y `hyprland.keyboard` para el diseño XKB, la variante y las opciones. Un ID de vinculación de manifiesto se asigna a un `config_key` estable (por ejemplo, `window.close` → `close_window` y `window.drag_mouse` → `drag_window__floating_window_only`). Las preferencias de pantalla usan `displays.monitors`, `displays.workspaces` y `displays.primary_monitor`. Los archivos TOML nativos y los archivos Lua generados bajo `data/generated/` siguen siendo proyecciones consumidas por Hyprland.

Cada interruptor `effects` está activado por defecto (animaciones, desenfoque, telemetría de widgets y cada superficie de transparencia), también en máquinas virtuales y sesiones de bajo consumo; la sombra de ventana (`shadow_enabled`) y el bamboleo (`wobble_enabled`) están desactivados por defecto. Las preferencias de efectos usan `effects` para el interruptor maestro de Animaciones (`animations`, aplicado a `hyprland.lua` desde el `hypr/state.lua` proyectado, independiente del interruptor de Efectos, que es `blur_global_enabled` más el `transparency_<surface>_enabled` de `taskbar`, `control-panel`, `widget-telemetry`, `terminal` y `launchers`, y se lee como activado cuando cualquiera de ellos lo está; `argvus effects-toggle animations|effects` los cambia), el interruptor global de desenfoque (`blur_global_enabled`), los parámetros de desenfoque de Hyprland (`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`), la sombra de ventana (`shadow_enabled`, `shadow_sharp`, `shadow_color`, `shadow_range`, `shadow_render_power`, `shadow_offset_x`, `shadow_offset_y`, `shadow_scale`), los valores de Animaciones (`animation_<leaf>_speed`, `_curve`, `_style`, `_popin` y `_enabled` para cada animación, más `animation_bezier_x1` a `animation_bezier_y2` y `animation_spring_mass`, `animation_spring_stiffness` y `animation_spring_dampening` para las curvas personalizadas) y los valores de transparencia por superficie. Los efectos son propiedad del tema, no de la variante: Sticky y Spaced usan la misma proyección de efectos `<theme>.conf` mientras que sus proyecciones de diseño difieren. Las superficies actualmente incluyen `taskbar`, `control-panel` y `widget-telemetry`; los valores de transparencia son porcentajes de `0` a `100` (las superficies del Centro de Control y del lanzador predeterminan a `15`, las demás a `50`), los parámetros de desenfoque son números dentro de los rangos declarados en `schema.json`, y los estados habilitados son booleanos. Los valores de animaciones también son validados por `argvus-config`: las velocidades son números de `1` a `10` (se permiten decimales), los porcentajes emergentes son números enteros de `1` a `100`, las curvas deben ser nombres de la página de Animaciones, los estilos deben ser nombres de la página de Animaciones y aceptados por la propia hoja de la animación (por ejemplo, `popin` se rechaza para `animation_workspace_in_style`, `gnomed` se acepta solo para abrir y cerrar ventanas, y las variantes de diapositivas se rechazan en las hojas de ventana y capa), los puntos Bézier Y permanecen entre `-1` y `2`, el rango que analiza Hyprland 0.56.2 y los demás parámetros de curva permanecen dentro de sus rangos documentados. Las hojas `layers` y `switching` ya no existen, por lo que las configuraciones guardadas antes de su eliminación pierden las claves `animation_layers_enabled`, `animation_layers_speed`, `animation_layers_curve` y `animation_switching_*` cuando se cargan. Los estilos fuera del conjunto actual se eliminan y los puntos Bézier Y fuera de `-1` a `2` se fijan, por lo que los valores predeterminados empaquetados se aplican nuevamente a lo que se eliminó.

Las preferencias de energía usan `power.screen_off_minutes`, `power.lock_minutes`, `power.keep_awake` y las políticas de tapa/botón de encendido. Los formularios proyectados son `data/generated/hypr/hypridle.conf` y el marcador `data/power/keep-awake`; `.keep-awake` es solo una entrada de migración. `session.language` está retirado: los valores existentes se ignoran, las nuevas escrituras se rechazan y no se proyecta ningún archivo de compatibilidad de idioma. Configure `LANG` en el usuario `locale.conf` en su lugar. Las preferencias del calendario usan `calendar` y se proyectan al TOML de usuario del calendario de la barra de tareas. Las preferencias de dispositivos extraíbles usan `removable_devices`; la aplicación de dispositivos extraíbles trata esta sección como una anulación del usuario sobre los valores predeterminados del paquete.

`migrate` importa los archivos y secciones raíz antiguos una vez, preservando un valor canónico existente. Los legados `defaults.json`, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, archivos con punto, `generated/` y directorios de componentes se trasladan debajo de `data/`; las recargas repetidas no recrean la estructura raíz antigua. Seleccionar el tema actual nuevamente es idempotente: si el estado canónico y las proyecciones ya coinciden, el plan de proyección no reporta secciones cambiadas y la sesión omite la pantalla de bienvenida y los reinicios de servicios.

La energía sigue el mismo contrato de proyección: el Centro de Control escribe `/power` en `config.json`, `argvus config project` renderiza `data/hypr/hypridle.conf` de manera atómica, y solo entonces puede `argvus sessionctl` reiniciar `argvus-hypridle.service`. Eliminar ese archivo es recuperable con `argvus config project`; un valor de `0` omite el tiempo de espera correspondiente y significa Nunca.

La apariencia de la pantalla de bloqueo es proyectada por Rust desde el tema canónico, el acento, los efectos, el fondo de pantalla y el estado de la fuente hacia `data/hypr/hyprlock.conf`. El punto de entrada histórico `hyprlock-theme.sh` es solo para compatibilidad y delega en `argvus config lock apply`; `--invalidate` solo invalida la caché derivada del fondo de pantalla desenfocado.

Los cambios de tiempo de espera de la energía usan `/power/lock_minutes` y `/power/screen_off_minutes` como su fuente de verdad. `argvus config project` reutiliza el renderizador existente Hypridle, y el punto de entrada `idle-timeout.sh` es solo un adaptador de compatibilidad; el reinicio del servicio sigue delegándose a `argvus sessionctl`.

La apariencia de las notificaciones se deriva del tema canónico, el acento y los efectos hacia `data/notifications/dunstrc`. El archivo histórico de notificaciones `theme.sh` solo expone funciones de compatibilidad que llaman a `argvus config notifications`; no analiza ni muta la configuración de Dunst.

Adaptadores orientados al hardware, como el descubrimiento de pantalla y el control de Bluetooth, ahora son importados por el dominio de pantalla de Rust: `nwg-displays` es un importador externo, mientras que `/displays` y `data/generated/hypr/monitors.lua` permanecen como estado canónico y derivado, respectivamente. Las herramientas de pantalla cambian el diseño con `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` y `monitor-remove`; los comandos de pantalla son `argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply`. El archivo de Hyprland se proyecta desde `/displays`; ningún script de shell lo escribe más. La selección de fondo de pantalla, el modo de grupo de utilidades de la barra de tareas y la búsqueda de aplicaciones predeterminadas también usan las APIs `argvus-config`; sus puntos de entrada históricos de Shell son envoltorios de UI o de compatibilidad. Mantener activo es una preferencia persistente `/power/keep_awake` proyectada al marcador `data/power/keep-awake`, mientras que bloquear-DPMS es un estado de tiempo de ejecución de sesión bajo
`$XDG_CACHE_HOME/argvus/lock-dpms`. El diseño del teclado ahora es propiedad del asistente Rust `argvus-keyboard-layout`: el orden configurado proviene de la proyección de entrada generada por Hyprland o del recurso de localización, mientras que el diseño activo permanece en el estado de ejecución de la sesión descubierto por Hyprland. El punto de entrada del ciclo y el demonio del diseño del teclado son solo envoltorios de compatibilidad; el servicio de usuario ejecuta el demonio Rust directamente y `argvus sessionctl keyboard-layout cycle` es el comando estable para los llamadores.

El estado de ejecución de Bluetooth sigue el límite del adaptador externo pero no forma parte de `argvus-config`: `argvus bluetooth` posee el análisis tipado y la validación en torno a `bluetoothctl`, los reintentos limitados, las comprobaciones de servicios del sistema y el estado de RFKill. `argvus bluetoothctl` es el punto de entrada. La energía del adaptador, los dispositivos emparejados y los dispositivos conectados permanecen como estado de ejecución de BlueZ en lugar de estado JSON canónico; la validación de Bluetooth en hardware real depende del entorno.
## Identidad de ejecución y reconciliación

Use `argvus config status --verbose` para inspeccionar el binario exacto, la identidad de compilación, la raíz de configuración, la raíz generada, la raíz de activos, la fuente del manifiesto y la generación activa. Informa los estados de proyección `synchronized`, `stale`, `incomplete`, `legacy` y `pending-transaction`.

`argvus config --version` informa la versión del paquete, la revisión de compilación cuando es proporcionada por el entorno de compilación, y el perfil. Las pruebas de desarrollo deben inyectar su binario y recursos de manera explícita; no deben usar silenciosamente `argvus config` o `~/.config/argvus`. `apply` normaliza de manera segura identificadores combinados heredados como `argvus-dark-spaced` en valores separados de tema y variante antes de reconstruir proyecciones. La identidad del recurso empaquetado no se registra en un manifiesto en esta versión.

## Luz nocturna

Abra **Escritorio e entrada > Pantallas > Luz nocturna** o ejecute `argvus control-center displays night-light`. Configure la temperatura, la activación manual y los horarios locales fijos a través de `argvus config display night-light`. Vea [Luz nocturna](/docs/user-guide/night-light/) para comandos, estado y requisitos de tiempo de ejecución.

### Configuración de balanceo (Hyprland 0.57+)

`effects.wobble_enabled`, `wobble_stiffness`, `wobble_damping`, `wobble_mass`, `wobble_intensity` y `wobble_mesh` se proyectan directamente en `hypr/state.lua`. Usa `argvus effects-toggle wobble-settings get` para leerlos, o `argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8` para guardar y solicitar una recarga. `enabled=disabled` apaga el efecto.

El Centro de Control comparte el borrador de Animaciones: `animations-settings get` también devuelve entradas `wobble_*`, y `animations-settings set` acepta estos nombres completos junto con sus argumentos de animación existentes. Esto persiste todo el borrador en un parche validado y solicita una recarga. Por ejemplo:

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```
El `enabled` global controla las animaciones; `wobble_enabled` es su preferencia independiente. Desactivar las animaciones globales suprime el balanceo en Lua sin borrar esa preferencia. Los valores no válidos rechazan todo el parche. Las claves ausentes reciben los valores predeterminados de ARGVUS sin reemplazar las elecciones existentes. Vea [Appearance](/docs/user-guide/appearance/#window-wobble) para límites y valores predeterminados.

### Atajos de teclado creados por el usuario

`/keyboard_shortcuts` mantiene sus valores existentes de cadena o nulos para acciones del manifiesto oficial. Los atajos de teclado globales personalizados usan un objeto `/hyprland/custom_shortcuts` separado, almacenado en `config/hyprland.json`. Sus IDs estables `custom_` son independientes de las descripciones y los locales. Cada registro tiene `description`, `keys`, `enabled` y un `action` tipado:

```json
{
  "custom_editor": {
    "description": "Open my editor",
    "keys": "SUPER + CTRL + E",
    "enabled": true,
    "action": { "type": "command", "command": "editor --new-window" }
  },
  "custom_float": {
    "description": "Toggle floating",
    "keys": "SUPER + CTRL + F",
    "enabled": false,
    "action": {
      "type": "dispatcher",
      "dispatcher": "window.float",
      "arguments": { "action": "toggle" }
    }
  }
}
```
El ejemplo es el valor de `/hyprland/custom_shortcuts`, no un documento de configuración completo. Los nombres de los despachadores provienen de las fábricas de despachadores Lua compatibles; los argumentos son un objeto JSON opcional o `null`, nunca código evaluado. Los atajos personalizados no tienen submapa, enlace de ratón ni flags avanzados. La proyección escribe `data/generated/hypr/custom-shortcuts.lua` como una tabla de datos escapada, incluyendo una tabla vacía cuando se elimina el último registro. Tanto la proyección completa como `project --only keyboard_shortcuts` la generan. Hyprland registra las entradas habilitadas con `hl.bind`; los fallos de registro de archivos opcionales e individuales se contienen y registran.

El Centro de Control crea/edita registros y desactiva conflictos confirmados usando una transacción `argvus config patch`. Se guarda antes de aplicar a través del servicio de configuración existente. Un intento fallido de aplicar no deshace un guardado exitoso. Restaurar los valores predeterminados oficiales conserva los registros personalizados y se rechaza si colisionaría con un acceso directo personalizado activo. Las descripciones de los usuarios permanecen como texto literal en todas las localidades y en la hoja de trucos generada efectiva.

## Modo de ventana global

Abra **Centro de Control → Escritorio e Entrada → Hyprland → Ventanas → Modo** y seleccione **Ventanas en mosaico (Predeterminado)** o **Ventanas flotantes**. El mismo selector está disponible desde el ícono de modo de ventana en el grupo de utilidad de la barra de tareas.

La preferencia se aplica a cada monitor y espacio de trabajo y se recuerda después de cerrar sesión. Flotar convierte las ventanas de aplicación existentes y las ventanas recién abiertas. En mosaico solo se restauran las ventanas que esta política convirtió; los diálogos y otras ventanas que ya estaban flotando permanecen flotando. Una ventana fijada explícitamente mientras Flotar está activo también permanece flotando para preservar su fijación. El diseño en mosaico se recalcula; su árbol de división exacto anterior no se restaura.

`SUPER + SHIFT + Space` mantiene su comportamiento normal por ventana en Mosaico. En Flotar no cambia una ventana de nuevo a mosaico. Los paneles, la barra de tareas y las notificaciones son superficies layer-shell y no se ven afectadas. La pantalla completa y la maximización continúan utilizando el comportamiento normal de Hyprland.

La preferencia canónica es `/hyprland/window_mode` en `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. Su proyección generada es `data/generated/hypr/window-mode.lua`; no la edite. El registro de restauración por ventana se encuentra bajo `$XDG_RUNTIME_DIR/argvus`, limitado a la instancia del compositor, y sobrevive a las recargas de configuración. No se reutiliza en una nueva sesión.

Si la aplicación está pendiente o falla, la interfaz lo distingue de la preferencia guardada. Seleccione la opción nuevamente para reintentar. Sin un compositor, la preferencia se guarda para el próximo inicio de sesión.

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```
## Botones de ventana y barra de aplicaciones

El modo flotante añade una barra de título ARGVUS con **Minimizar**, **Maximizar / Restaurar** y **Cerrar** a las ventanas normales de la aplicación. Hacer doble clic en la barra de título activa o desactiva la maximización. La decoración adicional se oculta en mosaico y pantalla completa. Las aplicaciones que dibujan sus propios controles pueden mostrar ambas barras; los controles uniformes de ARGVUS son intencionales.

Aparece una segunda Waybar en cada monitor, en el borde opuesto a la barra de tareas principal. Solo enumera las ventanas pertenecientes al espacio de trabajo actual de ese monitor, incluidas las ventanas minimizadas de ese espacio de trabajo. Un espacio de trabajo especial abierto tiene prioridad. Las ventanas fijadas permanecen visibles; las pestañas de grupo ocultas no crean entradas adicionales, y un grupo minimizado tiene una entrada de recuperación. Las pestañas dentro de una aplicación no se enumeran por separado. Las entradas minimizadas conservan su asociación original de monitor/espacio de trabajo. Haz clic en una ventana inactiva para activarla, en una ventana activa para minimizarla, o en una ventana minimizada para restaurarla; su menú contextual ofrece restaurar, minimizar, maximizar/restaurar y cerrar. Los íconos de aplicaciones desconocidas se sustituyen por un símbolo de ventana. Activar o restaurar una entrada preserva la posición del puntero. La barra secundaria utiliza el CSS primario efectivo (colores, fuentes, bordes y redondeo), con su altura y márgenes reflejados en el borde opuesto.

`SUPER + Alt + Up` coloca la barra principal en la parte superior y la barra de ventana en la parte inferior. `SUPER + Alt + Down` hace lo contrario. Repetir una dirección deja las posiciones sin cambios. Tile solo tiene la barra principal. Volver a Tile restaura todas las ventanas minimizadas antes de quitar la barra de ventana y luego preserva la política original de mosaico/flotante.

La minimización utiliza el espacio de trabajo reservado `special:argvus-minimized`. No use ese espacio de trabajo manualmente. El proceso `argvus windows service` registra la identidad, el espacio de trabajo/monitor original, la geometría, el estado de pantalla completa/maximización y el estado de fijación en un registro de tiempo de ejecución específico de la instancia del compositor bajo `$XDG_RUNTIME_DIR/argvus`. Los registros sobreviven a un reinicio del servicio o recarga de configuración, pero no cruzan sesiones del compositor. Cerrar una ventana elimina su registro. Si un monitor desaparece, la restauración utiliza una salida disponible. Una barra de recuperación faltante o con fallo evita nuevas minimizaciones; la restauración fallida conserva el registro y la barra de recuperación.

La sesión posee `argvus-windows.service` y `argvus-window-bar.service`. Esta última mantiene a su hijo Waybar inactivo en Tile. El plugin hyprbars está incluido por `argvus-hyprland`, compilado para el ABI de Hyprland 0.57.0 en x86_64; no se compila en el directorio de inicio del usuario. Reconstruya el plugin con una actualización del ABI del compositor.Comandos: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` y `minimize|restore|activate|maximize|close ADDRESS`. Las direcciones de las ventanas provienen de la lista. `position top --persist` almacena/proyecta la preferencia sin reiniciar la barra principal. Los detalles de configuración y errores en tiempo de ejecución se escriben en el diario; los fallos de acción visibles también muestran una notificación traducida.

En **Espaciado + Flotante**, las ventanas respetan los huecos exteriores y las áreas reservadas por ambas barras. Los límites incluyen el borde y el título, que se encuentra dentro del mismo borde de la ventana. Maximizar a través de los controles ARGVUS llena esta área con márgenes; restaurar recupera la geometría anterior, incluso después de reiniciar un servicio. Pantalla completa aún usa toda la salida. Las ventanas flotantes pueden superponerse; los huecos internos de mosaico no las separan.

Los márgenes internos reservados por las barras no se cuentan dos veces en los espacios exteriores de la ventana. Con espaciado simétrico, la distancia visible de la ventana a la barra es igual arriba y abajo, incluso después de intercambiar las barras.

`SUPER + Alt + Space` alterna el modo global de Mosaico/Flotante, usando la misma preferencia que la barra de tareas y el Centro de Control. El comando equivalente es `argvus config window-mode toggle`. Las invocaciones concurrentes se serializan; la alternancia usa la preferencia guardada incluso cuando la aplicación está pendiente. El atajo se puede cambiar o deshabilitar en Atajos de Teclado; `SUPER + Shift + Space` mantiene su acción de ventana individual.

---
title: Configuración y estado
description: Estado compartido, sobrescrituras y archivos de ejecución generados.
---
El usuario normal root es `$XDG_CONFIG_HOME/argvus` (generalmente `~/.config/argvus`). Las preferencias lógicas gestionadas se almacenan en `config.json`. Las preferencias de tema, acento, fondo de pantalla, espaciado y borde son canónicas allí; los archivos ocultos siguen siendo proyecciones de compatibilidad derivadas para los consumidores actuales de Hyprland Lua y shell mientras esos consumidores migran a `data/`.

`argvus-config` es el único escritor debajo de `generated/`. Cada archivo de consumidor propiedad de ARGVUS es una proyección de `config.json`, incluyendo Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, la barra de tareas de Waybar y los perfiles de telemetría de widgets, los árboles de configuración de Yazi y Superfile, los árboles de perfiles de terminal y Qt, los bloques gestionados de fuentes y la hoja de estilos de dispositivos extraíbles. Los componentes resuelven primero las sobrescrituras explícitas del usuario, luego la salida generada y después los valores predeterminados del paquete. Rofi no es una proyección: `argvus-launcher` posee sus archivos, y el usuario copia en vivo bajo `data/rofi/`. La proyección normal de tema/recarga nunca promueve archivos generados o empaquetados a una sobrescritura del usuario. La documentación y las herramientas deberían identificar la fuente lógica en lugar de pedir a los usuarios que editen la salida generada.

`argvus-appearance`, por lo tanto, posee la orquestación de tema y acento, mientras que `argvus-config` posee el estado canónico y la proyección estructurada. `argvus theme-switch` (la transacción `argvus config theme-apply`) y `argvus config accent` validan, bloquean y confirman a través de `argvus-config` cuando está disponible, luego reconcilian a los consumidores de compatibilidad que no pueden leer `config.json` directamente. Esos adaptadores son el archivo nativo Qt6ct, GTK `settings.ini` y las sugerencias `gsettings`, las aplicaciones de terminal y monitor del sistema, foot, el propio directorio de configuración de superfile y el saludo de preautenticación. Un ayudante que necesita un nuevo adaptador debe agregarlo allí, nunca dentro de `generated/`.

La propiedad del tiempo de ejecución se divide intencionalmente por responsabilidad: Rust posee la tienda canónica, la validación del esquema, las migraciones, el modelo de rutas, los bloqueos, las escrituras atómicas, los hashes de proyección y los renderizadores estructurados. Shell permanece solo como pequeños puntos de entrada de compatibilidad o adaptadores para consumidores externos como `hyprctl` y scripts de tema propiedad de activos. Python se reserva para pruebas, auditorías y ayudas de desarrollo; no forma parte del tiempo de ejecución crítico del escritorio.

`argvus config apply` es el límite de reconstrucción (`project` permanece como la ortografía de compatibilidad). Regenera salidas faltantes incluso cuando el documento canónico no ha cambiado, renderiza proyectores en staging, y reemplaza de forma atómica el árbol `data/` solo después de que cada proyección tenga éxito. El ciclo de vida de la sesión sigue siendo propiedad de `argvus sessionctl`, que consume ese manifiesto para planificar recargas dirigidas.

Cuando un llamante ya ha comprometido una generación — `apply-theme`, un cambio de acento, un `set`/`patch`, o un cambio de bloque de telemetría de widget — `reload` **aplica** esa generación en lugar de proyectarla de nuevo. Lee el plan desde el manifiesto, restaura los consumidores dirigidos y solo proyecta cuando el manifiesto está ausente o no tiene trabajo pendiente. Reproyectar en ese punto reconstruiría un árbol idéntico y descartaría el plan que la mutación acaba de producir. 

El manifiesto también registra la generación operativa, el hash de configuración efectiva, los dominios cambiados y la lista de archivos generados. `data/internal/generation.json` es un marcador de proyección local desechable y nunca es una preferencia del usuario. `argvus config status` compara estos registros con la configuración resuelta e informa los archivos generados que faltan o que sobran.

Las mutaciones siguen el mismo límite del motor: `UserConfig` se clona en un `NextUserConfig`, se valida, se resuelve en `EffectiveConfig`, se proyecta y solo entonces se persiste. `.config-transaction.json` es el estado de recuperación operativa; `argvus config recover` reproyecta de manera determinista el documento canónico previo después de un fallo entre la publicación de `data/` y `config.json`. La recarga se solicita solo después del commit, y una falla en la recarga no revierte una configuración persistente coherente.

## Modelo de resolución

```text
config.json logical model
    ↓
ARGVUS native compatibility state and per-app override
    ↓
ARGVUS generated runtime file
    ↓
/usr/share/argvus packaged default
    ↓
upstream default
```
Los candidatos exactos varían según el componente, pero el principio es consistente: una anulación del usuario prevalece, la salida generada es un estado derivado, y los archivos empaquetados permanecen como entradas de solo lectura. La sesión exporta `ARGVUS_CONFIG_HOME` y `ARGVUS_SYSTEM_CONFIG` para que los ayudantes y los servicios de usuario resuelvan las mismas raíces.

La recuperación de recarga sigue `recover → ensure → migrate → apply → runtime reload`, y el paso de proyección se omite cuando el manifiesto entrante ya describe una generación comprometida. `ensure` materializa `config.json` a partir de los valores predeterminados del esquema solo cuando el perfil falta, y completa los campos ausentes sin reemplazar `false`, `0` o valores personalizados explícitos. Un documento existente mal formado sigue siendo un error en lugar de ser reemplazado silenciosamente.

Las mutaciones persistentes comparan las configuraciones antiguas y nuevas resueltas a través de `ChangedDomains`. Después del commit, se envía una solicitud de recarga a través de `argvus sessionctl`; se informa y registra la falla de recarga sin revertir la configuración comprometida. El inicio de sesión realiza la recuperación antes de la materialización y proyección del arranque, por lo que los consumidores no comienzan con una transacción abandonada.

| Estado | Propósito |
| --- | --- |
| `config.json` | Preferencias lógicas gestionadas canónicas; la fuente única para todo lo que está debajo de `generated/`. |
| `.active-theme`, `.accent-color`, `.accent-custom` | Entradas de migración heredadas; no son fuentes de tiempo de ejecución y no se regeneran. |
| `.wallpaper-custom` | Entrada de migración heredada para `appearance.wallpaper`; no es una fuente de tiempo de ejecución. |
| `.spaces`, `.borders` | Entradas de migración de diseño heredadas; los valores canónicos se almacenan bajo `layout` en `config.json`. |
| `fonts.conf` | Proyección de compatibilidad de selecciones de fuentes canónicas. |
| `generated/effects` | Proyección de compatibilidad de efectos canónicos; desechable. |
| `$XDG_STATE_HOME/argvus/config-projection.json` | Manifiesto de proyección: secciones cambiadas, consumidores afectados y si se requiere una recarga. Leído por `reload`; nunca es una fuente de verdad. |
| `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` | Preferencias de bloque de telemetría heredadas. Leído solo como respaldo de migración; los valores canónicos se encuentran en `/control_panel/widget_telemetry_blocks` y el archivo ya no se escribe. |

Una propiedad es canónica cuando se lee desde `config.json` y se proyecta desde allí. Una propiedad es derivada cuando se proyecta pero también se consume como entrada para migración. Todo lo que un componente aún escribe en tiempo de ejecución —una caché, un archivo de configuración nativo bajo `$XDG_CONFIG_HOME/<app>`, un archivo de estado del saludo— es un objetivo de adaptador y no debe leerse de nuevo como configuración.

La proyección cubre a los consumidores para Hyprland, GTK/Qt, Waybar, Quickshell, terminales, notificaciones, pantalla de bloqueo y aplicaciones nativas. Hyprland lee los colores de borde activos e inactivos del tema seleccionado; un `appearance.accent_custom=true` explícito es el único caso que reemplaza el borde activo/de grupo con el acento del usuario. Cambiar de tema vuelve a aplicar estos valores mediante la recarga del compositor, por lo que no puede retener los colores del tema anterior. Si se recrea la configuración del usuario, los efectos se establecen por defecto con animaciones activadas. Eliminar `generated/` y ejecutar `argvus config apply` lo recrea byte a byte a partir de `config.json`; editar archivos generados directamente no es duradero. 
El diagnóstico en tiempo de ejecución debe identificar la tupla completa de ejecución: ruta del binario, versión/revisión, raíz de activos, raíz de configuración, raíz de datos, raíz generada, raíz interna y manifiesto de proyección. `status --verbose` expone esta tupla y
compara la generación/hash publicada con la configuración resuelta actual.  
Las pruebas de desarrollo y de paquetes instalados son modos separados; ninguno puede retroceder silenciosamente a un ejecutable no relacionado o al hogar del desarrollador. `apply` también es el límite de reconciliación para los identificadores de tema combinados heredados: un valor como `argvus-dark-spaced` se convierte en `theme=argvus-dark` y `variant=spaced` antes de la resolución. La identidad del recurso empaquetado no se registra en un manifiesto en esta versión, por lo que ningún archivo de diagnóstico la describe.

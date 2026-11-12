---
title:
description: Abre el terminal ARGVUS y los perfiles de la aplicación de terminal.
---
Abre la terminal configurada con:

```sh
argvus --terminal
```
`argvus terminal` (un subcomando de `argvus`) proporciona la integración del terminal y la configuración de Kitty. `argvus-app-profiles` suministra perfiles para aplicaciones de terminal como Superfile y Yazi; no proporciona el despachador `argvus`.

`argvus-config` proyecta el tema en `data/generated/terminal/`, y `argvus --yazi` y `argvus --spf` leen su configuración desde los árboles proyectados `data/generated/yazi/` y `data/generated/superfile/` en lugar de desde copias materializadas en su directorio personal. Una sobrescritura completa nativa en el propio directorio de configuración de la aplicación aún prevalece, por lo que puede reemplazar el árbol proyectado con su propia configuración.

Los cambios de tema regeneran los archivos derivados de Kitty bajo `$XDG_CACHE_HOME/argvus/argvus-terminal`. El generador serializa las actualizaciones y reemplaza cada archivo de manera atómica. Si el terminal aún no se abre, inspeccione el tema activo y regenere la caché con:

```sh
argvus config get /appearance/theme --raw
argvus terminal --apply
argvus terminal --print-config
```
El terminal lee primero el tema gestionado y el acento de `$XDG_CONFIG_HOME/argvus/config.json`. Los archivos heredados `.active-theme` y `.accent-color` siguen siendo recursos de respaldo para instalaciones más antiguas.

## Transparencia y desenfoque

En **Centro de Control → Apariencia → Terminal**, configura **Transparencia** y **Desenfoque** de manera independiente de `0%` a `100%`. Los cambios se aplican con el botón **Aplicar** y regeneran el perfil de Kitty bajo `$XDG_CACHE_HOME/argvus/argvus-terminal`; las ventanas existentes de Kitty reciben la configuración actualizada a través de una recarga.

`0%` mantiene el fondo opaco y `100%` lo hace completamente transparente. El desenfoque utiliza el radio de desenfoque de fondo soportado por Kitty y solo tiene un efecto visible cuando la ventana es transparente.

## Notificación de comando de larga duración

Los comandos que se ejecutan durante 30 segundos o más envían una notificación cuando terminan, siempre que la ventana del terminal no esté enfocada. La notificación utiliza `notify_on_cmd_finish` de Kitty con la integración de shell que ya está habilitada, por lo que `~/.zshrc` y `~/.bashrc` no necesitan cambios. El umbral de 30 segundos está fijado en la configuración de Kitty.

## Modo de Aplicación TUI

ARGVUS Control Center y los pop-ups TUI de la barra de tareas usan `argvus tui-terminal` con un perfil de aplicación dedicado de Kitty. Hereda el tema y la fuente activos de ARGVUS, asigna una clase de ventana estable y se cierra con su proceso hijo en lugar de abrir un shell. Control Center oculta las pestañas y tiene valores independientes de Transparencia y Desenfoque; los pop-ups de monitor de la barra de tareas y de temperatura usan una barra de pestañas inferior para su título localizado `Press q to close`. El lanzador acepta `--title TITLE` para establecer un título fijo de ventana y pestaña en Kitty. El terminal normal mantiene sus pestañas y configuraciones.

Cuando LANG está ausente en el entorno del lanzador, `argvus terminal` suministra la configuración regional personal de `$XDG_CONFIG_HOME/locale.conf` (inglés por defecto) antes de que Kitty inicialice la composición del teclado. Se conservan las anulaciones existentes del entorno de la configuración regional. Reinicie Kitty después de actualizar: exportar LANG dentro de una shell existente no reinicializa la tabla de composición de Kitty.

Se eliminan de LC_ALL y LC_CTYPE vacíos heredados de la inicialización de la sesión en el entorno hijo de Kitty. De lo contrario, Kitty trata una configuración regional vacía como explícita y no puede cargar su tabla de composición, incluso con un LANG válido. Se conservan las anulaciones no vacías. Esto también se aplica a los nuevos perfiles y requiere volver a abrir Kitty después de actualizar el lanzador.

La ventana dedicada del Centro de Control (`argvus-control-center`) recibe la anulación por lanzamiento de Kitty `confirm_os_window_close=0`, por lo que la acción de cierre del compositor no solicita confirmación. La anulación no se escribe en el perfil compartido del terminal; otros TUI y terminales regulares mantienen su política existente.

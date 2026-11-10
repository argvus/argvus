---
title: Máquinas virtuales
description: Ejecute ARGVUS con hardware gráfico virtual.
---
`argvus start` detecta la virtualización y exporta configuraciones de compatibilidad para las sesiones afectadas, incluyendo renderizado por software y alternativas para cursor/modificadores cuando el entorno detectado lo requiere.

Si una máquina virtual aún falla al renderizar, inspeccione el entorno de la sesión y los registros antes de agregar una anulación. Vea [depuración de gráficos](/docs/user-guide/troubleshooting/graphics-and-virtual-machines/).

Cuando se detecta virtualización, `argvus start` exporta variables de compatibilidad para la sesión, incluyendo renderizado por software, cursor por software y alternativas de modificadores. Esto hace que la ruta de inicio de sesión normal sea lo primero que se debe probar; no agregue una anulación permanente solo porque el escritorio se esté ejecutando en una VM.

Si la detección no es suficiente, use una anulación de Hyprland de usuario como alternativa de diagnóstico:

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
Agrega esto solo cuando la pila gráfica de la VM lo requiera:

```lua
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
Cierre sesión y vuelva a iniciarla después de cambiar el entorno de la sesión. Confirme el valor activo con `echo $LIBGL_ALWAYS_SOFTWARE`; la renderización por software puede hacer que el escritorio sea usable, pero puede reducir el rendimiento.

---
title: Componentes de la interfaz de usuario
description: Paquetes de interfaz de usuario ARGVUS.
---
Los paquetes de UI incluyen `argvus-control-panel`, `argvus-taskbar` y `argvus-launcher`, y los subcomandos `argvus` `argvus control-center` (ayudante en `argvus`), `argvus taskbar-calendar`, `argvus widget-telemetry`, `argvus terminal`, `argvus system-monitor`, `argvus removable-devices`, `argvus focus` y `argvus clock`. El calendario, Focus y Clock usan la crate compartida `argvus-taskbar-popup` para la posición y el desalojo de layer-shell; Focus también expone su lector de estado de Waybar a través de la ruta `argvus focus status` en proceso.

Su comportamiento frente al usuario está documentado por característica; esta página registra la propiedad de los paquetes.

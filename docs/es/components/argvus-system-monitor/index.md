---
title: Monitor del sistema
description: Abre el monitor del sistema ARGVUS.
---
Abre el monitor configurado con:

```sh
argvus --system-monitor
```
El subcomando `argvus system-monitor` proporciona el punto de entrada nativo y la integración con btop. La telemetría mostrada en la barra de tareas es una característica separada proporcionada por `argvus widget-telemetry`. Hacer clic en sus módulos de CPU, Memoria y GPU abre btop dentro de un panel TUI de ARGVUS con un encabezado personalizado y un pie de página `Esc Close`. El modo GPU muestra hasta seis cuadros de GPU y ningún cuadro de CPU, memoria, red o procesos. Presione `q` o `Esc` para cerrar el panel. La pestaña inferior Kitty muestra la sugerencia localizada `Press q to close`. Las llamadas independientes `argvus system-monitor --cpu`, `--mem` y `--gpu` inician btop directamente con los cuadros seleccionados.

El panel de GPU depende de las capacidades de la versión de btop instalada y de los proveedores de dispositivos del sistema. La supervisión de NVIDIA necesita soporte de GPU en btop, la Biblioteca de Gestión de NVIDIA del controlador NVIDIA instalado y acceso al dispositivo. La supervisión de AMD utiliza ROCm SMI, que es opcional y puede no estar instalada. Los datos de la GPU de Intel se leen desde sysfs y dependen del acceso del usuario a esas interfaces; btop 1.4.7 informa sobre la utilización de Intel, potencia y reloj, pero no proporciona la temperatura de Intel. Los controladores NVIDIA nouveau y otros pueden exponer la temperatura de una GPU en la barra de tareas a través de hwmon vinculado al dispositivo, mientras que el soporte del proveedor en btop depende del controlador y las bibliotecas disponibles. ARGVUS no cambia los permisos del dispositivo ni ejecuta btop con privilegios elevados. Si no se expone ninguna GPU, o btop no renderiza un cuadro de GPU dentro de tres segundos porque el soporte o un proveedor no están disponibles, el panel muestra un mensaje de no disponible localizado.

El monitor lee primero el tema y el acento seleccionados desde `$XDG_CONFIG_HOME/argvus/config.json`. Sus archivos btop generados permanecen en un estado de caché derivado; `.active-theme` y `.accent-color` son retrocesos de compatibilidad. Dado que btop mantiene su propio archivo de tema, el monitor se concilia mediante la transacción de tema (`argvus theme-switch`) como un adaptador externo después de que se confirme el cambio canónico; no forma parte del árbol generado.

## Contrato de tiempo de ejecución

- El tema y el acento provienen de la configuración canónica (` and ` and `). The files `). The files ~/.config/argvus/data/.active-theme y `.accent-color` son valores de reserva). 
- La configuración generada de btop y los temas se escriben en `$XDG_CACHE_HOME/argvus/argvus-system-monitor`. Son estados derivados; no los edite. 
- Los valores predeterminados se instalan bajo `/usr/share/argvus-system-monitor/btop`. 
- Los atajos de módulo (`--mem`, `--cpu`, `--net`, `--proc`) solo se leen como el primer argumento. Pasar los propios `-c`/`--config` o `--themes-dir` de btop ejecuta btop con esas rutas y no escribe ninguna configuración. 
- `--panel [--cpu|--mem|--gpu]` integra btop en la consola TUI de ARGVUS. Este modo es utilizado por la barra de tareas; sin `--panel`, el monitor continúa reemplazándose a sí mismo con btop.

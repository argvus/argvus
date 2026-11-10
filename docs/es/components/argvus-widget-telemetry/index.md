---
title: Telemetría de widget
description: Los bloques de telemetría de Waybar y su alternancia, a través del widget-telemetry de argvus.
---
El widget de telemetría es la información del sistema que se muestra en la barra de tareas (encabezado, claves, sistema, CPU y GPU, memoria, almacenamiento, procesos, red y el panel de desarrollador). Está en proceso en el paquete `argvus`: no hay recolectores de shell.

## Comandos

```text
argvus widget-telemetry block BLOCK ...   prints the text of one block (used by the bar)
argvus widget-telemetry toggle ...        switches the blocks on and off, and their order
```
`block` es la ruta rápida de la barra: se llama en cada actualización. `toggle` cambia qué bloques muestra la barra y desplaza la ventana de desplazamiento.

Los bloques son `header`, `keys`, `system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network` y `dev_dashboard`. La barra de tareas también usa las sondas `cpu-temp` y `gpu`. La sonda de GPU acepta `--present` (una GPU soportada es visible en DRM/sysfs), `--available` (una GPU y el comando btop están disponibles para el panel), `--available-temp` (se puede leer una temperatura atribuible) y `--temp` (la temperatura válida más alta, o `N/A`).

Las temperaturas de la GPU se corresponden con el dispositivo PCI de la GPU. Las lecturas propietarias de NVIDIA utilizan ID de bus `nvidia-smi`/NVML; las lecturas de NVIDIA nouveau, AMD e Intel utilizan hwmon solo cuando el enlace de dispositivo sysfs coincide de manera única con esa GPU. La temperatura de la barra de tareas es el valor máximo válido entre las GPUs detectadas. Las lecturas faltantes, inválidas o ambiguas permanecen no disponibles. La temperatura propietaria de NVIDIA requiere el proveedor del controlador; la temperatura de AMD e Intel depende del kernel que exponga un sensor hwmon vinculado. La temperatura de Intel no es suministrada por el panel de GPU de btop 1.4.7.

## Comenzando con la sesión

La unidad de usuario `argvus-widget-telemetry.service` se inicia con la sesión solo cuando el widget está habilitado. El interruptor principal es `effects.widget_telemetry_enabled` en la configuración, que está habilitado por defecto, por lo que una nueva cuenta inicia el widget al iniciar sesión.

Una vez que el interruptor ha escrito `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state`, ese archivo decide en su lugar. Solo una primera línea de `enabled` inicia el widget. Los archivos más antiguos `desktop-telemetry-state` y `sysinfo-state` todavía se leen cuando el archivo actual falta.

Las etiquetas de telemetría son compactas en los diez idiomas admitidos. Los identificadores técnicos como CPU, GPU, IP, DE, WM y Kernel mantienen su significado técnico. El tiempo de actividad se lee de `/proc/uptime` y se muestra con unidades abreviadas (por ejemplo, `1d 12h 49m`); ya no se expande en palabras completas desde la configuración regional del sistema. Los nombres de modelos de hardware y otros valores del proveedor todavía pueden afectar el ancho del widget.

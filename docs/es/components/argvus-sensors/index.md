---
title: Sensores de temperatura
description: Resumen de temperatura en vivo respaldado por JSON de lm_sensors.
---
`argvus sensors` abre el panel de Ratatui en proceso utilizado por el popup de temperatura de la CPU en la barra de tareas. Ejecuta `sensors -j`, agrupa las lecturas por los nombres de los chips y las etiquetas reportadas por el hardware, y se actualiza cada dos segundos. El monitoreo de la GPU lo maneja `argvus system-monitor --gpu`.

Las lecturas usan Celsius. El panel muestra `max`, `crit` y el estado de alarma cuando lm_sensors los proporciona. Se muestra un medidor solo cuando hay un límite positivo disponible. Las `lm_sensors` faltantes, los fallos de comando, el JSON inválido y las lecturas vacías permanecen visibles como estados recuperables; la sondeo se reintenta en el próximo ciclo.

Presiona `q` o `Esc` para cerrar. La barra de tareas abre el panel en Kitty a través de `argvus tui-terminal`; su título de pestaña inferior localizado muestra la sugerencia de cierre. Este comando posee los detalles de la temperatura de la CPU; el icono de temperatura de la GPU abre el monitor btop solo de GPU. El paquete de la barra de tareas ya depende de `lm_sensors`, por lo que no se necesita ningún paquete o dependencia separada.

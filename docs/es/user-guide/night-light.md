---
title: Luz nocturna
description: Ajusta la temperatura del color y los horarios locales fijos en Hyprland.
---
Abre **Centro de Control > Escritorio y Entrada > Pantallas > Luz Nocturna**, busca **Luz Nocturna** o ejecuta `argvus control-center displays night-light`.

La Luz Nocturna y su horario están apagados inicialmente. Elige una temperatura de **1000 a 20000 Kelvin** (por defecto **4500 K**). Los valores más bajos dan colores más cálidos. La página muestra la activación manual guardada, el estado efectivo y la siguiente transición. Presiona Enter para editar un valor; Esc cancela o regresa a Pantallas. Los valores no válidos se rechazan antes de guardar. Los cambios se guardan y aplican de inmediato. Si la aplicación falla después de guardar, la preferencia guardada permanece visible junto con el estado efectivo real y el error.

## Horario y acciones rápidas

El horario fijo usa la zona horaria local actual. El inicio está incluido y el fin está excluido; **20:00–07:00** cruza la medianoche. El inicio y el fin deben ser diferentes y usar `HH:MM`. No se utiliza ubicación ni cálculo solar.

Usa la tarjeta **Pantalla** del Panel de Control o **SUPER + Shift + B** para accesos rápidos. El selector de brillo mantiene todas sus acciones de brillo y agrega Luz Nocturna cuando el backend está disponible, incluso en una pantalla sin un backend de brillo. No hay un módulo permanente en la barra de tareas.

Con la programación desactivada, la activación manual es persistente. Con la programación activada, un acceso rápido crea una anulación temporal hasta la siguiente transición programada. Cambiar la programación o la temperatura elimina esa anulación. Cerrar sesión, una nueva instancia del compositor o un cambio en la siguiente transición (incluido un cambio de zona horaria) también la invalida.

## Comandos y estado

Todas las interfaces utilizan el mismo comando público:

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```
`status` devuelve el esquema JSON versión 1: `available`, `unavailable_reason`, `settings` (habilitado, temperatura, programación_habilitada, inicio, fin), `active`, `effective_temperature`, `mode` (`manual` o `scheduled`), `manual_override`, `next_transition` (ISO 8601 con desplazamiento), `minimum_temperature` y `maximum_temperature`. Los campos efectivos son nulos cuando no están disponibles; la temperatura efectiva es nula cuando la identidad está activa. Las fallas de disponibilidad se informan en el estado; las fallas de acción devuelven un valor distinto de cero con un diagnóstico.

La preferencia canónica es `/displays/night_light`, almacenada atómicamente por `argvus config` en `$XDG_CONFIG_HOME/argvus/config/displays.json`. Las anulaciones volátiles y el bloqueo de acción residen bajo `$XDG_RUNTIME_DIR/argvus/night-light/`. Los archivos de monitor generados son independientes. No edite los archivos generados para configurar esta función.

## Tiempo de ejecución

Requiere Hyprland y **hyprsunset >= 0.4.0**. La sesión inicia la `hyprsunset.service` ascendente una vez, aplica el estado guardado a través de `argvus-night-light-init.service` y evalúa los horarios cada minuto mediante `argvus-night-light.timer` y `argvus-night-light.service`. Los cambios de reloj y zona horaria también disparan la evaluación. Con la programación desactivada, la operación periódica deja el estado manual intacto. Las aplicaciones repetidas consultan la identidad/temperatura efectiva y omiten escrituras sin cambios. No se necesita recargar el compositor.

ARGVUS envía un drop-in de systemd para el servicio upstream: comienza con `--identity` y aísla el descubrimiento de su configuración a `/usr/share/argvus/session/config/night-light`. Esto evita perfiles nativos en competencia y deja intactos los archivos personales `hyprsunset.conf`. El drop-in se aplica a ese servicio upstream mientras el paquete de sesión ARGVUS esté instalado, incluyendo los inicios fuera de ARGVUS. Las anulaciones de systemd existentes del usuario pueden tener prioridad; inspeccione `systemctl --user cat hyprsunset.service` al diagnosticar esa situación. Hyprsunset 0.4.0 rechaza el argumento de ruta separado de `--config`, por lo que el descubrimiento de configuración se aísla con variables XDG locales del servicio.

El backend utiliza el IPC oficial de Hyprsunset (`temperature`, `identity`, `identity get`), con E/S limitada y lectura después de los cambios. No se utiliza ningún shader, backend de compositor alternativo, escritura de configuración del sistema ni escalamiento de privilegios. Los controles no disponibles están deshabilitados en el Centro de Control y ocultos en la tarjeta de Pantalla.

---
title: Reloj ARGVUS
description: Reloj, relojes mundiales, cronómetro, temporizador de cuenta regresiva y alarmas con sonido.
---
ARGVUS Clock es una aplicación GTK 4 con la hora actual y las herramientas de tiempo: relojes mundiales, un cronómetro, un temporizador de cuenta regresiva y alarmas. Ábrela desde el lanzador, haciendo clic en la hora en la barra de tareas, con **SUPER + Shift + C**, o con:

```sh
argvus clock                     # open the popup near the pointer
argvus clock --toggle            # hide the popup if it is visible, show it otherwise
argvus clock --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus clock --page alarms       # open on a page: clock, world, stopwatch, timer, alarms
argvus clock --background        # run without a window, to keep alarms active
```
El reloj ARGVUS se abre como una ventana emergente alineada con la barra de tareas, con un espacio de 4 píxeles. La barra de tareas pasa las coordenadas del clic raíz, de modo que la ventana emergente usa el monitor clicado; iniciarla sin coordenadas usa la posición del puntero. Si el puntero permanece fuera de la ventana emergente durante 5 segundos, esta se cierra automáticamente; mover el puntero dentro de la ventana cancelará la cuenta regresiva. **Esc** en una pestaña principal todavía la oculta inmediatamente, y **Esc** en una página completa vuelve a la pestaña anterior. `argvus --clock` es un alias. Hacer clic en la fecha en la barra de tareas aún abre el calendario; Reloj ARGVUS no muestra eventos.

ARGVUS Focus, Reloj ARGVUS y el Panel de Control comparten el mismo borde de pantalla. Abrir Reloj cierra primero Focus y el Panel de Control; abrir cualquiera de esas superficies cierra Reloj.

## Páginas

- **Reloj**: la hora en dígitos grandes (24 horas por defecto, 12 horas en la configuración, segundos opcionales), la fecha localizada, la zona horaria local y su desfase UTC, y la próxima alarma.
- **Mundo**: ciudades agregadas buscando una ciudad o zona horaria IANA (los acentos y las mayúsculas se ignoran). Cada una muestra su hora, la diferencia con la hora local y si allí es ayer, hoy o mañana. El horario de verano se obtiene de la base de datos de zonas horarias IANA; nada se calcula con desfases fijos. Las ciudades se pueden reordenar y eliminar.
- **Cronómetro**: iniciar, pausar, reanudar, reiniciar y vueltas (tiempo de vuelta y total).
- **Temporizador**: horas, minutos y segundos, o los preajustes de 5, 10, 15, 25, 30 y 60 minutos. Pausar, reanudar, reiniciar y cancelar. Cuando termina suena como una alarma. Este temporizador es independiente de las sesiones de ARGVUS Focus.
- **Alarmas**: crear, editar, activar y desactivar, y eliminar alarmas. Cada una tiene una hora, un nombre opcional, los días de la semana en que se repite (ninguno significa que suena una sola vez), un sonido y una duración de posposición.

Teclado: **Ctrl+1** a **Ctrl+5** cambian de página, **Ctrl+N** crea una alarma, **Ctrl+,** abre la configuración, **Esc** regresa, **Ctrl+W** cierra la ventana y **Ctrl+Q** sale.

## Sonido de alarma

Las alarmas reproducen un sonido real hasta que las atiendes:

- Sonidos: **Chime** (predeterminado) y **Beacon**, incluidos en `/usr/share/argvus/sounds/`. 
- **Volumen de la alarma** (10 a 100, por defecto 80) y **Escuchar el sonido de la alarma** están en los ajustes; el editor tiene un botón de prueba para el sonido elegido. 
- Una alarma que suena muestra una tarjeta en la parte superior de la pantalla, sobre otras ventanas, y una notificación, ambas con **Posponer** y **Descartar**. Con la tarjeta seleccionada, **Enter** descarta y **S** pospone. 
- Posponer hace que la alarma suene de nuevo tras la duración de posponer de la alarma (1 a 60 minutos, por defecto 10). 
- Una alarma que no se atiende se detiene después de **Detener el timbre después de** (5, 10, 15 o 30 minutos, por defecto 10) y deja una notificación. 
- Dos alarmas sonando juntas se listan ambas; la segunda nunca silencia a la primera. 
- Si el sonido no se puede reproducir, la tarjeta y la ventana lo indican con el error, y el error se registra. ARGVUS Clock nunca informa un sonido que no se haya reproducido.

La reproducción utiliza el soporte de medios de GTK (GStreamer), por lo que funciona con PipeWire, PulseAudio o ALSA. El paquete depende de `gst-plugins-base` (Ogg Vorbis) y `gst-plugins-good` (salida de audio).

## Cuando suenan las alarmas

Las alarmas suenan solo mientras ARGVUS Clock se ejecuta en tu sesión:

- Cerrar la ventana mantiene ARGVUS Clock en ejecución mientras sea necesario para alarmas, un temporizador en marcha o un cronómetro en funcionamiento (**Mantener en ejecución cuando la ventana esté cerrada**, activado por defecto). Si no hay nada pendiente, se cierra.
- **Iniciar con la sesión** habilita la unidad de usuario `argvus-clock.service`, que ejecuta `argvus clock --background` después del inicio de sesión. Está desactivado por defecto.
- Las alarmas no pueden sonar mientras hayas cerrado sesión, la computadora esté apagada o suspendida.
- Una alarma notada hasta 5 minutos tarde aún suena. Más tarde que eso (el reloj no estaba funcionando, o el sistema estaba en suspensión o apagado) no suena: se informa como **perdida** en una notificación y en la página de Alarmas. Una alarma repetitiva pasa a su siguiente día; una alarma única se apaga. 
- Cada ocurrencia se guarda como manejada antes de que suene, por lo que al reabrir ARGVUS Clock la misma alarma nunca suena dos veces, y solo se ejecuta una instancia por sesión. 
- Las alarmas repetitivas se calculan en la hora local en los días de la semana seleccionados, por lo que mantienen su hora a través de cambios de horario de verano. Una hora saltada por el cambio de primavera suena en el primer minuto después del hueco; una hora que ocurre dos veces suena la primera vez. 
- Un temporizador sigue contando durante una suspensión. Si terminó mientras el sistema estaba en suspensión, suena al despertar y dice cuán tarde está. Después de cerrar ARGVUS Clock, un temporizador o cronómetro en funcionamiento vuelve pausado.

## Rutas

| Ruta | Contenido |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/clock/settings.json` | Configuraciones y relojes mundiales |
| `$XDG_DATA_HOME/argvus/clock/alarms.json` | Alarmas |
| `$XDG_STATE_HOME/argvus/clock/timers.json` | Temporizador y punto de control del cronómetro |
| `/usr/lib/systemd/user/argvus-clock.service` | Iniciar con la sesión |
| `/usr/share/argvus/sounds/` | Sonidos empaquetados |

Los archivos se escriben de manera atómica; un archivo que no se puede leer se renombra a `<name>.corrupt-<time>` y se utilizan valores predeterminados. Los colores siguen el tema activo de ARGVUS y el color de resaltado (consumidor del tema `time`). Las cadenas provienen de los dominios de traducción `clock` y `time`.

El calendario de la barra de tareas, ARGVUS Focus, ARGVUS Clock y el Panel de Control son mutuamente excluyentes: abrir uno cierra los demás.

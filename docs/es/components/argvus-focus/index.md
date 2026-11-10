---
title: Enfoque ARGVUS
description: Pomodoro y sesiones de concentración con pausas, notificaciones y un historial de sesiones local.
---
ARGVUS Focus es una aplicación GTK 4 para trabajar en sesiones de concentración separadas por descansos. Se abre como un popup junto a la barra de tareas. Ábrela desde el lanzador, haciendo clic en su indicador de la barra de tareas, con **SUPER + Shift + F**, o con:

```sh
argvus focus              # open the popup near the pointer
argvus focus --toggle     # hide the popup if it is visible, show it otherwise
argvus focus --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus focus --start      # start or resume the current phase without opening the window
argvus focus --pause      # pause the running phase
argvus focus status --waybar         # print one Waybar JSON status and exit
argvus focus status --waybar --follow  # stream status changes for Waybar
```
La barra de tareas pasa las coordenadas del clic raíz para que el popup use el monitor en el que se hizo clic y se alinee con la barra; un lanzamiento sin coordenadas usa la posición del puntero. Si el puntero permanece fuera del popup durante 5 segundos, este se cierra automáticamente; mover el puntero dentro del popup cancela esa cuenta regresiva. **Esc** en la página principal todavía lo oculta inmediatamente, y **Esc** en una subpágina regresa a la página anterior. El indicador de la barra de tareas muestra solo su ícono mientras está inactivo y el `MM:SS` restante durante una fase, con el color de la fase; una fase pausada mantiene su tiempo restante y añade un estado de pausa. El comando `status` se ejecuta dentro de `argvus` sin iniciar GTK. `argvus --focus` es un alias. Solo hay una instancia de Focus por sesión: un segundo lanzamiento entrega sus opciones a la instancia en ejecución.

ARGVUS Focus, ARGVUS Clock y el Panel de Control comparten el mismo borde de la pantalla. Abrir Focus cierra primero Clock y el Panel de Control; abrir cualquiera de esas superficies cierra Focus.

## Sesiones

- **Modo Pomodoro** alterna concentración y descansos cortos y termina cada ciclo con un descanso largo. Las duraciones predeterminadas son 25, 5 y 15 minutos, con un descanso largo después de 4 sesiones de concentración.
- **Modo de sesión única** ejecuta una sesión de concentración de su propia duración (45 minutos por defecto) sin descansos.
- La ventana muestra la fase (concentración, descanso corto, descanso largo, cada una en su propio color), el tiempo restante dentro de un anillo de progreso, la posición en el ciclo y el total de hoy.
- **Iniciar concentración / Iniciar descanso**, **Pausar**, **Reanudar**, **Cancelar sesión** (abandona la fase y no registra nada) y **Completar ahora** (termina la sesión de concentración temprano y registra el tiempo realmente empleado). Durante un descanso, **Saltar descanso** vuelve a la concentración.
- Cuando una fase termina, la siguiente espera por ti, a menos que **Iniciar descansos automáticamente** o **Iniciar concentración automáticamente después de un descanso** esté activado.

Teclado: **Espacio** inicia o pausa, **Ctrl+,** abre la configuración, **Ctrl+H** el historial, **Esc** regresa, **Ctrl+W** oculta la ventana y **Ctrl+Q** sale.

## Configuración

Preajustes (Clásico 25/5/15, Extendido 50/10/30, Trabajo profundo 90/20/30) rellenan las duraciones, las cuales permanecen editables. La configuración también incluye las sesiones antes de un descanso largo, los inicios automáticos, la notificación al final de una fase, y los **sonidos de la sesión**: una campanita corta con su propio volumen y un botón de **Probar sonido**. Estos sonidos son independientes de los sonidos de alarma de [ARGVUS Clock](/docs/components/argvus-clock/)].

## Historial

Cada sesión de enfoque completada se registra con su inicio, final y el tiempo empleado; los descansos y sesiones canceladas no se registran. La página de historial muestra las sesiones y minutos de hoy, los totales y las sesiones recientes. Se eliminan los registros de más de 365 días. **Borrar historial** solicita confirmación.

## Ventana, fondo y suspensión

- Cerrar la ventana durante un ciclo solo la oculta: la sesión continúa y te notifica cuando termina la fase. Sin nada en curso, cerrar termina el programa.
- Si el sistema se suspende mientras una fase está en marcha, la fase se pausa en el momento de la suspensión, y un mensaje indica cuánto tiempo estuvo el sistema en reposo. El tiempo de suspensión no se cuenta como tiempo de enfoque.
- La sesión se guarda en cada cambio y cada 30 segundos. Si ARGVUS Focus se cierra durante una fase (cierre de sesión, fallo, **Ctrl+Q**), vuelve pausada en ese punto la próxima vez que se abra; nunca finge que el temporizador siguió funcionando mientras no podía notificar.
- Los tiempos se miden en el reloj de arranque del sistema, por lo que cambiar el reloj de pared o la zona horaria no acorta ni alarga una fase.

## Rutas

| Ruta | Contenido |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/focus/settings.json` | Configuración |
| `$XDG_DATA_HOME/argvus/focus/history.json` | Sesiones completadas |
| `$XDG_STATE_HOME/argvus/focus/session.json` | Punto de control de la sesión actual |
| `/usr/share/argvus/sounds/argvus-bell.oga` | Timbre de la sesión |

Los archivos se escriben de manera atómica. Un archivo que no se puede leer se renombra a `<name>.corrupt-<time>` y se utilizan los valores predeterminados; un mensaje indica a dónde fue.

Los colores siguen el tema activo de ARGVUS y el color de resaltado (consumidor de tema `time`). Las cadenas provienen de los dominios de traducción `focus` y `time`.

El calendario de la barra de tareas, ARGVUS Focus, ARGVUS Clock y el Panel de control son mutuamente excluyentes: abrir uno cierra los otros.

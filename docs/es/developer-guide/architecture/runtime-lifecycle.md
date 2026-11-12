---
title: Ciclo de vida de ejecución
description: Inicio de sesión y propiedad del servicio de usuario.
---
`argvus-session` importa el entorno, inicia Hyprland a través de `argvus start` y arranca `argvus-session.target` después de que el compositor esté listo. El objetivo incluye servicios de shell, notificaciones, fondo de pantalla, inactividad y portapapeles. `argvus sessionctl` coordina iniciar, detener, recargar, reiniciar, estado y registros.

La sesión detiene el objetivo cuando Hyprland finaliza. Los proyectos individuales siguen siendo responsables de sus propios binarios y configuración.

## Inicio

El punto de entrada gráfico es `/usr/share/wayland-sessions/argvus.desktop`, que invoca `argvus session`. La sesión importa el entorno de configuración ARGVUS y XDG, lo sincroniza con `systemd --user` y la activación de D-Bus, y delega el inicio del compositor a `argvus start`.

`argvus start` valida la configuración de Hyprland, aplica la configuración de respaldo empaquetada cuando se encuentra un archivo generado por defecto o cuando la configuración de un usuario es inválida, detecta la compatibilidad con virtualización y lanza Hyprland. El puente de preparación del compositor luego llama a `argvus sessionctl ready`, que inicia el objetivo ARGVUS con el entorno actual de Wayland y Hyprland.

El objetivo utiliza una unidad de preparación de disparo único antes de iniciar los componentes de larga duración. La preparación aplica la configuración generada, el estado del tema y los acentos, el estado del monitor, la integración de aplicaciones y el entorno de la sesión. Los servicios persistentes son propiedad de `systemd --user`, no de procesos de fondo ad-hoc.

`argvus start` pone en cola `argvus-config.service` (`--no-block`) justo antes de lanzar Hyprland: la proyección no necesita compositor, así que se superpone al inicio del compositor. `argvus sessionctl ready` luego inicia el objetivo, que encuentra esa unidad activa o espera su trabajo, por lo que la configuración canónica todavía se proyecta antes de que se ejecute la preparación. Una vez que el objetivo ha comenzado, `sessionctl ready` espera hasta 1,5 segundos para que se mapeen el fondo de pantalla (una superficie de capa de fondo) y la barra de tareas antes de eliminar la superposición de carga, porque un servicio simple se considera iniciado antes de que dibuje; una superficie cuya unidad no está en ejecución no se espera, y la espera se registra como `session.desktop-surfaces`, con `session.wallpaper-surface` y `session.taskbar-surface` para el tiempo que tomó cada superficie. Un fallo fatal de proyección o de inicio del objetivo se registra y detiene la superposición de carga independiente, evitando un aparentemente interminable indicador de carga. Opcional Los consumidores visuales deben resolver una alternativa o informar de un fallo degradado sin reemplazar el objetivo de la sesión.

### Costo de inicio de sesión

Un inicio de sesión sin cambios de configuración no repite trabajo costoso:

- La proyección ejecuta el cambio de tema y el adaptador de efectos solo cuando las secciones de apariencia o efectos han cambiado, o cuando sus salidas reales (las copias de usuario materializadas y `data/state/effects/<theme>.conf`) están ausentes.
- La preparación aplica el tema de inicio y el acento encima de él, solo cuando el sello en `$XDG_CACHE_HOME/argvus/session/theme-startup.stamp` ya no coincide con los archivos de tema empaquetados y el marcador de acento. El sello obtiene la huella digital del contenido de los archivos, por lo que reinstalar archivos idénticos no lo invalida. Los cambios en las entradas del tema o un nuevo acento lo invalidan una vez. Migrar un sello basado en marca de tiempo más antiguo también lo reconstruye una vez. Cuando el tema se reaplica, la preparación también recarga Hyprland, que había iniciado con los archivos anteriores.
- La configuración de GTK (`gsettings`) se escribe solo cuando se vuelve a aplicar el tema de inicio. Entre inicios de sesión, dconf las conserva, por lo que un inicio de sesión estable no las ejecuta nuevamente.
- `argvus-dunst.service` es `Type=dbus`: se considera iniciado una vez que Dunst posee `org.freedesktop.Notifications`, por lo que el objetivo de la sesión no espera a que se restaure el estado de pausa.
- La preparación ejecuta `spaces-switch.sh` y `borders-switch.sh` (`--apply-static`) en paralelo: escriben archivos independientes (los márgenes JSONC y la geometría efectiva de las barras, y el CSS de las barras y el tema de Rofi).
- La proyección reescribe un archivo generado solo cuando su contenido cambia. Los archivos sin cambios se dejan sin modificar, por lo que un inicio de sesión estable no los sincroniza de nuevo.
- El paso de visualización de la preparación (`argvus config display session-prepare`) se aplica solo a DPMS y salidas deshabilitadas: no vuelve a proyectar ni recarga el compositor.
- `argvus-session-reload.service` ejecuta `argvus sessionctl reload-desktop --login`, que recarga Hyprland solo cuando el manifiesto de proyección lista a `hyprland` entre los consumidores afectados. Los componentes del escritorio no esperan a esto.

El spinner de sesión (el ayudante de transferencia en modo Wayland) se asigna desde la transferencia del compositor hasta que `argvus sessionctl ready` haya iniciado el objetivo, por lo que un solo spinner cubre todo el inicio. No tiene `PartOf=`, porque recrear el objetivo en un nuevo inicio de sesión no debe detenerlo; `sessionctl` lo detiene explícitamente, y `RuntimeMaxSec=30` lo finaliza si no llega una detención.

Cada paso de preparación, recarga y adaptador de proyección añade su duración al registro de la sesión como `<epoch-ms> timing <step> <N>ms`; ver [solución de problemas del inicio de sesión](/docs/user-guide/troubleshooting/session-startup/).

La resolución de rutas, la preparación de la sesión y el plan de recarga de Hyprland son propiedad de Rust. `argvus config session-prepare` y `argvus config session-reload-config` ejecutan los pasos de inicio y recarga; el script heredado `hypr-init.sh` ha sido eliminado.

## Servicios y apagado

El objetivo comúnmente posee `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service` cuando está habilitado, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, los observadores del portapapeles, las notificaciones de distribución del teclado y la integración de PolicyKit. La superposición de carga de la sesión es iniciada por `argvus start`, no por el objetivo. Un fallo en un servicio opcional no reemplaza toda la sesión.

Los servicios que necesitan una conexión de cliente Wayland, incluyendo el Panel de Control y el gestor de inactividad, esperan a que haya un runtime de Hyprland utilizable antes de lanzar su cliente. Esto evita iniciar Quickshell o hypridle contra un socket obsoleto durante la transferencia del compositor; si Hyprland es reemplazado, systemd puede iniciar el componente nuevamente después de que el nuevo runtime esté disponible.

Un cambio de tema serializa su commit, construcción, proyección y recargas de consumidores como una sola operación de ciclo de vida bajo el bloqueo del tema. No recarga `argvus-config.service` ni reinicia servicios: la proyección que ejecuta solo escribe archivos y deja el tema para el cambio mientras se mantiene el bloqueo, y los consumidores en ejecución se recargan en su lugar (ver [Apariencia y temas](/docs/developer-guide/subsystems/appearance-and-themes/)).

Cuando Hyprland se cierra, `argvus start` detiene `argvus-session.target` y los servicios vinculados a él. Esto evita que los procesos de la barra de tareas, shell, notificaciones, inactividad y portapapeles permanezcan después de cerrar sesión.

## Recargando

```sh
argvus sessionctl reload
```

La recarga vuelve a importar el entorno gráfico y, a continuación, **aplica** una generación confirmada cuando un comando de dominio previo —como `apply-theme`, `accent`, `set`/`patch` o un bloque de telemetría de widgets— ya ha proyectado el cambio. En ese caso, restaura los consumidores incluidos en el plan del manifiesto y no vuelve a ejecutar `argvus config project`, ya que la nueva proyección reconstruiría el mismo árbol y descartaría el plan. La proyección, la migración y `ensure` solo se ejecutan cuando no hay un plan pendiente válido: al iniciar sesión o después de que `argvus config recover` tenga que restaurar el documento anterior. Solo se considera que no hay cambios cuando tanto las proyecciones como el entorno de ejecución gestionado están actualizados. Si el estado canónico no ha cambiado, pero un socket del compositor, un servicio o un consumidor está obsoleto, la recarga lo reconcilia en lugar de finalizar antes. Según el manifiesto de proyección, la propagación habitual incluye Hyprland, la barra de tareas, la telemetría de widgets, el Panel de control, las notificaciones, el fondo de pantalla y el modo inactivo; los consumidores opcionales que no hayan cambiado permanecen intactos. `argvus-config` escribe en este punto todos los archivos de `data/generated/`, por lo que la sesión no vuelve a generar temas por su cuenta. Después, la recarga ejecuta `theme-adapters.sh` para reconciliar los consumidores que no pueden leer por sí mismos el documento canónico o la proyección publicada. Estos adaptadores solo leen la generación confirmada: escriben sus propios archivos y cachés, pero nunca el documento canónico ni `data/generated/`. Así, una única acción del usuario sigue implicando una modificación, una proyección, una publicación y una recarga. Use `argvus sessionctl restart <component>` solo cuando sea necesario reiniciar explícitamente un servicio de forma aislada. Consulte la [referencia de servicios](/docs/reference/services/) para ver los nombres exactos de las unidades.

Las operaciones de proyección y recarga se serializan con un bloqueo de tiempo de ejecución de usuario. Para un rastro de tiempo temporal, configure `ARGVUS_SESSION_PERF=1` antes de invocar la recarga e inspeccione `~/.local/state/argvus/session.log`; el rastro registra los límites de las etapas sin cambiar el comportamiento normal de la recarga.

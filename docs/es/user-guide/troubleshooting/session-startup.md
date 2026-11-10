---
title: Solución de problemas de inicio de sesión
description: Diagnosticar fallos en el saludo, Hyprland y la sesión de usuario de systemd.
---
Correr:

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```
Verifica por separado el greeter, `argvus start`, Hyprland y la unidad de usuario correspondiente. La compilación exitosa del paquete no demuestra que la carga instalada o la anulación activa del usuario esté actualizada.

Usa las verificaciones en este orden:

1. Confirma que el greeter está usando la configuración ARGVUS greetd instalada.
2. Lee el registro de la sesión y busca una configuración de Lua de Hyprland faltante o inválida.
3. Verifica la preparación y el estado objetivo con `systemctl --user status argvus-session.target`.
4. Inspecciona el componente que falla en lugar de reiniciar todos los servicios a la vez.

Si `argvus-session.target` está inactivo mientras la superposición de carga está activa, inspecciona el error de proyección antes de reiniciar los servicios:

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```
`argvus-config` es el único escritor por debajo de `data/generated/`, así que `project` es la forma autorizada de reproducir una generación mientras se realiza el diagnóstico. El inicio de la sesión lo ejecuta porque aún no existe un plan comprometido; el plan que escribe en `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` es lo que un posterior `argvus sessionctl reload` aplica, sin proyectar nuevamente.

La sesión detiene la superposición de carga cuando la preparación fatal o el inicio del objetivo fallan. Un fallo de consumidor visual debería usar su alternativa documentada o ser reportado explícitamente; no debe ocultarse como una sesión exitosa.

## Inicio de sesión lento

Si `greeter.handoff-takeover-failed` aparece sin `handoff.master-acquired`, inspeccione `handoff.master-unavailable card=<name> errno=<n>` por el error de adquisición. Un observador abierto mientras otro compositor era maestro puede devolver errno 13 (`EACCES`) incluso después de que ese compositor libere el maestro. El ayudante vuelve a abrir la misma GPU para adquirir el maestro implícitamente; `handoff.master-reopened` registra esta recuperación y se prepara un nuevo framebuffer en ese descriptor. El recepcionista cancela el ayudante fallido (`greeter.handoff-cancelled`), por lo que no puede añadir otra espera antes de que el compositor del usuario comience. Una sesión que envía `release` mientras la adquisición está pendiente recibe `handoff.release-before-master`: el ayudante cierra sus descriptores DRM antes de reconocer. Un bloqueo del compositor recepcionista en Hyprland/Aquamarine es un hallazgo aparte; cancelar al ayudante no establece que el problema del controlador o del compositor se haya solucionado.

`greeter.vt-switch-returned active=<tty>` registra el VT activo cuando la llamada switch de logind retorna. Un tiempo de espera de dibujo también registra `active-vt=<tty>` en su error. Estas son instantáneas, no pruebas de una transferencia de DRM completada.

Para una comparación antes/después, guarde el trabajo y use cinco inicios de sesión por versión en la misma máquina, con la misma configuración de tema y monitor. Antes de cada cierre de sesión siguiente, copie `/run/argvus-greeter/timing.log`, el registro de la sesión y el diario de arranque a un directorio de muestras separado bajo `/tmp/argvus/`; la siguiente ejecución del gestor de inicio reemplaza su registro de tiempos. Registre el ID de arranque con cada muestra y compare las marcas de tiempo únicamente dentro de ese arranque. Mida la autenticación hasta la transferencia del primer cuadro, la entrada del envoltorio de sesión, el lanzamiento final de Hyprland y la detención del indicador de carga. Compare la mediana y el peor tiempo de autenticación hasta la detención del indicador y verifique visualmente si hay cuadros negros, destellos o un indicador atascado. La duración solo de las superficies de escritorio no es el tiempo de inicio de sesión de extremo a extremo. Repita las pruebas de fallo y recuperación en una sesión desechable, sin terminar una sesión que contenga trabajo no guardado.

Cada paso de inicio registra su duración en el registro de la sesión. Para ver los pasos del último inicio de sesión, de más lento a más rápido:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```
Para cronometrar el cambio del saludo al escritorio, lea las marcas del saludo y de transferencia junto con el diario. Todos ellos usan `CLOCK_MONOTONIC`, por lo que se alinean:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log` conserva solo el último inicio de sesión: el saludaor, la superficie de transferencia de DRM (`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`) y marcas `greeter.*`. Las líneas `handoff.crtc` registran cada cambio de CRTC mientras la transferencia espera al maestro de DRM; una línea con `fb=0` o `mode_valid=false`, o `handoff.output-found ... framebuffer=0`, significa que el compositor del saludaor apagó la salida antes de que la transferencia la tomara, por lo que el monitor tuvo que sincronizarse de nuevo. `greeter.watchdog-stop-requested` significa que la sesión del saludaor terminó sin esperar a su compositor, y `greeter.compositor-exit-deferred` que se le pide al compositor que salga una vez que la superficie de transferencia desaparezca; `greeter.compositor-exit-requested` por sí solo significa que se le pidió salir de inmediato. Si las líneas de greetd y logind en el diario muestran un intervalo de aproximadamente medio segundo entre el cierre de la sesión del saludaor y la sesión del usuario apertura, la desinstalación del compositor del saludo se ejecutó durante el inicio de sesión. Eso es esperado mientras la marca muestre `compositor-in-foreground=true`: la desconexión del terminal del saludo termina el compositor antes de la salida diferida. Eso ocurre cuando el saludo no pudo abandonar su VT antes de que su compositor saliera: busca `greeter.vt-switch-requested` seguido de `greeter.handoff-drawing`; cuando falta el segundo, `greeter.handoff-takeover-failed error=<reason>` dice por qué. `handoff.page-flip` significa que la transferencia reemplazó el último cuadro del saludo sin un cambio de modo; `handoff.page-flip-failed errno=<n>` seguido de `handoff.modeset` significa que el kernel rechazó el cambio de página. `handoff.overlay plane=<id>` y luego `handoff.retired` significan que la superposición ocultó los primeros cuadros de Hyprland hasta que el indicador de sesión apareció en pantalla; `handoff.overlay-unavailable` significa que el CRTC no tenía plano de superposición libre, y `handoff.retire-timeout` significa que el indicador de sesión nunca se envió `retire`. El spinner de sesión (`argvus-greeter-handoff`) escribe `first layer-shell frame callback` y `stop requested` en el diario.

Se espera una entrada `project.theme-switch` o `prepare.startup-theme` distinta de cero después de un cambio de apariencia, archivos de tema modificados, un nuevo acento o la migración de un sello de inicio anterior. Reinstalar archivos idénticos preserva la caché de inicio basada en contenido. Si aparece en cada inicio de sesión, los archivos de tema materializados están ausentes; ejecute `argvus theme-switch <theme>` una vez.

El registro de sesión se mantiene bajo `$XDG_STATE_HOME/argvus/session.log` (normalmente `~/.local/state/argvus/session.log`). Después de cambiar una anulación de usuario Hyprland, cierre sesión y vuelva a iniciarla para que el punto de entrada de la sesión se valide y se seleccione nuevamente. Consulte [Sesión gráfica](/docs/argvus-session/graphical-session/) para el flujo normal de inicio.

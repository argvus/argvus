---
title: Capa de carga
description: La superposición que se muestra mientras la sesión se recarga directamente.
---
La superposición de carga es una superficie layer-shell de GTK 4 que se muestra mientras ARGVUS recarga la sesión directamente (`argvus sessionctl reload`). Un cambio de tema no la utiliza: el escritorio cambia en el lugar. Es un asistente incluido en el paquete `argvus` en `/usr/lib/argvus/loading-theme/splash`, y no se coloca en `/usr/bin`.

El inicio de sesión no lo utiliza: el splash del saludador y el spinner de inicio de sesión son el modo Wayland de
el asistente de entrega, que dibuja el mismo spinner sin GTK y muestra su primer fotograma dentro
milisegundos (véase [Recibidor, bloqueo y salpicadura](/docs/developer-guide/subsystems/greeter-lock-and-splash/)). 
Con `--spinner-only` la superposición solo muestra ese spinner y sale en `SIGTERM`. Con `--detached` no sale cuando el proceso que lo inició sale; una recarga de sesión usa ambas banderas y la detiene a través de su archivo PID. 
Lee el tema seleccionado del directorio de datos ARGVUS y vuelve al ARGVUS empaquetado
Paleta oscura. Informa de preparación solo después del primer fotograma de cada superficie que mapea, no cuando
El proceso comienza.

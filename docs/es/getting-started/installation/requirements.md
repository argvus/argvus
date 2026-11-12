---
title: Requisitos
description: Requisitos del sistema y prerrequisitos para ARGVUS.
---
ARGVUS apunta a Arch Linux y requiere un stack gráfico compatible con Wayland, Hyprland y las dependencias de tiempo de ejecución que instalan los paquetes. Un gestor de pantalla es opcional: ARGVUS soporta un flujo de inicio gráfico a través de greetd y un flujo manual por TTY.

Antes de la instalación, asegúrese de que:

- el sistema esté completamente actualizado con `pacman`;
- un kernel y un controlador gráfico funcionando estén instalados;
- el usuario pueda ejecutar una sesión de Wayland;
- la red esté disponible cuando se configure el repositorio de paquetes.

Las máquinas virtuales pueden necesitar los ajustes de compatibilidad descritos en [Virtual machines](/docs/user-guide/virtual-machines/).

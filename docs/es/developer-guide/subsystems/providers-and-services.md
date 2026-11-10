---
title: Proveedores y servicios
description: Paquetes de proveedores y sus interfaces de sistema.
---
Los proyectos de proveedores exponen interfaces enfocadas a los consumidores de la interfaz de usuario:

- `argvus network` y `argvus bluetooth` integran los ayudantes de NetworkManager y Bluetooth;
- `argvus-display` integra el estado del monitor con Hyprland;
- `argvus-power` integra las acciones de inactividad, bloqueo y energía;
- `argvus-firewall` gestiona `argvus-firewall.service`;
- el ayudante `argvus removable-devices` en `argvus` consume UDisks2;
- el ayudante `argvus control-center` en `argvus` consume las APIs del proveedor y las interfaces del sistema D-Bus.

El Centro de Control es un cliente de estos proveedores, no su reemplazo.

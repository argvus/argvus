---
title: Empaque
description: Construir e inspeccionar paquetes ARGVUS Arch.
---
Cada componente posee su propio empaquetado Arch bajo su repositorio. Inspeccione `packaging/archlinux/`, las funciones del paquete y la lista de archivos instalados antes de documentar una carga útil. Los valores predeterminados del paquete bajo `/usr/share/argvus` y la configuración del administrador bajo `/etc/argvus` no son intercambiables.

Para la verificación en tiempo de ejecución, compare el contenido del paquete con `pacman -Ql <package>` e inspeccione las anulaciones de usuario activas por separado.

Las compilaciones locales mantienen los objetos de Cargo en `build/cargo-target`, fuera del árbol de origen del paquete, por lo que una `make build` repetida recompila solo lo que cambió. `make clean` elimina ese directorio, y la siguiente compilación comienza desde cero.

No infiera la carga útil en tiempo de ejecución únicamente por el nombre del repositorio. Un paquete puede contener binarios, servicios de usuario, valores predeterminados `/usr/share/argvus`, configuración de administrador `/etc/argvus` o recursos. Confirme la carga útil real con los `PKGBUILD` y `pacman -Ql` del repositorio después de la instalación.

---
title: Sesión y systemd
description: El objetivo de la sesión ARGVUS y sus servicios de usuario.
---
Las definiciones de unidad están empaquetadas por `argvus-session` bajo `/usr/share/argvus/session/config/systemd/user/`. El administrador de sesiones posee el ciclo de vida y la importación del entorno. Los repositorios de componentes poseen los comandos ejecutados por sus unidades.

Use `argvus sessionctl` en lugar de iniciar manualmente un componente al diagnosticar la sesión integrada.

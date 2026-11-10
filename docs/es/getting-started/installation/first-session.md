---
title: Primera sesión
description: Inicie ARGVUS después de la instalación.
---
Para un inicio de sesión gráfico, configure la integración empaquetada de greetd con `argvus greeter-setup --help` y use el flujo del gestor de pantalla descrito en [Sessions](/docs/user-guide/sessions/).

Para una sesión TTY, inicie sesión en un terminal virtual y ejecute:

```sh
argvus --start-desktop
```
Después de iniciar sesión, use `argvus sessionctl status` para inspeccionar el objetivo de usuario ARGVUS. Si el compositor o el shell no se inicia, siga [solución de problemas de sesión](/docs/user-guide/troubleshooting/session-startup/).

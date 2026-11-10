---
title: Recepcionista
description: El saludador de inicio de sesión que greetd inicia, se envió en el paquete argvus.
---
`argvus-greeter` es el saludo de inicio de sesión para greetd. Es un frontend de terminal centrado en el teclado que descubre las sesiones de Wayland instaladas, muestra las cuentas con sus avatares y aplica el tema ARGVUS activo para la cuenta seleccionada.

Es parte del paquete `argvus`, el único paquete del escritorio. Su código fuente se encuentra en `crates/argvus-greeter` del repositorio `argvus`, como miembro del espacio de trabajo raíz de Cargo, y se construye junto con el resto de `argvus`.

El saludo no autentica a los usuarios por sí mismo. greetd maneja la autenticación a través de PAM. El lanzador abre el saludo con `argvus tui-terminal`, por lo que el saludo no necesita un programa de terminal separado.

Los archivos de configuración del saludo se instalan bajo `/etc/argvus`. Usa `argvus greeter-setup --help` para ver las opciones de configuración que el paquete proporciona. Para el flujo de inicio de sesión, consulta [Sessions](/docs/user-guide/sessions/)].

El socket de transferencia de inicio de sesión autoriza a cada cliente con credenciales de par del kernel. Solo root, el propietario del asistente y la cuenta autenticada por greetd pueden enviar comandos de control. A otras cuentas locales se les niega el acceso aunque el socket permita conexiones entre usuarios. Un marco DRM preparado permanece vinculado a su descriptor original; el asistente espera ese descriptor antes de hacerse cargo de la pantalla.

Cuando el campo de autenticación está enfocado, los caracteres imprimibles (incluyendo `j` y `k`) ingresan en la respuesta. Tab y Shift-Tab mueven el enfoque; Backspace elimina el último carácter. Esto se aplica a solicitudes secretas y no secretas.

La lista de cuentas se desplaza para mantener visible la cuenta seleccionada mientras navegas o redimensionas la terminal.

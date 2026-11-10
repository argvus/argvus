---
title: Núcleo y despachador
description: El espacio de trabajo argvus, su despachador y los crates compartidos de Rust.
---
`argvus` es el espacio de trabajo de Cargo en `de/argvus` y el paquete que proporciona `/usr/bin/argvus`. La caja `argvus-cli` es el despachador: lee el primer argumento y ejecuta un comando de biblioteca en proceso, ejecuta un asistente GTK 4 o un ejecutable privado, o ejecuta un script empaquetado. La lista completa de subcomandos está en la [referencia de línea de comandos](/docs/reference/command-line/)].

El espacio de trabajo también contiene las cajas que los comandos comparten:

- `argvus-config`: la tienda de configuración canónica y el único escritor del árbol generado (`data/generated/`). También es una biblioteca utilizada en proceso por `argvus config`, `theme-switch` y `effects-toggle`.
- `argvus-i18n` y `argvus-i18n-core`: los catálogos en `resources/locales/`, su validación y el módulo QML `org.argvus.i18n`.
- `argvus-tui` y `argvus-theme`: interfaz de usuario de terminal compartida y carga de temas.
- `argvus-runtime`: los mensajes traducidos y los asistentes de proceso compartidos por los comandos.

`argvus-session` sigue siendo un repositorio separado. Proporciona los puntos de entrada de sesión (`session`, `start`, `tty`) y las unidades de ciclo de vida, y `argvus sessionctl` alcanza su controlador.

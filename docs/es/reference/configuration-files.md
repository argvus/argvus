---
title: Archivos de configuración
description: Configuración ARGVUS de usuario, sistema, paquete por defecto y generada.
---

| Capa | Ubicación | Significado |
| --- | --- | --- |
| Sistema | `/etc/argvus/` | Configuración del administrador para servicios y proveedores. |
| Valores predeterminados del paquete | `/usr/share/argvus/` | Valores predeterminados de componentes de solo lectura y recursos de tiempo de ejecución. |
| Usuario | `$XDG_CONFIG_HOME/argvus/config.json` | Preferencias portátiles canónicas. |
| Datos gestionados | `$XDG_CONFIG_HOME/argvus/data/` | Datos de componentes, archivos de compatibilidad y metadatos internos. |
| Generado | `$XDG_CONFIG_HOME/argvus/data/generated/` | Proyecciones de `config.json`, escritas solo por `argvus-config`. Nunca editar, nunca copiar a otro lugar como configuración, y nunca leer de nuevo como fuente. |
| Tiempo de ejecución | `$XDG_RUNTIME_DIR/` y `$XDG_CACHE_HOME/` | Sockets, bloqueos y estado transitorio. |

El tema activo, el acento, el fondo de pantalla personalizado y el estado de los efectos son estados lógicos del usuario. El modelo lógico gestionado está documentado en [Canonical configuration](/docs/reference/argvus-config/). Los archivos generados de Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, Yazi, Superfile, terminal, fuente y dispositivos extraíbles no deben tratarse como fuentes de verdad independientes.

`argvus-config` también refleja la capa de tema en `data/waybar/argvus-taskbar.{jsonc,css}` y `data/waybar/argvus-widget-telemetry.{jsonc,css}`. Esas copias se reemplazan desde los valores predeterminados empaquetados cuando hay un cambio de apariencia, luego se reescriben sus bloques gestionados delimitados, por lo que deben considerarse como derivados en lugar de archivos mantenidos manualmente. La configuración de aplicaciones nativas bajo `$XDG_CONFIG_HOME/<app>` es diferente: es un objetivo adaptador escrito por los orquestadores de apariencia, nunca por el proyector.

## Excepción a la regla de escritor único

`argvus config` es el único escritor de `data/generated/`. Existe una excepción para la migración: cuando se crea una copia de usuario de un archivo empaquetado, un archivo que una versión anterior dejó bajo `data/generated/` se mueve a su ubicación de usuario (`data/`), para que el usuario conserve sus ediciones. El movimiento ocurre solo para archivos que la versión actual no genera. No confíe en esta ruta para archivos nuevos: agregue un archivo generado en la proyección, no aquí.

---
title: Temas
description: Instala, selecciona, personaliza, importa y exporta temas de ARGVUS.
---
Un tema en ARGVUS establece los colores, superficies, fondos de pantalla predeterminados y la geometría del diseño utilizada en todo el escritorio: Hyprland, Waybar, el Panel de Control, la barra de tareas, la pantalla de bloqueo, las notificaciones y las aplicaciones de ARGVUS. Esta guía cubre todo lo que necesitas para trabajar con temas como usuario.

## Qué se incluye en un tema

Cada tema oficial es una identidad visual completa con dos modos:

- **Sticky** — un diseño compacto con pequeños espacios entre ventanas, sin espacios exteriores y una barra de tareas cerca del borde de la pantalla.
- **Spaced** — un diseño espacioso con espacios más grandes, esquinas redondeadas y márgenes alrededor de la barra de tareas y las superficies del shell.

Seleccionar un tema no cambia su identidad: la paleta, el color de resaltado predeterminado y la asociación del fondo de pantalla permanecen igual entre los modos. El modo es una configuración separada, vea [Temas oficiales](/docs/components/argvus-themes/official-themes/).

## Páginas en esta guía

- [Instalación de temas](/docs/components/argvus-themes/installing-themes/) — de dónde provienen los temas oficiales y cómo instalar uno o todos ellos con `pacman`.
- [Temas oficiales](/docs/components/argvus-themes/official-themes/) — el catálogo, el color de resaltado predeterminado de cada tema y cómo elegir un tema y su modo.
- [Personalización del color de resaltado](/docs/components/argvus-themes/highlight-color/) — cambia el color de acento independientemente del tema.
- [Importación y exportación de temas](/docs/components/argvus-themes/import-and-export/) — exporta el tema activo, con tu color de resaltado y fondo de pantalla, como un tema propio e impórtalo en otra máquina.

## Referencia rápida

| Tarea | Dónde |
| --- | --- |
| Elegir un tema | **Centro de control → Apariencia → Temas → Temas oficiales** |
| Alternar Sticky y Spaced | **Centro de control → Apariencia → Modo** |
| Cambiar el color de resaltado | **Centro de control → Apariencia → Color de resaltado** |
| Exportar el tema activo | **Centro de control → Apariencia → Temas → Exportar tema** |
| Importar un tema | **Centro de control → Apariencia → Temas → Importar tema** |
| Abrir el selector de temas desde el teclado | `SUPER + SHIFT + T` |

## De dónde vienen los temas

ARGVUS incluye dos temas integrados, **ARGVUS Dark** y **ARGVUS Light**, con el paquete `argvus-appearance`. Cada otro tema oficial es un paquete `argvus-theme-<name>` separado. Instalar un paquete de temas lo añade al selector de temas; no se necesita reiniciar la sesión. Tus propios temas, exportados o importados, se listan por separado como **Temas personalizados**.

## Sugerencias de teclado

Las páginas de temas del Centro de Control muestran sus teclas en el pie de página:

- `↑` / `↓` — navegar.
- `Enter` — aplicar o abrir.
- `e` — exportar el tema activo, en una página de Temas.
- `i` — importar un tema, en una página de Temas.
- `d` — eliminar un tema personalizado.
- `Esc` — volver.

## Solución de problemas

- **Un tema que instalé no aparece.** Confirma que el paquete está instalado con `pacman -Q argvus-theme-<name>`, luego vuelve a abrir el Centro de Control. El paquete debe estar instalado con `argvus-appearance` presente.
- **El color de resaltado cambió después de cambiar de tema.** Un color de resaltado personalizado permanece tras cambiar de tema. Usa **Restablecer al valor predeterminado del tema** para devolver al tema su propio color.
- **Un archivo de tema no aparece listado para importar.** Debe estar directamente en tu directorio personal y contener `<id>/theme.toml`. Consulta [Importar y exportar temas](/docs/components/argvus-themes/import-and-export/).

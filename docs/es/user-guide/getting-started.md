---
title: Primera configuración
description: Un camino práctico de primer arranque para configurar ARGVUS.
---
Después de la primera sesión gráfica, no necesitas configurar cada parte de ARGVUS. Comienza con los ajustes que afectan cómo se siente el escritorio, luego configura el hardware y los servicios según sea necesario.

## Un orden útil

1. Lee la [visión general del escritorio](/docs/user-guide/desktop/) para identificar ventanas, espacios de trabajo, la barra de tareas, el Panel de Control y el lanzador.
2. Abre el [Centro de Control](/docs/components/argvus-control-center/) y elige un tema, modo, acento y fondo de pantalla en **Apariencia**.
3. Ajusta [ventanas y diseño](/docs/argvus-hyprland/windows-and-layout/) si los espacios predeterminados, bordes o el espaciado de la barra de tareas no se ajustan a tu pantalla.
4. Configura la [barra de tareas](/docs/argvus-taskbar/taskbar/) y decide qué tarjetas del Panel de Control quieres visibles.
5. Revisa [atajos de teclado](/docs/argvus-hyprland/keyboard-shortcuts/) y cambia solo las combinaciones que realmente uses.
6. Configura [ratón y panel táctil](/docs/argvus-hyprland/input/), pantallas, red y energía desde las páginas de configuración correspondientes.

Los primeros cuatro pasos son personalización opcional. Los pasos restantes normalmente solo son necesarios cuando tu hardware, idioma o flujo de trabajo lo requieren.

## Dos lugares para cambiar la configuración

Usa **Centro de Control** para configuraciones que describen cómo debe configurarse ARGVUS: apariencia, diseño, fuentes, entrada, atajos, idioma, región y aplicaciones predeterminadas. Usa **Panel de Control** para el estado y acciones frecuentes de la sesión, como volumen, brillo, estado de la red, notificaciones, energía y controles de sesión.

Consulta [Centro de Control vs Panel de Control](/docs/components/argvus-control-center/) para la distinción completa y [Dónde configurar cosas](/docs/user-guide/where-to-configure/) para un índice rápido.

## Cambios, persistencia y recuperación

La mayoría de los cambios en el Centro de Control se aplican mientras la sesión está en ejecución y se guardan como estado de usuario de ARGVUS. Los cambios de apariencia se comprometen a la configuración canónica, y `argvus-config` proyecta los archivos consumidos por Hyprland y la shell a partir de ella. Un interruptor del Panel de Control puede ser una acción directa o un cambio de estado de la sesión en lugar de una configuración permanente.

Cuando una página ofrece **Restablecer valores predeterminados**, usa esa acción de la página en lugar de eliminar archivos generados. Apariencia tiene acciones de restablecimiento separadas para acentos, fondos de pantalla y valores de diseño; la página de atajos de teclado tiene su propia acción de restauración. No existe un botón de restablecimiento universal único para todas las configuraciones del escritorio.

## Próximos pasos

- [Apariencia](/docs/user-guide/appearance/) — temas, fondos de pantalla, efectos y diseño.
- [Diseño del escritorio](/docs/argvus-hyprland/windows-and-layout/) — comprender los espacios, bordes y espacio del panel.
- [Dispositivos de entrada](/docs/argvus-hyprland/input/) — controles del ratón y touchpad.
- [Solución de problemas de tema y fondo de pantalla](/docs/user-guide/troubleshooting/theme-and-wallpaper/) — recuperación cuando un cambio visual no es lo que esperabas.

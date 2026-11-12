---
title: Temas oficiales
description: El catálogo oficial de temas de ARGVUS, los colores de resaltado predeterminados y cómo seleccionar un tema y su modo Pegajoso o Espaciado.
---
Esta página enumera los temas oficiales disponibles como paquetes de temas y explica cómo seleccionar uno. Para instalarlos, vea [Instalando temas](/docs/components/argvus-themes/installing-themes/).

## Seleccionar un tema

1. Abra **Centro de Control → Apariencia → Temas**.
2. Elija **Temas Oficiales**.
3. Elija **Oscuro** o **Claro**.
4. Elija la familia de temas. El tema se aplica tan pronto como lo seleccione.

El tema seleccionado se aplica a todo el escritorio de inmediato: la barra de tareas en ejecución, los paneles, las notificaciones, las ventanas y el fondo de pantalla cambian de color en el lugar, sin una pantalla de carga y sin reiniciar. ARGVUS mantiene su selección de fondo de pantalla a menos que el tema se aplique con su propia asociación de fondo.

También puede presionar `SUPER + SHIFT + T` para abrir el selector de temas desde el teclado. Use las teclas de flecha para moverse por las categorías, `Enter` para elegir y `Esc` para cerrar.

## Pegajoso y Espaciado

Cada tema funciona en dos modos. El modo es independiente del tema: cambiar el tema mantiene el modo actual, y cambiar el modo mantiene el tema actual.

| Modo | Espacios entre ventanas | Margen exterior | Redondeo de ventanas | Barra de tareas y margen del shell |
| --- | ---: | ---: | ---: | ---: |
| Fijo | `2` | `0` | `0` | cerca del borde |
| Separado | `10` | `18` | `4` | `18` |

Para cambiar el modo, abre **Centro de Control → Apariencia → Modo** y elige **Fijo** o **Separado**. Tus ajustes manuales de estos valores de diseño permanecen vigentes hasta la próxima selección explícita de tema.

## Temas oscuros

| Tema | Paquete | Resaltado predeterminado |
| --- | --- | --- |
| Uno oscuro | `argvus-theme-one-dark` | `#61AFEF` |
| Drácula | `argvus-theme-dracula` | `#BD93F9` |
| Plata Oscuro | `argvus-theme-silver-dark` | `#595959` |
| Pizarra Oscura | `argvus-theme-slate-dark` | `#7391A5` |
| Universo | `argvus-theme-universe` | `#EEEEEE` |
| Gruvbox Alto Oscuro | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| Gruvbox Oscuro | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| Pino rosado | `argvus-theme-rose-pine` | `#C4A7E7` |
| Noche de Tokio | `argvus-theme-tokyo-night` | `#7AA2F7` |
| Soledad | `argvus-theme-solitude` | `#798186` |
| Atardecer | `argvus-theme-sunset` | `#E2BE8A` |
| Matriz | `argvus-theme-matrix` | `#82FB9C` |
| Monokai Oscuro | `argvus-theme-monokai-dark` | `#78DCE8` |
| Señora Niebla | `argvus-theme-lady-mist` | `#354B78` |

## Temas claros

| Tema | Paquete | Resaltado predeterminado |
| --- | --- | --- |
| GitHub Luz | `argvus-theme-github-light` | `#0969DA` |
| Luz Solarizada | `argvus-theme-solarized-light` | `#268BD2` |
| Luz del Norte | `argvus-theme-nord-light` | `#5E81AC` |
| Una luz | `argvus-theme-one-light` | `#4078F2` |
| Luz del bosque eterno | `argvus-theme-everforest-light` | `#3A94C5` |
| Catuchino Latte | `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Luz Gruvbox | `argvus-theme-gruvbox-light` | `#458588` |
| Acuarela | `argvus-theme-watercolor` | `#1A639C` |
| Luz de obsidiana | `argvus-theme-obsidian-light` | `#334A78` |

## Temas incorporados

**ARGVUS Dark** y **ARGVUS Light** son proporcionados por `argvus-appearance` y siempre están disponibles bajo **Temas oficiales**. ARGVUS Dark usa el resaltado predeterminado `#3590BD`; ARGVUS Light usa `#181818`.

## Qué controla un tema

Un tema define la paleta y el tratamiento de superficie utilizado por los componentes del escritorio, el color de resaltado predeterminado, la asociación de fondo de pantalla predeterminada y los valores predeterminados de transparencia y desenfoque. Los componentes del escritorio leen este estado en lugar de almacenar su propia copia, por lo que un cambio de tema llega al compositor, a la barra de tareas, al Panel de control, a la pantalla de bloqueo, a las notificaciones y a las aplicaciones en conjunto.

El color de resaltado predeterminado es solo el punto de partida. Puedes reemplazarlo con cualquier color de seis dígitos, consulta [Personalización del color de resaltado](/docs/components/argvus-themes/highlight-color/).

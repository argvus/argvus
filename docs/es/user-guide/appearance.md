---
title: Apariencia
description: Configure el lenguaje visual compartido por los componentes de ARGVUS.
---
El sistema de apariencia aplica un estado de tema a Hyprland, GTK, Qt, Waybar, Quickshell, Rofi, Dunst, Hyprlock y aplicaciones ARGVUS.

## Familias de temas

Las veintitrés familias a continuación están definidas por la carga útil del tema actual. La columna de color es el color predeterminado de borde activo/acento de la familia; los usuarios pueden elegir otro acento RGB válido.

| Familia | Color de resaltado de reinicio | Fondo base | Modos |
| --- | --- | --- | --- |
| One Dark | `#61AFEF` | `#282C34` | Pegajoso, Espaciado |
| Dracula | `#BD93F9` | `#282A36` | Pegajoso, Espaciado |
| ARGVUS Dark | `#3590bd` | `#111316` | Pegajoso, Espaciado |
| Dark Silver | `#595959` | `#111316` | Pegajoso, Espaciado |
| Dark Slate | `#7391a5` | `#2f3541` | Pegajoso, Espaciado |
| Dark Universe | `#eeeeee` | `#000000` | Pegajoso, Espaciado |
| ARGVUS Dark Gruvbox High | `#D79921` | `#282828` | Pegajoso, Espaciado |
| ARGVUS Dark Gruvbox | `#D4BE98` | `#282828` | Pegajoso, Espaciado |
| Luz ARGVUS | `#181818` | `#f7f7f7` | Pegajoso, espaciado |
| Luz de GitHub | `#0969DA` | `#FFFFFF` | Pegajoso, espaciado |
| Catuchino Latte | `#1E66F5` | `#EFF1F5` | Pegajoso, espaciado |
| ARGVUS Gruvbox ligero | `#458588` | `#FBF1C7` | Pegajoso, espaciado |
| Pino rosado | `#C4A7E7` | `#191724` | Pegajoso, espaciado |
| Tokio-Noche | `#7AA2F7` | `#1A1B26` | Pegajoso, espaciado |
| Soledad | `#798186` | `#101315` | Pegajoso, espaciado |
| Puesta de sol | `#E2BE8A` | `#0F0F0F` | Pegajoso, espaciado |
| Matriz | `#82FB9C` | `#0B0C16` | Pegajoso, espaciado |
| Monokai Oscuro | `#78DCE8` | `#2D2A2E` | Pegajoso, espaciado |
| Señora Niebla | `#354B78` | `#131313` | Pegajoso, espaciado |
| Una luz | `#4078F2` | `#FAFAFA` | Pegajoso, espaciado |
| Luz del bosque eterno | `#3A94C5` | `#FDF6E3` | Pegajoso, espaciado |
| Luz Solarizada | `#268BD2` | `#FDF6E3` | Pegajoso, espaciado |
| Luz del Norte | `#5E81AC` | `#ECEFF4` | Pegajoso, espaciado |
| Acuarela | `#1A639C` | `#E9E3D3` | Pegajoso, espaciado |

El color mostrado en la tabla es el valor escrito en `config.json` cuando se selecciona esa familia o cuando se usa **Restablecer al tema predeterminado**. Un acento RGB de seis dígitos elegido manualmente se aplica hasta el siguiente cambio de tema, que lo reemplaza con el valor predeterminado del tema seleccionado.

**Sticky** es la disposición compacta: las ventanas en mosaico utilizan pequeños espacios internos, los espacios exteriores son cero y la barra de tareas se encuentra cerca del borde de la pantalla. **Spaced** agrega espacios más grandes, esquinas redondeadas, sombras y márgenes alrededor de la barra de tareas y las superficies del sistema.

Los valores de disposición existentes `float` y los identificadores de tema `-float` se reconocen como nombres heredados para Spaced. Los nuevos valores de disposición y los identificadores de tema resueltos usan `spaced` y `-spaced`.

El modo cambia la geometría y el tratamiento de la superficie; no crea una sexta familia de color. Vea [Themes](/docs/user-guide/appearance/) para la selección y [Desktop](/docs/user-guide/desktop/) para la colocación de la barra de tareas y las ventanas.

- [Temas](/docs/user-guide/appearance/)
- [Fondos de pantalla](/docs/argvus-appearance/wallpapers/)
- [Fuentes e íconos](/docs/argvus-appearance/fonts-and-icons/)
- [Efectos](/docs/argvus-appearance/effects/)

El Centro de Control expone la configuración visible para el usuario. La implementación es compartida por `argvus-appearance`, `argvus-session`, los componentes del shell y los paquetes de recursos. Ver [arquitectura de apariencia](/docs/developer-guide/subsystems/appearance-and-themes/).

## Oscilación de ventanas

Abre **Escritorio y entrada → Hyprland → Animaciones → Ventanas** y usa los controles de **Oscilación**. Busca `wobble`, o abre la página directamente:

```sh
argvus control-center hyprland animations windows
```
El movimiento oscilante está desactivado por defecto. Sus valores predeterminados en ARGVUS son rigidez `100`, amortiguamiento `10`, masa `1`, intensidad `1` y resolución de malla `8`. La rigidez y la masa aceptan `0.0001–1000`, el amortiguamiento `0–1000`, la intensidad `0–2`, y la malla un número entero `2–32`. Una mayor resolución de malla aumenta el coste de renderizado. Usa las teclas de flecha para ajustar los valores o Enter para escribirlos. **Restaurar los valores predeterminados de Wobble** cambia el borrador; **Aplicar** guarda todas las ediciones pendientes de animación y recarga Hyprland una vez, sin reiniciar la sesión. Volver dentro de Animations mantiene el borrador compartido. 

El interruptor global de Animations suprime Wobble mientras mantiene sus configuraciones individuales. Los valores canónicos se encuentran bajo `effects.wobble_*` en el estado ARGVUS; `hypr/state.lua` se genera y no debe editarse manualmente.

Esto requiere Hyprland **0.57.0 o más reciente**, impuesto por la dependencia del paquete de `argvus-hyprland`. La versión 0.56 no es compatible con esta configuración. El máximo de intensidad sigue la [fuente publicada 0.57.0 ](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp), que lo limita a `2`, aunque la documentación más reciente pueda listar `3`.

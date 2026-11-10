---
title: Introducción
description: Qué es ARGVUS, cómo funciona su escritorio y dónde encajan sus componentes.
---
ARGVUS es un entorno de escritorio modular para Hyprland y Wayland en Arch Linux. Es una colección integrada de paquetes en lugar de una aplicación monolítica: la sesión, el compositor, las superficies del shell, las aplicaciones de configuración, los proveedores y los recursos visuales funcionan juntos a través de un estado compartido y contratos de ejecución explícitos.

Esta página es el mapa de la documentación. Explica todo el escritorio a un alto nivel; las páginas vinculadas contienen los detalles operativos y de implementación.

## Cómo funciona ARGVUS

La sesión normal sigue este camino:

```text
graphical login or TTY
        ↓
argvus-session → argvus start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```
`argvus-session` posee el ciclo de vida de la sesión y la configuración del entorno. No implementa todas las funciones del escritorio: cada componente mantiene la propiedad de sus propios comandos, configuración y carga útil de servicio.

Para la instalación y el inicio, consulte [Installation](/docs/getting-started/installation/)] y [Sessions](/docs/user-guide/sessions/)]. Para la propiedad del servicio, vea el [ciclo de vida en tiempo de ejecución](/docs/developer-guide/architecture/runtime-lifecycle/)] del desarrollador.

## Capas principales

### Puntos de entrada e infraestructura

El paquete `argvus` proporciona el despachador `/usr/bin/argvus` y el punto de entrada principal del paquete. `argvus-session` proporciona puntos de entrada gráficos y TTY, el objetivo del usuario y `argvus sessionctl`. La funcionalidad compartida de TUI y localización proviene del espacio de trabajo `argvus` (los crates `argvus-tui` y `argvus-i18n`) y el comando `argvus i18n`.

### Compositor y shell de escritorio

`argvus-hyprland` suministra la configuración y los atajos de teclado de Hyprland. `argvus-waybar` suministra el binario de Waybar parcheado, mientras que `argvus-taskbar` posee la configuración de la barra de tareas y las acciones. `argvus-control-panel` suministra el panel Quickshell, y `argvus widget-telemetry` suministra widgets de sistema opcionales, en proceso en `argvus`.

Vea [Desktop](/docs/user-guide/desktop/) para el uso y [shell surfaces](/docs/developer-guide/subsystems/shell-surfaces/) para la propiedad de implementación.

### Configuraciones y aplicaciones

`argvus control-center` es la aplicación de configuraciones orientada al teclado, distribuida como un ayudante en `argvus`. Las herramientas de calendario y de dispositivos extraíbles son ayudantes GTK 4 en `argvus`; el monitor del sistema y la terminal son subcomandos de `argvus`; el lanzador es `argvus-launcher`. Todos ellos están integrados con el shell y el despachador.

Vea [Applications](/docs/user-guide/applications/) y la [referencia de comandos](/docs/reference/command-line/)].

### Proveedores e integraciones

Pantalla, red, Bluetooth, energía, notificaciones, firewall y almacenamiento extraíble son proporcionados por proyectos específicos. El Centro de Control y el Panel de Control consumen estos proveedores; no los reemplazan.

El inicio de sesión, el bloqueo y los elementos visuales de inicio también son separados: `argvus-greeter` maneja greetd, `argvus-lock` maneja Hyprlock, el ayudante de transferencia incluido en `argvus` dibuja el spinner de inicio de sesión, la superposición de carga maneja las transiciones de tema, y `argvus-boot-splash` maneja la pantalla de inicio.

Vea [Hardware](/docs/user-guide/hardware/), [Privacidad y seguridad](/docs/user-guide/privacy-and-security/) y la [referencia de componentes](/docs/developer-guide/) para desarrolladores.

## Estado de apariencia compartido

Las preferencias de tema, acento, fondo de pantalla, fuente y efectos se almacenan como estado lógico bajo `$XDG_CONFIG_HOME/argvus` (normalmente `~/.config/argvus`). `argvus-config` es el único componente que escribe `data/generated/`, donde proyecta archivos de usuario para Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, notificaciones, la pantalla de bloqueo, Yazi, Superfile y terminales. Rofi es la excepción: `argvus launcher` empaqueta su configuración y temas como archivos simples, y las copias editadas por el usuario se encuentran bajo `data/rofi/`. Los ayudantes de apariencia confirman el cambio de estado y luego reconcilian las aplicaciones externas; nunca escriben el árbol generado. 

El directorio generado no es la fuente de verdad. Ver [Appearance](/docs/user-guide/appearance/)] y [configuration and state](/docs/developer-guide/architecture/configuration-and-state/)].

El sistema visual también controla la disposición. El modo pegajoso es compacto y cuadrado; el modo espaciado utiliza espacios más grandes en las ventanas, esquinas redondeadas y márgenes más amplios alrededor de la barra de tareas y el entorno de escritorio. La barra de tareas se coloca por defecto en el borde superior, pero admite colocación en la parte superior/inferior a través del estado de disposición compartido. Los efectos controlan la transparencia, el desenfoque, las sombras y las animaciones a través del compositor y las superficies del escritorio.

Consulte la [tabla de temas](/docs/user-guide/appearance/)] para las familias y colores actuales, [efectos](/docs/argvus-appearance/effects/)] para el contrato de efectos compartido, y [barra de tareas](/docs/argvus-taskbar/taskbar/)] para la colocación de barras y ventanas.

## Qué es —y qué no es— ARGVUS

ARGVUS es la capa del entorno de escritorio sobre Arch Linux. Arch Linux proporciona el sistema operativo y los paquetes; Hyprland proporciona la composición Wayland; ARGVUS coordina la sesión gráfica, las superficies del entorno, la configuración, la apariencia, los servicios de escritorio y las aplicaciones integradas.

Por lo tanto, es más que una configuración de Hyprland o una colección de dotfiles personales. ARGVUS define un punto de entrada de sesión compatible, gestiona el ciclo de vida de sus servicios de escritorio, proporciona un modelo común de configuración y apariencia, y conecta componentes enfocados como redes, pantallas, energía, notificaciones, cuentas y almacenamiento extraíble.

ARGVUS no necesita reemplazar todas las aplicaciones. Integra aplicaciones seleccionadas de upstream cuando esa es la opción más confiable, mientras mantiene coherente el comportamiento y la configuración de todo el escritorio.

El nombre combina dos ideas. *ARGV*, de *argument vector*, es la lista de argumentos que recibe un programa, y refleja el objetivo de priorizar el teclado para manejar el escritorio mediante comandos y atajos. *US*, de *User System*, refleja el objetivo de un escritorio centrado en su usuario. Para la explicación completa y el audio de la pronunciación, vea [About ARGVUS](/about/)].

Para los límites de implementación y la propiedad de los paquetes, consulte la [visión general de la arquitectura del desarrollador](/docs/developer-guide/architecture/overview/)] y [roles del repositorio](/docs/developer-guide/architecture/repository-roles/)].

## Continúe por objetivo

- [Lea las preguntas frecuentes](/docs/user-guide/faq/)].
- [Instale ARGVUS](/docs/getting-started/installation/)].
- [Inicie una sesión gráfica o TTY](/docs/user-guide/sessions/)].
- [Configure temas y fondos de pantalla](/docs/user-guide/appearance/)].
- [Use la barra de tareas y el Panel de control](/docs/user-guide/desktop/)].
- [Comprenda comandos, rutas y servicios](/docs/reference/)].
- [Estudie la arquitectura](/docs/developer-guide/architecture/overview/)].

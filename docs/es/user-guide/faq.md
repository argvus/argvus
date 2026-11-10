---
title: Preguntas frecuentes
description: Preguntas comunes sobre ARGVUS, su arquitectura y uso diario.
---
Esta página responde preguntas comunes sobre qué es ARGVUS, cómo se distribuye y dónde configurarlo. Para procedimientos detallados, siga los enlaces en cada respuesta.

## ¿Qué es ARGVUS?

ARGVUS es un entorno de escritorio modular para Wayland y Arch Linux, construido alrededor de Hyprland. No es un entorno de escritorio independiente de Hyprland; es una experiencia de escritorio ensamblada sobre Hyprland y adaptada a Arch Linux. ARGVUS coordina la sesión gráfica, superficies de shell, aplicaciones de configuración, apariencia, servicios de escritorio y herramientas integradas a través de una colección de paquetes.

Es más que una configuración de Hyprland o una colección de dotfiles personales: proporciona un punto de entrada de sesión soportado, configuración compartida y estado de apariencia, y propiedad definida para componentes como la barra de tareas, el Centro de Control, las notificaciones, la red y las pantallas. Consulta la [introducción](/docs/introduction/) para una visión general de cómo encajan las piezas.

## ¿Es ARGVUS gratuito?

Sí. ARGVUS es gratuito para descargar y usar, y sus paquetes de proyecto principales se distribuyen como software libre bajo la [Licencia Pública General de GNU versión 3](https://www.gnu.org/licenses/gpl-3.0.html). Esto da a los usuarios la libertad de ejecutar, estudiar, modificar y redistribuir el software cubierto bajo los términos de la licencia.

ARGVUS se distribuye como paquetes firmados de Arch Linux. El software en sí no tiene precio de compra, pero la descarga de paquetes aún puede implicar los costos normales de Internet o de alojamiento. Los componentes y aplicaciones proporcionados por terceros pueden tener sus propias licencias.

## ¿Es ARGVUS una distribución de Linux?

No. ARGVUS es la capa del entorno de escritorio instalada sobre Arch Linux. Arch Linux proporciona el sistema operativo, el kernel, el gestor de paquetes y los paquetes base; Hyprland proporciona la composición de Wayland; ARGVUS coordina la sesión de escritorio y sus componentes integrados.

Consulta [installation](/docs/getting-started/installation/) para el modelo de instalación compatible y [Qué es ARGVUS y qué no es](/docs/introduction/#what-argvus-isand-is-not) para la distinción arquitectónica.

## ¿Por qué un entorno de escritorio en lugar de una distribución?

Mantener ARGVUS como un entorno de escritorio da a los usuarios más control sobre el resto de su sistema. Una distribución normalmente elige y mantiene una base completa, incluyendo el kernel, la selección de paquetes, los repositorios, los valores predeterminados y las políticas del sistema. ARGVUS se centra en la experiencia de escritorio mientras deja esas decisiones más amplias al usuario y a la instalación subyacente de Arch Linux.

Este modelo también mantiene el proyecto modular: la gestión de sesiones, la configuración de Hyprland, la apariencia, la barra de tareas, la configuración y los proveedores pueden desarrollarse y empaquetarse por separado mientras trabajan juntos como un solo escritorio.

## ¿Qué sistemas son compatibles?

La base actualmente soportada es Arch Linux. ARGVUS está empaquetado para Arch Linux y depende de Wayland, Hyprland y los componentes del sistema descritos en los [requisitos de instalación](/docs/getting-started/installation/requirements/).

Otras distribuciones pueden proporcionar algunos de los mismos componentes upstream, pero eso no hace que el conjunto completo de paquetes o la integración de sesión de ARGVUS esté soportada allí.

## ¿Cómo instalo ARGVUS?

Sigue la [guía de instalación](/docs/getting-started/installation/)]. Explica los requisitos, cómo configurar el repositorio de paquetes firmado de ARGVUS, cómo instalar el conjunto de paquetes y cómo iniciar la primera sesión.

ARGVUS se entrega como un conjunto coordinado de paquetes de Arch. El paquete `argvus` es el objetivo principal de la instalación y trae los componentes de escritorio que coordina.

## ¿Cómo cambio la configuración de ARGVUS?

Usa el **Centro de Control** para la configuración persistente del escritorio, incluyendo temas, fondos de pantalla, efectos, diseño, fuentes, teclado, entrada, idioma, región y aplicaciones predeterminadas. Las formas más eficientes de abrirlo son el acceso directo `SUPER + Alt + C`, o el lanzador con `SUPER + D`: escribe **Centro de Control ARGVUS**. También puedes escribir `argvus` para listar las aplicaciones ARGVUS.

Como opción adicional, puedes abrirlo desde un terminal con:

```sh
argvus --control-center
```
También puedes abrir un área enfocada con `argvus control-center`, por ejemplo `argvus control-center appearance themes`. Ejecuta `argvus control-center --help` para ver las rutas disponibles en la versión instalada.

Usa **Panel de Control** para el estado actual y acciones frecuentes como volumen, brillo, red, notificaciones, control de energía y de sesión. No es una segunda base de datos de configuración. Consulta [Dónde configurar las cosas](/docs/user-guide/where-to-configure/)] y [Centro de Control](/docs/components/argvus-control-center/)] para la distinción.

## ¿Dónde se almacenan los ajustes de ARGVUS?

La configuración propiedad del usuario se almacena debajo de `$XDG_CONFIG_HOME/argvus` (normalmente `~/.config/argvus`). La configuración lógica canónica se mantiene allí, mientras que los archivos derivados para consumidores como Hyprland, Waybar y el widget de telemetría, GTK, Qt6ct, Quickshell, Dunst, Hyprlock, Yazi, Superfile, terminales, fuentes, efectos, calendarios, entrada, teclado, energía y dispositivos extraíbles viven bajo `~/.config/argvus/data/generated/`. Ese árbol es escrito solamente por `argvus-config` y se reconstruye a partir de la configuración canónica; `argvus-appearance` confirma los cambios de apariencia y luego reconcilia únicamente a los consumidores externos que no pueden leer la configuración canónica por sí mismos. Rofi es la excepción: `argvus launcher` empaqueta su configuración y temas, y las copias editadas por el usuario viven bajo `~/.config/argvus/data/rofi/`.

No edite archivos generados como un método de configuración permanente. Use el Centro de Control o la ruta de sobrescritura nativa documentada para el componente que desea personalizar. Vea [configuration files](/docs/reference/configuration-files/) y [configuration and state](/docs/developer-guide/architecture/configuration-and-state/).

## ¿Qué debo hacer si un cambio no se aplica?

Primero confirme que el cambio fue guardado o aplicado en la página correspondiente. Luego, verifique si el componente afectado necesita una recarga de sesión o de servicio. Un cambio confirmado se proyecta en los archivos del consumidor mediante `argvus-config`, pero proyectarlos no garantiza que todos los consumidores en ejecución los hayan recargado. `argvus sessionctl reload` aplica una generación ya confirmada desde el manifiesto de proyección en lugar de proyectarla nuevamente, y omite la recarga gráfica cuando el estado canónico no ha cambiado.

Utilice la [guía de solución de problemas](/docs/user-guide/troubleshooting/) relevante para la instalación, el inicio de sesión, gráficos, pantallas, Hyprland, temas y fondos de pantalla. Al reportar un problema, incluya el componente, el ajuste modificado, el comando o página utilizado y la salida del comando de estado o diagnóstico correspondiente.

## ¿Dónde puedo aprender más?

- [Primera configuración](/docs/user-guide/getting-started/) para el flujo recomendado de la primera sesión.
- [Guía del usuario](/docs/user-guide/) para instrucciones específicas de funciones.
- [Referencia](/docs/reference/) para comandos, rutas, servicios y contratos de configuración.
- [Guía del desarrollador](/docs/developer-guide/) para arquitectura, propiedad de paquetes y flujos de trabajo de contribución.

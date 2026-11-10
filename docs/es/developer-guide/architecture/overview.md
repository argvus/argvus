---
title: Visión general de la arquitectura
description: Cómo ARGVUS forma un entorno de escritorio integrado.
---
ARGVUS tiene cinco capas de tiempo de ejecución:

1. puntos de entrada y despacho en `argvus`;
2. ciclo de vida de la sesión en `argvus-session`;
3. compositor y superficies de shell en `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar` y `argvus-control-panel`;
4. aplicaciones de usuario como el Centro de Control, calendario y monitor del sistema;
5. proveedores y recursos para pantalla, red, energía, apariencia, cuentas y notificaciones.

El estado compartido conecta estas capas. `argvus-config` contiene el documento canónico y es el único escritor de `data/generated/`, por lo que un cambio de tema o efectos regenera los archivos consumidos por varios paquetes en una sola transacción; los servicios de sesión proporcionan propiedad del ciclo de vida sin asumir la propiedad de la implementación de cada componente.

## Límites de responsabilidad

| Capa | Responsabilidad principal | Ejemplos |
| --- | --- | --- |
| Puntos de entrada y despacho | Iniciar aplicaciones de escritorio y resolver perfiles ARGVUS. | `argvus`, `argvus-app-profiles` |
| Ciclo de vida de la sesión | Preparar el entorno, lanzar Hyprland, sincronizar el entorno del servicio del usuario y el cierre de su propia sesión. | `argvus-session`, `argvus sessionctl` |
| Compositor y shell | Definir el comportamiento de las ventanas y las superficies de escritorio visibles. | `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar`, `argvus-control-panel` |
| Configuración | Propietario de `config.json`, el modelo de rutas y todos los archivos de consumidor generados. | `argvus-config` |
| Aplicaciones | Proporcionar configuraciones y flujos de trabajo de escritorio enfocados. | Centro de control, calendario, lanzador, monitor del sistema y dispositivos extraíbles |
| Proveedores y recursos | Implementar operaciones de dominio y recursos visuales compartidos. | pantalla, red, energía, notificaciones, apariencia, fuentes y fondos de pantalla |

El paquete `argvus` es el objetivo de instalación y coordinador, pero no es un runtime monolítico. Los paquetes enfocados conservan la propiedad de sus binarios, configuración y servicios. El gestor de sesiones los inicia en el momento apropiado de la sesión gráfica sin absorber su implementación.

Esta distinción importa al depurar: un fallo de sesión pertenece primero a `argvus-session`; un fallo en la barra de tareas pertenece a `argvus-taskbar`/`argvus-waybar`; y un fallo en un tema o fondo de pantalla generado pertenece a la ruta de configuración de apariencia y sesión.

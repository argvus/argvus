---
title: Proyectos
description: El selector de proyectos y la configuración de proyectos, a través de proyectos argvus.
---
`argvus projects` lista y abre tus proyectos. Es un comando en proceso del paquete `argvus`, y `SUPER + O` abre el selector.

## Comandos

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position, full path or unique name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```
## Qué es un proyecto

Un proyecto es un directorio listado en la configuración (`/projects/paths`), o un subdirectorio de
una raíz listada en `/projects/roots`. Se listan como máximo nueve proyectos: los números de los
atajos `SUPER + CTRL + N`. Abrir un proyecto centra el espacio de trabajo de Hyprland. La primera vez que un
proyecto se abre, ARGVUS inicia un terminal y el editor configurado en `/projects/editor` dentro de él.

La configuración se lee y se escribe a través de `argvus config`, por lo que el Centro de Control y el
shell ven la misma lista.

## Espacios de trabajo del proyecto y la barra de tareas

Los proyectos usan espacios de trabajo de Hyprland con nombres ordinarios, no espacios de trabajo especiales. `SUPER + O` muestra el número del proyecto, el directorio y el acceso directo directo `SUPER + CTRL + N`. Ambos puntos de entrada enfocan el mismo espacio de trabajo. Solo los espacios de trabajo abiertos aparecen en la barra de tareas, con una etiqueta `Project: name`, estado activo y una información sobre herramientas que contiene la ruta completa y el acceso directo. Haga clic en el botón del espacio de trabajo para regresar a él. La barra de tareas sigue al monitor del espacio de trabajo.

La ruta de directorio canónica determina una identidad estable del espacio de trabajo; cambiar el orden de la lista cambia el acceso directo, no el espacio de trabajo. Los proyectos con el mismo nombre de directorio permanecen separados, y los enlaces simbólicos al mismo directorio se deduplican. Los lanzamientos del terminal y del editor apuntan explícitamente al espacio de trabajo del proyecto. Los editores que reutilizan un proceso/ventana existente pueden mantener el comportamiento propio del espacio de trabajo de esa aplicación.

Los espacios de trabajo abiertos antes de la actualización conservan sus ventanas existentes y nombres nativos. Abrir un proyecto después de la actualización utiliza su nueva identidad derivada de la ruta; cierre las ventanas de proyectos antiguos cuando sea conveniente. No se intenta ninguna asociación o movimiento automático porque un nombre antiguo no identifica de manera confiable al directorio. Con nombres de directorios duplicados, use el selector, un número o la ruta canónica completa en lugar de un nombre ambiguo.

Abrir un proyecto regenera `$XDG_RUNTIME_DIR/argvus/project-workspaces.json` de manera atómica a partir de la lista de proyectos configurada y las traducciones. Esto es metadato de sesión para etiquetas y descripciones emergentes de espacios de trabajo, no configuración; no lo edite. Vuelva a abrir un proyecto después de cambiar la lista o el idioma para actualizar sus etiquetas. Actualice tanto `argvus` como `argvus-waybar`, además de la configuración `argvus-taskbar`, para esta integración.

---
title: Contribuyendo
description: Límites de contribución para proyectos y documentación de ARGVUS.
---
Trace una característica a través de todos los repositorios consumidores antes de cambiarla. Preserve los contratos de estado compartido, la propiedad del paquete, la propiedad del servicio y ambos catálogos de localización.

`argvus-config` es el único escritor de `data/generated/`. Agregue o cambie un archivo de consumidor allí, en `crates/argvus-config/src/project.rs` y sus submódulos bajo `project/` (archivos de Hyprland en `project/hyprland_projection.rs`), no en un asistente de shell. Si una aplicación consumidora no puede leer `config.json`, agregue un adaptador en el paquete propietario y llámelo desde el orquestador después del commit canónico. Elimine `data/generated/`, ejecute `argvus config apply` y confirme que el árbol se reconstruyó byte a byte desde `config.json`. Valide la fuente, la carga del paquete y el comportamiento de la sesión en vivo por separado; una compilación de la fuente por sí sola no prueba que una sesión de Wayland instalada use los nuevos archivos.

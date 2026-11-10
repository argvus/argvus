---
title: Integración de Hyprland
description: Contratos de configuración e integración del compositor ARGVUS.
---
`argvus-hyprland` empaqueta `/usr/share/argvus/hyprland/config/hyprland.lua`, atajos de teclado y scripts auxiliares. `argvus-session` lanza Hyprland y aplica la configuración de compatibilidad de la sesión; `argvus-config` proyecta los archivos de tema, diseño, borde y espaciado de Hyprland en `data/generated/hypr/`, y `argvus-appearance` reconcilia el compositor sobre `hyprctl`; `argvus-display` proporciona sobreescrituras de monitor.

La ruta del paquete heredado no es parte del diseño instalado actual.

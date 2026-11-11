---
title: Componentes de UI
description: Pacotes de interface do ARGVUS.
---

Incluem `argvus-control-panel`, `argvus-taskbar` e `argvus-launcher`, e os subcomandos `argvus control-center` (auxiliar no `argvus`), `argvus taskbar-calendar`, `argvus widget-telemetry`, `argvus terminal`, `argvus system-monitor`, `argvus removable-devices`, `argvus focus` e `argvus clock`. Calendário, Focus e Clock usam o crate compartilhado `argvus-taskbar-popup` para posicionamento e dispensa via layer-shell; o Focus também expõe seu leitor de status Waybar pela rota em processo `argvus focus status`.

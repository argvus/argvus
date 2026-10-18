---
title: Calendário
description: Use o calendário da taskbar ARGVUS.
slug: pt/0.4.0/docs/user-guide/applications/calendar
---

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```

`argvus-taskbar-calendar` é o popup GTK4 com eventos locais, lembretes e importação/exportação ICS. O serviço é `argvus-taskbar-calendar.service`.

O calendário acompanha o tema ativo do ARGVUS, incluindo as variantes Sticky e Float. A mudança é aplicada pelo cache gerado do usuário; use `argvus taskbar-calendar reload` se um popup já aberto precisar de atualização explícita.

Quando aberto pelo módulo de data da taskbar, o popup fica preso à borda da barra: abre abaixo da taskbar no topo e acima da taskbar na parte inferior, alinhado à borda direita do módulo de data. Execuções diretas sem geometria da taskbar usam o ponteiro ou o fallback do monitor.

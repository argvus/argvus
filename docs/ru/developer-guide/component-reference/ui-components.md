---
title: Компоненты пользовательского интерфейса
description: Пакеты пользовательского интерфейса ARGVUS.
---
Пакеты пользовательского интерфейса включают `argvus-control-panel`, `argvus-taskbar` и `argvus-launcher`, а также подкоманды `argvus` `argvus control-center` (помощник в `argvus`), `argvus taskbar-calendar`, `argvus widget-telemetry`, `argvus terminal`, `argvus system-monitor`, `argvus removable-devices`, `argvus focus` и `argvus clock`. Календарь, Focus и Часы используют общий crate `argvus-taskbar-popup` для позиционирования layer-shell и его закрытия; Focus также предоставляет считыватель статуса Waybar через маршрут `argvus focus status` в процессе.

Их поведение для пользователей документируется по функциям; на этой странице указана принадлежность пакетов.

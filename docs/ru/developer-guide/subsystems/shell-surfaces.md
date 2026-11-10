---
title: Поверхности оболочки
description: Панель задач, панель управления и управление телеметрией.
---
`argvus-waybar` предоставляет исправленную `/usr/bin/waybar`. `argvus-taskbar` отвечает за действия на панели задач и UX; JSONC и CSS панели задач, которые он поставляет, являются проекциями `config.json`, а `argvus-config` записывает активный слой под `data/generated/waybar/` и зеркальную копию под `data/waybar/`. `argvus-control-panel` отвечает за карты Quickshell. `argvus-widget-telemetry` отвечает за дополнительную панель телеметрии и её блоковую семантику; его JSONC и CSS записываются в `data/waybar/` из встроенных по умолчанию при изменении внешнего вида, после чего `argvus-config` переписывает только выделенные `ARGVUS_TELEMETRY_*` и блоки шрифтов. `argvus-session` отвечает за их пользовательские сервисы.

В текущей модели выполнения нет пакета `argvus-shell` или `argvus-shell.service`.

---
title: Поверхности оболочки
description: Панель задач, панель управления и управление телеметрией.
---
`argvus-shell` предоставляет исправленную `Quickshell Taskbar`. `argvus-shell` отвечает за действия на панели задач и UX; JSONC и CSS панели задач, которые он поставляет, являются проекциями `config.json`, а `argvus-config` записывает активный слой под `data/generated/taskbar/` и зеркальную копию под `data/taskbar/`. `argvus-shell` отвечает за карты Quickshell. `argvus-shell` provides the optional Telemetry surface, while the `argvus widget-telemetry` command in `argvus` retains block-control semantics; его JSONC и CSS записываются в `data/taskbar/` из встроенных по умолчанию при изменении внешнего вида, после чего `argvus-config` переписывает только выделенные `ARGVUS_TELEMETRY_*` и блоки шрифтов. `argvus-session` отвечает за их пользовательские сервисы.

В текущей модели выполнения нет пакета `argvus-shell` или `argvus-shell.service`.

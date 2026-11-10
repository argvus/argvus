---
title: Пакет matrix
description: Обязанности пакета ARGVUS.
---

| Пакет | Ответственность |
| --- | --- |
| `argvus` | Основной пакет: диспетчер, команды в процессе выполнения (конфигурация, переводы, ключи, проекты, терминал, системный монитор, сеть, Bluetooth, телеметрия, TUI терминал, установка), общие данные и каталоги i18n. От него зависят все компоненты пакетов. |
| `argvus` (вспомогательные) | Вспомогательные элементы в том же пакете: Центр управления и Анализатор использования диска (терминальные приложения), а также GTK 4 приложения для календаря панели задач, съемных устройств, загрузочного оверлея, ARGVUS Focus и ARGVUS Clock. |
| `argvus` (greeter) | Программа приветствия для greetd, поставляется в том же пакете и собирается из `crates/argvus-greeter`. |
| `argvus-desktop` | Профиль рабочего стола и повседневных приложений, необходимый для основного пакета `argvus`. Он не зависит от `argvus`, поэтому зависимость остается односторонней. |
| `argvus-devtools`, `argvus-pentest` | Необязательные независимые метапакеты для инструментов разработки и уполномоченного аудита безопасности. Основной пакет не устанавливает эти профили. |
| `argvus-session` | Точки входа сессии и жизненный цикл пользователя. |
| `argvus-hyprland` | Конфигурация Hyprland. |
| `argvus-control-panel` | Панель управления Quickshell. |
| `argvus-taskbar`, `argvus-waybar` | Конфигурация Waybar и исправленный бинарный файл. |
| `argvus-branding` | Логотип и фирменные элементы. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Состояние внешнего вида и визуальные элементы. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | Провайдеры рабочего стола. |
| `argvus-lock`, `argvus-boot-splash` | Экран блокировки и заставка при загрузке. |
| `argvus-firewall` | Брандмауэр с собственными привилегиями. |
| `argvus-launcher` | Конфигурация лаунчера и точки входа. |
| `argvus-app-profiles` | Профили только для данных для комплектных терминальных приложений. |
| `argvus-portal` | Портал XDG и настройки по умолчанию для Wayland/DBus. |
| `argvus-theme-*` | Один дополнительный пакет на каждую официальную тему. |
| `argvus-game-snake`, `argvus-games` | Дополнительно. Snake — независимый проект; `argvus-games` объединяет дополнительные игры. |

## Заменённые пакеты

`argvus` объявляет `replaces`, `conflicts` и `provides` для пакетов, которые он поглотил, поэтому обновление удаляет их, и их файлы не остаются как сироты:

`argvus-config`, `argvus-i18n` (с `argvus-language-en-us` и `argvus-language-pt-br`), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` и `argvus-du`. `argvus` также заменяет `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices`, а также прежние разделенные пакеты `argvus-gtk` и `argvus-greeter`, которые он все еще предоставляет, чтобы пакеты, зависящие от них, продолжали разрешаться. Менеджер учетных записей (`argvus accounts`) собирается из `crates/argvus-accounts` и устанавливается как `/usr/lib/argvus/commands/accounts`, с его правилом polkit. Анализатор использования диска собирается из `crates/argvus-du` и устанавливается как `/usr/lib/argvus/commands/argvus-du`.

`/usr/bin/argvus-terminal` остаётся установленным как совместимый лаунчер для `argvus terminal`. Терминал по умолчанию в Hyprland, каталог Центра управления и сохранённые приложения по умолчанию по-прежнему называют эту команду.

Удалённые пакеты `argvus-pwvucontrol` и `argvus-snappy-switcher` не предоставляются никаким пакетом.

Очередность установки среди компонентов ARGVUS не требуется: `argvus` является их главным пакетом, и каждый компонент зависит от него. Установка `argvus` также устанавливает профиль `argvus-desktop`; `argvus-devtools` и `argvus-pentest` остаются по выбору пользователя. Наличие пакета в этой таблице не означает, что он владеет всей функцией: например, панель задач сочетает `argvus-waybar`, `argvus-taskbar`, `argvus-session` и провайдеров доменов.

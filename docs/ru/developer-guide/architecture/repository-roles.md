---
title: Роли репозитория
description: Ответственность исходных репозиториев ARGVUS после консолидации рабочего пространства.
---
Репозитории сгруппированы по ответственности, а не по навигационной странице:

- рабочее пространство: `argvus` (диспетчер, команды в процессе выполнения, помощники GTK 4, общие пакеты, приветствие в `crates/argvus-greeter` и данные команд);
- сессия и интеграция: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash` и приветствие (`argvus-greeter`, построено из `argvus/crates/argvus-greeter`);
- пользовательский интерфейс вне рабочего пространства: Панель управления, панель задач (`argvus-taskbar`), лаунчер и Waybar;
- провайдеры: отображение, питание, брандмауэр, уведомления;
- ресурсы и упаковка: внешность, обои, шрифты, иконки, профили.

## Консолидировано в рабочее пространство

Эти репозитории были объединены в `de/argvus`. Их документация теперь находится под `docs/` рабочего пространства, а их репозитории сохраняются только для истории:

| Бывший репозиторий | Сейчас | Документация |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [командная строка](/docs/reference/command-line/), [файлы конфигурации](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [компонент интернационализации](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | командная строка |
| `argvus-terminal` | `argvus terminal` | [компонент терминала](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [компонент системного монитора](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [компонент сети](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | командная строка |
| `argvus-control-center` | `argvus control-center` (терминальное приложение) | [компонент Центра управления](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (помощник GTK 4) | [Компонент календаря панели задач](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (помощник GTK 4) | [Компонент съемных устройств](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (помощник `splash`) | командная строка |
| `argvus-tui-terminal` | `argvus tui-terminal` (пакет на рабочем пространстве) | командная строка |
| `argvus-tui`, `argvus-theme` | Общие пакеты `crates/argvus-tui` и `crates/argvus-theme` | заметки разработчика |
| `argvus-greeter` | Исходники в `crates/argvus-greeter`, поставляется в пакете `argvus` | [Компонент приветствия](/docs/components/argvus-greeter/) |
| `argvus-themes` | Выведен из эксплуатации. Каждая официальная тема имеет собственный пакет `argvus-theme-*`. | [установка тем](/docs/user-guide/appearance/) |

Примечания разработчика по объединённым компонентам находятся в `docs/internal/<repository>/`.

Язык не является границей владения: каждый компонент сохраняет свои вспомогательные элементы времени выполнения вместе с пакетами. Единственное твёрдое правило, которое распространяется на все пакеты: только `argvus-config` записывает данные ниже `data/generated/`. Компонент, которому нужно обращаться к приложению, которое не может читать `config.json`, добавляет адаптер в свой собственный вспомогательный модуль и позволяет оркестратору вызвать его после канонического коммита; он никогда не записывает сгенерированный файл. Публичные CLI должны находиться в `/usr/bin`; внутренние вспомогательные элементы располагаются в `/usr/lib/argvus/<component>` или в директории commands, когда они нужны во время выполнения; данные, независимые от архитектуры, остаются в `/usr/share/argvus/<component>`.

См. [package matrix](/docs/reference/package-matrix/) для границы установленного пакета.

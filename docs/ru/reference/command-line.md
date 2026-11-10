---
title: Командная строка
description: Команда argvus, её подкоманды, флаги, используемые для привязки клавиш, и где выполняется каждая из них.
---
`/usr/bin/argvus` является публичной точкой входа ARGVUS. Это одна программа на Rust, собранная из crates `argvus-cli`. Каждая подкоманда либо выполняется внутри этой программы, либо запускает вспомогательный исполняемый файл из директории команд, либо запускает упакованный скрипт.

Запустите `argvus --help` для списка, установленного с вашей версией. Подкоманды записываются без ведущего `--`. Формы `--` сохраняются только там, где они используются в привязке клавиш или записи на рабочем столе.

## Где выполняется каждая команда

- **В процессе:** команда является библиотекой рабочего пространства и выполняется внутри `argvus`. Отдельного исполняемого файла не существует. Это `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`, `tui-terminal`, `sensors` и `setup`.
- **Помощник:** отдельный исполняемый файл в `/usr/lib/argvus/commands/`, устанавливаемый пакетом `argvus`. Это `control-center` (терминальное приложение), `argvus-du` (запущено с помощью `argvus du`) и приложения GTK 4 `taskbar-calendar`, `removable-devices`, `focus` и `clock`.
- **Частный исполняемый файл:** исполняемый файл, который устанавливается другим пакетом в `/usr/lib/argvus/commands/`. `argvus` только запускает его. К ним относятся `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` и `greeter-*` точки входа.
- **Скрипт:** упакованный сценарий оболочки, запускаемый с помощью `/bin/sh`. Это команды сценария Hyprland, перечисленные ниже.

`ARGVUS_COMMANDS_DIR` изменяет каталог команд для тестов и проверок `DESTDIR`. Частный исполняемый файл, которого там нет, ищется на `PATH`.

## Подкоманды

| Команда | Бежит | Заметки |
| --- | --- | --- |
| `argvus config ...` | В процессе | Модель конфигурации и инструменты. Также доступно через `theme-switch` и `effects-toggle`. |
| `argvus theme ...` | В процессе | `apply ID` — это переключатель темы (такой же, как `theme-switch`). Сборки тем: `list` установленные темы, `check DIRECTORY` тема для установленных шаблонов, `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`. См. [Внешний вид и темы](/docs/developer-guide/subsystems/appearance-and-themes/)]. |
| `argvus i18n ...` | В процессе | Переводы: `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`. |
| `argvus keys ...` | В процессе | Менеджер SSH-ключей. `gpg` — это псевдоним для подкоманд GPG. |
| `argvus gpg ...` | В процессе | GPG-ключи: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | В процессе | Выбор проекта. |
| `argvus terminal ...` | В процессе | Профиль терминала ARGVUS. |
| `argvus system-monitor ...` | В процессе | Системный монитор btop; `--gpu` выбирает только GPU-боксы, а `--panel [--cpu|--mem|--gpu]` внедряет его в оболочку ARGVUS TUI. |
| `argvus du` | Помощник (терминальное приложение) | Открывает ARGVUS Disk Usage Analyzer в отдельном окне Kitty и запускает ncdu. |
| `argvus network ...` | В процессе | Статус и управление NetworkManager (`networkctl` сохранён как псевдоним). |
| `argvus bluetooth ...` | В процессе | Статус и управление Bluetooth (`bluetoothctl` сохранён как псевдоним). |
| `argvus widget-telemetry block ...` | В процессе | Блок телеметрии Waybar. Это горячая зона панели. |
| `argvus widget-telemetry toggle ...` | В процессе | Переключает блоки телеметрии и их порядок. |
| `argvus game-snake` | Частное исполняемое | Игра «Змейка» в терминале из независимого проекта `argvus-game-snake`, опциональный пакет. За пределами терминала она открывает один. |
| `argvus control-center [page]` | Помощник (терминальное приложение) | Открывает Центр управления или фокусирует окно, которое уже открыто. |
| `argvus welcome [--if-needed]` | Диспетчер → Центр управления | Открывает встроенную страницу приветствия в Центре управления. `--if-needed` открывается один раз, если записи о прогрессе пользователя не показывают, что страница приветствия уже была показана, завершена или пропущена. |
| `argvus taskbar-calendar [args]` | Приложение GTK 4 | По умолчанию `toggle`. |
| `argvus removable-devices [args]` | Приложение GTK 4 | По умолчанию `menu`. |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Приложение GTK 4 | Всплывающее окно фокуса ARGVUS. Один экземпляр за сессию. |
| `argvus focus status --waybar [--follow]` | В процессе | Отправляет индикатор панели задач Focus в формате Waybar JSON; `--follow` транслирует изменения без загрузки GTK. |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Приложение GTK 4 | Всплывающее окно часов ARGVUS. Один экземпляр на сессию; `--background` сохраняет активность будильников без всплывающего окна. |
| `argvus loading-theme ...` | Упакованный `splash` в `/usr/lib/argvus/loading-theme/splash` | Наложение прямых перезагрузок сессии; смена темы его не использует. |
| `argvus session`, `start`, `tty`, `sessionctl` | Частные исполняемые файлы | Точки входа сессии и контроллер сессии. |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | Частные исполняемые файлы | Точки входа приветственного экрана. |
| `argvus accounts`, `firewall` | Частные исполняемые файлы | Инструменты для управления учетными записями и файерволлом. |
| `argvus launcher`, `snippets`, `ssh`, `git` | Приватные исполняемые файлы | Инструменты запуска и выбора. |
| `argvus tui-terminal` | В процессе | Терминальная оболочка для TUI-приложений. |
| `argvus sensors` | В процессе | Обзор текущей температуры из `sensors -j`. |
| `app-profiles-yazi-keymap` | Приватный исполняемый файл | Помощник по раскладке клавиш. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | Приватные исполняемые файлы | Уведомления, действия с дисплеем и панелью задач. |
| `argvus setup [options]` | В процессе | Копирует упакованные настройки по умолчанию в конфигурацию пользователя. См. ниже. |

Аргументы передаются без изменений, включая пробелы и пустые значения. Стандартный ввод, вывод, ошибки, сигналы и статус выхода относятся к выполняемой команде. Неизвестное имя вызывает ошибку с подсказкой выполнить `argvus --help`. Команда, выполняющаяся в процессе, отвергает аргумент, который не является допустимым UTF-8 (статус выхода 2), вместо того чтобы передавать изменённое значение; команды, запускающие отдельную программу, получают байты без изменений.

## Флаги и псевдонимы

Эти имена существуют для того, чтобы сочетания клавиш, записи в рабочем столе и старые сценарии продолжали работать. В новом коде предпочтительнее использовать форму с подкомандой.

| Флаг или псевдоним | Эквивалент |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (с страницами `about` и `apps`) |
| `--displays`, страница `--monitors` | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | файловый менеджер (`spf`) |
| `--yazi`, `--yazy` | файловый браузер (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | запускается `tty` (аргументы не пересылаются) |
| `--version`, `-V`, `version` | принты `ARGVUS <version>` |
| `help`, `--help`, `-h` | печатает использование |
| `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

Центр управления принимает флаги устаревших страниц в качестве первого аргумента и сопоставляет их с именами страниц. Примеры: `--apps` становится `apps`, `--keyboard-layout` становится `locale`, а `--hostname` становится `system`.

## Команды сценария Hyprland

Эти упакованные скрипты запускаются и используются конфигурацией Hyprland:

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`.

Скрипт в конфигурации пользователя переопределяет упакованный. Порядок поиска такой же, как у `_config_path` в `hyprland.lua`: копия конфигурации пользователя, затем копия данных ARGVUS, затем копия состояния, затем сгенерированная копия, и только потом упакованный файл под `/usr/share/argvus`.

## Примеры

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```
## Настройка

`argvus setup` копирует упакованные настройки из `/usr/share/argvus/<app>` в `~/.config/argvus/data/<app>`, чтобы вы могли их настроить. Он никогда не редактирует упакованные файлы.

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```
Уже существующее место назначения сообщается и остаётся без изменений, если не задан `--force`. При использовании `--force` существующий каталог переименовывается в `<name>.bak-<timestamp>` перед копированием.

Приложение `argvus` особое: оно копирует содержимое своего упакованного каталога в корень конфигурации. Поскольку корень конфигурации уже существует во время запуска установки, `--copy argvus` в текущем коде требует `--force`.

Установка проверяет каждый параметр перед тем, как что-либо изменить. Недопустимые имена приложений (пустые, начинающиеся с `.` или содержащие `/`) отклоняются.

## Сессия и перезагрузка

`SUPER + SHIFT + R` запускает `argvus sessionctl reload`. Контроллер сессии применяет созданную конфигурацию и перезапускает только те компоненты, которыми он управляет. Он не копирует `/usr/share/argvus` в домашний каталог и не заменяет некорректный JSON-файл.

## Ночной свет

Откройте **Рабочий стол и ввод > Дисплеи > Ночной свет** или выполните `argvus control-center displays night-light`. Настройте температуру, ручное включение и фиксированные локальные расписания через `argvus config display night-light`. См. [Ночной свет](/docs/user-guide/night-light/) для команд, состояния и требований к выполнению.

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` запускает интерактивную команду внутри общего терминального фрейма ARGVUS. Ctrl+E закрывает фрейм; его вызывающий элемент предоставляет локализованный нижний колонтитул. Используйте это внутри терминала или как команду, переданную в `argvus tui-terminal --term kitty -- ...`.

`argvus tui-terminal --term kitty --profile ai-agents` использует специальную конфигурацию Kitty с скрытыми вкладками и только комбинациями для буфера обмена. Вызывающий элемент задает команду и заголовок; размещение окна обрабатывается правилом Hyprland для класса `argvus-ai`.

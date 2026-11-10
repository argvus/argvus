---
title: Расположения файлов
description: Важные установленные файлы и каталоги ARGVUS.
---

- `/usr/bin/argvus*` — общедоступные команды.
- `/usr/share/argvus/hyprland/` — конфигурация Hyprland и помощники.
- `/usr/share/argvus/control-panel/` — панель управления QML и скрипты.
- `/usr/share/argvus/taskbar/` — конфигурация панели задач и скрипты.
- `/usr/share/argvus/lock/` — конфигурация Hyprlock.
- `/usr/share/argvus/portal/` — файлы интеграции портала.
- `/usr/share/backgrounds/argvus/` — ресурсы обоев.
- `/etc/argvus/` — конфигурация системного провайдера.
- `/etc/argvus/greeter.toml` — конфигурация приветственного экрана.
- `/etc/argvus/removable-devices/config.json` — системные настройки для сменных носителей.
- `$XDG_CONFIG_HOME/argvus/config.json` — стандартные переносимые настройки Canonical.
- `$XDG_CONFIG_HOME/argvus/data/` — данные управляемых компонентов, резервные копии, блокировки и сгенерированный вывод.
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — необязательные полные нативные замены панели задач Waybar.
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — управляемые копии панели задач ARGVUS.  
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — сгенерированные профили Waybar, включая `argvus-taskbar.css` и `argvus-widget-telemetry.{jsonc,css}`; они имеют приоритет над вышеуказанными копиями.  
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — также зеркально отображены под `data/waybar/`, который заменяется на стандартную версию из пакета при изменении внешнего вида и затем его блоки `ARGVUS_TELEMETRY_*` и шрифтов переписываются.  
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — проецируемое дерево конфигурации Yazi с резервной схемой, учитывающей варианты для тем, которые не содержат Yazi-версии.  
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — проецируемый набор тем Superfile.  
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — проецируемые профили терминала.
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — проектируемая палитра Qt6ct.  
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — проектируемые файлы темы GTK.  
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — пользовательские копии конфигурации Rofi, тем и режима. ARGVUS создаёт каждую копию при первом изменении файла; неизменённые файлы читаются из `/usr/share/argvus/launcher/config/`.  
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — проектируемый стиль для съёмных устройств.  
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — настройки ARGVUS Focus, история сеансов и контрольная точка текущего сеанса.  
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — эфемерный статус ARGVUS Focus, используемый панелью задач; удаляется при выходе Focus.
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — настройки часов ARGVUS и мировые часы, будильники и контрольные точки таймера/секундомера.  
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — цвета ARGVUS Focus и ARGVUS Clock (тема-потребитель `time`).  
- `/usr/share/argvus/sounds/` — звуки ARGVUS Focus и ARGVUS Clock (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`).  
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — сборки тем (`builds/<id>-<variant>-<hash>/`), написанные `argvus theme render`, а также при каждой смене темы и изменении акцента, и символьная ссылка `current` на сборку активной темы.  
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — пользовательские переопределения Hyprland на Lua и нативные проекты Hyprland.
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — необязательная полная родная конфигурация Hyprland, выбранная `argvus start`; это не инкрементальный оверлей.  
- `$XDG_STATE_HOME/argvus/config-projection.json` — манифест проекции: генерация, хэш эффективной конфигурации, измененные разделы и ожидаемые задачи во время выполнения. Читается `argvus sessionctl reload`; никогда не является источником истины.  
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — устаревшие предпочтения блока телеметрии. Только для миграции; канонические значения находятся в `config.json`, и этот файл больше не записывается.  
- `$XDG_STATE_HOME/argvus/session.log` — журнал графической сессии (обычно `~/.local/state/argvus/session.log`).  
- `/usr/share/wayland-sessions/argvus.desktop` — запись графической сессии.  
- `/usr/share/plymouth/themes/argvus/` — упакованные ресурсы загрузочного сплеша.  

Точные полезные данные зависят от пакета; см. справку по компонентам для разработчиков и соответствующий манифест пакета.

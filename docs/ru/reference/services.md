---
title: Услуги
description: Системные и пользовательские службы ARGVUS.
---
Основной целевой пользователь — `argvus-session.target`. Общие пользовательские сервисы включают `argvus-taskbar.service`, `argvus-control-panel.service`, `argvus-widget-telemetry.service`, `argvus-hyprland.service`, `argvus-dunst.service`, `argvus-wallpaper.service`, `argvus-hypridle.service`, сервисы буфера обмена и `argvus-session-loading.service`.

Межсетевой экран — это системный сервис с именем `argvus-firewall.service`. Пакет календаря устанавливает `argvus-taskbar-calendar.service` как пользовательскую единицу, а `argvus-clock.service` (по умолчанию отключена) запускает ARGVUS Clock скрытно после входа в систему, когда включена опция **Запускать вместе с сеансом**.

Проверяйте активные единицы с помощью:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```
## Сессионные юниты

Пакет сессии устанавливает эти пользовательские юниты под `/usr/lib/systemd/user` (исходный payload находится в общем дереве конфигурации пакета сессии):

| Юнит | Роль |
| --- | --- |
| `argvus-session.target` | Граница времени жизни для графической сессии ARGVUS. |
| `argvus-session-prepare.service` | Одноразовая подготовка запуска и сгенерированная конфигурация. |
| `argvus-control-panel.service` | Панель управления Quickshell. |
| `argvus-taskbar.service` | Основная панель задач ARGVUS. |
| `argvus-widget-telemetry.service` | Опциональная поверхность телеметрии/системной информации. |
| `argvus-hyprland.service` | Одноразовый юнит по требованию, который запускает `hyprctl reload`. |
| `argvus-config.service` | Проецирует каноническую конфигурацию (`argvus config project`); перезагрузка запускает `argvus sessionctl reload-config-transaction`. |
| `argvus-session-reload.service` | Перезагрузка входа (`argvus sessionctl reload-desktop --login`): перезагружает Hyprland только когда проекция на него влияет. |
| `argvus-ssh-agent.service` | SSH-агент для сессии. |
| `argvus-secret-service.service` | Запускает службу секретов через `argvus sessionctl start-secret-service`. |
| `argvus-wallpaper.service` | Активный бэкенд обоев. |
| `argvus-dunst.service` | Демон уведомлений Dunst. |
| `argvus-hypridle.service` | Политика простоя и блокировки. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | Наблюдатели истории буфера обмена. |
| `argvus-keyboard-layout.service` | Уведомления раскладки клавиатуры. |
| `argvus-polkit.service` | Интеграция агента PolicyKit. |
| `argvus-session-loading.service` | Наслаивание загрузки сессии от передачи композитору до запуска компонентов рабочего стола. |

`argvus-blueman-applet.service` также отправляется с пакетной сессией. Некоторые элементы являются необязательными или активируются условно; они не все активны в каждой сессии.

Чтобы проверить фактическое определение элемента, а не только его исходный код:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```
Системный уровень `argvus-firewall.service` принадлежит `argvus-firewall`. Пакет календаря отдельно поставляется как пользовательский модуль `argvus-taskbar-calendar.service`. `argvus-clock.service` является пользовательским модулем пакета `argvus`, `WantedBy=graphical-session.target`, включенным через настройки часов ARGVUS.

## Команды управления сессией

Команда `argvus sessionctl` управляет жизненным циклом сессии и может перезапускать конкретные компоненты без выхода из системы:

### Жизненный цикл сессии

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```
### Перезапуск отдельных компонентов

Перезапускайте отдельные службы, не влияя на всю сессию:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```
### Синхронизация окружения

Импортируйте переменные окружения в сессию:

```sh
argvus sessionctl import-environment
```
Это синхронизирует переменные Wayland, Hyprland, Qt, курсора и DBus между оболочкой и пользовательскими службами systemd.

## Справка по юнитам службы

Дополнительные пользовательские юниты могут управляться напрямую с помощью `systemctl --user`:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```
Журналы для начальной загрузки сессии сохраняются по адресу:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```

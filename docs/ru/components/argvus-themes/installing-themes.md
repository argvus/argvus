---
title: Установка тем
description: Установите официальные пакеты темы ARGVUS с помощью pacman.
---
Официальные темы публикуются как пакеты Arch Linux. Каждая тема является отдельным пакетом с именем `argvus-theme-<THEME_NAME>`.

## Где получить темы

Источник каждой официальной темы находится в организации проекта ARGVUS:

- Каждая тема — это репозиторий с именем `argvus-theme-<THEME_NAME>` (например, `argvus-theme-dracula`), построенный как пакет с тем же именем.

Пакеты тем зависят от `argvus-appearance`. Сначала установите ARGVUS, как описано в [Installation](/docs/getting-started/installation/), чтобы компонент внешнего вида был присутствующим.

## Установка одной темы

Чтобы установить сразу все официальные темы, установите метапакет `argvus-themes`:

```bash
sudo pacman -S argvus-themes
```
Это зависит от всех опубликованных пакетов `argvus-theme-*`. ARGVUS Dark и ARGVUS Light уже поставляются `argvus-appearance`.

Установите одну тему с её именем пакета:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```
Например:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```
Пакет устанавливает манифест темы `/usr/share/argvus/themes/<theme-id>/theme.toml` (её имя, категорию, обои и палитру, с опциональным `overrides/`), а также обои темы под `/usr/share/backgrounds/argvus/`. Файлы каждого компонента формируются из палитры при применении темы. Он не записывает данные в ваш домашний каталог и не изменяет вашу текущую тему. Выберите её через Центр управления после установки, см. [Официальные темы](/docs/components/argvus-themes/official-themes/).

## Установка нескольких тем

Не существует мета-пакета, который устанавливает все темы. Установите нужные вам темы одной командой:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## Доступные пакеты

Ниже представлены официальные пакеты тем, опубликованные сегодня.

**Темные темы**

| Пакет | Тема |
| --- | --- |
| `argvus-theme-one-dark` | Одна тьма |
| `argvus-theme-dracula` | Дракула |
| `argvus-theme-silver-dark` | Серебряный Темный |
| `argvus-theme-slate-dark` | Сланцевый темный |
| `argvus-theme-universe` | Вселенная |
| `argvus-theme-gruvbox-high-dark` | Грувбокс Хай Темный |
| `argvus-theme-gruvbox-dark` | Грувбокс Темный |
| `argvus-theme-rose-pine` | Розэ Пайн |
| `argvus-theme-tokyo-night` | Токио Ночь |
| `argvus-theme-solitude` | Одиночество |
| `argvus-theme-sunset` | Закат |
| `argvus-theme-matrix` | Матрица |
| `argvus-theme-monokai-dark` | Монокай Темный |

**Светлые темы**

| Пакет | Тема |
| --- | --- |
| `argvus-theme-github-light` | GitHub Свет |
| `argvus-theme-solarized-light` | Соляризованный свет |
| `argvus-theme-nord-light` | Норд Лайт |
| `argvus-theme-one-light` | Один свет |
| `argvus-theme-everforest-light` | Эверфорест Лайт |
| `argvus-theme-catppuccin-latte` | Катпучин Латте |
| `argvus-theme-gruvbox-light` | Грувбокс Лайт |
| `argvus-theme-watercolor` | Акварель |

Встроенные темы **ARGVUS Dark** и **ARGVUS Light** не требуют отдельного пакета; они идут в комплекте с `argvus-appearance`.

## Проверка установленных тем

Список установленных пакетов тем можно вывести с помощью:

```bash
pacman -Qs argvus-theme
```
Откройте Панель управления и перейдите в **Внешний вид → Темы → Официальные темы**, чтобы увидеть темы, которые в настоящее время доступны в селекторе. Тема, которая установлена, но отсутствует в этом списке, является проблемой упаковки; см. раздел устранения неполадок в руководстве [Themes](/docs/components/argvus-themes/).

## Добавление собственной темы

Тема — это манифест, поэтому собственная тема не требует пакета. Создайте `~/.local/share/argvus/themes/<id>/theme.toml`, где имя директории — это ID темы (строчные буквы, цифры и одинарные дефисы):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```
Только `bg`, `fg` и `accent` обязательны: каждый другой цвет имеет значение по умолчанию, производное от них, и манифест определяет его только для изменения. Проверьте это, затем примените, как любую другую тему:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```
`argvus theme list` показывает ваши темы рядом с официальными (официальная тема с тем же ID имеет приоритет). Ваши темы появляются в меню тем (`SUPER + Shift + T`); приветствие перед входом не может прочитать ваш домашний каталог, поэтому оно сохраняет последнюю применённую официальную тему, пока активна одна из ваших тем.

Копии на тему, которые старые версии ARGVUS записывали под `~/.config/argvus/data/<app>/themes/`, больше не читаются. Первый переключение темы после обновления перемещает их в `~/.config/argvus/data/backups/theme-copies-<time>/`; чтобы сохранить файл, который вы редактировали, поместите его в вашу собственную тему под `overrides/` и укажите его в таблице `[overrides]` манифеста.

## Удаление темы

Удалите один пакет темы с помощью:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```
Удаление пакета тем не удаляет ваши импортированные профили. Если удаляемая тема является текущей применяемой, выберите другую тему в Центре управления перед её удалением.

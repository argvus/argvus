---
title: Счета
description: Локальные пользовательские профили, отображаемые имена, аватары и членство в группах через учетные записи argvus.
---
`argvus accounts` перечисляет и редактирует локальные учетные записи пользователей рабочего стола ARGVUS. Это приватный исполняемый файл из пакета `argvus`, установленного в `/usr/lib/argvus/commands/accounts`. Команда `argvus` запускает его, и только операции, которые изменяют другие учетные записи или системные данные, выполняются с правами root.

## Команды

```text
argvus accounts list [--all]                   list users (--all includes system accounts)
argvus accounts show USER                      show account details
argvus accounts name USER "DISPLAY NAME"       change another user's display name
argvus accounts avatar USER IMAGE              set another user's avatar
argvus accounts avatar USER --remove           remove another user's avatar
argvus accounts groups USER                    list groups
argvus accounts groups USER --add G --remove G change groups (authorization required)
argvus accounts passwd USER                    change a password (reads three lines from stdin)
argvus accounts self                           show your own account
argvus accounts self name "DISPLAY NAME"       change your own display name
argvus accounts self avatar IMAGE              set your avatar
argvus accounts self avatar --remove           remove your avatar
argvus accounts self groups                    list your groups
argvus accounts --verbose ...                  print diagnostic information to stderr
```
## Отображаемые имена и аватары

Отображаемое имя хранится в стандартном поле GECOS. Имя пользователя Unix никогда не изменяется.

Аватар — это изображение в формате PNG, JPEG или WebP, размером до 20 МБ. Изображение масштабируется до 256×256 пикселей и сохраняется в формате PNG с режимом `0644` по адресу `~/.face`, в местоположении freedesktop, которое считывается приветственным экраном и большинством дисплейных менеджеров. Ваш собственный аватар записывается внутри вашего домашнего каталога и не требует авторизации.

## Авторизация

Только `list`, `show`, `self`, `self groups` и ваш собственный аватар работают без авторизации. Любое другое изменение требует подтверждения со стороны агента polkit рабочего стола. Правило ARGVUS (`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) определяет, кто должен аутентифицироваться:

- пользователь в группе администраторов (`wheel`, значение по умолчанию в Arch, или `sudo`) подтверждает с помощью своего собственного пароля;
- любой другой пользователь подтверждает с помощью пароля root.

| Операция | Без авторизации | С авторизацией |
| --- | --- | --- |
| Список пользователей, показать аккаунт, список групп | ✓ | ✓ |
| Изменить собственный аватар | ✓ | ✓ |
| Изменить собственное отображаемое имя | — | ✓ |
| Изменить собственный пароль | — | ✓ (после подтверждения текущего пароля) |
| Изменить имя, аватар, группы или пароль другого пользователя | — | ✓ |

Изменение собственного пароля сначала проверяет текущий пароль. Неправильный пароль отклоняется до появления запроса авторизации.

Нет необходимости запускать команду с `sudo`. Команда запрашивает авторизацию через polkit, когда это необходимо.

## Ограничения безопасности

- `passwd` считывает текущий, новый и подтверждающий пароли со стандартного ввода, по одному на строку, и никогда не принимает их как аргументы. Строка с текущим паролем может быть пустой, когда администратор сбрасывает пароль другого пользователя. Пример:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```
Интерактивно введите три строки и нажимайте Enter после каждой. Ввод отображается на терминале по мере набора.
- Редактирование учетной записи с UID ниже 1000 (и `nobody`) запрещено, так же как её блокировка, удаление и истечение срока действия пароля. Ваша собственная учетная запись не может быть заблокирована или удалена.
- Переименование или удаление группы с GID ниже 1000, или группы администраторов (`wheel`, `sudo`) запрещено; при этом состав группы может быть изменен, за исключением того, что вы не можете удалить себя из группы администраторов.
- Членство в администраторах — это группа `wheel` или `sudo`. Центр управления добавляет новых администраторов в `wheel`, если система поддерживает её, и в `sudo` в противном случае. Удаление собственного членства администратора через редактирование запрещено.

## Примеры

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the wheel group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```
## Устранение неполадок

- `pkexec was not found`: установите `polkit`. Пакет `argvus` зависит от него.
- Всплывающее окно запроса авторизации не появляется: проверьте, что в вашей сессии работает агент аутентификации polkit. Пакет `argvus` зависит от `hyprpolkitagent`, который его предоставляет.
- Ошибки `unix_chkpwd` при смене пароля: помощник предоставляется пакетом `pam`.
- Запустите `argvus accounts --verbose ...`, чтобы выводить шаги команды в stderr.

Для заметок разработчика о границе привилегий см. `docs/internal/argvus-accounts/`.

Когда администратор блокирует или удаляет учетную запись, настроенную для автоматического входа, поставщик учетной записи сначала отключает её запись автоматического входа в greetd. Если эта очистка не удается, операция с учетной записью отклоняется. Если блокировка или удаление позже не удаются, автоматический вход остается отключенным. Автоматический вход для другой учетной записи сохраняется; разблокировка учетной записи не включает автоматический вход. Существующие сеансы не закрываются и greetd не перезапускается.

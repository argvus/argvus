---
title: Первая сессия
description: Запустите ARGVUS после установки.
---
Для графического входа настройте интеграцию greetd, поставляемую в комплекте, с `argvus greeter-setup --help`, и используйте поток дисплей-менеджера, описанный в [Sessions](/docs/user-guide/sessions/).

Для сессии TTY войдите в виртуальный терминал и выполните:

```sh
argvus --start-desktop
```
После входа в систему используйте `argvus sessionctl status`, чтобы проверить целевого пользователя ARGVUS. Если компоновщик или оболочка не запускаются, следуйте [инструкции по устранению неполадок с сессией](/docs/user-guide/troubleshooting/session-startup/).

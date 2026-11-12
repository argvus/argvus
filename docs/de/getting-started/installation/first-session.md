---
title: Erste Sitzung
description: Starten Sie ARGVUS nach der Installation.
---
Für ein grafisches Login konfigurieren Sie die bereitgestellte greetd-Integration mit `argvus greeter-setup --help` und verwenden den Display-Manager-Flow, der in [Sessions](/docs/user-guide/sessions/) beschrieben ist.

Für eine TTY-Sitzung melden Sie sich an einem virtuellen Terminal an und führen aus:

```sh
argvus --start-desktop
```
Nach dem Login verwenden Sie `argvus sessionctl status`, um das ARGVUS-Benutzerziel zu überprüfen. Wenn der Compositor oder die Shell nicht startet, folgen Sie [Sitzungsproblembehandlung](/docs/user-guide/troubleshooting/session-startup/).

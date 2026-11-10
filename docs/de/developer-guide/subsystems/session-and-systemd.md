---
title: Sitzung und systemd
description: Das ARGVUS-Sitzungsziel und seine Benutzerdienste.
---
Die Einheitendefinitionen werden von `argvus-session` unter `/usr/share/argvus/session/config/systemd/user/` verpackt. Der Sitzungsmanager ist für den Lebenszyklus und den Umgebungsimport verantwortlich. Komponenten-Repositorys besitzen die von ihren Einheiten ausgeführten Befehle.

Verwenden Sie `argvus sessionctl`, anstatt eine Komponente manuell zu starten, wenn Sie die integrierte Sitzung diagnostizieren.

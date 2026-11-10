---
title: Anbieter und Dienstleistungen
description: Anbieterpakete und ihre Systemschnittstellen.
---
Anbieterprojekte stellen fokussierte Schnittstellen für UI-Nutzer bereit:

- `argvus network` und `argvus bluetooth` integrieren NetworkManager- und Bluetooth-Hilfsprogramme;
- `argvus-display` integriert den Monitorzustand mit Hyprland;
- `argvus-power` integriert Leerlauf-, Sperr- und Energieaktionen;
- `argvus-firewall` verwaltet `argvus-firewall.service`;
- der `argvus removable-devices`-Helfer in `argvus` nutzt UDisks2;
- der `argvus control-center`-Helfer in `argvus` nutzt Provider-APIs und System-D-Bus-Schnittstellen.

Das Kontrollzentrum ist ein Client dieser Anbieter, nicht deren Ersatz.

---
title: Anforderungen
description: Systemanforderungen und Voraussetzungen für ARGVUS.
---
ARGVUS richtet sich an Arch Linux und erfordert einen Wayland-fähigen Grafik-Stack, Hyprland und die Laufzeitabhängigkeiten, die durch die Pakete bereitgestellt werden. Ein Display-Manager ist optional: ARGVUS unterstützt einen grafischen Login über greetd und einen manuellen TTY-Flow.

Vor der Installation stellen Sie sicher, dass:

- das System vollständig mit `pacman` aktualisiert ist;
- ein funktionierender Kernel und Grafikkartentreiber installiert sind;
- der Benutzer eine Wayland-Sitzung starten kann;
- Netzwerk verfügbar ist, wenn das Paket-Repository konfiguriert ist.

Virtuelle Maschinen benötigen möglicherweise die Kompatibilitätseinstellungen, die in [Virtuelle Maschinen](/docs/user-guide/virtual-machines/) beschrieben sind.

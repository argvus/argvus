---
title: Verpackung
description: Erstellen und überprüfen Sie ARGVUS-Arch-Pakete.
---
Jede Komponente besitzt ihr Arch-Paket unter ihrem Repository. Untersuche `packaging/archlinux/`, die Paketfunktionen und die installierte Dateiliste, bevor du ein Payload dokumentierst. Paketstandards unter `/usr/share/argvus` und Administrator-Konfiguration unter `/etc/argvus` sind nicht austauschbar.

Für die Laufzeitüberprüfung vergleiche den Paketinhalt mit `pacman -Ql <package>` und überprüfe aktive Benutzerüberschreibungen separat.

Lokale Builds halten die Cargo-Objekte in `build/cargo-target`, außerhalb des Paketquellbaums, sodass eine wiederholte `make build` nur das neu kompiliert, was sich geändert hat. `make clean` entfernt dieses Verzeichnis, und der nächste Build startet kalt.

Leite das Runtime-Payload nicht allein aus dem Repository-Namen ab. Ein Paket kann Binärdateien, Benutzerdienste, `/usr/share/argvus`-Standards, `/etc/argvus`-Administrator-Konfiguration oder Assets besitzen. Bestätige das tatsächliche Payload mit dem `PKGBUILD` und `pacman -Ql` des Repositorys nach der Installation.

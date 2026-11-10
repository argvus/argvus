---
title: Virtuelle Maschinen
description: Führen Sie ARGVUS mit virtueller Grafikhardware aus.
---
`argvus start` erkennt Virtualisierung und exportiert Kompatibilitätseinstellungen für betroffene Sitzungen, einschließlich Software-Rendering und Cursor-/Modifier-Fallbacks, wenn dies durch die erkannte Umgebung erforderlich ist.

Wenn eine virtuelle Maschine immer noch nicht rendern kann, überprüfen Sie die Sitzungsumgebung und die Protokolle, bevor Sie eine Überschreibung hinzufügen. Siehe [Grafik-Fehlerbehebung](/docs/user-guide/troubleshooting/graphics-and-virtual-machines/).

Wenn Virtualisierung erkannt wird, exportiert `argvus start` Kompatibilitätsvariablen für die Sitzung, einschließlich Software-Rendering, Software-Cursor und Modifier-Fallbacks. Dies macht den normalen Anmeldepfad zum ersten Versuch; fügen Sie keine dauerhafte Überschreibung hinzu, nur weil der Desktop in einer VM läuft.

Wenn die Erkennung nicht ausreicht, verwenden Sie eine Benutzer-Hyprland-Überschreibung als diagnostischen Fallback:

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
Fügen Sie dies nur hinzu, wenn der VM-Grafikstack es erfordert:

```lua
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
Melden Sie sich nach dem Ändern der Sitzungsumgebung ab und wieder an. Bestätigen Sie den aktiven Wert mit `echo $LIBGL_ALWAYS_SOFTWARE`; Software-Rendering kann den Desktop nutzbar machen, die Leistung jedoch verringern.

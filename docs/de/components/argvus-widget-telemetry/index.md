---
title: Widget-Telemetrie
description: Die Waybar-Telemetrie-Blöcke und deren Umschaltung über das argvus Widget-Telemetrie.
---
Das Telemetrie-Widget ist die Systeminformation, die in der Taskleiste angezeigt wird (Header, Tasten, System, CPU und GPU, Speicher, Speicherplatz, Prozesse, Netzwerk und das Entwickler-Dashboard). Es befindet sich im Prozess im `argvus`-Paket: Es gibt keine Shell-Sammler.

## Befehle

```text
argvus widget-telemetry block BLOCK ...   prints the text of one block (used by the bar)
argvus widget-telemetry toggle ...        switches the blocks on and off, and their order
```
`block` ist der heiße Pfad der Leiste: Er wird bei jeder Aktualisierung aufgerufen. `toggle` ändert, welche Blöcke die Leiste anzeigt und bewegt das Scrollfenster.

Die Blöcke sind `header`, `keys`, `system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network` und `dev_dashboard`. Die Taskleiste verwendet auch die `cpu-temp`- und `gpu`-Sonden. Die GPU-Sonde akzeptiert `--present` (eine unterstützte GPU ist in DRM/sysfs sichtbar), `--available` (eine GPU und der Befehl btop stehen für das Panel zur Verfügung), `--available-temp` (eine zuordenbare Temperatur kann gelesen werden) und `--temp` (die höchste gültige Temperatur, oder `N/A`).

GPU-Temperaturen werden einem PCI-Gerät der GPU zugeordnet. Proprietäre NVIDIA-Ablesungen verwenden `nvidia-smi`/NVML-Bus-IDs; NVIDIA nouveau, AMD- und Intel-Ablesungen verwenden hwmon nur, wenn dessen sysfs-Gerätelink eindeutig dieser GPU zugeordnet ist. Die Taskleisten-Temperatur ist der maximal gültige Wert über die erkannten GPUs. Fehlende, ungültige oder mehrdeutige Ablesungen bleiben nicht verfügbar. Proprietäre NVIDIA-Temperaturen erfordern den Treiberanbieter; AMD- und Intel-Temperaturen hängen davon ab, dass der Kernel einen verbundenen hwmon-Sensor bereitstellt. Intel-Temperatur wird im GPU-Panel von btop 1.4.7 nicht bereitgestellt.

## Start mit der Sitzung

Die `argvus-widget-telemetry.service`-Benutzereinheit startet mit der Sitzung nur, wenn das Widget aktiviert ist. Der Hauptschalter ist `effects.widget_telemetry_enabled` in der Konfiguration, der standardmäßig aktiviert ist, sodass ein neues Konto das Widget beim Login startet.

Sobald der Schalter `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state` geschrieben hat, entscheidet diese Datei stattdessen. Nur eine erste Zeile von `enabled` startet das Widget. Die älteren Dateien `desktop-telemetry-state` und `sysinfo-state` werden weiterhin gelesen, wenn die aktuelle Datei fehlt.

Telemetry-Bezeichnungen sind in allen zehn unterstützten Sprachen kompakt. Technische Bezeichner wie CPU, GPU, IP, DE, WM und Kernel behalten ihre technische Bedeutung. Die Betriebszeit wird aus `/proc/uptime` ausgelesen und mit abgekürzten Einheiten angezeigt (zum Beispiel `1d 12h 49m`); sie wird nicht mehr in vollständige Wörter aus der Systemsprache erweitert. Hardwaremodellnamen und andere Anbieterwerte können weiterhin die Breite des Widgets beeinflussen.

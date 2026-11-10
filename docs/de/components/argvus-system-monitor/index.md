---
title: Systemmonitor
description: Öffnen Sie den ARGVUS-Systemmonitor.
---
Öffnen Sie den konfigurierten Monitor mit:

```sh
argvus --system-monitor
```
Der Unterbefehl `argvus system-monitor` bietet den nativen Einstiegspunkt und die btop-Integration. Die im Taskleistenbereich angezeigte Telemetrie ist ein separates Feature, das von `argvus widget-telemetry` bereitgestellt wird. Ein Klick auf die CPU-, Speicher- und GPU-Module öffnet btop innerhalb eines ARGVUS-TUI-Panels mit einem gebrandeten Header und einem `Esc Close`-Footer. Der GPU-Modus zeigt bis zu sechs GPU-Felder und keine CPU-, Speicher-, Netzwerk- oder Prozessfelder an. Drücken Sie `q` oder `Esc`, um das Panel zu schließen. Die untere Kitty-Registerkarte zeigt den lokalisierten `Press q to close`-Hinweis. Eigenständige `argvus system-monitor --cpu`-, `--mem`- und `--gpu`-Aufrufe starten btop direkt mit den ausgewählten Feldern.

Das GPU-Panel hängt von den Fähigkeiten der installierten btop-Version und den Geräteanbietern des Systems ab. Die NVIDIA-Überwachung benötigt GPU-Unterstützung in btop, die NVIDIA Management Library aus dem installierten NVIDIA-Treiber und Zugriff auf das Gerät. Die AMD-Überwachung verwendet ROCm SMI, das optional ist und möglicherweise nicht installiert ist. Intel-GPU-Daten werden aus sysfs gelesen und hängen vom Zugriff des Benutzers auf diese Schnittstellen ab; btop 1.4.7 zeigt die Intel-Auslastung, Leistung und Taktrate an, bietet jedoch keine Intel-Temperatur. NVIDIA nouveau und andere Treiber können eine GPU-Temperatur über mit dem Gerät verbundenes hwmon in der Taskleiste anzeigen, während die Unterstützung durch btop-Anbieter von den verfügbaren Treibern und Bibliotheken abhängt. ARGVUS ändert keine Geräteberechtigungen und führt btop nicht mit erhöhten Rechten aus. Wenn keine GPU angezeigt wird oder btop innerhalb von drei Sekunden kein GPU-Feld rendert, weil Unterstützung oder ein Anbieter nicht verfügbar ist, zeigt das Panel eine lokalisierte nicht verfügbare Nachricht an.

Der Monitor liest zuerst das ausgewählte Thema und den Akzent von `$XDG_CONFIG_HOME/argvus/config.json`. Seine generierten btop-Dateien behalten den abgeleiteten Cache-Zustand; `.active-theme` und `.accent-color` sind Kompatibilitäts-Fallbacks. Da btop seine eigene Themendatei behält, wird der Monitor durch die Thementransaktion (`argvus theme-switch`) als externer Adapter nach der Bestätigung der kanonischen Änderung abgeglichen; er ist kein Teil des generierten Baums.

## Laufzeitvertrag

- Das Thema und der Akzent stammen aus der kanonischen Konfiguration (` and ` and `). The files `). The files ~/.config/argvus/data/.active-theme und `.accent-color` sind Fallbacks). 
- Die generierte btop-Konfiguration und die Themes werden in `$XDG_CACHE_HOME/argvus/argvus-system-monitor` geschrieben. Sie sind abgeleiteter Zustand; bearbeiten Sie sie nicht. 
- Die Standardwerte sind unter `/usr/share/argvus-system-monitor/btop` installiert. 
- Die Modul-Shortcuts (`--mem`, `--cpu`, `--net`, `--proc`) werden nur als erstes Argument gelesen. Das Übergeben von btops eigenen `-c`/`--config` oder `--themes-dir` startet btop mit diesen Pfaden und schreibt keine Konfiguration. 
- `--panel [--cpu|--mem|--gpu]` bettet btop in die ARGVUS TUI-Shell ein. Dieser Modus wird von der Taskleiste verwendet; ohne `--panel` ersetzt der Monitor weiterhin sich selbst durch btop.

---
title: Temperatursensoren
description: Live-Temperaturübersicht unterstützt durch lm_sensors JSON.
---
`argvus sensors` öffnet das in Bearbeitung befindliche Ratatui-Panel, das vom Popup der CPU-Temperatur in der Taskleiste verwendet wird. Es führt `sensors -j` aus, gruppiert Messwerte nach den vom Hardware gemeldeten Chipnamen und -bezeichnungen und aktualisiert alle zwei Sekunden. Die GPU-Überwachung wird von `argvus system-monitor --gpu` übernommen.

Die Messwerte werden in Celsius angegeben. Das Panel zeigt `max`, `crit` und den Alarmzustand, wenn lm_sensors diese bereitstellt. Ein Messanzeiger wird nur angezeigt, wenn ein positiver Grenzwert verfügbar ist. Fehlende `lm_sensors`, Befehlsfehler, ungültiges JSON und leere Messwerte bleiben als wiederherstellbare Zustände sichtbar; die Abfrage wird im nächsten Zyklus erneut versucht.

Drücken Sie `q` oder `Esc`, um zu schließen. Die Taskleiste öffnet das Panel in Kitty über `argvus tui-terminal`; der lokalisierte untere Tab-Titel zeigt den Schließhinweis. Dieser Befehl besitzt die CPU-Temperaturdetails; das GPU-Temperatursymbol öffnet den GPU-Only-btop-Monitor. Das Taskleistenpaket hängt bereits von `lm_sensors` ab, daher sind kein separates Paket oder keine weitere Abhängigkeit erforderlich.

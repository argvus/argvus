---
title: Netzwerk und Bluetooth
description: Verwalten Sie NetworkManager und Bluetooth über ARGVUS.
---
Der `argvus network`-Unterbefehl meldet und steuert NetworkManager, und `argvus bluetooth` meldet und steuert den Zustand und die Verfügbarkeit des Bluetooth-Adapters. Beide laufen im Prozess innerhalb von `argvus`; `networkctl` und `bluetoothctl` bleiben als Aliase bestehen. Das Kontrollpanel und Waybar rufen diese Befehle auf.

Verwenden Sie `--help` bei einem der Befehle für die installierten Unterbefehle. Die Integration der Netzwerk- und Bluetooth-Benutzeroberfläche ist getrennt von optionalen erweiterten Managern wie Blueman.

Das Kontrollzentrum trennt den Netzwerkstatus von Konfigurationsseiten: **Netzwerk → Status** fasst den aktuellen Zustand zusammen, während Schnittstellen, WLAN, Ethernet, VPN, DNS, Proxy und Firewall separate Wege darstellen, wenn ihre Anbieter verfügbar sind. **Bluetooth → Zustand**, **Geräte** und **Koppeln** behandeln den Adapterzustand und Geräteaktionen. Diese Seiten führen anbieterunterstützte Systemaktionen aus; die Taskleiste und das Kontrollpanel bieten die Schnellstatus-/Aktionsoberfläche.

Netzwerk- und Bluetooth-Änderungen können eine Autorisierung erfordern und können sofort über NetworkManager oder den Bluetooth-Dienst wirksam werden. Sie sind nicht dasselbe wie das Ändern des ARGVUS-Taskleistenanzeigers.

Das Kontrollzentrum sendet Wi-Fi-Passwörter über die Standard-Eingabe mit `--ask` an nmcli; Passwörter sind von Prozessargumenten ausgeschlossen und werden aus zurückgegebenen Befehlsfehlern geschwärzt.

## Netzwerkverwaltung über die Befehlszeile

Verwenden Sie `argvus networkctl`, um das Netzwerk ohne grafische Benutzeroberfläche abzufragen und zu steuern:

```sh
# Show network status
argvus networkctl status

# Toggle networking on/off
argvus networkctl enable
argvus networkctl disable

# Wi-Fi control
argvus networkctl wifi on
argvus networkctl wifi off

# Open the network manager
argvus networkctl manager
```
## Bluetooth-Verwaltung über die Befehlszeile

Verwenden Sie `argvus bluetoothctl` für den Bluetooth-Adapter:

```sh
# Show the adapter state (key=value lines)
argvus bluetoothctl status

# Exit 0 when an adapter is available, 1 otherwise
argvus bluetoothctl available

# Turn the adapter on or off, or toggle it
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Open the Bluetooth manager (Blueman)
argvus bluetoothctl manager
```
Wenn der Kernel keinen Controller in `/sys/class/bluetooth` auflistet, antwortet `status` sofort auf `status=no-controller`, ohne `bluetoothctl` zu fragen (was sonst auf BlueZ warten würde, wenn der Dienst gestoppt ist). Das Taskleisten-Modul und die Systemsteuerungs-Karten lesen denselben Status.

Verwenden Sie `--help` bei einem der Befehle, um zusätzliche Optionen und Unterbefehle zu sehen, die spezifisch für Ihre installierte Version sind.

Während das Wi-Fi-Passwortdialogfeld geöffnet ist, behalten Scan-Updates die ausgewählte SSID und den Sicherheitstyp bei. Wenn dieses Netzwerk verschwindet oder sich seine Sicherheit ändert, wird die Bestätigung mit einem Fehler abgebrochen und erfordert eine neue Auswahl.

Manuelles DNS sendet IPv4- und IPv6-Server an die jeweiligen NetworkManager-Einstellungen, bevor die Resolver-Datei aktualisiert wird. Dienstverwaltete Resolver-Symlinks werden durch die Ablehnung einer Dateiersetzung beibehalten. Eine erfolgreiche Rückkehr zum automatischen DNS stellt dies wieder her und entfernt die Sicherung, sodass ein späterer manueller Zyklus den aktuellen Resolver-Status speichert.

Das Schließen des Netzwerkprofil-Editors wartet, bis der aktuelle Vorgang abgeschlossen ist, sodass das Speichern oder Löschen eines Profils nicht durch das Verwerfen des Editors unterbrochen werden kann. Hintergrundaktualisierungen bleiben abbrechbar.

Wi-Fi-Verbindungsanforderungen teilen die Netzwerk-Aktionssperre: Eine laufende Aktion oder ein Profil-Editor-Vorgang wird abgeschlossen, bevor eine weitere Änderung gestartet werden kann. Fortschritts- und Abschlussmeldungen erfolgen in der gewählten Sprache.

Die Bestätigung der Bluetooth-Entfernung behält die ursprüngliche Geräteadresse bei, selbst wenn die Geräteliste aktualisiert wird. Batteriestände von bluetoothctl werden erkannt, und Abfragen zu Gerätedetails werden mit begrenzter Parallelität und individuellen Zeitlimits ausgeführt.

Wi-Fi-Passwort, manuelle DNS- und Netzwerksuchfelder akzeptieren eingefügten Text in Klammern. Eingefügte Steuerzeichen und Zeilenumbrüche werden weggelassen. Nicht unterstützte Tastenkombinationen mit nur Strg oder nur Alt fügen keine Buchstaben ein. Editoren akzeptieren druckbare AltGr-Ereignisse, die als Strg+Alt codiert sind; Strg+S speichert nur ohne Alt. Der gemeinsame Texteditor behält die vertikale Navigation in visuellen Spalten bei und richtet das horizontale Scrollen für breite Zeichen aus.

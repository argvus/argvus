---
title: Lokalisierung
description: Verwenden Sie ARGVUS auf Englisch oder Portugiesisch.
---
`argvus-i18n` stellt die gemeinsamen Kataloge, den Befehl `argvus i18n validate` und das QML-Modul `org.argvus.i18n` bereit. ARGVUS liefert derzeit englische und portugiesische Kataloge aus.

Native Anwendungen und Shell-Komponenten verwenden die gemeinsamen Kataloge. Ein übersetzter String kann daher von mehr als einem Repository verwendet werden.

Verwenden Sie **Control Center → Sprache & Region**, um die Sprache, die regionale Gebietsschema-Einstellung, die generierten Systemgebietsschemata, die Zeitzone, Datum/Uhrzeit und die Tastatur zu konfigurieren. Die aktuellen ARGVUS-Kataloge bieten Englisch und Portugiesisch; dies ist getrennt von der Liste der vom Betriebssystem generierten Systemgebietsschemata.

Das Ändern der Anzeigesprache speichert die ARGVUS-Einstellung und startet `argvus-control-panel.service` automatisch neu, sodass die Systemsteuerung die Änderung sofort widerspiegelt. Andere Anwendungen und Oberflächen der Shell benötigen möglicherweise einen Sitzungsladevorgang oder eine neue Sitzung. Das Erstellen oder Ändern von `/etc/locale.gen`-Einträgen erfordert eine Autorisierung und startet `locale-gen`; die Auswahl eines nicht generierten Gebietsschemas wird abgelehnt. Tastaturlayout, Variante und Konsolen-Tastaturbelegung sind separate Einstellungen und sollten nicht mit der Benutzerschnittstellensprache verwechselt werden.

## Wenn Umgebungsvariablen die Änderung überschreiben

ARGVUS-Anwendungen erkennen die Sprache zunächst aus der vorhandenen ARGVUS-Sprachdatei, dann aus `LC_ALL`, `LC_MESSAGES` und `LANG`. Lokale Umgebungsvariablen können daher dazu führen, dass die Systemsteuerung eine Sprachänderung scheinbar ignoriert, insbesondere wenn ein Benutzerdienst oder eine grafische Sitzung eine ältere Umgebung beibehält.

Überprüfen Sie die effektiven Werte mit:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```
Wenn `~/.config/locale.conf` existiert und `LANG`, `LC_ALL` oder `LC_MESSAGES` definiert, benennen Sie es vorübergehend um, damit diese Werte nicht erneut eingeführt werden. Erstellen Sie stattdessen eine Sicherungskopie, anstatt es zu löschen:

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```
Wenden Sie die Sprache erneut in **Systemsteuerung → Gebietsschema & Region** an. Die Systemsteuerung wird automatisch neu gestartet; wenn andere Komponenten weiterhin die vorherige Sprache anzeigen, melden Sie sich ab und wieder an, damit die grafische Sitzung und `systemd --user` die neue Umgebung erhalten. Wiederholen Sie danach die Diagnosebefehle. Siehe [Umgebungsvariablen](/docs/reference/environment-variables/) für die technischen Verträge und Prioritäten.

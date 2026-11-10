---
title: Ladeüberlagerung
description: Die Überlagerung wird direkt angezeigt, während die Sitzung neu geladen wird.
---
Die Ladeüberlagerung ist eine GTK 4 Layer-Shell-Oberfläche, die angezeigt wird, während ARGVUS die Sitzung direkt neu lädt (`argvus sessionctl reload`). Ein Themenwechsel verwendet sie nicht: Der Desktop ändert sich direkt vor Ort. Es ist ein Hilfsprogramm, das im Paket `argvus` bei `/usr/lib/argvus/loading-theme/splash` ausgeliefert wird, und es wird nicht in `/usr/bin` platziert.

Das Login verwendet es nicht: der Greeter-Splash und der Session-Start-Spinner sind der Wayland-Modus des Handoff-Helfers, der denselben Spinner ohne GTK zeichnet und seinen ersten Frame innerhalb von Millisekunden zeigt (siehe [Greeter, Sperre und Splash](/docs/developer-guide/subsystems/greeter-lock-and-splash/)). Mit `--spinner-only` zeigt die Overlay nur diesen Spinner und beendet sich bei `SIGTERM`. Mit `--detached` beendet es sich nicht, wenn der Prozess, der es gestartet hat, endet; ein Session-Reload verwendet beide Flags und stoppt es über seine PID-Datei. Es liest das ausgewählte Thema aus dem ARGVUS-Datenverzeichnis und fällt auf die mitgelieferte ARGVUS Dark-Palette zurück. Es meldet die Bereitschaft erst nach dem ersten Frame jeder Oberfläche, die es abbildet, nicht, wenn der Prozess startet.

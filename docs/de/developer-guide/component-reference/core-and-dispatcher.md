---
title: Kern und Dispatcher
description: Der argvus-Arbeitsbereich, sein Dispatcher und die gemeinsamen Rust-Crates.
---
`argvus` ist der Cargo-Arbeitsbereich bei `de/argvus` und das Paket, das `/usr/bin/argvus` bereitstellt. Die Kiste `argvus-cli` ist der Dispatcher: Sie liest das erste Argument und führt entweder einen Bibliotheksbefehl im Prozess aus, startet einen GTK 4-Helfer oder eine private ausführbare Datei, oder führt ein gepacktes Skript aus. Die vollständige Unterbefehlsliste befindet sich in der [Befehlszeilenreferenz](/docs/reference/command-line/).

Der Arbeitsbereich enthält auch die Kisten, die die Befehle gemeinsam nutzen:

- `argvus-config`: der kanonische Konfigurationsspeicher und der einzige Schreiber des generierten Baums (`data/generated/`). Es ist auch eine Bibliothek, die im Prozess von `argvus config`, `theme-switch` und `effects-toggle` verwendet wird.
- `argvus-i18n` und `argvus-i18n-core`: die Kataloge in `resources/locales/`, deren Validierung und das `org.argvus.i18n` QML-Modul.
- `argvus-tui` und `argvus-theme`: geteilte Terminal-Benutzeroberfläche und Themenladen.
- `argvus-runtime`: die übersetzten Nachrichten und die Prozesshelfer, die von den Befehlen gemeinsam genutzt werden.

`argvus-session` bleibt ein separates Repository. Es stellt die Sitzungseinstiegspunkte (`session`, `start`, `tty`) und die Lebenszykluseinheiten bereit, und `argvus sessionctl` erreicht seinen Controller.

---
title: Beitragend
description: Beitragsgrenzen für ARGVUS-Projekte und Dokumentation.
---
Verfolgen Sie eine Funktion durch alle verbrauchenden Repositories, bevor Sie sie ändern. Bewahren Sie Verträge für gemeinsamen Zustand, Paketbesitz, Servicebesitz und beide Lokalisierungskataloge. 

`argvus-config` ist der einzige Schreiber von `data/generated/`. Fügen Sie dort eine Verbraucherdatei hinzu oder ändern Sie sie, in `crates/argvus-config/src/project.rs` und seinen Untermodulen unter `project/` (Hyprland-Dateien in `project/hyprland_projection.rs`), nicht in einem Shell-Helfer. Wenn eine Verbraucheranwendung `config.json` nicht lesen kann, fügen Sie einen Adapter im besitzenden Paket hinzu und rufen Sie ihn vom Orchestrator nach dem kanonischen Commit auf. Löschen Sie `data/generated/`, führen Sie `argvus config apply` aus und bestätigen Sie, dass der Baum byte-für-byte aus `config.json` wiederhergestellt wird. Validieren Sie Quelle, Paketpayload und Verhalten in der Live-Sitzung separat; ein reiner Source-Build beweist nicht, dass eine installierte Wayland-Sitzung die neuen Dateien verwendet.

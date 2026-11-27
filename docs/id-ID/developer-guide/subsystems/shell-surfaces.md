---
title: Permukaan cangkang
description: Taskbar, panel kontrol dan kepemilikan telemetri.
---

`argvus-shell` menyediakan `Quickshell Taskbar` yang dipatch. `argvus-shell` memiliki tindakan bilah tugas dan UX; bilah tugas JSONC dan CSS yang dikirimkannya merupakan proyeksi `config.json`, dan `argvus-config` menulis lapisan aktif di bawah `data/generated/taskbar/` ditambah salinan cermin di bawah `data/taskbar/`. `argvus-shell` memiliki kartu Quickshell. `argvus-shell` provides the optional Telemetry surface, while the `argvus widget-telemetry` command in `argvus` retains block-control semantics; JSONC dan CSS-nya ditulis ke `data/taskbar/` dari paket default pada perubahan tampilan, setelah itu `argvus-config` hanya menulis ulang `ARGVUS_TELEMETRY_*` dan blok font yang dibatasi. `argvus-session` memiliki layanan penggunanya.

Tidak ada paket `argvus-shell` atau `argvus-shell.service` saat ini dalam model runtime ini.
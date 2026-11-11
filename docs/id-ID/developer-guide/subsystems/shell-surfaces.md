---
title: Permukaan cangkang
description: Taskbar, panel kontrol dan kepemilikan telemetri.
---

`argvus-waybar` menyediakan `/usr/bin/waybar` yang dipatch. `argvus-taskbar` memiliki tindakan bilah tugas dan UX; bilah tugas JSONC dan CSS yang dikirimkannya merupakan proyeksi `config.json`, dan `argvus-config` menulis lapisan aktif di bawah `data/generated/waybar/` ditambah salinan cermin di bawah `data/waybar/`. `argvus-control-panel` memiliki kartu Quickshell. `argvus-widget-telemetry` memiliki bilah telemetri opsional dan semantik bloknya; JSONC dan CSS-nya ditulis ke `data/waybar/` dari paket default pada perubahan tampilan, setelah itu `argvus-config` hanya menulis ulang `ARGVUS_TELEMETRY_*` dan blok font yang dibatasi. `argvus-session` memiliki layanan penggunanya.

Tidak ada paket `argvus-shell` atau `argvus-shell.service` saat ini dalam model runtime ini.
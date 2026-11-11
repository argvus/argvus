---
title: Configuration files
description: Pengguna, sistem, paket-default, dan konfigurasi ARGVUS yang dihasilkan.
---

| Lapisan | Lokasi | Arti |
| --- | --- | --- |
| Sistem | `/etc/argvus/` | Konfigurasi administrator untuk layanan dan penyedia. |
| Default paket | `/usr/share/argvus/` | Default komponen hanya baca dan aset runtime. |
| Pengguna | `$XDG_CONFIG_HOME/argvus/config.json` | Preferensi portabel kanonik. |
| Data yang dikelola | `$XDG_CONFIG_HOME/argvus/data/` | Data komponen, file kompatibilitas, dan metadata internal. |
| Dihasilkan | `$XDG_CONFIG_HOME/argvus/data/generated/` | Proyeksi `config.json`, hanya ditulis oleh `argvus-config`. Jangan pernah mengedit, jangan pernah menyalin di tempat lain sebagai konfigurasi, dan jangan pernah membaca kembali sebagai sumber. |
| Waktu proses | `$XDG_RUNTIME_DIR/` dan `$XDG_CACHE_HOME/` | Soket, kunci dan keadaan sementara. |

Status tema aktif, aksen, wallpaper khusus, dan efek adalah status pengguna logis. Model logika terkelola didokumentasikan dalam [Konfigurasi kanonik](/docs/reference/argvus-config/). File Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, Yazi, Superfile, terminal, font, dan perangkat yang dapat dilepas yang dihasilkan tidak boleh diperlakukan sebagai sumber kebenaran independen.

`argvus-config` juga mencerminkan lapisan tema ke `data/waybar/argvus-taskbar.{jsonc,css}` dan `data/waybar/argvus-widget-telemetry.{jsonc,css}`. Salinan tersebut diganti dari paket default dengan perubahan tampilan, kemudian blok terkelola yang dibatasi ditulis ulang, jadi perlakukan salinan tersebut sebagai file turunan, bukan sebagai file yang dikelola secara manual. Konfigurasi aplikasi asli di bawah `$XDG_CONFIG_HOME/<app>` berbeda: ini adalah target adaptor yang ditulis oleh orkestrator tampilan, tidak pernah oleh proyektor.

## Pengecualian terhadap aturan penulis tunggal

`argvus config` adalah satu-satunya penulis `data/generated/`. Ada satu pengecualian untuk migrasi:
ketika salinan pengguna dari file paket dibuat, file yang ditinggalkan oleh versi yang lebih lama
`data/generated/` dipindahkan ke lokasi penggunanya (`data/`), sehingga pengguna tetap menyimpan hasil editnya. Itu
pemindahan hanya terjadi untuk file yang tidak dihasilkan oleh versi saat ini. Jangan mengandalkan ini
jalur untuk file baru: tambahkan file yang dihasilkan dalam proyeksi, bukan di sini.
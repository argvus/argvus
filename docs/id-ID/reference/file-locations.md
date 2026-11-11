---
title: Lokasi file
description: File dan direktori ARGVUS penting yang diinstal.
---

- `/usr/bin/argvus*` — perintah publik.
- `/usr/share/argvus/hyprland/` — Konfigurasi dan pembantu Hyprland.
- `/usr/share/argvus/control-panel/` — Panel Kontrol QML dan skrip.
- `/usr/share/argvus/taskbar/` — konfigurasi dan skrip bilah tugas.
- `/usr/share/argvus/lock/` — Konfigurasi Hyprlock.
- `/usr/share/argvus/portal/` — file integrasi portal.
- `/usr/share/backgrounds/argvus/` — aset wallpaper.
- `/etc/argvus/` — konfigurasi penyedia sistem.
- `/etc/argvus/greeter.toml` — konfigurasi penyambut.
- `/etc/argvus/removable-devices/config.json` — default sistem untuk penyimpanan yang dapat dilepas.
- `$XDG_CONFIG_HOME/argvus/config.json` — preferensi portabel kanonik.
- `$XDG_CONFIG_HOME/argvus/data/` — data komponen yang dikelola, cadangan, kunci, dan keluaran yang dihasilkan.
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — penggantian bilah tugas Waybar asli opsional yang lengkap.
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — salinan bilah tugas ARGVUS yang dikelola.
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — menghasilkan profil Waybar, termasuk `argvus-taskbar.css` dan `argvus-widget-telemetry.{jsonc,css}`; ini lebih diutamakan daripada salinan di atas.
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — juga dicerminkan di bawah `data/waybar/`, yang diganti dari paket default dengan perubahan tampilan dan kemudian `ARGVUS_TELEMETRY_*` dan blok fontnya ditulis ulang.
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — proyeksi pohon konfigurasi Yazi, dengan fallback yang peka terhadap varian untuk tema yang tidak memberikan cita rasa Yazi.
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — kumpulan tema Superfile yang diproyeksikan.
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — profil terminal yang diproyeksikan.
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — palet Qt6ct yang diproyeksikan.
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — file tema GTK yang diproyeksikan.
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — salinan pengguna dari konfigurasi, tema, dan mode Rofi. ARGVUS membuat setiap salinan saat pertama kali mengubah file; file yang tidak diubah dibaca dari `/usr/share/argvus/launcher/config/`.
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — proyeksi stylesheet perangkat yang dapat dilepas.
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — Pengaturan Fokus ARGVUS, riwayat sesi, dan pos pemeriksaan sesi saat ini.
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — status Fokus ARGVUS sementara yang digunakan oleh bilah tugas; dihapus saat Fokus keluar.
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — ARGVUS Pengaturan jam dan jam dunia, alarm, dan pos pemeriksaan pengatur waktu/stopwatch.
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — warna ARGVUS Focus dan ARGVUS Clock (konsumen tema `time`).
- `/usr/share/argvus/sounds/` — suara Fokus ARGVUS dan Jam ARGVUS (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`).
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — pembuatan tema (`builds/<id>-<variant>-<hash>/`), ditulis oleh `argvus theme render` dan setiap peralihan tema dan perubahan aksen, serta symlink `current` ke pembuatan tema aktif.
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — penggantian Hyprland Lua pengguna dan proyeksi Hyprland asli.
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — konfigurasi Hyprland asli lengkap opsional yang dipilih oleh `argvus start`; ini bukan hamparan tambahan.
- `$XDG_STATE_HOME/argvus/config-projection.json` — manifes proyeksi: pembuatan, hash konfigurasi efektif, bagian yang diubah, dan pekerjaan runtime yang tertunda. Dibaca oleh `argvus sessionctl reload`; tidak pernah menjadi sumber kebenaran.
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — preferensi blok telemetri lama. Hanya masukan migrasi; nilai kanonik ada di `config.json` dan file ini tidak lagi ditulis.
- `$XDG_STATE_HOME/argvus/session.log` — log sesi grafis (biasanya `~/.local/state/argvus/session.log`).
- `/usr/share/wayland-sessions/argvus.desktop` — entri sesi grafis.
- `/usr/share/plymouth/themes/argvus/` — paket aset boot splash.

Muatan yang tepat bersifat spesifik paket; lihat referensi komponen pengembang dan manifes paket yang relevan.
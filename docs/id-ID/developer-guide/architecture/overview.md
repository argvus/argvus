---
title: Architecture overview
description: Bagaimana ARGVUS membentuk lingkungan desktop terintegrasi.
---

ARGVUS memiliki lima lapisan runtime:

1. titik masuk dan pengiriman di `argvus`;
2. siklus hidup sesi di `argvus-session`;
3. permukaan kompositor dan shell di `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar` dan `argvus-control-panel`;
4. aplikasi pengguna seperti Pusat Kontrol, kalender dan monitor sistem;
5. penyedia dan aset untuk tampilan, jaringan, daya, tampilan, akun, dan notifikasi.

Status bersama menghubungkan lapisan-lapisan ini. `argvus-config` memegang dokumen kanonik dan merupakan satu-satunya penulis `data/generated/`, sehingga perubahan tema atau efek membuat ulang file yang digunakan oleh beberapa paket dalam satu transaksi; layanan sesi memberikan kepemilikan siklus hidup tanpa mengambil kepemilikan atas implementasi setiap komponen.

## Batasan tanggung jawab

| Lapisan | Tanggung jawab utama | Contoh |
| --- | --- | --- |
| Titik masuk dan pengiriman | Mulai aplikasi desktop dan selesaikan profil ARGVUS. | `argvus`, `argvus-app-profiles` |
| Siklus hidup sesi | Mempersiapkan lingkungan, meluncurkan Hyprland, menyinkronkan lingkungan layanan pengguna dan mematikan sesi sendiri. | `argvus-session`, `argvus sessionctl` |
| Kompositor dan shell | Tentukan perilaku jendela dan permukaan desktop yang terlihat. | `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar`, `argvus-control-panel` |
| Konfigurasi | `config.json` sendiri, model jalur dan setiap file konsumen yang dihasilkan. | `argvus-config` |
| Aplikasi | Menyediakan pengaturan dan alur kerja desktop yang terfokus. | Pusat Kontrol, kalender, peluncur, monitor sistem, dan perangkat yang dapat dilepas |
| Penyedia dan aset | Menerapkan operasi domain dan sumber daya visual bersama. | tampilan, jaringan, daya, notifikasi, tampilan, font dan wallpaper |

Paket `argvus` adalah target instalasi dan koordinator, tetapi ini bukan runtime monolitik. Paket terfokus mempertahankan kepemilikan biner, konfigurasi, dan layanannya. Manajer sesi memulainya pada titik yang tepat dalam sesi grafis tanpa menyerap implementasinya.

Perbedaan ini penting ketika melakukan debug: kegagalan sesi adalah milik `argvus-session`; kegagalan bilah tugas milik `argvus-taskbar`/`argvus-waybar`; dan kegagalan tema atau wallpaper yang dihasilkan termasuk dalam jalur konfigurasi tampilan dan sesi.
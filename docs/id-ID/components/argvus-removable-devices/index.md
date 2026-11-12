---
title: Perangkat yang dapat dilepas
description: Pasang dan kelola penyimpanan yang dapat dilepas dari ARGVUS.
---

`argvus-removable-devices` mengawasi UDisks2 dan menyediakan modul penyimpanan ARGVUS Taskbar dan menu perangkatnya. Menu ini juga tersedia sebagai antarmuka bergaya Rofi yang terpisah, sehingga tindakan perangkat yang sama dapat digunakan dari bilah tugas atau dari alur kerja peluncur.

Tindakan yang tersedia adalah `open`, `mount`, `unmount`, `eject`, `poweroff`, `lock`, `unlock` dan `copy`. Perintah `list`, `watch` dan `menu` memaparkan status yang sesuai, aliran bilah tugas, dan antarmuka menu untuk integrasi.

## Menu bilah tugas

Dengan `"mode": "gui"` (default sistem), klik pada ikon penyimpanan taskbar akan membuka menu GTK yang terpasang pada Taskbar ARGVUS, seperti popup kalender: di bawah bar ketika taskbar berada di bagian atas layar, dan di atasnya ketika taskbar berada di bawah. Setiap baris perangkat memiliki submenu tindakan yang hanya berisi tindakan yang diizinkan oleh statusnya.
- **Pointer:** melayang atau mengklik baris perangkat akan membuka submenunya. Submenu tetap terbuka ketika penunjuk melintasi tepi menu atau meninggalkan menu, dan mengikuti baris lain hanya setelah penunjuk berhenti di sana sebentar, sehingga jalur diagonal ke submenu tidak menutupnya. Klik di luar kedua menu akan menutup popup.
- **Keyboard:** `Up`/`Down` pilih satu baris, `Right`, `Enter` atau `Space` buka submenu (atau jalankan **Refresh**/**Quit**), `Enter` atau `Space` menjalankan tindakan yang dipilih, `Left` atau `Escape` kembali ke daftar perangkat, dan `Escape` menutup popup.

**Salin jalur** menempatkan titik pemasangan di papan klip melalui `copy_command` (default `wl-copy`, dari `wl-clipboard`), sehingga jalur tetap ada setelah menu ditutup; pemberitahuan mengonfirmasinya ketika `show_notification` aktif. **Kunci** ditawarkan untuk setiap volume terenkripsi yang tidak terkunci dan melepasnya terlebih dahulu bila diperlukan.

Tindakan pengelola file default mengikuti pengelola file yang dipilih di Pusat Kontrol ARGVUS. Hal ini menjaga tindakan perangkat yang dapat dilepas tetap konsisten dengan default aplikasi pengguna.

Konfigurasi sistemnya adalah `/etc/argvus/removable-devices/config.json`. Proyeksi per pengguna yang dikelola ARGVUS adalah `~/.config/argvus/data/removable-devices/config.json`, dengan `theme.css` yang cocok di sebelahnya. Stylesheet tersebut juga diproyeksikan ke `~/.config/argvus/data/generated/removable-devices/theme.css` dari tema, aksen, dan mode kanonik, dengan fallback keluarga/varian untuk tema yang tidak mengirimkan aset perangkat lepasan khusus, sehingga perubahan tema menjaga modul tetap konsisten tanpa langkah manual apa pun. Jalur komponen lama adalah masukan migrasi; konfigurasi pengguna selanjutnya mengesampingkan default sistem.

Pengaturan yang berguna mencakup apakah modul disembunyikan saat tidak ada perangkat (`hide_when_empty`), apakah nama dan kapasitas ditampilkan, urutan perangkat, perilaku notifikasi, backend menu, dan ikon status. Stylesheetnya dirender dari tema ARGVUS aktif (`removable-devices/theme.css` dari pembuatan tema) dan disalin ke panel `theme.css` pada setiap perubahan tema atau aksen.

Antarmuka menggunakan bahasa pengguna dari `LANG` di `$XDG_CONFIG_HOME/locale.conf`, dengan bahasa Inggris sebagai default untuk pengguna yang belum dikonfigurasi. Variabel lokal yang diwarisi tidak memilih bahasa antarmuka.

## Katalog bahasa bersama

Antarmukanya menggunakan katalog `argvus-i18n` bersama untuk sepuluh bahasa. Teks disimpan di `resources/locales/<locale>/removable-devices.json` di ruang kerja master dan diinstal oleh `argvus`. Bahasa Inggris adalah penggantinya; bahasa baru tidak lagi memerlukan kamus bilingual lokal.

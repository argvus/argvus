---
title: Lokalisasi
description: Gunakan ARGVUS dalam bahasa Inggris atau Portugis.
---

`argvus-i18n` menyediakan katalog bersama, yang `argvus i18n validate` perintah dan QML modul `org.argvus.i18n`. ARGVUS Kapal 18 bahasa interface: Arab, Bengali, Jerman, Inggris, Spanyol, Prancis, Hindi, Indonesia, Italia, Jepang, Korea, Portugis Brasil, Portugis Eropa, Rusia, Turki, Vietnam dan Cina Sederhana.

Asli ARGVUS aplikasi dan komponen shell mengkonsumsi katalog bersama. String yang diterjemahkan dapat digunakan oleh lebih dari satu repositori. Arab dan Urdu gunakan tata letak -to-left dengan didukung GTK, Quickshell dan antarmuka terminal.

Gunakan * * Control Center Averence Language & Daerah Bahasa * * untuk memilih ARGVUS Bahasa antarmuka. Ini menulis `ARGVUS_LANGUAGE` ke `$XDG_CONFIG_HOME/locale.conf` (biasanya `~/.config/locale.conf`). Misalnya, `ARGVUS_LANGUAGE=pt-PT` Memilih Portugis Eropa. Portugis Brasil (`pt-BR`) adalah pilihan yang terpisah. Katalog bahasa diinstal dengan ARGVUS, sehingga tidak tergantung pada `locale-gen`.

Panel Kendali dijalankan ulang secara otomatis setelah perubahan bahasa antar muka. Mulai sesi baru untuk menerapkannya ke komponen sesi. Pengaturan terpisah * * Regional Locale * * menulis `LANG` dan mengontrol format regional dan aplikasi pihak ketiga. Ini memerlukan lokal sistem yang dihasilkan; memungkinkan satu di `/etc/locale.gen` berjalan `locale-gen` dengan otorisasi administrator. Tata letak papan tik, varian dan peta kunci konsol adalah pengaturan terpisah.

# # Ketika variabel lingkungan menimpa perubahan

ARGVUS aplikasi dibaca `ARGVUS_LANGUAGE` dari pengguna `locale.conf`Jika tidak ada, mereka menyelesaikan regional `LANG` nilai ke katalog yang terpasang. Variabel lokal tidak override sebuah eksplisit `ARGVUS_LANGUAGE` preferensi. Aplikasi pihak ketiga terus menggunakan sistem lokal regional.

Inspeksi nilai efektif dengan:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

Periksa `~/.config/locale.conf` berisi sebuah valid `ARGVUS_LANGUAGE=<catalog>` nilai. Untuk mengubah format regional atau bahasa aplikasi pihak ketiga, gunakan * * Regional Locale * * dan pilih lokal yang dihasilkan. Panel Kendali dijalankan ulang setelah perubahan preferensi; log keluar dan masuk bila komponen sesi lain masih menampilkan bahasa antar muka lama. Lihat [Variabel Lingkungan] (/docs/reference/environment-variables/) untuk kontrak teknis dan mendahului.

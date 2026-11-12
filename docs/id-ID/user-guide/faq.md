---
title: Pertanyaan yang sering diajukan
description: Pertanyaan umum tentang ARGVUS, arsitekturnya, dan penggunaan sehari-hari.
---

Halaman ini menjawab pertanyaan umum tentang apa itu ARGVUS, bagaimana distribusinya, dan di mana mengkonfigurasinya. Untuk prosedur terperinci, ikuti tautan di setiap jawaban.

## Apa itu ARGVUS?

ARGVUS adalah lingkungan desktop modular untuk Wayland dan Arch Linux, dibangun di sekitar Hyprland. Ini bukan lingkungan desktop yang independen dari Hyprland; ini adalah pengalaman desktop yang dirakit di atas Hyprland dan disesuaikan dengan Arch Linux. ARGVUS mengoordinasikan sesi grafis, permukaan shell, pengaturan aplikasi, tampilan, layanan desktop, dan alat terintegrasi melalui kumpulan paket.

Ini lebih dari sekadar konfigurasi Hyprland atau kumpulan dotfile pribadi: ia menyediakan titik masuk sesi yang didukung, konfigurasi bersama dan status tampilan, serta kepemilikan yang ditentukan untuk komponen seperti bilah tugas, Pusat Kontrol, notifikasi, jaringan, dan tampilan. Lihat [pendahuluan](/docs/introduction/) untuk gambaran umum tentang bagaimana potongan-potongan tersebut cocok satu sama lain.

## Apakah ARGVUS gratis?

Ya. ARGVUS gratis untuk diunduh dan digunakan, dan paket proyek intinya didistribusikan sebagai perangkat lunak gratis di bawah [GNU General Public License versi 3](https://www.gnu.org/licenses/gpl-3.0.html). Hal ini memberikan kebebasan kepada pengguna untuk menjalankan, mempelajari, memodifikasi, dan mendistribusikan ulang perangkat lunak yang tercakup dalam ketentuan lisensi.

ARGVUS didistribusikan sebagai paket Arch Linux yang ditandatangani. Perangkat lunaknya sendiri tidak memiliki harga pembelian, tetapi pengunduhan paket mungkin masih memerlukan biaya internet atau hosting normal. Komponen dan aplikasi yang disediakan oleh pihak ketiga dapat memiliki lisensinya sendiri.

## Apakah ARGVUS merupakan distribusi Linux?

Tidak. ARGVUS adalah lapisan lingkungan desktop yang diinstal di atas Arch Linux. Arch Linux menyediakan sistem operasi, kernel, manajer paket dan paket dasar; Hyprland menyediakan komposisi Wayland; ARGVUS mengoordinasikan sesi desktop dan komponen terintegrasinya.

Lihat [instalasi](/docs/getting-started/installation/) untuk model instalasi yang didukung dan [Apa itu ARGVUS—dan bukan](/docs/introduction/#what-argvus-isand-is-not) untuk perbedaan arsitekturalnya.

## Mengapa lingkungan desktop dan bukan distribusi?

Mempertahankan ARGVUS sebagai lingkungan desktop memberi pengguna kontrol lebih besar terhadap seluruh sistem mereka. Distribusi biasanya memilih dan memelihara garis dasar yang lengkap, termasuk kernel, pemilihan paket, repositori, default, dan kebijakan sistem. ARGVUS berfokus pada pengalaman desktop sambil menyerahkan keputusan yang lebih luas kepada pengguna dan instalasi Arch Linux yang mendasarinya.

Model ini juga menjaga proyek tetap modular: manajemen sesi, konfigurasi Hyprland, tampilan, taskbar, pengaturan dan penyedia dapat dikembangkan dan dikemas secara terpisah sambil bekerja bersama sebagai satu desktop.

## Sistem mana yang didukung?

Basis yang didukung saat ini adalah Arch Linux. ARGVUS dikemas untuk Arch Linux dan bergantung pada Wayland, Hyprland, dan komponen sistem yang dijelaskan dalam [persyaratan instalasi](/docs/getting-started/installation/requirements/).

Distribusi lain mungkin menyediakan beberapa komponen upstream yang sama, tetapi itu tidak membuat rangkaian paket ARGVUS lengkap atau integrasi sesi didukung di sana.

## Bagaimana cara menginstal ARGVUS?

Ikuti [panduan instalasi](/docs/getting-started/installation/). Ini menjelaskan persyaratan, cara mengkonfigurasi repositori paket ARGVUS yang ditandatangani, cara menginstal kumpulan paket dan cara memulai sesi pertama.

ARGVUS dikirimkan sebagai kumpulan paket Arch yang terkoordinasi. Paket `argvus` adalah target instalasi utama dan menarik komponen desktop yang dikoordinasikannya.

## Bagaimana cara mengubah pengaturan ARGVUS?Gunakan **Pusat Kontrol** untuk konfigurasi desktop persisten, termasuk tema, wallpaper, efek, tata letak, font, keyboard, input, bahasa, wilayah, dan aplikasi default. Cara paling efisien untuk membukanya adalah dengan pintasan `SUPER + Alt + C`, atau peluncur dengan `SUPER + D`: ketik **ARGVUS Control Center**. Anda juga dapat mengetik `argvus` untuk membuat daftar aplikasi ARGVUS.

Sebagai opsi tambahan, Anda dapat membukanya dari terminal dengan:

```sh
argvus --control-center
```

Anda juga dapat membuka area fokus dengan `argvus control-center`, misalnya `argvus control-center appearance themes`. Jalankan `argvus control-center --help` untuk melihat rute yang tersedia di versi terinstal.

Gunakan **Panel Kontrol** untuk status saat ini dan tindakan yang sering dilakukan seperti volume, kecerahan, jaringan, notifikasi, daya, dan kontrol sesi. Ini bukan database konfigurasi kedua. Lihat [Tempat mengonfigurasi sesuatu](/docs/user-guide/where-to-configure/) dan [Pusat Kontrol](/docs/components/argvus-control-center/) untuk perbedaannya.

## Di mana pengaturan ARGVUS disimpan?

Konfigurasi milik pengguna disimpan di bawah `$XDG_CONFIG_HOME/argvus` (biasanya `~/.config/argvus`). Konfigurasi logis kanonik dipertahankan di sana, sedangkan file konsumen turunan untuk Hyprland, Waybar dan widget telemetri, GTK, Qt6ct, Quickshell, Dunst, Hyprlock, Yazi, Superfile, terminal, font, efek, kalender, input, keyboard, daya, dan perangkat yang dapat dilepas berada di bawah `~/.config/argvus/data/generated/`. Pohon itu hanya ditulis oleh `argvus-config` dan dibangun kembali dari konfigurasi kanonik; `argvus-appearance` melakukan perubahan tampilan dan kemudian merekonsiliasi hanya konsumen eksternal yang tidak dapat membaca sendiri konfigurasi kanoniknya. Rofi adalah pengecualian: `argvus launcher` mengemas konfigurasi dan temanya, dan salinan yang diedit pengguna berada di bawah `~/.config/argvus/data/rofi/`.

Jangan mengedit file yang dihasilkan sebagai metode konfigurasi permanen. Gunakan Pusat Kontrol atau jalur penggantian asli yang terdokumentasi untuk komponen yang ingin Anda sesuaikan. Lihat [file konfigurasi](/docs/reference/configuration-files/) dan [konfigurasi dan status](/docs/developer-guide/architecture/configuration-and-state/).

## Apa yang harus saya lakukan jika perubahan tidak diterapkan?

Konfirmasikan terlebih dahulu bahwa perubahan telah disimpan atau diterapkan di halaman pemilik. Kemudian periksa apakah komponen yang terpengaruh memerlukan sesi atau layanan dimuat ulang. Perubahan yang dilakukan diproyeksikan ke dalam file konsumen dengan `argvus-config`, namun memproyeksikannya tidak menjamin bahwa setiap konsumen yang berjalan telah memuat ulang file tersebut. `argvus sessionctl reload` menerapkan pembuatan yang sudah dilakukan dari manifes proyeksi alih-alih memproyeksikan lagi, dan melewatkan pemuatan ulang grafis ketika status kanonik tidak berubah.

Gunakan [panduan pemecahan masalah](/docs/user-guide/troubleshooting/) yang relevan untuk instalasi, permulaan sesi, grafik, tampilan, Hyprland, tema dan wallpaper. Saat melaporkan masalah, sertakan komponen, perubahan pengaturan, perintah atau halaman yang digunakan, dan keluaran status atau perintah diagnostik yang relevan.

## Di mana saya dapat mempelajari lebih lanjut?

- [Konfigurasi pertama](/docs/user-guide/getting-started/) untuk alur sesi pertama yang direkomendasikan.
- [Panduan pengguna](/docs/user-guide/) untuk instruksi spesifik fitur.
- [Referensi](/docs/reference/) untuk perintah, jalur, layanan, dan kontrak konfigurasi.
- [Panduan pengembang](/docs/developer-guide/) untuk arsitektur, kepemilikan paket, dan alur kerja kontribusi.

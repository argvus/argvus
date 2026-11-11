---
title: Pendahuluan
description: Apa itu ARGVUS, cara kerja desktopnya, dan lokasi komponennya.
---

ARGVUS adalah lingkungan desktop Hyprland dan Wayland modular untuk Arch Linux. Ini adalah kumpulan paket yang terintegrasi, bukan satu aplikasi monolitik: sesi, kompositor, permukaan shell, aplikasi pengaturan, penyedia, dan aset visual bekerja sama melalui status bersama dan kontrak runtime eksplisit.

Halaman ini adalah peta dokumentasi. Ini menjelaskan keseluruhan desktop pada tingkat tinggi; halaman tertaut berisi rincian operasional dan implementasi.

## Cara kerja ARGVUS

Sesi normal mengikuti jalur ini:

```text
graphical login or TTY
        ↓
argvus-session → argvus start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```

`argvus-session` memiliki siklus hidup sesi dan pengaturan lingkungan. Itu tidak mengimplementasikan setiap fitur desktop: setiap komponen mempertahankan kepemilikan atas perintah, konfigurasi, dan payload layanannya sendiri.

Untuk instalasi dan startup, lihat [Instalasi](/docs/getting-started/installation/) dan [Sesi](/docs/user-guide/sessions/). Untuk kepemilikan layanan, lihat pengembang [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/).

## Lapisan utama

### Titik masuk dan infrastruktur

Paket `argvus` menyediakan operator `/usr/bin/argvus` dan titik masuk paket utama. `argvus-session` menyediakan titik masuk grafis dan TTY, target pengguna dan `argvus sessionctl`. Fungsi TUI dan pelokalan bersama berasal dari ruang kerja `argvus` (peti `argvus-tui` dan `argvus-i18n`) dan perintah `argvus i18n`.

### Kompositor dan shell desktop

`argvus-hyprland` memasok konfigurasi Hyprland dan pengikatan kunci. `argvus-waybar` memasok biner Waybar yang dipatch, sedangkan `argvus-taskbar` memiliki konfigurasi dan tindakan bilah tugas. `argvus-control-panel` memasok panel Quickshell, dan `argvus widget-telemetry` menyediakan widget sistem opsional, dalam proses di `argvus`.

Lihat [Desktop](/docs/user-guide/desktop/) untuk penggunaan dan [permukaan shell](/docs/developer-guide/subsystems/shell-surfaces/) untuk kepemilikan implementasi.

### Pengaturan dan aplikasi

`argvus control-center` adalah aplikasi pengaturan keyboard-first, dikirimkan sebagai pembantu di `argvus`. Kalender dan alat perangkat yang dapat dilepas adalah pembantu GTK 4 di `argvus`; monitor sistem dan terminal adalah sub-perintah dari `argvus`; peluncurnya adalah `argvus-launcher`. Semuanya terintegrasi dengan shell dan operator.

Lihat [Aplikasi](/docs/user-guide/applications/) dan [referensi perintah](/docs/reference/command-line/).

### Penyedia dan integrasi

Tampilan, jaringan, Bluetooth, daya, notifikasi, firewall, dan penyimpanan yang dapat dilepas disediakan oleh proyek terfokus. Pusat Kontrol dan Panel Kontrol menggunakan penyedia ini; mereka tidak menggantikannya.

Visual login, kunci, dan startup juga terpisah: `argvus-greeter` menangani salam, `argvus-lock` menangani Hyprlock, pembantu handoff yang dikirimkan dalam `argvus` menggambar pemintal login, overlay pemuatan menangani transisi tema, dan `argvus-boot-splash` menangani boot splash.

Lihat [Perangkat Keras](/docs/user-guide/hardware/), [Privasi dan keamanan](/docs/user-guide/privacy-and-security/) dan pengembang [referensi komponen](/docs/developer-guide/).

## Status penampilan bersama

Preferensi tema, aksen, wallpaper, font dan efek disimpan sebagai keadaan logis di bawah `$XDG_CONFIG_HOME/argvus` (biasanya `~/.config/argvus`). `argvus-config` adalah satu-satunya komponen yang menulis `data/generated/`, yang memproyeksikan file konsumen untuk Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, notifikasi, layar kunci, Yazi, Superfile, dan terminal. Rofi adalah pengecualian: `argvus launcher` mengemas konfigurasi dan temanya sebagai file biasa, dan salinan yang diedit pengguna berada di bawah `data/rofi/`. Pembantu penampilan melakukan perubahan keadaan dan kemudian merekonsiliasi aplikasi eksternal; mereka tidak pernah menulis pohon yang dihasilkan.Direktori yang dihasilkan bukanlah sumber kebenaran. Lihat [Penampilan](/docs/user-guide/appearance/) dan [konfigurasi dan status](/docs/developer-guide/architecture/configuration-and-state/).

Sistem visual juga mengontrol tata letak. Mode tempel kompak dan berbentuk persegi; Mode spasi menggunakan celah jendela yang lebih besar, sudut membulat, dan margin yang lebih lebar di sekitar bilah tugas dan shell. Bilah tugas defaultnya adalah tepi atas tetapi mendukung penempatan atas/bawah melalui status tata letak bersama. Efek mengontrol transparansi, keburaman, bayangan, dan animasi di seluruh kompositor dan permukaan desktop.

Lihat [tabel tema](/docs/user-guide/appearance/) untuk kelompok dan warna saat ini, [efek](/docs/argvus-appearance/effects/) untuk kontrak efek bersama, dan [bilah tugas](/docs/argvus-taskbar/taskbar/) untuk penempatan bilah dan jendela.

## Apa itu ARGVUS—dan apa yang bukan

ARGVUS adalah lapisan lingkungan desktop di atas Arch Linux. Arch Linux menyediakan sistem operasi dan paket; Hyprland menyediakan komposisi Wayland; ARGVUS mengoordinasikan sesi grafis, permukaan shell, pengaturan, tampilan, layanan desktop, dan aplikasi terintegrasi.

Oleh karena itu, ini lebih dari sekadar konfigurasi Hyprland atau kumpulan dotfile pribadi. ARGVUS menentukan titik masuk sesi yang didukung, memiliki siklus hidup layanan desktopnya, menyediakan pengaturan umum dan model tampilan, dan menghubungkan komponen terfokus seperti jaringan, layar, daya, notifikasi, akun, dan penyimpanan yang dapat dilepas.

ARGVUS tidak perlu mengganti setiap aplikasi. Ini mengintegrasikan aplikasi upstream terpilih yang merupakan pilihan paling andal, sekaligus menjaga perilaku dan konfigurasi seluruh desktop tetap koheren.

Nama tersebut menggabungkan dua ide. *ARGV*, dari *argument vector*, adalah daftar argumen yang diterima program, dan mencerminkan tujuan keyboard-first dalam mengarahkan desktop melalui perintah dan pintasan. *AS*, dari *Sistem Pengguna*, mencerminkan tujuan desktop yang berpusat pada penggunanya. Untuk penjelasan lengkap dan audio pengucapannya, lihat [Tentang ARGVUS](/about/).

Untuk batasan implementasi dan kepemilikan paket, lihat pengembang [ikhtisar arsitektur](/docs/developer-guide/architecture/overview/) dan [peran repositori](/docs/developer-guide/architecture/repository-roles/).

## Lanjutkan berdasarkan tujuan

- [Baca FAQ](/docs/user-guide/faq/).
- [Instal ARGVUS](/docs/getting-started/installation/).
- [Mulai sesi grafis atau TTY](/docs/user-guide/sessions/).
- [Konfigurasi tema dan wallpaper](/docs/user-guide/appearance/).
- [Gunakan bilah tugas dan Panel Kontrol](/docs/user-guide/desktop/).
- [Memahami perintah, jalur, dan layanan](/docs/reference/).
- [Pelajari arsitekturnya](/docs/developer-guide/architecture/overview/).
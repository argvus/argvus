---
title: Runtime lifecycle
description: Sesi startup dan kepemilikan layanan pengguna.
---

`argvus-session` mengimpor lingkungan, memulai Hyprland melalui `argvus start` dan memulai `argvus-session.target` setelah kompositor siap. Targetnya menggunakan layanan shell, notifikasi, wallpaper, idle, dan clipboard. `argvus sessionctl` mengoordinasikan mulai, berhenti, memuat ulang, memulai ulang, status, dan mencatat.

Sesi ini menghentikan target ketika Hyprland keluar. Masing-masing proyek tetap bertanggung jawab atas biner dan konfigurasinya sendiri.

## Memulai

Titik masuk grafisnya adalah `/usr/share/wayland-sessions/argvus.desktop`, yang memanggil `argvus session`. Sesi ini mengimpor lingkungan konfigurasi ARGVUS dan XDG, menyinkronkannya dengan `systemd --user` dan aktivasi D-Bus, dan mendelegasikan startup kompositor ke `argvus start`.

`argvus start` memvalidasi konfigurasi Hyprland, menerapkan fallback paket ketika file yang dihasilkan stok ditemukan atau konfigurasi pengguna tidak valid, mendeteksi kompatibilitas virtualisasi, dan meluncurkan Hyprland. Jembatan kesiapan kompositor kemudian memanggil `argvus sessionctl ready`, yang memulai target ARGVUS dengan lingkungan Wayland dan Hyprland saat ini.

Targetnya menggunakan unit persiapan sekali pakai sebelum memulai komponen jangka panjang. Persiapan menerapkan konfigurasi yang dihasilkan, status tema dan aksen, status monitor, integrasi aplikasi, dan lingkungan sesi. Layanan persisten dimiliki oleh `systemd --user`, bukan oleh proses latar belakang ad-hoc.

`argvus start` mengantri `argvus-config.service` (`--no-block`) tepat sebelum meluncurkan Hyprland: proyeksi tidak memerlukan compositor, sehingga tumpang tindih dengan startup compositor. `argvus sessionctl ready` kemudian memulai target, yang menemukan unit tersebut aktif atau menunggu tugasnya, sehingga konfigurasi kanonik masih diproyeksikan sebelum persiapan dijalankan. Setelah target dimulai, `sessionctl ready` menunggu hingga 1,5 detik hingga wallpaper (permukaan lapisan latar belakang) dan bilah tugas dipetakan sebelum menghapus overlay pemuatan, karena layanan sederhana dihitung sebagai dimulai sebelum digambar; permukaan yang unitnya tidak berjalan tidak ditunggu, dan penantiannya dicatat sebagai `session.desktop-surfaces`, dengan `session.wallpaper-surface` dan `session.taskbar-surface` untuk waktu yang dibutuhkan setiap permukaan. Proyeksi fatal atau kegagalan target-start dicatat dan menghentikan overlay pemuatan independen, mencegah putaran yang tampaknya tak ada habisnya. Konsumen visual opsional harus menyelesaikan fallback atau melaporkan kegagalan tanpa mengganti target sesi.

### Biaya masuk

Login tanpa perubahan konfigurasi tidak mengulangi pekerjaan yang mahal:

- Proyeksi menjalankan saklar tema dan adaptor efek hanya ketika tampilan atau bagian efek berubah, atau ketika output sebenarnya (salinan pengguna yang terwujud dan `data/state/effects/<theme>.conf`) hilang.
- Persiapan menerapkan tema startup, dan aksen di atasnya, hanya jika stempel di `$XDG_CACHE_HOME/argvus/session/theme-startup.stamp` tidak lagi cocok dengan file tema yang dikemas dan penanda aksen. Isi file cap sidik jari, jadi menginstal ulang file yang identik tidak membuatnya tidak valid. Input tema yang diubah atau aksen baru akan membatalkannya satu kali. Memigrasikan stempel berbasis stempel waktu lama juga akan dibuat ulang satu kali. Saat tema diterapkan kembali, persiapan juga memuat ulang Hyprland, yang dimulai pada file sebelumnya.
- Pengaturan GTK (`gsettings`) ditulis hanya ketika tema startup diterapkan kembali. Di antara login, dconf menyimpannya, sehingga login yang stabil tidak menjalankannya lagi.
- `argvus-dunst.service` adalah `Type=dbus`: dihitung sebagai dimulai setelah Dunst memiliki `org.freedesktop.Notifications`, sehingga target sesi tidak menunggu hingga status jeda dipulihkan.
- Persiapan berjalan `spaces-switch.sh` dan `borders-switch.sh` (`--apply-static`) secara berdampingan: mereka menulis file terpisah (margin JSONC batang dan geometri efektif, serta CSS batang dan tema Rofi).- Proyeksi menulis ulang file yang dihasilkan hanya ketika isinya berubah. File yang tidak diubah dibiarkan begitu saja, jadi login yang stabil tidak akan menyinkronkannya lagi.
- Langkah tampilan persiapan (`argvus config display session-prepare`) hanya menerapkan DPMS dan keluaran yang dinonaktifkan: tidak memproyeksikan lagi atau memuat ulang kompositor.
- `argvus-session-reload.service` menjalankan `argvus sessionctl reload-desktop --login`, yang memuat ulang Hyprland hanya ketika manifes proyeksi mencantumkan `hyprland` di antara konsumen yang terpengaruh. Komponen desktop tidak menunggunya.

Pemintal sesi (pembantu handoff dalam mode Wayland) dipetakan dari handoff kompositor hingga `argvus sessionctl ready` memulai target, sehingga satu spinner mencakup keseluruhan startup. Ia tidak memiliki `PartOf=`, karena membuat ulang target pada login baru tidak boleh menghapusnya; `sessionctl` menghentikannya secara eksplisit, dan `RuntimeMaxSec=30` mengakhirinya jika tidak ada pemberhentian yang tiba.

Setiap langkah persiapan, pemuatan ulang, dan adaptor proyeksi menambahkan durasinya ke log sesi sebagai `<epoch-ms> timing <step> <N>ms`; lihat [pemecahan masalah permulaan sesi](/docs/user-guide/troubleshooting/session-startup/).

Resolusi jalur, persiapan sesi, dan rencana isi ulang Hyprland adalah milik Rust.
`argvus config session-prepare` dan `argvus config session-reload-config` menjalankan
langkah memulai dan memuat ulang; skrip `hypr-init.sh` lama telah dihapus.

## Layanan dan penutupan

Target umumnya memiliki `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service` saat diaktifkan, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, pengamat papan klip, pemberitahuan tata letak keyboard, dan PolicyKit integrasi. Hamparan pemuatan sesi dimulai oleh `argvus start`, bukan oleh target. Kegagalan dalam layanan opsional tidak menggantikan seluruh sesi.

Layanan yang memerlukan koneksi klien Wayland, termasuk Panel Kontrol dan manajer idle, menunggu runtime Hyprland yang dapat digunakan sebelum meluncurkan kliennya. Hal ini untuk menghindari memulai Quickshell atau hypridle pada soket basi selama handoff kompositor; jika Hyprland diganti, systemd dapat memulai komponen lagi setelah runtime baru tersedia.

Peralihan tema membuat serial penerapan, pembuatan, proyeksi, dan pemuatan ulang konsumen sebagai satu operasi siklus hidup di bawah kunci tema. Itu tidak memuat ulang `argvus-config.service` atau memulai ulang layanan: proyeksi yang dijalankannya hanya menulis file dan membiarkan tema ke sakelar saat kunci ditahan, dan konsumen yang berjalan memuat ulang di tempatnya (lihat [Tampilan dan tema](/docs/developer-guide/subsystems/appearance-and-themes/)).

Saat Hyprland keluar, `argvus start` menghentikan `argvus-session.target` dan layanan terkait dengannya. Hal ini mencegah proses logout pada taskbar, shell, notifikasi, idle, dan clipboard.

## Memuat ulang

```sh
argvus sessionctl reload
```

Muat ulang akan mengimpor ulang lingkungan grafis, lalu **menerapkan** pembuatan komitmen ketika perintah domain sebelumnya seperti `apply-theme`, `accent`, `set`/`patch` atau perubahan blok widget-telemetri telah memproyeksikan perubahan. Dalam hal ini, ia memulihkan konsumen yang tercantum dalam rencana manifes dan tidak memanggil `argvus config project` lagi, karena proyeksi ulang akan membangun kembali pohon yang identik dan membuang rencana tersebut. Proyeksi, migrasi, dan `ensure` dilakukan hanya jika tidak ada rencana valid yang tertunda — saat sesi dimulai, atau setelah `argvus config recover` harus memulihkan dokumen sebelumnya. Larangan pengoperasian yang bersih hanya diperbolehkan jika proyeksi dan waktu proses terkelola sama-sama terkini. Jika status kanonik tidak berubah tetapi soket kompositor, layanan, atau konsumen sudah usang, muat ulang akan merekonsiliasinya alih-alih kembali lebih awal. Fan-out normal mencakup Hyprland, taskbar, telemetri widget, Panel Kontrol, notifikasi, wallpaper, dan idle sesuai dengan manifes proyeksi, sementara konsumen opsional yang tidak berubah tetap tidak tersentuh. Setiap file di bawah `data/generated/` ditulis oleh `argvus-config` pada saat ini, sehingga sesi tidak pernah merender ulang tema itu sendiri. Muat ulang kemudian menjalankan `theme-adapters.sh`, yang merekonsiliasi konsumen yang tidak dapat membaca sendiri dokumen kanonik atau proyeksi yang diterbitkan. Adaptor tersebut hanya membaca generasi yang dikomit: mereka menulis file dan cache konsumen mereka sendiri, dan mereka tidak pernah menulis dokumen kanonik atau `data/generated/`, jadi satu tindakan pengguna masih memerlukan satu mutasi, satu proyeksi, satu publikasi, dan satu muat ulang. Gunakan `argvus sessionctl restart <component>` hanya ketika satu layanan yang dimiliki memerlukan restart yang terisolasi secara eksplisit. Periksa [referensi layanan](/docs/reference/services/) untuk nama unit yang tepat.

Operasi proyeksi dan muat ulang diserialkan dengan kunci waktu proses pengguna. Untuk pelacakan waktu sementara, atur `ARGVUS_SESSION_PERF=1` sebelum menjalankan pemuatan ulang dan periksa `~/.local/state/argvus/session.log`; batas tahapan catatan jejak tanpa mengubah perilaku muat ulang normal.
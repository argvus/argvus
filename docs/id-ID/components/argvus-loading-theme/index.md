---
title: Memuat hamparan
description: Hamparan ditampilkan saat sesi dimuat ulang secara langsung.
---

Hamparan pemuatan adalah permukaan cangkang lapisan GTK 4 yang ditampilkan saat ARGVUS memuat ulang sesi secara langsung
(`argvus sessionctl reload`). Pengalih tema tidak menggunakannya: desktop berubah di tempatnya. Ini adalah pembantu yang dikirimkan dalam paket `argvus` di
`/usr/lib/argvus/loading-theme/splash`, dan tidak ditempatkan di `/usr/bin`.

Login tidak menggunakannya: splash penyapa dan spinner awal sesi adalah mode Wayland
pembantu handoff, yang menggambar spinner yang sama tanpa GTK dan menampilkan frame pertamanya di dalamnya
milidetik (lihat [Menyapa, mengunci, dan memercik](/docs/developer-guide/subsystems/greeter-lock-and-splash/)).
Dengan `--spinner-only` overlay hanya menampilkan spinner tersebut dan keluar pada `SIGTERM`. Dengan `--detached` ia tidak keluar ketika proses yang memulainya keluar; pemuatan ulang sesi menggunakan kedua tanda dan menghentikannya melalui file pidnya.
Ia membaca tema yang dipilih dari direktori data ARGVUS dan kembali ke paket ARGVUS
Palet gelap. Ia melaporkan kesiapan hanya setelah frame pertama dari setiap permukaan yang dipetakannya, bukan kapan
prosesnya dimulai.
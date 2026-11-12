---
title: ARGVUS Fokus
description: Pomodoro dan sesi fokus dengan jeda, notifikasi, dan riwayat sesi lokal.
---

ARGVUS Focus adalah aplikasi GTK 4 untuk bekerja dalam sesi fokus yang dipisahkan oleh istirahat. Ini terbuka sebagai popup di samping taskbar. Buka dari peluncur, dengan mengeklik indikator bilah tugas, dengan **SUPER + Shift + F**, atau dengan:

```sh
argvus focus              # open the popup near the pointer
argvus focus --toggle     # hide the popup if it is visible, show it otherwise
argvus focus --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus focus --start      # start or resume the current phase without opening the window
argvus focus --pause      # pause the running phase
argvus focus status --waybar         # print one Waybar JSON status and exit
argvus focus status --waybar --follow  # stream status changes for Waybar
```

Bilah tugas meneruskan koordinat klik akar sehingga popup menggunakan monitor yang diklik dan sejajar dengan bilah; peluncuran tanpa koordinat menggunakan posisi penunjuk. Jika penunjuk tetap berada di luar popup selama 5 detik, penunjuk akan menutup secara otomatis; memindahkan penunjuk ke dalam popup akan membatalkan hitungan mundur tersebut. **Esc** di halaman utama tetap menyembunyikannya dengan segera, dan **Esc** di subhalaman kembali ke halaman sebelumnya. Indikator bilah tugas hanya menampilkan ikonnya saat idle dan `MM:SS` yang tersisa selama suatu fase, dengan warna fase; fase yang dijeda mempertahankan waktu yang tersisa dan menambahkan status jeda. Perintah `status` berjalan di dalam `argvus` tanpa memulai GTK. `argvus --focus` adalah alias. Hanya satu instans Fokus yang berjalan per sesi: peluncuran kedua menyerahkan opsinya kepada instans yang sedang berjalan.

Fokus ARGVUS, Jam ARGVUS, dan Panel Kontrol berbagi tepi layar yang sama. Membuka Fokus akan menutup Jam dan Panel Kontrol terlebih dahulu; membuka salah satu permukaan tersebut akan menutup Fokus.

## Sesi

- Mode **Pomodoro** bergantian fokus dan istirahat pendek dan mengakhiri setiap siklus dengan istirahat panjang. Durasi defaultnya adalah 25, 5, dan 15 menit, dengan jeda panjang setelah 4 sesi fokus.
- Mode **Sesi tunggal** menjalankan satu sesi fokus dengan durasinya sendiri (45 menit secara default) tanpa jeda.
- Jendela menunjukkan fase (fokus, istirahat pendek, istirahat panjang, masing-masing dengan warna tersendiri), waktu yang tersisa di dalam cincin kemajuan, posisi dalam siklus dan total hari ini.
- **Mulai fokus / Mulai istirahat**, **Jeda**, **Lanjutkan**, **Batalkan sesi** (mengabaikan fase dan tidak mencatat apa pun) dan **Selesaikan sekarang** (mengakhiri sesi fokus lebih awal dan mencatat waktu yang sebenarnya dihabiskan). Saat istirahat, **Lewati istirahat** kembali fokus.
- Saat fase berakhir, fase berikutnya menunggu Anda, kecuali **Mulai berhenti secara otomatis** atau **Mulai fokus secara otomatis setelah jeda** aktif.

Keyboard: **Spasi** memulai atau menjeda, **Ctrl+,** membuka pengaturan, **Ctrl+H** riwayat, **Esc** kembali, **Ctrl+W** menyembunyikan jendela dan **Ctrl+Q** keluar.

## Pengaturan

Preset (Klasik 25/5/15, Diperpanjang 50/10/30, Pekerjaan mendalam 90/20/30) mengisi panjangnya, yang tetap dapat diedit. Pengaturannya juga mencakup sesi sebelum jeda panjang, permulaan otomatis, notifikasi di akhir fase, dan **suara sesi**: bel pendek dengan volumenya sendiri dan tombol **Uji suara**. Suara ini terpisah dari suara alarm [ARGVUS Clock](/docs/components/argvus-clock/).

## Sejarah

Setiap sesi fokus yang diselesaikan dicatat dengan awal, akhir, dan waktu yang dihabiskan; istirahat dan sesi yang dibatalkan tidak. Halaman riwayat menunjukkan sesi dan menit hari ini, total dan sesi terkini. Catatan yang lebih lama dari 365 hari akan dihapus. **Hapus riwayat** meminta konfirmasi.

## Jendela, latar belakang, dan tangguhkan

- Menutup jendela selama satu siklus hanya menyembunyikannya: sesi berlanjut dan memberi tahu Anda ketika fase berakhir. Karena tidak ada yang sedang berjalan, penutupan terhenti.
- Jika sistem ditangguhkan saat suatu fase berjalan, fase tersebut dijeda pada saat penghentian tersebut, dan sebuah pesan menyatakan berapa lama sistem tidur. Tidur tidak dihitung sebagai waktu fokus.
- Sesi ini disimpan pada setiap perubahan dan setiap 30 detik. Jika ARGVUS Focus ditutup selama suatu fase (logout, crash, **Ctrl+Q**), ARGVUS Focus akan kembali dijeda pada saat itu saat dibuka lagi; ia tidak pernah berpura-pura bahwa pengatur waktu terus berjalan sementara ia tidak dapat memberi tahu.
- Waktu diukur pada jam waktu boot sistem, sehingga mengubah jam dinding atau zona waktu tidak memperpendek atau memperpanjang suatu fase.## Jalur

| Jalur | Konten |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/focus/settings.json` | Pengaturan |
| `$XDG_DATA_HOME/argvus/focus/history.json` | Sesi selesai |
| `$XDG_STATE_HOME/argvus/focus/session.json` | Pos pemeriksaan sesi saat ini |
| `/usr/share/argvus/sounds/argvus-bell.oga` | Bel sesi |

File ditulis secara atom. File yang tidak dapat dibaca diubah namanya menjadi `<name>.corrupt-<time>` dan defaultnya digunakan; sebuah pesan memberitahu Anda kemana perginya.

Warnanya mengikuti tema ARGVUS aktif dan warna highlight (konsumen tema `time`). String berasal dari domain terjemahan `focus` dan `time`.

Kalender bilah tugas, Fokus ARGVUS, Jam ARGVUS, dan Panel Kontrol saling eksklusif: membuka satu akan menutup yang lain.

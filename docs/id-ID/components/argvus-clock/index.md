---
title: ARGVUS Jam
description: Jam, jam dunia, stopwatch, penghitung waktu mundur, dan alarm dengan suara.
---

ARGVUS Clock adalah aplikasi GTK 4 dengan waktu saat ini dan alat waktu: jam dunia, stopwatch, penghitung waktu mundur, dan alarm. Buka dari peluncur, dengan mengeklik waktu di bilah tugas, dengan **SUPER + Shift + C**, atau dengan:

```sh
argvus clock                     # open the popup near the pointer
argvus clock --toggle            # hide the popup if it is visible, show it otherwise
argvus clock --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus clock --page alarms       # open on a page: clock, world, stopwatch, timer, alarms
argvus clock --background        # run without a window, to keep alarms active
```

ARGVUS Clock terbuka sebagai popup yang sejajar dengan taskbar, dengan celah 4 piksel. Bilah tugas meneruskan koordinat klik akar sehingga popup menggunakan monitor yang diklik; meluncurkannya tanpa koordinat menggunakan posisi penunjuk. Jika penunjuk tetap berada di luar popup selama 5 detik, penunjuk akan menutup secara otomatis; memindahkan penunjuk ke dalam popup akan membatalkan hitungan mundur. **Esc** pada tab utama tetap menyembunyikannya dengan segera, dan **Esc** pada halaman penuh kembali ke tab sebelumnya. `argvus --clock` adalah alias. Mengklik tanggal di bilah tugas masih membuka kalender; Jam ARGVUS tidak menampilkan acara.

Fokus ARGVUS, Jam ARGVUS, dan Panel Kontrol berbagi tepi layar yang sama. Jam Pembukaan menutup Fokus dan Panel Kontrol terlebih dahulu; membuka salah satu permukaan tersebut akan menutup Jam.

## Halaman

- **Jam**: waktu dalam digit besar (24 jam secara default, 12 jam dalam pengaturan, detik opsional), tanggal lokal, zona waktu lokal dan offset UTC, dan alarm berikutnya.
- **Dunia**: kota ditambahkan dengan mencari kota atau zona waktu IANA (aksen dan huruf besar/kecil diabaikan). Masing-masing menunjukkan waktunya, perbedaannya dengan waktu setempat dan apakah kemarin, hari ini atau besok ada. Waktu musim panas berasal dari database zona waktu IANA; tidak ada yang dihitung dengan offset tetap. Kota dapat diatur ulang dan dihapus.
- **Stopwatch**: memulai, menjeda, melanjutkan, mengatur ulang, dan putaran (waktu putaran dan total).
- **Timer**: jam, menit dan detik, atau preset 5, 10, 15, 25, 30 dan 60 menit. Jeda, lanjutkan, mulai ulang, dan batalkan. Ketika itu berakhir, itu berdering seperti alarm. Pengatur waktu ini tidak bergantung pada sesi Fokus ARGVUS.
- **Alarm**: membuat, mengedit, menghidupkan dan mematikan, serta menghapus alarm. Masing-masing memiliki waktu, nama opsional, hari kerja yang berulang (tidak ada yang berarti berdering satu kali), suara, dan durasi tunda.

Keyboard: **Ctrl+1** ke **Ctrl+5** berpindah halaman, **Ctrl+N** membuat alarm, **Ctrl+,** membuka pengaturan, **Esc** kembali, **Ctrl+W** menutup jendela dan **Ctrl+Q** keluar.

## Suara alarm

Alarm mengeluarkan suara nyata sampai Anda menjawabnya:

- Suara: **Berpadu** (default) dan **Beacon**, dikirimkan dalam `/usr/share/argvus/sounds/`.
- **Volume alarm** (10 hingga 100, default 80) dan **Dengarkan suara alarm** ada dalam pengaturan; editor memiliki tombol tes untuk suara yang dipilih.
- Alarm berdering menampilkan kartu di bagian atas layar, di atas jendela lain, dan pemberitahuan, baik dengan **Tunda** dan **Tutup**. Dengan kartu terfokus, **Enter** ditutup dan **S** ditunda.
- Tunda berdering lagi setelah durasi tunda alarm (1 hingga 60 menit, default 10).
- Alarm yang tidak dijawab akan berhenti setelah **Berhenti berdering setelah** (5, 10, 15 atau 30 menit, default 10) dan meninggalkan notifikasi.
- Dua alarm yang berbunyi bersamaan keduanya terdaftar; yang kedua tidak pernah membungkam yang pertama.
- Jika suara tidak dapat diputar, kartu dan jendela menyatakan demikian dengan kesalahannya, dan kesalahan tersebut dicatat. ARGVUS Clock tidak pernah melaporkan suara yang tidak diputar.

Pemutaran menggunakan dukungan media GTK (GStreamer), sehingga dapat berfungsi dengan PipeWire, PulseAudio, atau ALSA. Paketnya bergantung pada `gst-plugins-base` (Ogg Vorbis) dan `gst-plugins-good` (output audio).

## Saat alarm berbunyi

Alarm hanya berdering saat Jam ARGVUS berjalan di sesi Anda:

- Menutup jendela membuat Jam ARGVUS tetap berjalan saat alarm, pengatur waktu berjalan, atau stopwatch yang berjalan membutuhkannya (**Terus berjalan saat jendela ditutup**, diaktifkan secara default). Tanpa menunggu apa pun, ia berhenti.
- **Mulai dengan sesi** mengaktifkan unit pengguna `argvus-clock.service`, yang menjalankan `argvus clock --background` setelah login. Ini dinonaktifkan secara default.- Alarm tidak dapat berdering saat Anda logout, komputer mati, atau ditangguhkan.
- Alarm yang terlambat hingga 5 menit masih berdering. Lebih dari itu (jam tidak berjalan, atau sistem tertidur atau mati) jam tidak berdering: dilaporkan sebagai **tidak terjawab** dalam pemberitahuan dan di halaman Alarm. Alarm yang berulang berlanjut ke hari berikutnya; alarm satu kali akan mati.
- Setiap kejadian disimpan sebagai ditangani sebelum berdering, jadi membuka kembali Jam ARGVUS tidak akan pernah membunyikan alarm yang sama dua kali, dan hanya satu kejadian yang berjalan per sesi.
- Alarm yang berulang dihitung dalam waktu lokal pada hari kerja yang dipilih, sehingga alarm tetap menjaga waktunya pada perubahan musim panas. Waktu yang dilewati oleh cincin pergantian pegas pada menit pertama setelah jeda; waktu yang terjadi dua kali berdering pertama kali.
- Pengatur waktu terus menghitung selama penangguhan. Jika ini berakhir saat sistem tidur, sistem akan berdering saat bangun dan memberitahukan seberapa larutnya. Setelah Jam ARGVUS ditutup, pengatur waktu atau stopwatch yang berjalan kembali dijeda.

## Jalur

| Jalur | Konten |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/clock/settings.json` | Pengaturan dan jam dunia |
| `$XDG_DATA_HOME/argvus/clock/alarms.json` | Alarm |
| `$XDG_STATE_HOME/argvus/clock/timers.json` | Pos pemeriksaan pengatur waktu dan stopwatch |
| `/usr/lib/systemd/user/argvus-clock.service` | Mulailah dengan sesi |
| `/usr/share/argvus/sounds/` | Suara yang dikemas |

File ditulis secara atom; file yang tidak dapat dibaca diubah namanya menjadi `<name>.corrupt-<time>` dan default digunakan. Warnanya mengikuti tema ARGVUS aktif dan warna highlight (konsumen tema `time`). String berasal dari domain terjemahan `clock` dan `time`.

Kalender bilah tugas, Fokus ARGVUS, Jam ARGVUS, dan Panel Kontrol saling eksklusif: membuka satu akan menutup yang lain.
---
title: Calendar
description: Gunakan kalender bilah tugas ARGVUS.
---

`argvus-taskbar-calendar` adalah popup kalender GTK4 yang dibuka dari modul tanggal Waybar atau dengan:

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```

Ini mendukung acara lokal, pengingat dan impor/ekspor ICS. Layanan penggunanya adalah `argvus-taskbar-calendar.service`. Gunakan `argvus taskbar-calendar --help` untuk opsi yang didukung oleh versi yang diinstal.

Kalender mengikuti tema ARGVUS yang aktif, termasuk varian Sticky dan Spaced. Perubahan tema diterapkan melalui cache pengguna yang dihasilkan; gunakan `argvus taskbar-calendar reload` jika popup yang sedang berjalan memerlukan penyegaran eksplisit.

Saat dibuka dari modul tanggal bilah tugas, popup ditautkan ke tepi bilah: popup terbuka di bawah bilah tugas atas dan di atas bilah tugas bawah, sejajar dengan tepi kanan modul tanggal. Peluncuran langsung tanpa geometri bilah tugas menggunakan penunjuk atau penggantian monitor.

## Perintah

```bash
argvus taskbar-calendar                 open the calendar
argvus taskbar-calendar toggle          show or hide it (the Waybar date module uses this)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          open the system configuration for editing (sudo)
argvus taskbar-calendar config-path     print the configuration path
argvus taskbar-calendar reload
argvus taskbar-calendar import file.ics
argvus taskbar-calendar export --output calendar.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```

## Acara dan pengingat

Acara dinonaktifkan secara default. Aktifkan di popup pada **Pengaturan > Acara**; itu
pilihan disimpan di direktori negara bagian dan juga diaktifkan dan dimulai
`argvus-taskbar-calendar.service`, yang menampilkan pengingat. Mematikannya akan menghentikan
layanan.

- **+ EVENT** (atau `n`) membuka editor untuk hari yang dipilih. Acara dapat dibuat dan
  berubah mulai hari ini; acara di hari yang lalu masih dapat dibuka dan dihapus.
- Acara baru dimulai pada pukul 09:00 dan berlangsung sesuai durasi default yang ditetapkan di Pengaturan. Memindahkan
  waktu mulai menjaga panjangnya; waktu berakhir lebih awal dari waktu mulai berarti peristiwa tersebut
  berakhir pada hari berikutnya.
- Setiap acara menampilkan notifikasi dengan suara di awal, apa pun itu
  **Pengingat** dan **Ulangi** tertulis di kotak. Acara yang disimpan setelah dimulai (sepanjang hari
  acara yang dibuat pada siang hari, misalnya) memberi tahu dalam waktu 30 detik.
- **Pengingat** menambahkan pemberitahuan sebelumnya, waktu yang dipilih sebelum permulaan (di
  dimulai ketika waktu itu telah berlalu).
- Pada acara sepanjang hari, kolom waktunya menjadi **BERITAHU DI**: acara mencakup keseluruhan
  hari dan memberi tahu pada waktu itu (00:00 memberi tahu pada awal hari). Waktunya adalah
  disimpan saat acara dibuka kembali.
- Untuk acara sepanjang hari, **Ulangi** memberi tahu lagi pada interval yang dipilih setelah waktu tersebut.
- Mengedit acara tanpa memindahkan permulaannya tidak mengulangi pengingat yang sudah ditampilkan.
- Notifikasi mengikuti bahasa kalender (Pengaturan > Bahasa).
- Suatu hari menunjukkan sebuah titik (`•`) ketika suatu peristiwa terjadi di sana, termasuk setiap hari dalam suatu
  acara semalam atau beberapa hari. Hari mengikuti zona waktu setempat.
- Acara lokal dihapus secara otomatis 10 menit setelah berakhir. Jika suatu acara sudah
  berakhir ketika disimpan, itu tetap tersedia setidaknya 10 menit setelah disimpan.

File `.ics` yang diimpor menyimpan tanggal, zona waktu (`TZID`) dan alarm (`VALARM`).
Acara yang diimpor juga memberi tahu saat dimulainya, selain alarmnya sendiri. Peristiwa berulang (`RRULE`)
disimpan dan diekspor, namun hanya kejadian pertama yang ditampilkan dan diingatkan.
Setelah pembaruan paket, layanan pengingat keluar dan systemd memulai versi baru
dalam waktu 30 detik. `sync` hanya melaporkan kalender jarak jauh yang dikonfigurasi; Sinkronisasi CalDAV tidak
belum diimplementasikan.

## Jalur

- Konfigurasi: `/etc/argvus/taskbar/calendar/config.toml` (seluruh sistem; default dikirimkan bersama paket).
- Gaya: `/etc/argvus/taskbar/calendar/style.css` dan `theme.css`; warna tema ARGVUS yang aktif berasal dari tema yang dibangun melalui cache di bawah.
- Data: `$XDG_DATA_HOME/argvus-taskbar-calendar/` (atau `~/.local/share/argvus-taskbar-calendar/`).
- Negara: `$XDG_STATE_HOME/argvus/taskbar/calendar/` (atau `~/.local/state/...`); tombol acara disimpan di sana.
- Tembolok: `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (atau `~/.cache/argvus-taskbar-calendar/`).

Warna highlight dibaca dari `~/.config/argvus/data/.accent-color` dan diterapkan saat runtime
di atas warna tema.

Pemilihan bahasa otomatis mengikuti pengguna `locale.conf`, dengan bahasa Inggris sebagai default. Preferensi bahasa kalender eksplisit masih mengesampingkan pemilihan otomatis.Kalender bilah tugas, Fokus ARGVUS, Jam ARGVUS, dan Panel Kontrol saling eksklusif: membuka satu akan menutup yang lain.

Kegagalan pembacaan kalender ditampilkan secara terpisah dari agenda kosong. Membuka database, memuat peristiwa harian, dan memuat indikator bulanan memiliki pesan yang berbeda. Pilih hari lagi untuk mencoba lagi. Penyegaran yang gagal mempertahankan data yang dimuat sebelumnya hanya untuk hari atau bulan yang sama; hasil yang valid dari kueri lain tetap tersedia.

## Katalog bahasa bersama

Antarmukanya menggunakan katalog `argvus-i18n` bersama untuk sepuluh bahasa. Teks disimpan di `resources/locales/<locale>/taskbar-calendar.json` di ruang kerja master dan diinstal oleh `argvus`. Bahasa Inggris adalah penggantinya; bahasa baru tidak lagi memerlukan kamus bilingual lokal.
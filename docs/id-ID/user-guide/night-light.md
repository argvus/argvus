---
title: Lampu Malam
description: Menyesuaikan suhu warna dan memperbaiki jadwal lokal di Hyprland.
---

Buka **Pusat Kontrol > Desktop & Input > Tampilan > Lampu Malam**, cari **Lampu Malam**, atau jalankan `argvus control-center displays night-light`.

Night Light dan jadwalnya pada awalnya tidak aktif. Pilih suhu dari **1000 hingga 20000 Kelvin** (default **4500 K**). Nilai yang lebih rendah memberikan warna yang lebih hangat. Halaman tersebut menunjukkan aktivasi manual yang disimpan, status efektif, dan transisi berikutnya. Masukkan nilai editan; Esc membatalkannya atau kembali ke Tampilan. Nilai yang tidak valid ditolak sebelum disimpan. Perubahan simpan dan terapkan segera. Jika penerapan gagal setelah penyimpanan, preferensi yang disimpan tetap terlihat di samping status efektif sebenarnya dan kesalahannya.

## Jadwalkan dan tindakan cepat

Jadwal tetap menggunakan zona waktu lokal saat ini. Awal disertakan dan akhir dikecualikan; **20:00–07:00** melewati tengah malam. Awal dan akhir harus berbeda dan menggunakan `HH:MM`. Tidak ada lokasi atau perhitungan matahari yang digunakan.

Gunakan kartu **Display** pada Panel Kontrol atau **SUPER + Shift + B** untuk beralih cepat. Pemilih kecerahan mempertahankan semua tindakan kecerahannya dan menambahkan Cahaya Malam saat backend tersedia, termasuk pada layar tanpa backend kecerahan. Tidak ada modul bilah tugas permanen.

Jika penjadwalan dinonaktifkan, aktivasi manual tetap dilakukan. Dengan penjadwalan aktif, peralihan cepat akan membuat penggantian waktu proses hingga transisi terjadwal berikutnya. Mengubah jadwal atau suhu akan menghapus kesalahan tersebut. Logout, instance kompositor baru, atau perubahan transisi berikutnya (termasuk perubahan zona waktu) juga akan membatalkannya.

## Perintah dan status

Semua antarmuka menggunakan perintah publik yang sama:

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```

`status` mengembalikan skema JSON versi 1: `available`, `unavailable_reason`, `settings` (diaktifkan, suhu, schedule_enabled, mulai, akhir), `active`, `effective_temperature`, `mode` (`manual` atau `scheduled`), `manual_override`, `next_transition` (ISO 8601 dengan offset), `minimum_temperature` dan `maximum_temperature`. Bidang yang efektif bernilai nol jika tidak tersedia; suhu efektif adalah nol ketika identitas aktif. Kegagalan ketersediaan dilaporkan dalam status; kegagalan tindakan mengembalikan bukan nol dengan diagnostik.

Preferensi kanonik adalah `/displays/night_light`, disimpan secara atom oleh `argvus config` di `$XDG_CONFIG_HOME/argvus/config/displays.json`. Penggantian yang mudah menguap dan kunci tindakan aktif di bawah `$XDG_RUNTIME_DIR/argvus/night-light/`. File monitor yang dihasilkan bersifat independen. Jangan edit file yang dihasilkan untuk mengonfigurasi fitur ini.

## Waktu proses

Memerlukan Hyprland dan **hyprsunset >= 0.4.0**. Sesi ini memulai `hyprsunset.service` upstream satu kali, menerapkan status tersimpan melalui `argvus-night-light-init.service`, dan mengevaluasi jadwal setiap menit melalui `argvus-night-light.timer` dan `argvus-night-light.service`. Perubahan jam dan zona waktu juga memicu evaluasi. Jika penjadwalan dinonaktifkan, operasi periodik hanya menyisakan status manual. Aplikasi berulang menanyakan identitas/suhu efektif dan melewatkan penulisan yang tidak berubah. Tidak perlu memuat ulang kompositor.

ARGVUS mengirimkan drop-in systemd untuk layanan upstream: dimulai dengan `--identity` dan mengisolasi penemuan konfigurasinya ke `/usr/share/argvus/session/config/night-light`. Hal ini menghindari persaingan profil asli dan membiarkan file `hyprsunset.conf` pribadi tidak tersentuh. Drop-in berlaku untuk layanan upstream tersebut saat paket sesi ARGVUS diinstal, termasuk dimulai di luar ARGVUS. Penggantian systemd pengguna yang ada dapat diutamakan; periksa `systemctl --user cat hyprsunset.service` saat mendiagnosis situasi itu. Hyprsunset 0.4.0 menolak argumen jalur terpisah `--config`, jadi penemuan konfigurasi diisolasi dengan variabel XDG lokal layanan.Backend menggunakan IPC Hyprsunset resmi (`temperature`, `identity`, `identity get`), dengan I/O terbatas dan pembacaan kembali setelah perubahan. Tidak ada shader, backend kompositor alternatif, penulisan konfigurasi sistem, atau peningkatan hak istimewa yang digunakan. Kontrol yang tidak tersedia dinonaktifkan di Pusat Kontrol dan disembunyikan di kartu Tampilan.
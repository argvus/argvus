---
title: ARGVUS Penganalisis Penggunaan Disk
description: Jelajahi penggunaan disk secara interaktif dengan ncdu di jendela bertema Kitty.
---

Buka **ARGVUS Disk Usage Analyzer** dari peluncur aplikasi atau jalankan:

```sh
argvus du
```

Penganalisis membuka jendela Kitty khusus dan terpusat dan menyematkan antarmuka ncdu asli di dalam bingkai ARGVUS-nya. Ini dimulai di `$HOME`. Edit jalur direktori atau pilih **Jelajahi** untuk membuka pengelola file yang dikonfigurasi. Dengan Superfile, navigasikan ke direktori dan tekan **Spasi** untuk memilihnya; `q` atau `Esc` dibatalkan. Manajer file terkonfigurasi lainnya terbuka secara normal, tetapi mungkin tidak mengembalikan direktori yang dipilih, jadi masukkan atau tempel jalurnya di penganalisis.

Gunakan Tab atau tombol panah untuk berpindah antar direktori, opsi, dan tindakan. Grafik, persentase, skema warna gelap, dan pengurutan pertama direktori diaktifkan secara default. Opsi pemindaian dan tampilan yang tersisa dapat diubah sebelum memilih **Analyze**. Di Hasil, gunakan kontrol keyboard ncdu; `q` kembali ke formulir dan `Esc` menghentikan pemindaian. Tekan `d` untuk menghapus file atau direktori yang dipilih. ncdu meminta konfirmasi; penghapusan bersifat permanen dan menggunakan izin pengguna saat ini. Tidak ada hak istimewa yang lebih tinggi yang diminta.

Antarmuka luar mengikuti tema dan lokal ARGVUS yang aktif. Kitty dan ncdu diperlukan; paket `argvus-devtools` menyediakan ncdu. Jika Kitty atau ncdu hilang, aplikasi melaporkan ketergantungan yang hilang.
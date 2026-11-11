---
title: Menyesuaikan warna highlight
description: Mengubah warna sorotan ARGVUS secara terpisah dari tema yang dipilih, dan kembali ke default tema.
---

**Warna sorotan** adalah aksen yang digunakan untuk batas fokus, pemilihan, dan status aktif di seluruh desktop. Setiap tema memiliki warna sorotan default, namun Anda dapat menggantinya dengan warna apa pun yang Anda inginkan tanpa mengubah tema.

## Ubah warna sorotan

1. Buka **Pusat Kontrol → Tampilan → Warna sorotan**.
2. Pilih **Edit warna sorotan**. Nilai saat ini ditampilkan di sebelah opsi.
3. Ketik warna RGB enam digit, dengan atau tanpa `#` di depannya. Misalnya `3590BD` dan `#3590BD` keduanya diterima.
4. Tekan `Enter` untuk menerapkan.

Anda juga dapat menempelkan warna ke dalam bidang. ARGVUS menerima teks yang ditempel hanya jika teks tersebut merupakan nilai RGB enam digit yang valid. Bentuk pendek seperti `#123` dan nilai dengan karakter non-heksadesimal ditolak dengan pesan kesalahan, dan warnanya tidak berubah.

Warna baru segera diterapkan ke desktop dan disimpan dalam konfigurasi ARGVUS Anda.

## Kembali ke tema default

Untuk kembali ke warna yang ditentukan oleh tema yang dipilih, buka **Pusat Kontrol → Tampilan → Warna sorotan** dan pilih **Setel ulang ke default tema**.

## Bagaimana warna highlight berinteraksi dengan tema

- Warna highlight yang Anda pilih merupakan warna yang independen. Warna ini tetap bertahan saat Anda beralih ke tema lain, sehingga warna yang Anda pilih tetap sama.
- Tema yang Anda terapkan setelah **Setel ulang ke default tema** mengambil alih warna sorotan lagi dan menggunakan defaultnya sendiri.
- Mengekspor tema aktif menyimpan warna highlight yang aktif saat Anda mengekspor sebagai aksen palet tema baru. Lihat [Mengimpor dan mengekspor tema](/docs/components/argvus-themes/import-and-export/).

## Dimana nilai disimpan

Warna highlight adalah bagian dari konfigurasi ARGVUS Anda, di `$XDG_CONFIG_HOME/argvus/config.json` (biasanya `~/.config/argvus/config.json`). Jangan edit file yang dihasilkan di bawah `$XDG_CONFIG_HOME/argvus/data/generated`: ARGVUS menulis ulang file tersebut dari konfigurasi.
---
title:
description: Buka terminal ARGVUS dan profil aplikasi terminal.
---

Buka terminal yang dikonfigurasi dengan:

```sh
argvus --terminal
```

`argvus terminal` (sub-perintah dari `argvus`) menyediakan integrasi terminal dan konfigurasi Kitty. `argvus-app-profiles` memasok profil untuk aplikasi terminal seperti Superfile dan Yazi; itu tidak menyediakan operator `argvus`.

`argvus-config` memproyeksikan tema ke dalam `data/generated/terminal/`, dan `argvus --yazi` dan `argvus --spf` membaca konfigurasinya dari pohon `data/generated/yazi/` dan `data/generated/superfile/` yang diproyeksikan, bukan dari salinan yang diwujudkan ke dalam direktori home Anda. Penggantian asli lengkap dalam direktori konfigurasi aplikasi tetap menang, sehingga Anda dapat mengganti pohon yang diproyeksikan dengan konfigurasi Anda sendiri.

Perubahan tema membuat ulang file turunan Kitty di bawah `$XDG_CACHE_HOME/argvus/argvus-terminal`. Generator membuat serial pembaruan dan mengganti setiap file secara atom. Jika terminal masih tidak terbuka, periksa tema aktif dan buat ulang cache dengan:

```sh
argvus config get /appearance/theme --raw
argvus terminal --apply
argvus terminal --print-config
```

Terminal membaca tema dan aksen yang dikelola
`$XDG_CONFIG_HOME/argvus/config.json` dulu. Warisan `.active-theme` dan
File `.accent-color` tetap menjadi cadangan untuk instalasi lama.

## Transparansi dan buram

Di bawah **Pusat Kontrol → Tampilan → Terminal**, konfigurasikan **Transparansi**
dan **Blur** secara terpisah dari `0%` hingga `100%`. Perubahan dilakukan dengan
Tombol **Terapkan** dan buat ulang profil Kitty di bawah
`$XDG_CACHE_HOME/argvus/argvus-terminal`; jendela Kitty yang ada menerima
konfigurasi yang diperbarui melalui muat ulang.

Transparansi `0%` menjaga latar belakang tetap buram dan `100%` membuatnya sepenuhnya
transparan. Blur menggunakan radius blur latar belakang yang didukung Kitty dan hanya memiliki a
efek terlihat ketika jendela transparan.

## Pemberitahuan perintah yang berjalan lama

Perintah yang dijalankan selama 30 detik atau lebih mengirimkan pemberitahuan ketika selesai,
selama jendela terminal tidak fokus. Notifikasinya menggunakan milik Kitty
`notify_on_cmd_finish` dengan integrasi shell yang sudah diaktifkan, jadi
`~/.zshrc` dan `~/.bashrc` tidak memerlukan perubahan. Ambang batas 30 detik telah ditetapkan
dalam konfigurasi Kitty.

## Mode Aplikasi TUI

Pusat Kontrol ARGVUS dan popup TUI bilah tugas menggunakan `argvus tui-terminal` dengan
profil aplikasi Kitty khusus. Ini mewarisi tema dan font ARGVUS aktif,
menetapkan kelas jendela yang stabil, dan keluar bersama anaknya alih-alih membuka a
cangkang. Pusat Kontrol menyembunyikan tab dan memiliki Transparansi dan Keburaman independen
nilai-nilai; monitor bilah tugas dan popup suhu menggunakan bilah tab bawah untuknya
judul `Press q to close` yang dilokalkan. Peluncur menerima `--title TITLE` untuk disetel
jendela Kitty dan judul tab yang tetap. Terminal normal menyimpan tabnya dan
pengaturan.

Ketika LANG tidak ada di lingkungan peluncur, `argvus terminal` menyediakan lokal pribadi dari `$XDG_CONFIG_HOME/locale.conf` (Bahasa Inggris secara default) sebelum Kitty menginisialisasi komposisi keyboard. Penggantian lingkungan lokal yang ada dipertahankan. Mulai ulang Kitty setelah memperbarui: mengekspor LANG ke dalam shell yang sudah ada tidak menginisialisasi ulang tabel penulisan Kitty.

LC_ALL dan LC_CTYPE kosong yang diwarisi dari inisialisasi sesi akan dihapus dari lingkungan anak Kitty. Jika tidak, Kitty akan memperlakukan lokal kosong sebagai eksplisit dan tidak dapat memuat tabel penulisannya, bahkan dengan LANG yang valid. Penggantian yang tidak kosong dipertahankan. Ini juga berlaku untuk profil baru dan memerlukan pembukaan kembali Kitty setelah memperbarui peluncur.

Jendela Pusat Kontrol khusus (`argvus-control-center`) menerima penggantian Kitty per peluncuran `confirm_os_window_close=0`, sehingga tindakan penutupan kompositor tidak meminta konfirmasi. Penggantian tidak ditulis ke dalam profil terminal bersama; TUI lain dan terminal reguler tetap mempertahankan kebijakan yang ada.

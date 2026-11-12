---
title: Kemasan
description: Membangun dan memeriksa paket ARGVUS Arch.
---

Setiap komponen memiliki kemasan Arch di bawah repositorinya. Periksa `packaging/archlinux/`, fungsi paket dan daftar file yang diinstal sebelum mendokumentasikan payload. Default paket pada `/usr/share/argvus` dan konfigurasi administrator pada `/etc/argvus` tidak dapat dipertukarkan.

Untuk verifikasi runtime, bandingkan konten paket dengan `pacman -Ql <package>` dan periksa penggantian pengguna aktif secara terpisah.

Build lokal menyimpan objek Cargo di `build/cargo-target`, di luar pohon sumber paket, sehingga `make build` yang berulang hanya mengkompilasi ulang apa yang berubah. `make clean` menghapus direktori itu, dan build berikutnya mulai dingin.

Jangan menyimpulkan payload runtime hanya dari nama repositori. Sebuah paket mungkin memiliki biner, layanan pengguna, default `/usr/share/argvus`, konfigurasi atau aset administrator `/etc/argvus`. Konfirmasikan payload sebenarnya dengan `PKGBUILD` dan `pacman -Ql` repositori setelah instalasi.

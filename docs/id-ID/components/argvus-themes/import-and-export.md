---
title: Mengimpor dan mengekspor tema
description: Ekspor tema aktif sebagai tema Anda sendiri, impor arsip tema, terapkan dan hapus.
---

Ekspor dan impor berfungsi dengan **tema**, dalam format yang sama dengan tema resmi. Mengekspor akan menyimpan tema aktif, dengan warna highlight dan wallpaper Anda, sebagai tema baru Anda sendiri. Mengimpor akan menginstal arsip tema di antara tema Anda. Keduanya ada di **Pusat Kontrol → Tampilan → Tema**, dan tema Anda tercantum di **Tema Khusus**.

## Isi tema yang diekspor

Arsipnya, `argvus-theme-<id>.tar.gz`, berisi satu direktori tema, ditata seperti tema resmi:

```text
<id>/theme.toml        the manifest (schema 2): name, category, palette, wallpaper
<id>/appearance.json  your visual settings, including layout, effects, fonts and panel presentation
<id>/effects.conf     per-surface effect values for the active theme, when present
<id>/overrides/...     the overrides of the theme it was made from, if any
<id>/wallpaper.<ext>   the wallpaper on screen when you exported
```

Manifes didasarkan pada tema aktif, dengan nama Anda, id baru, warna highlight Anda sebagai aksen palet dan wallpaper di sampingnya. Profil tampilan mempertahankan pengaturan visual yang saat ini diterapkan di ARGVUS, termasuk spasi jendela, batas, margin bilah tugas, presentasi panel, warna, efek, dan font. Wallpaper khusus dan warna aksen disimpan bersama tema. Menerapkan tema apa pun akan menerapkan kembali profil lengkapnya (atau default ARGVUS ketika tidak ada profil), menggantikan perubahan visual independen yang dibuat sejak terakhir kali diterapkan. Mengalihkan Sticky atau Spaced selalu menerapkan kembali geometri jendela dan taskbar mode tersebut; simpan geometri pribadi dalam tema khusus yang diekspor agar tetap tersedia saat tema tersebut diterapkan. Pengaturan non-visual seperti preferensi audio, keyboard, dan jaringan bukan bagian dari tema.

Tampilan yang diterapkan tetap ada selama sesi dimulai ulang dan dimuat ulang. Persiapan sesi membangun kembali file yang diperlukan dari konfigurasi saat ini tanpa mengembalikan geometri tema asli.

Pengaturan yang diubah setelah menerapkan tema bersifat sementara pada perubahan tema atau mode berikutnya. Ekspor tema dari **Pusat Kontrol → Tampilan → Tema → Ekspor tema** untuk mempertahankan konfigurasi visual saat ini dan memulihkannya nanti dengan menerapkan tema yang diekspor.

Tema Anda dipasang di `~/.local/share/argvus/themes/<id>/`. Mengekstrak arsip di sana akan menginstal tema juga.

## Ekspor tema aktif

1. Buka **Pusat Kontrol → Tampilan → Tema**.
2. Pilih **Ekspor tema**, atau tekan `e` di halaman Tema.
3. Ketikkan nama untuk tema tersebut. Bidang tersebut menunjukkan file yang akan ditulis.
4. Tekan `Enter`.

Id dibuat dari nama: huruf kecil, angka dan tanda hubung. Ketika id sudah digunakan oleh tema yang diinstal atau oleh arsip di direktori home Anda, `-2`, `-3` dan seterusnya akan ditambahkan. Arsip ditulis ke direktori home Anda, misalnya `~/argvus-theme-my-theme.tar.gz`. ARGVUS memeriksa tema sebelum menulisnya: manifes harus dimuat dan setiap komponen harus dirender dalam kedua mode.

## Impor tema

1. Salin arsip `.tar.gz` ke direktori home Anda (`~`). ARGVUS mencantumkan arsip di sana yang memiliki tema.
2. Buka **Pusat Kontrol → Tampilan → Tema → Impor tema**, atau tekan `i` di halaman Tema.
3. Pilih arsip dan tekan `Enter`.

ARGVUS memeriksa arsip sebelum menginstalnya. Itu harus menampung satu direktori dengan `theme.toml`, dengan jalur relatif saja (tidak ada `..`, tanpa tautan), paling banyak 128 entri dan batas ukuran per file; id harus cocok dengan direktori, wallpaper harus ada di arsip dan setiap komponen harus dirender dalam kedua mode. Profil tampilan yang disertakan harus menggunakan bagian yang didukung dan JSON yang valid. Jika ada pemeriksaan yang gagal, tidak ada yang diinstal.

Tema ditambahkan ke **Tema Khusus**. Mengimpor tidak berlaku: Anda memilih kapan.

### Nama bertabrakan

- Jika salah satu tema Anda memiliki id yang sama, ARGVUS menanyakan **Ganti tema Anda dengan ID yang sama?**. Pilih **Ganti** untuk menggantinya, atau tekan `Esc` untuk menyimpannya. Tema yang diganti disimpan sebagai cadangan hingga tema baru diinstal.- Arsip dengan id tema resmi ditolak. Ekspor lagi dengan nama lain.

## Terapkan tema khusus

1. Buka **Pusat Kontrol → Tampilan → Tema → Tema Khusus**.
2. Pilih tema dan tekan `Enter`.

Konfigurasi lengkap tema yang diekspor diterapkan, termasuk wallpaper, spasi, batas, warna aksen, efek, font, dan presentasi panel. Nilai visualnya menggantikan pengaturan saat ini. ARGVUS kemudian memulai ulang bilah tugas, layanan muat ulang Hyprland, wallpaper, dan Panel Kontrol sehingga setiap permukaan menggunakan tema khusus.

## Hapus tema khusus

1. Buka **Pusat Kontrol → Tampilan → Tema → Tema Khusus**.
2. Pilih tema dan tekan `d`.
3. Konfirmasi dengan `y`, atau tekan `n` atau `Esc` untuk membatalkan.

Direktori tema dihapus dari `~/.local/share/argvus/themes`. Jika itu adalah tema aktif, ARGVUS beralih ke **ARGVUS Dark** terlebih dahulu.

## Profil versi sebelumnya

Versi sebelumnya mengekspor *profil tema* (`profile.tar.gz`), salinan pengaturan tampilan. Saat pertama kali halaman Tema dibuka, setiap profil di `~/.local/share/argvus/themes/<name>/` diubah menjadi tema yang dibuat dari tema dasarnya, dengan warna highlight dan wallpapernya sendiri. Profil asli, dan daftar `custom-themes.json` lama, dipindahkan ke `~/.config/argvus/data/backups/theme-profiles-<seconds>/`, dan baris status melaporkan konversi. Profil yang tema dasarnya tidak diinstal hanya dipindahkan ke cadangan.

## Bagikan tema

Tema yang diekspor adalah arsip biasa. Salin ke komputer lain yang menjalankan ARGVUS dan impor ke sana.

## Pemecahan masalah

- **Arsip tidak terdaftar.** Arsip harus langsung berada di dalam direktori home Anda, diakhiri dengan `.tar.gz` dan tahan `<id>/theme.toml`.
- **Impor ditolak.** Baris status menyatakan alasannya: tema resmi dengan id yang sama, wallpaper yang hilang, manifes yang tidak dapat dimuat, atau komponen yang tidak dirender.
- **Ekspor ditolak.** Nama memerlukan setidaknya satu huruf atau angka, dan tema aktif harus diinstal.
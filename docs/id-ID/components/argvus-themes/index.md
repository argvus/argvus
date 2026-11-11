---
title: Themes
description: Instal, pilih, sesuaikan, impor dan ekspor tema ARGVUS.
---

Tema di ARGVUS mengatur warna, permukaan, default wallpaper, dan geometri tata letak yang digunakan di seluruh desktop: Hyprland, Waybar, Panel Kontrol, bilah tugas, layar kunci, notifikasi, dan aplikasi ARGVUS. Panduan ini mencakup semua yang Anda perlukan untuk bekerja dengan tema sebagai pengguna.

## Apa yang termasuk dalam sebuah tema

Setiap tema resmi merupakan identitas visual lengkap dengan dua mode:

- **Sticky** — tata letak ringkas dengan celah jendela kecil, tanpa celah luar, dan bilah tugas dekat dengan tepi layar.
- **Spaced** — tata letak yang luas dengan celah lebih besar, sudut membulat, dan margin di sekitar bilah tugas dan permukaan shell.

Memilih tema tidak mengubah identitasnya: palet, warna sorotan default, dan asosiasi wallpaper tetap sama antar mode. Mode ini merupakan pengaturan terpisah, lihat [Tema resmi](/docs/components/argvus-themes/official-themes/).

## Halaman dalam panduan ini

- [Menginstal tema](/docs/components/argvus-themes/installing-themes/) — dari mana tema resmi berasal dan cara menginstal salah satu atau semuanya dengan `pacman`.
- [Tema resmi](/docs/components/argvus-themes/official-themes/) — katalog, warna highlight default setiap tema, dan cara memilih tema serta modenya.
- [Menyesuaikan warna sorotan](/docs/components/argvus-themes/highlight-color/) — mengubah warna aksen apa pun temanya.
- [Mengimpor dan mengekspor tema](/docs/components/argvus-themes/import-and-export/) — mengekspor tema aktif, dengan warna highlight dan wallpaper, sebagai tema Anda sendiri dan mengimpornya ke komputer lain.

## Referensi cepat

| Tugas | Dimana |
| --- | --- |
| Pilih tema | **Pusat Kontrol → Tampilan → Tema → Tema Resmi** |
| Ganti Lengket dan Spasi | **Pusat Kontrol → Tampilan → Mode** |
| Ubah warna sorotan | **Pusat Kontrol → Tampilan → Warna sorotan** |
| Ekspor tema aktif | **Pusat Kontrol → Tampilan → Tema → Ekspor tema** |
| Impor tema | **Pusat Kontrol → Tampilan → Tema → Impor tema** |
| Buka pemilih tema dari keyboard | `SUPER + SHIFT + T` |

## Dari mana tema berasal

ARGVUS mengirimkan dua tema bawaan, **ARGVUS Dark** dan **ARGVUS Light**, dengan paket `argvus-appearance`. Setiap tema resmi lainnya adalah paket `argvus-theme-<name>` terpisah. Menginstal paket tema akan menambahkannya ke pemilih tema; tidak perlu memulai kembali sesi. Tema Anda sendiri, yang diekspor atau diimpor, dicantumkan secara terpisah sebagai **Tema Khusus**.

## Petunjuk papan ketik

Halaman tema Pusat Kontrol memperlihatkan kuncinya di footer:

- `↑` / `↓` — navigasi.
- `Enter` — terapkan atau buka.
- `e` — mengekspor tema aktif, pada halaman Tema.
- `i` — mengimpor tema, pada halaman Tema.
- `d` — menghapus tema khusus.
- `Esc` — kembali.

## Pemecahan masalah

- **Tema yang saya instal tidak muncul.** Konfirmasikan bahwa paket diinstal dengan `pacman -Q argvus-theme-<name>`, lalu buka kembali Pusat Kontrol. Paket harus diinstal dengan kehadiran `argvus-appearance`.
- **Warna sorotan berubah setelah saya mengganti tema.** Warna sorotan khusus tetap bertahan dalam peralihan tema. Gunakan **Setel ulang ke default tema** untuk mengembalikan warna pada tema.
- **Arsip tema tidak terdaftar untuk diimpor.** Arsip tersebut harus langsung berada di direktori home Anda dan berisi `<id>/theme.toml`. Lihat [Mengimpor dan mengekspor tema](/docs/components/argvus-themes/import-and-export/).
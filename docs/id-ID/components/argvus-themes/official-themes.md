---
title: Tema resmi
description: Katalog tema ARGVUS resmi, warna sorotan default, dan cara memilih tema serta mode Lengket atau Spasi.
---

Halaman ini mencantumkan tema resmi yang tersedia sebagai paket tema dan menjelaskan cara memilihnya. Untuk menginstalnya, lihat [Menginstal tema](/docs/components/argvus-themes/installing-themes/).

## Pilih tema

1. Buka **Pusat Kontrol → Tampilan → Tema**.
2. Pilih **Tema Resmi**.
3. Pilih **Gelap** atau **Terang**.
4. Pilih tema keluarga. Tema diterapkan segera setelah Anda memilihnya.

Tema yang dipilih diterapkan ke seluruh desktop sekaligus: bilah tugas yang sedang berjalan, panel, notifikasi, jendela, dan wallpaper berubah warna di tempatnya, tanpa layar pemuatan dan tanpa memulai ulang. ARGVUS menyimpan pilihan wallpaper Anda kecuali tema tersebut diterapkan dengan asosiasi wallpapernya sendiri.

Anda juga dapat menekan `SUPER + SHIFT + T` untuk membuka pemilih tema dari keyboard. Gunakan tombol panah untuk berpindah kategori, `Enter` untuk memilih, dan `Esc` untuk menutup.

## Lengket dan Berjarak

Setiap tema bekerja dalam dua mode. Mode ini tidak bergantung pada tema: mengubah tema akan mempertahankan mode saat ini, dan mengubah mode akan mempertahankan tema saat ini.

| Modus | Celah jendela | Celah luar | Pembulatan jendela | Bilah tugas dan margin shell |
| --- | ---: | ---: | ---: | ---: |
| Lengket | `2` | `0` | `0` | dekat tepi |
| Spasi | `10` | `18` | `4` | `18` |

Untuk beralih mode, buka **Pusat Kontrol → Tampilan → Mode** dan pilih **Sticky** atau **Spaced**. Penyesuaian manual Anda terhadap nilai tata letak ini tetap berlaku hingga pemilihan tema eksplisit berikutnya.

## Tema gelap

| Tema | Paket | Sorotan bawaan |
| --- | --- | --- |
| Satu Gelap | `argvus-theme-one-dark` | `#61AFEF` |
| Drakula | `argvus-theme-dracula` | `#BD93F9` |
| Perak Gelap | `argvus-theme-silver-dark` | `#595959` |
| Batu Tulis Gelap | `argvus-theme-slate-dark` | `#7391A5` |
| Alam Semesta | `argvus-theme-universe` | `#EEEEEE` |
| Gruvbox Tinggi Gelap | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| Gruvbox Gelap | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| Pinus Rosé | `argvus-theme-rose-pine` | `#C4A7E7` |
| Malam Tokyo | `argvus-theme-tokyo-night` | `#7AA2F7` |
| Kesendirian | `argvus-theme-solitude` | `#798186` |
| Matahari terbenam | `argvus-theme-sunset` | `#E2BE8A` |
| Matriks | `argvus-theme-matrix` | `#82FB9C` |
| Monokai Gelap | `argvus-theme-monokai-dark` | `#78DCE8` |
| Nyonya Kabut | `argvus-theme-lady-mist` | `#354B78` |

## Tema ringan

| Tema | Paket | Sorotan bawaan |
| --- | --- | --- |
| Cahaya GitHub | `argvus-theme-github-light` | `#0969DA` |
| Cahaya Tersolaris | `argvus-theme-solarized-light` | `#268BD2` |
| Cahaya Utara | `argvus-theme-nord-light` | `#5E81AC` |
| Satu Cahaya | `argvus-theme-one-light` | `#4078F2` |
| Cahaya Hutan Abadi | `argvus-theme-everforest-light` | `#3A94C5` |
| Catpuccin Latte | `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Lampu Gruvbox | `argvus-theme-gruvbox-light` | `#458588` |
| Cat Air | `argvus-theme-watercolor` | `#1A639C` |
| Cahaya Obsidian | `argvus-theme-obsidian-light` | `#334A78` |

## Tema bawaan

**ARGVUS Dark** dan **ARGVUS Light** disediakan oleh `argvus-appearance` dan selalu tersedia di **Tema Resmi**. ARGVUS Dark menggunakan sorotan default `#3590BD`; Lampu ARGVUS menggunakan `#181818`.

## Tema apa yang dikontrol

Tema menentukan palet dan perlakuan permukaan yang digunakan oleh komponen desktop, warna sorotan default, asosiasi wallpaper default, serta default transparansi dan blur. Komponen desktop membaca status ini dibandingkan menyimpan salinannya sendiri, sehingga perubahan tema menjangkau kompositor, bilah tugas, Panel Kontrol, layar kunci, notifikasi, dan aplikasi secara bersamaan.Warna sorotan default hanyalah titik awal. Anda dapat menggantinya dengan warna enam digit apa pun, lihat [Menyesuaikan warna sorotan](/docs/components/argvus-themes/highlight-color/).
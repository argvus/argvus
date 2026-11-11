---
title: Appearance
description: Konfigurasikan bahasa visual yang digunakan bersama oleh komponen ARGVUS.
---

Sistem tampilan menerapkan satu status tema pada aplikasi Hyprland, GTK, Qt, Waybar, Quickshell, Rofi, Dunst, Hyprlock, dan ARGVUS.

## Tema keluarga

Dua puluh tiga keluarga di bawah ini ditentukan oleh muatan tema saat ini. Kolom warna adalah warna batas aktif/aksen default keluarga; pengguna dapat memilih aksen RGB lain yang valid.

| Keluarga | Setel ulang warna sorotan | Latar belakang dasar | Mode |
| --- | --- | --- | --- |
| Satu Gelap | `#61AFEF` | `#282C34` | Lengket, Spasi |
| Drakula | `#BD93F9` | `#282A36` | Lengket, Spasi |
| ARGVUS Gelap | `#3590bd` | `#111316` | Lengket, Spasi |
| Perak Gelap | `#595959` | `#111316` | Lengket, Spasi |
| Batu Tulis Gelap | `#7391a5` | `#2f3541` | Lengket, Spasi |
| Alam Semesta Gelap | `#eeeeee` | `#000000` | Lengket, Spasi |
| ARGVUS Dark Gruvbox Tinggi | `#D79921` | `#282828` | Lengket, Spasi |
| Kotak Gruv Gelap ARGVUS | `#D4BE98` | `#282828` | Lengket, Spasi |
| Cahaya ARGVUS | `#181818` | `#f7f7f7` | Lengket, Spasi |
| Cahaya GitHub | `#0969DA` | `#FFFFFF` | Lengket, Spasi |
| Catpuccin Latte | `#1E66F5` | `#EFF1F5` | Lengket, Spasi |
| Kotak Gruv Ringan ARGVUS | `#458588` | `#FBF1C7` | Lengket, Spasi |
| Pinus Rosé | `#C4A7E7` | `#191724` | Lengket, Spasi |
| Tokyo-Malam | `#7AA2F7` | `#1A1B26` | Lengket, Spasi |
| Kesendirian | `#798186` | `#101315` | Lengket, Spasi |
| Matahari terbenam | `#E2BE8A` | `#0F0F0F` | Lengket, Spasi |
| Matriks | `#82FB9C` | `#0B0C16` | Lengket, Spasi |
| Monokai Gelap | `#78DCE8` | `#2D2A2E` | Lengket, Spasi |
| Nyonya Kabut | `#354B78` | `#131313` | Lengket, Spasi |
| Satu Cahaya | `#4078F2` | `#FAFAFA` | Lengket, Spasi |
| Cahaya Hutan Abadi | `#3A94C5` | `#FDF6E3` | Lengket, Spasi |
| Cahaya Tersolaris | `#268BD2` | `#FDF6E3` | Lengket, Spasi |
| Cahaya Utara | `#5E81AC` | `#ECEFF4` | Lengket, Spasi |
| Cat Air | `#1A639C` | `#E9E3D3` | Lengket, Spasi |

Warna yang ditampilkan dalam tabel adalah nilai yang ditulis ke `config.json` ketika kelompok tersebut dipilih atau ketika **Reset to theme default** digunakan. Aksen RGB enam digit yang dipilih secara manual berlaku hingga peralihan tema berikutnya, yang menggantikannya dengan default tema yang dipilih.

**Sticky** adalah tata letak yang ringkas: jendela ubin menggunakan celah internal kecil, celah luar nol, dan bilah tugas berada dekat dengan tepi layar. **Spasi** menambahkan celah yang lebih besar, sudut membulat, bayangan, dan margin di sekitar bilah tugas dan permukaan shell.

Nilai tata letak `float` dan pengidentifikasi tema `-float` yang ada diakui sebagai nama lama untuk Spaced. Nilai tata letak baru dan pengidentifikasi tema yang diselesaikan menggunakan `spaced` dan `-spaced`.

Mode ini mengubah geometri dan perawatan permukaan; itu tidak menciptakan keluarga warna keenam. Lihat [Tema](/docs/user-guide/appearance/) untuk pemilihan dan [Desktop](/docs/user-guide/desktop/) untuk penempatan bilah tugas dan jendela.

- [Tema](/docs/user-guide/appearance/)
- [Wallpaper](/docs/argvus-appearance/wallpapers/)
- [Font dan ikon](/docs/argvus-appearance/fonts-and-icons/)
- [Efek](/docs/argvus-appearance/effects/)

Pusat Kontrol memperlihatkan pengaturan yang dilihat pengguna. Implementasinya dibagikan oleh `argvus-appearance`, `argvus-session`, komponen shell dan paket aset. Lihat [arsitektur tampilan](/docs/developer-guide/subsystems/appearance-and-themes/).

## Jendela goyah

Buka **Desktop & Input → Hyprland → Animasi → Windows** dan gunakan
Kontrol **Goyangan**. Cari `wobble`, atau langsung buka halamannya:

```sh
argvus control-center hyprland animations windows
```

Goyangan dinonaktifkan secara default. Default ARGVUS-nya adalah kekakuan `100`, redaman`10`, massa `1`, intensitas `1` dan resolusi mesh `8`. Kekakuan dan penerimaan massa
`0.0001–1000`, redaman `0–1000`, intensitas `0–2`, dan mesh bilangan bulat `2–32`.
Resolusi mesh yang lebih tinggi meningkatkan biaya rendering. Gunakan tombol panah untuk menyesuaikan
nilai atau Enter untuk mengetikkannya. **Kembalikan default Wobble** mengubah draf;
**Terapkan** menyimpan semua pengeditan animasi yang tertunda dan memuat ulang Hyprland satu kali, tanpa perlu
memulai kembali sesi. Kembali ke dalam Animasi akan menyimpan draf bersama.

Sakelar Animasi global menekan Wobble sambil mempertahankan individualnya
pengaturan. Nilai kanonik berada di bawah `effects.wobble_*` di negara bagian ARGVUS;
`hypr/state.lua` dibuat dan tidak boleh diedit secara manual.

Hal ini memerlukan Hyprland **0.57.0 atau lebih baru**, yang diterapkan oleh `argvus-hyprland`
ketergantungan paket. Versi 0.56 tidak didukung untuk konfigurasi ini. Itu
intensitas maksimum mengikuti [sumber 0.57.0 dirilis](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp),
yang membatasinya pada `2`, meskipun dokumentasi yang lebih baru mungkin mencantumkan `3`.
---
title: Menginstal tema
description: Instal paket tema ARGVUS resmi dengan pacman.
---

Tema resmi diterbitkan sebagai paket Arch Linux. Setiap tema adalah paketnya sendiri yang diberi nama `argvus-theme-<THEME_NAME>`.

## Di mana mendapatkan tema

Sumber dari setiap tema resmi ada di organisasi proyek ARGVUS:

- Setiap tema adalah repositori bernama `argvus-theme-<THEME_NAME>` (misalnya `argvus-theme-dracula`), dibuat sebagai paket dengan nama yang sama.

Paket tema bergantung pada `argvus-appearance`. Instal ARGVUS terlebih dahulu seperti dijelaskan pada [Instalasi](/docs/getting-started/installation/), sehingga komponen tampilannya ada.

## Instal satu tema

Untuk menginstal setiap tema resmi sekaligus, instal metapackage `argvus-themes`:

```bash
sudo pacman -S argvus-themes
```

Itu tergantung pada semua paket `argvus-theme-*` yang diterbitkan. ARGVUS Gelap dan ARGVUS
Lampu sudah dikirimkan oleh `argvus-appearance`.

Instal satu tema dengan nama paketnya:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```

Misalnya:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```

Paket ini menginstal manifes tema, `/usr/share/argvus/themes/<theme-id>/theme.toml` (nama, kategori, wallpaper dan paletnya, dengan opsional `overrides/`), dan wallpaper tema di bawah `/usr/share/backgrounds/argvus/`. File setiap komponen dirender dari palet saat tema diterapkan. Itu tidak menulis ke direktori home Anda, dan tidak mengubah tema Anda saat ini. Pilih dari Pusat Kontrol setelahnya, lihat [Tema resmi](/docs/components/argvus-themes/official-themes/).

## Instal beberapa tema

Tidak ada paket meta yang menginstal setiap tema. Instal tema yang Anda inginkan dalam satu perintah:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## Paket yang tersedia

Paket di bawah ini adalah paket tema resmi yang diterbitkan hari ini.

**Tema gelap**

| Paket | Tema |
| --- | --- |
| `argvus-theme-one-dark` | Satu Gelap |
| `argvus-theme-dracula` | Drakula |
| `argvus-theme-silver-dark` | Perak Gelap |
| `argvus-theme-slate-dark` | Batu Tulis Gelap |
| `argvus-theme-universe` | Alam Semesta |
| `argvus-theme-gruvbox-high-dark` | Gruvbox Tinggi Gelap |
| `argvus-theme-gruvbox-dark` | Gruvbox Gelap |
| `argvus-theme-rose-pine` | Pinus Rosé |
| `argvus-theme-tokyo-night` | Malam Tokyo |
| `argvus-theme-solitude` | Kesendirian |
| `argvus-theme-sunset` | Matahari terbenam |
| `argvus-theme-matrix` | Matriks |
| `argvus-theme-monokai-dark` | Monokai Gelap |

**Tema ringan**

| Paket | Tema |
| --- | --- |
| `argvus-theme-github-light` | Cahaya GitHub |
| `argvus-theme-solarized-light` | Cahaya Tersolaris |
| `argvus-theme-nord-light` | Cahaya Utara |
| `argvus-theme-one-light` | Satu Cahaya |
| `argvus-theme-everforest-light` | Cahaya Hutan Abadi |
| `argvus-theme-catppuccin-latte` | Catpuccin Latte |
| `argvus-theme-gruvbox-light` | Lampu Gruvbox |
| `argvus-theme-watercolor` | Cat Air |

Tema bawaan **ARGVUS Dark** dan **ARGVUS Light** tidak memerlukan paket terpisah; mereka datang dengan `argvus-appearance`.

## Periksa apa yang diinstal

Daftar paket tema yang diinstal dengan:

```bash
pacman -Qs argvus-theme
```

Buka Pusat Kontrol dan buka **Penampilan → Tema → Tema Resmi** untuk melihat tema yang saat ini tersedia di pemilih. Tema yang diinstal tetapi hilang dari daftar tersebut adalah masalah pengemasan; lihat bagian pemecahan masalah di [Panduan tema](/docs/components/argvus-themes/).

## Tambahkan tema Anda sendiri

Tema adalah sebuah manifes, jadi tema Anda sendiri tidak memerlukan paket. Buat
`~/.local/share/argvus/themes/<id>/theme.toml`, dimana nama direktorinya adalah
ID tema (huruf kecil, angka, dan tanda hubung tunggal):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```

Hanya `bg`, `fg` dan `accent` yang diperlukan: setiap warna lain memiliki default
berasal dari mereka, dan manifes mendefinisikan seseorang hanya untuk mengubahnya. Periksa itu,
lalu terapkan seperti tema lainnya:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```

`argvus theme list` menampilkan tema Anda di sebelah tema resmi (resmi
tema dengan ID yang sama menang). Tema Anda muncul di menu tema
(`SUPER + Shift + T`); penyambut pra-login tidak dapat membaca direktori home Anda, jadi
itu mempertahankan tema resmi terakhir yang Anda terapkan saat salah satu tema Anda aktif.

Salinan per tema yang ditulis oleh versi ARGVUS lama`~/.config/argvus/data/<app>/themes/` tidak lagi terbaca. Peralihan tema pertama
setelah pembaruan memindahkannya ke `~/.config/argvus/data/backups/theme-copies-<time>/`;
untuk menyimpan file yang Anda edit, masukkan ke dalam tema Anda sendiri di bawah `overrides/` dan
beri nama di tabel manifes `[overrides]`.

## Hapus tema

Hapus satu paket tema dengan:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```

Menghapus paket tema tidak menghapus profil yang Anda impor. Jika tema yang dihapus adalah tema yang sedang diterapkan, pilih tema lain dari Pusat Kontrol sebelum menghapusnya.

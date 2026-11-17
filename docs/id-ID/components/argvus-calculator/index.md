---
title: Kalkulator ARGVUS
description: Kalkulator ARGVUS — GTK4 / ARGVUS
---

# Kalkulator ARGVUS

`argvus calculator` · launcher: **Kalkulator ARGVUS**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **Ilmiah**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **Radian**: Tidak dicentang: derajat.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **Hapus**; Backspace: **Hapus karakter**; Tab / Shift+Tab.
- **Salin hasil**; **Riwayat**: 100; **Hapus riwayat**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## Riwayat

Riwayat tetap tersimpan setelah ditutup (hingga 100 hasil). Menghapus riwayat juga menghapus data tersimpan. Pengaturan: ↑/↓ untuk navigasi, Enter atau klik untuk aktivasi, Escape untuk kembali.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

Selalu terbuka sebagai jendela mengambang kecil di tengah (370 × 420); mode ilmiah dapat memperbesarnya.

Layar utama hanya berisi ekspresi, hasil dan papan tombol. Menu menyediakan mode ilmiah, satuan sudut, salin dan layar riwayat terpisah. Kembali atau Escape kembali ke perhitungan; memilih hasil menggunakannya kembali. Mode dasar memulihkan jendela kecil dan mempertahankan ekspresi.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

Memperbesar jendela memperbesar tombol. Setiap peluncuran baru memakai ukuran bawaan. Layar sekunder memiliki panah kembali di atas.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

Mode dasar atau ilmiah yang dipilih langsung disimpan dan dipulihkan saat dibuka kembali.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).

---
title: Kalkulator ARGVUS
description: Dasar · Ilmiah · Pemrograman · Keuangan · Konversi satuan
---

# Kalkulator ARGVUS

`argvus calculator` · `SUPER + C` · launcher: **ARGVUS Calculator**

Pilih profil di Pengaturan → Profil dengan ↑/↓ dan Enter atau klik baris. Profil dan opsi langsung disimpan dan dipulihkan saat dibuka kembali. Setiap profil menyimpan ekspresinya sendiri selama sesi. Enter atau = mengganti ekspresi dengan hasil tanpa memilihnya, dengan kursor di akhir.

Suku bunga adalah persentase per periode; gunakan periode sama untuk bunga, jumlah dan pembayaran. Pembayaran dilakukan di akhir periode; bunga nol didukung. Bunga sederhana dan majemuk menghasilkan total akumulasi. Konversi offline tanpa mata uang. Riwayat menyimpan 100 catatan beserta profil, opsi dan masukan keuangan; memilih hasil memulihkan konteks. Menghapus riwayat juga mengosongkan berkas tersimpan.

## Pengaturan

- `s Pengaturan` → **Profil** → Dasar / Ilmiah / Pemrograman / Keuangan / Konversi satuan.
- ↑ / ↓ · Enter · Escape / ←.
- **Salin hasil** · **Riwayat** · **Hapus riwayat**.
- **Dasar**: 370 × 420; **Ilmiah**: 440 × 640.
- **Pemrograman**: 520 × 640; **Keuangan**: 440 × 640; **Konversi satuan**: 440 × 520.

## Dasar / Ilmiah

`+ - * / %` · `50% = 0.5` · `1.5 = 1,5` · `2+2` → Enter → `4` → `+2` → Enter → `6`.

`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `log`, `ln`, `exp`, `pi`, `e`, `^`, `(`, `)`.

**Satuan sudut**: Derajat / Radian. `-2^2 = -4`; `2^3^2 = 512`; `2*pi`.

IEEE 754 / f64 · 15 digit signifikan · 4096 byte · 64 tingkat pengurai.

## Pemrograman

Bawaan: desimal, 64 bit bertanda. Aritmetika berputar sesuai lebar kata; geser kanan bertanda memperluas tanda. Pembagian bilangan bulat dipotong menuju nol. Pembagian nol dan geser di luar [0, bits) adalah kesalahan. Tanpa awalan, basis terpilih digunakan. Mengubah basis mengonversi ekspresi valid; lebar atau tanda menafsirkan ulang bit yang sama.

- **Basis**: 2 / 8 / 10 / 16; **Ukuran kata (bit)**: 8 / 16 / 32 / 64.
- **Bertanda** / **Tanpa tanda** · `0b1010`, `0o12`, `0xA`.
- `+ - * / %`, `AND OR XOR NOT`, `<< >>`, `& | ^ ~`, `(`, `)`.
- `9007199254740993+2 = 9007199254740995`; `0xFF AND 0b1010 = 10` (base 10).
- 8-bit: `255+1 = 0`; `NOT 0 = -1` (Bertanda); `-7/2 = -3`.

## Keuangan

Bunga sederhana · Bunga majemuk · Angsuran tetap · Nilai sekarang · Nilai masa depan · Perubahan persentase.

- `P`: Modal awal; `FV`: Jumlah masa depan; `A`: Pembayaran per periode.
- `r`: Suku bunga per periode (%) / 100; `n`: Jumlah periode.
- `G=(1+r)^n`; `H=(G-1)/r` (`r=0`: `H=n`).
- Bunga sederhana: `P*(1+r*n)`; Bunga majemuk: `P*G`.
- Angsuran tetap: `P*G/H`; Nilai sekarang: `(FV+A*H)/G`; Nilai masa depan: `P*G+A*H`.
- Perubahan persentase: `(Jumlah akhir - Jumlah awal) / Jumlah awal * 100`.

## Konversi satuan

Panjang · Massa · Suhu · Luas · Volume · Waktu · Kecepatan · Penyimpanan data.

**Dari** / **Ke**: Pengaturan · `1500 m → 1.5 km` · `32 °F → 0 °C` · `K ≥ 0`.

`1 B = 8 bit`; `1 kB = 1000 B`; `1 KiB = 1024 B`; `US gal = 3.785411784 L`.

## Riwayat

100 · Profil + `context` · `Hapus riwayat`.

`$XDG_DATA_HOME/argvus/calculator/history.json` (→ `~/.local/share/argvus/calculator/history.json`).

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (→ `~/.config/argvus/calculator/settings.json`).

`{"scientific":true}` → `{"profile":"scientific"}`; `{"scientific":false}` → `{"profile":"basic"}`.

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`.

Teks aplikasi mengikuti Applications Font di Control Center → Appearance → Desktop Style → Fonts. GTK mewarisi gtk-font-name; ukuran angka dan judul dipertahankan. Peluncur membaca apps_family yang dihasilkan dan memantau perubahan.

Header menggabungkan lencana ARGVUS berwarna aksen dengan nama aplikasi. Footer s Settings yang dapat diklik berada pada pita selebar jendela dengan latar tema berbeda, mengikuti gaya Control Center.

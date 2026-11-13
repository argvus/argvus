---
title: Internasionalisasi ARGVUS
description: Katalog bahasa, pilihan dan dukungan Unicode.
---

# Internasionalisasi ARGVUS

ARGVUS kapal 18 bahasa: Arab (`ar`), Bengali (`bn-BD`), Jerman (`de-DE`)
Inggris (`en-US`), Spanyol (`es-ES`), Perancis (`fr-FR`), Hindi (`hi-IN`), Indonesia
(`id-ID`), Italia (`it-IT`), Jepang (`ja-JP`), Korea (`ko-KR`), Portugis Brasil
(`pt-BR`), Portugis Eropa (`pt-PT`), Rusia (`ru-RU`), Turki (`tr-TR`), Urdu
(`ur-PK`), Vietnam (`vi-VN`(Dan Cina yang sederhana)`zh-CN`). Inggris adalah
katalog kanonikal dan mundur. Terjemahan dihasilkan dengan AI Bantuan dan
Pembenaran teknis; ulasan independen-speaker belum disertifikasi.

## Pilih bahasa

Preferensi Open Control > Preferensi > Bahasa & Daerah > Bahasa. Halaman menemukan
katalog terpasang dan menampilkan nama asli mereka. Ini menyimpan `ARGVUS_LANGUAGE` in
`$XDG_CONFIG_HOME/locale.conf` (baku `~/.config/locale.conf`), independen
`LANG`; menghasilkan sebuah sistem lokal tidak diperlukan. Portugis Brasil (`pt-BR`) dan
Portugis Eropa (`pt-PT`) adalah pilihan yang terpisah.
Pilihan tidak diubah `/etc/locale.conf` atau regional `LANG` Mengatur.
Komentar dan pengaturan kategori regional dipertahankan. Panel Kendali dijalankan ulang
setelah menyimpan; memulai sesi baru untuk menerapkan bahasa di seluruh komponen sesi.
Pengguna tak terkonfigurasi memakai bahasa Inggris. Kontrol preferensi Locale Regional terpisah
`LANG`, format regional, dan aplikasi pihak ketiga. Pelahap log masuk berjalan
sebelum preferensi pengguna tersedia dan mempertahankan layanan sendiri-akun
bahasa, bahasa Inggris secara default.
Varian regional memakai katalog terinstal untuk bahasa mereka ketika tepat
katalog tidak ada; sebagai contoh `es_MX.UTF-8` gunakan `es-ES`. `zh-Hans` memilih
Cina sederhana. Cina tradisional (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`)
tidak dipetakan diam-diam untuk Sederhana Cina dan jatuh kembali ke Inggris.

## Katalog dan kontribusi

Sumbernya adalah `resources/locales/<locale>/<domain>.json` di repositori `argvus`.
Katalog yang terpasang ada di `/usr/share/argvus/i18n/`. Mereka mengirimkan masternya
Paket `argvus`. `locale.json` menyediakan pengidentifikasi dan nama asli/tampilan.
Tidak ada katalog PO terpisah atau platform terjemahan online yang diperlukan.

Gunakan kunci semantik dan pertahankan nama placeholder, perintah, opsi, jalur, dan
pintasan keyboard. Terjemahkan pesan lengkap alih-alih menyusun kalimat
dari fragmen yang diterjemahkan. Kunci kanonik baru memerlukan terjemahan di sepuluh kunci tersebut
bahasa. Pertahankan konsistensi terminologi dalam setiap domain dan tinjau terjemahannya
dalam konteks, terutama otentikasi, izin, dan tindakan destruktif.

```sh
argvus i18n list
argvus i18n locale
argvus i18n get common welcome name=Alice
argvus i18n validate
```

`ARGVUS_I18N_DIR` memilih akar katalog alternatif untuk pengembangan dan dipentaskan
pemeriksaan paket. Validasi memeriksa domain, kunci, nilai string, placeholder,
terjemahan kosong dan bentuk jamak bilangan bulat khusus lokal. Validasi membuktikan
integritas struktural, bukan keakuratan linguistik.

## Unicode dan font

`argvus-fonts` membutuhkan font Noto CJK selain Latin, simbol, dan emoji
keluarga. Fontconfig menyediakan fallback untuk bahasa Mandarin, Jepang, dan Hangul tanpa
menggantikan keluarga utama yang dipilih pengguna. Aplikasi dan terminal masih
perlu font fallback diaktifkan.

Pembantu teks Ratatui yang dibagikan mengukur sel terminal, mempertahankan grafem yang diperluas
cluster selama pemotongan dan membungkus teks lebar. Editor teks memindahkan dan menghapus
seluruh grafem. Area yang sangat sempit tidak dapat menampilkan mesin terbang dua sel dalam satu
sel; helper hard-wrap menggunakan karakter pengganti dalam kasus tersebut. Emoji dan
karakter dengan lebar ambigu masih dapat berbeda antara implementasi terminal/font.
Kebijakan lebar terminal harus sesuai dengan lebar Unicode normal Ratatui.

Pemilihan bahasa tidak menentukan metode input atau mengubah tata letak keyboard.Gunakan metode masukan yang kompatibel dengan Wayland saat memasukkan bahasa Mandarin, Jepang, atau Korea.

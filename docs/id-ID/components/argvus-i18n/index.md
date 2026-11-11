---
title: Internasionalisasi ARGVUS
description: Katalog bahasa, pilihan dan dukungan Unicode.
---

# Internasionalisasi ARGVUS

ARGVUS mengirimkan bahasa Inggris (`en-US`), Portugis Brasil (`pt-BR`), Spanyol (`es-ES`),
Prancis (`fr-FR`), Italia (`it-IT`), Jerman (`de-DE`), Rusia (`ru-RU`), Sederhana
Cina (`zh-CN`), Jepang (`ja-JP`) dan Korea (`ko-KR`). Bahasa Inggris adalah kanonik
katalog dan cadangan. Delapan terjemahan tambahan diproduksi dengan mesin
bantuan terjemahan dan koreksi teknis; ulasan penutur asli independen
belum tersertifikasi.

## Pilih bahasa

Buka Pusat Kontrol > Preferensi > Bahasa & Wilayah > Bahasa, atau Pengaturan >
Bahasa. Halaman ini menemukan katalog yang terinstal dan menampilkan nama aslinya.
Ini menghemat `LANG` di `$XDG_CONFIG_HOME/locale.conf` (default `~/.config/locale.conf`).
Aktifkan lokal sistem UTF-8 yang sesuai di Lokal Sistem terlebih dahulu. Contoh
termasuk `es_ES.UTF-8`, `zh_CN.UTF-8`, `ja_JP.UTF-8` dan `ko_KR.UTF-8`.

Preferensi tidak mengubah `/etc/locale.conf`. Komentar dan kategori regional
pengaturan dipertahankan. Panel Kontrol dimulai ulang setelah disimpan; keluar dan
kembali untuk menerapkan bahasa tersebut sepanjang sesi. Pengguna yang tidak dikonfigurasi menggunakan
bahasa Inggris. Variabel lingkungan yang diwarisi dan file bahasa ARGVUS yang lama tidak
ganti preferensi ini. Penyambut login berjalan sebelum preferensi pengguna
tersedia dan mempertahankan bahasa akun layanannya sendiri, bahasa Inggris secara default.

Varian regional menggunakan katalog yang terinstal untuk bahasa mereka jika tepat
katalog tidak ada; misalnya `es_MX.UTF-8` menggunakan `es-ES`. `zh-Hans` memilih
Cina Sederhana. Bahasa Cina Tradisional (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) adalah
tidak secara diam-diam dipetakan ke bahasa Mandarin Sederhana dan kembali ke bahasa Inggris.

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
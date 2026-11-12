---
title: Konfigurasi dan status
description: Status bersama, menimpa dan menghasilkan file runtime.
---

Root pengguna normal adalah `$XDG_CONFIG_HOME/argvus` (biasanya `~/.config/argvus`). Preferensi logis terkelola disimpan di `config.json`. Preferensi tema, aksen, wallpaper, spasi, dan batas bersifat kanonik; file dot tetap merupakan proyeksi kompatibilitas turunan untuk konsumen Hyprland Lua dan shell saat ini sementara konsumen tersebut bermigrasi ke `data/`.

`argvus-config` adalah satu-satunya penulis di bawah `generated/`. Setiap milik ARGVUS
file konsumen adalah proyeksi `config.json`, termasuk Hyprland, GTK, Qt6ct,
Waybar, Quickshell, Dunst, Hyprlock, taskbar Waybar dan widget-telemetri
profil, pohon konfigurasi Yazi dan Superfile, terminal dan profil Qt
pohon, blok yang dikelola font, dan stylesheet perangkat yang dapat dilepas. Komponen
selesaikan penggantian pengguna secara eksplisit, lalu hasilkan keluaran, lalu paket default. Rofi bukan proyeksi: `argvus-launcher` memiliki filenya, dan pengguna menyalin langsung di bawah `data/rofi/`.
Proyeksi tema/muat ulang normal tidak pernah mempromosikan file yang dihasilkan atau dikemas
menjadi penggantian pengguna. Dokumentasi dan alat harus mengidentifikasi sumber logis
alih-alih meminta pengguna untuk mengedit keluaran yang dihasilkan.

Oleh karena itu `argvus-appearance` memiliki orkestrasi tema dan aksen
`argvus-config` memiliki status kanonik dan proyeksi terstruktur.
`argvus theme-switch` (transaksi `argvus config theme-apply`) dan `argvus config accent` memvalidasi, mengunci, dan melakukan melalui
`argvus-config` bila tersedia, kemudian rekonsiliasi kompatibilitas konsumen itu
tidak dapat membaca `config.json` secara langsung. Adaptor itu adalah
file Qt6ct asli, petunjuk GTK `settings.ini` dan `gsettings`, terminal dan
aplikasi monitor sistem, kaki, konfigurasi superfile sendiri
direktori dan penyambut pra-otentikasi. Seorang pembantu yang membutuhkan adaptor baru
harus menambahkannya di sana, jangan pernah di dalam `generated/`.

Kepemilikan runtime sengaja dibagi berdasarkan tanggung jawab: Rust memilikinya
penyimpanan kanonik, validasi skema, migrasi, model jalur, kunci, atomik
penulisan, hash proyeksi, dan penyaji terstruktur. Shell tetap hanya sebagai
titik masuk kompatibilitas kecil atau adaptor untuk konsumen eksternal seperti
`hyprctl` dan skrip tema milik aset. Python dicadangkan untuk pengujian, audit
dan pembantu pembangunan; ini bukan bagian dari runtime desktop yang penting.

`argvus config apply` adalah batas pembangunan kembali (`project` tetap menjadi batas
ejaan kompatibilitas). Ini menghasilkan kembali keluaran yang hilang bahkan ketika kanonik
dokumen tidak berubah, merender proyektor dalam pementasan, dan menukar secara atom
Pohon `data/` hanya setelah setiap proyeksi berhasil. Siklus hidup sesi tetap dimiliki oleh
`argvus sessionctl`, yang menggunakan manifes tersebut untuk merencanakan pemuatan ulang yang ditargetkan.

Ketika penelepon telah melakukan satu generasi — `apply-theme`, sebuah aksen
perubahan, `set`/`patch`, atau perubahan blok telemetri widget — `reload` **berlaku**
generasi itu alih-alih memproyeksikannya lagi. Bunyinya rencana dari
manifes, mengembalikan konsumen yang ditargetkan dan hanya memproyeksikan ketika manifesnya ada
tidak hadir atau tidak ada pekerjaan yang tertunda. Memproyeksikan ulang pada saat itu akan membangun kembali sebuah
pohon yang identik dan membuang rencana mutasi yang baru saja dihasilkan.

Manifes juga mencatat pembuatan operasional, konfigurasi efektif
hash, mengubah domain dan menghasilkan daftar file. `data/internal/generation.json`
adalah penanda proyeksi lokal sekali pakai dan tidak pernah menjadi pilihan pengguna.
`argvus config status` membandingkan catatan ini dengan konfigurasi yang diselesaikan
dan melaporkan file yang hilang atau tambahan yang dihasilkan.

Mutasi mengikuti batas mesin yang sama: `UserConfig` dikloning menjadi
`NextUserConfig`, divalidasi, diselesaikan menjadi `EffectiveConfig`, diproyeksikan, dan
baru kemudian bertahan. `.config-transaction.json` adalah status pemulihan operasional;`argvus config recover` secara deterministik memproyeksikan ulang kanonik sebelumnya
dokumen setelah crash antara penerbitan `data/` dan `config.json`. Muat ulang adalah
diminta hanya setelah komit, dan kegagalan memuat ulang tidak mengembalikan a
konfigurasi bertahan yang koheren.

## Model resolusi

```text
config.json logical model
    ↓
ARGVUS native compatibility state and per-app override
    ↓
ARGVUS generated runtime file
    ↓
/usr/share/argvus packaged default
    ↓
upstream default
```

Kandidat yang tepat berbeda-beda berdasarkan komponen, namun prinsipnya konsisten: penggantian pengguna menang, keluaran yang dihasilkan adalah status turunan, dan file yang dikemas tetap menjadi masukan hanya-baca. Sesi ini mengekspor `ARGVUS_CONFIG_HOME` dan `ARGVUS_SYSTEM_CONFIG` sehingga pembantu dan layanan pengguna menyelesaikan akar yang sama.

Muat ulang pemulihan mengikuti `recover → ensure → migrate → apply → runtime reload`,
dan langkah proyeksi dilewati ketika manifes yang masuk sudah menjelaskan a
generasi yang berkomitmen. `ensure`
terwujud `config.json` dari default skema hanya jika profilnya ada
hilang, dan mengisi kolom yang tidak ada tanpa mengganti `false`, `0`, atau
nilai-nilai adat. Dokumen yang salah formatnya tetap merupakan kesalahan
digantikan secara diam-diam.

Mutasi persisten membandingkan konfigurasi lama dan baru yang terselesaikan
`ChangedDomains`. Setelah komit, satu permintaan muat ulang dikirim
`argvus sessionctl`; kegagalan memuat ulang dilaporkan dan dicatat tanpa bergulir
kembali konfigurasi yang dilakukan. Startup sesi melakukan pemulihan sebelumnya
perwujudan dan proyeksi bootstrap, sehingga konsumen tidak memulai dengan an
transaksi yang ditinggalkan.

| Negara | Tujuan |
| --- | --- |
| `config.json` | Preferensi logis yang dikelola kanonik; satu-satunya sumber untuk semua hal di bawah `generated/`. |
| `.active-theme`, `.accent-color`, `.accent-custom` | Masukan migrasi lama; bukan sumber runtime dan tidak dibuat ulang. |
| `.wallpaper-custom` | Masukan migrasi lama untuk `appearance.wallpaper`; bukan sumber runtime. |
| `.spaces`, `.borders` | Masukan migrasi tata letak lama; nilai kanonik disimpan di bawah `layout` di `config.json`. |
| `fonts.conf` | Proyeksi kompatibilitas pilihan font kanonik. |
| `generated/effects` | Proyeksi kompatibilitas efek kanonik; sekali pakai. |
| `$XDG_STATE_HOME/argvus/config-projection.json` | Manifes proyeksi: bagian yang diubah, konsumen yang terpengaruh, dan apakah diperlukan pemuatan ulang. Dibaca oleh `reload`; tidak pernah menjadi sumber kebenaran. |
| `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` | Preferensi blok telemetri lama. Baca sebagai cadangan migrasi saja; nilai kanonik berada di `/control_panel/widget_telemetry_blocks` dan file tidak lagi ditulis. |

Sebuah properti bersifat kanonik ketika dibaca dari `config.json` dan diproyeksikan dari
di sana. Suatu properti diperoleh ketika diproyeksikan tetapi juga dapat dikonsumsi sebagai a
masukan migrasi. Apa pun yang masih ditulis oleh komponen saat runtime — cache, yang asli
file config di bawah `$XDG_CONFIG_HOME/<app>`, file status penyambut — adalah adaptor
target dan tidak boleh dibaca kembali sebagai konfigurasi.

Proyeksi mencakup konsumen untuk Hyprland, GTK/Qt, Waybar, Quickshell, terminal, notifikasi, layar kunci, dan aplikasi asli. Hyprland membaca warna batas aktif dan tidak aktif dari tema yang dipilih; `appearance.accent_custom=true` eksplisit adalah satu-satunya kasus yang mengesampingkan batas aktif/grup dengan aksen pengguna. Peralihan tema menerapkan kembali nilai-nilai ini melalui pemuatan ulang kompositor sehingga tidak dapat mempertahankan warna tema sebelumnya. Jika konfigurasi pengguna dibuat ulang, efek default ke animasi diaktifkan. Menghapus `generated/` dan menjalankan `argvus config apply` akan membuatnya kembali byte demi byte dari `config.json`; mengedit file yang dihasilkan secara langsung tidak tahan lama.
Diagnosis runtime harus mengidentifikasi tupel eksekusi lengkap: jalur biner,
versi/revisi, root aset, root konfigurasi, root data, root yang dihasilkan, internal
akar dan manifes proyeksi. `status --verbose` mengekspos tupel ini dan
membandingkan generasi/hash yang dipublikasikan dengan konfigurasi yang diselesaikan saat ini.
Pengembangan dan pengujian paket terinstal adalah mode terpisah; tidak ada yang bisa jatuhkembali secara diam-diam ke executable yang tidak terkait atau ke rumah pengembang. `apply` adalah
juga batas rekonsiliasi untuk pengidentifikasi tema gabungan warisan: a
nilai seperti `argvus-dark-spaced` menjadi `theme=argvus-dark` dan
`variant=spaced` sebelum resolusi. Identitas aset yang dikemas tidak dicatat dalam manifes dalam rilis ini, sehingga tidak ada file diagnostik yang menjelaskannya.

---
title: Berkontribusi
description: Batasan kontribusi untuk proyek dan dokumentasi ARGVUS.
---

Lacak fitur melalui semua repositori yang digunakan sebelum mengubahnya. Pertahankan kontrak negara bagian bersama, kepemilikan paket, kepemilikan layanan, dan kedua katalog pelokalan.

`argvus-config` adalah penulis tunggal `data/generated/`. Tambahkan atau ubah file konsumen di sana, di `crates/argvus-config/src/project.rs` dan submodulnya di bawah `project/` (file Hyprland di `project/hyprland_projection.rs`), bukan di shell helper. Jika aplikasi konsumen tidak dapat membaca `config.json`, tambahkan adaptor dalam paket pemiliknya dan panggil adaptor tersebut dari orkestrator setelah penerapan kanonik. Hapus `data/generated/`, jalankan `argvus config apply` dan konfirmasikan bahwa pohon dibangun kembali byte demi byte dari `config.json`. Validasi sumber, muatan paket, dan perilaku sesi langsung secara terpisah; pembuatan sumber saja tidak membuktikan bahwa sesi Wayland yang diinstal menggunakan file baru.
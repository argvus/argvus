---
title: Mesin virtual
description: Jalankan ARGVUS dengan perangkat keras grafis virtual.
---

`argvus start` mendeteksi virtualisasi dan mengekspor pengaturan kompatibilitas untuk sesi yang terpengaruh, termasuk rendering perangkat lunak dan fallback kursor/pengubah bila diperlukan oleh lingkungan yang terdeteksi.

Jika mesin virtual masih gagal dirender, periksa lingkungan sesi dan log sebelum menambahkan penggantian. Lihat [pemecahan masalah grafis](/docs/user-guide/troubleshooting/graphics-and-virtual-machines/).

Ketika virtualisasi terdeteksi, `argvus start` mengekspor variabel kompatibilitas untuk sesi tersebut, termasuk rendering perangkat lunak, kursor perangkat lunak, dan fallback pengubah. Ini menjadikan jalur login normal sebagai hal pertama yang dicoba; jangan menambahkan penggantian permanen hanya karena desktop berjalan di VM.

Jika deteksi tidak mencukupi, gunakan penggantian Hyprland pengguna sebagai cadangan diagnostik:

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```

Tambahkan ini hanya ketika tumpukan grafis VM memerlukannya:

```lua
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```

Keluar dan masuk kembali setelah mengubah lingkungan sesi. Konfirmasikan nilai aktif dengan `echo $LIBGL_ALWAYS_SOFTWARE`; rendering perangkat lunak dapat membuat desktop dapat digunakan tetapi dapat mengurangi kinerja.
---
title: Monitor sistem
description: Buka monitor sistem ARGVUS.
---

Buka monitor yang dikonfigurasi dengan:

```sh
argvus --system-monitor
```

Sub-perintah `argvus system-monitor` menyediakan titik masuk asli dan integrasi btop. Telemetri yang ditampilkan di bilah tugas adalah fitur terpisah yang disediakan oleh `argvus widget-telemetry`. Mengklik modul CPU, Memori, dan GPU akan membuka bagian atas di dalam panel ARGVUS TUI dengan header bermerek dan footer `Esc Close`. Mode GPU menampilkan hingga enam kotak GPU dan tidak ada kotak CPU, memori, jaringan, atau proses. Tekan `q` atau `Esc` untuk menutup panel. Tab Kitty bawah menunjukkan petunjuk `Press q to close` yang dilokalkan. Panggilan mandiri `argvus system-monitor --cpu`, `--mem`, dan `--gpu` diluncurkan secara langsung dengan kotak yang dipilih.

Panel GPU bergantung pada kemampuan btop build yang diinstal dan penyedia perangkat sistem. Pemantauan NVIDIA memerlukan dukungan GPU di btop, Perpustakaan Manajemen NVIDIA dari driver NVIDIA yang diinstal, dan akses ke perangkat. Pemantauan AMD menggunakan ROCm SMI, yang bersifat opsional dan mungkin tidak diinstal. Data GPU Intel dibaca dari sysfs dan bergantung pada akses pengguna ke antarmuka tersebut; btop 1.4.7 melaporkan pemanfaatan, daya, dan jam Intel, tetapi tidak memberikan suhu Intel. NVIDIA nouveau dan driver lain dapat memaparkan suhu GPU ke taskbar melalui hwmon yang terhubung dengan perangkat, sementara dukungan penyedia btop bergantung pada driver dan perpustakaan yang tersedia. ARGVUS tidak mengubah izin perangkat atau menjalankan btop dengan hak istimewa yang lebih tinggi. Jika tidak ada GPU yang terekspos, atau btop tidak merender kotak GPU dalam waktu tiga detik karena dukungan atau penyedia tidak tersedia, panel akan menampilkan pesan tidak tersedia yang dilokalkan.

Monitor membaca tema dan aksen yang dipilih
`$XDG_CONFIG_HOME/argvus/config.json` dulu. File btop yang dihasilkan tetap ada
status cache turunan; `.active-theme` dan `.accent-color` adalah kompatibilitas
mundur. Karena btop menyimpan file temanya sendiri, monitor direkonsiliasi dengan
transaksi tema (`argvus theme-switch`) sebagai adaptor eksternal setelah perubahan kanonik
berkomitmen; itu bukan bagian dari pohon yang dihasilkan.

## Kontrak waktu proses

- Tema dan aksennya berasal dari konfigurasi kanonik (`argvus config get
/penampilan/tema` and `/penampilan/aksen`). The files `~/.config/argvus/data/.tema-aktif`
  dan `.accent-color` adalah fallback.
- Konfigurasi dan tema btop yang dihasilkan ditulis
  `$XDG_CACHE_HOME/argvus/argvus-system-monitor`. Mereka adalah negara turunan; jangan mengeditnya.
- Defaultnya diinstal di bawah `/usr/share/argvus-system-monitor/btop`.
- Pintasan modul (`--mem`, `--cpu`, `--net`, `--proc`) hanya dibaca sebagai argumen pertama.
  Melewati `-c`/`--config` milik btop atau `--themes-dir` menjalankan btop dengan jalur tersebut dan menulis tidak
  konfigurasi.
- `--panel [--cpu|--mem|--gpu]` menyematkan btop di shell ARGVUS TUI. Mode ini digunakan oleh bilah tugas;
  tanpa `--panel`, monitor terus tergantikan dengan btop.
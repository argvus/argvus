---
title: Telemetri widget
description: Blok telemetri Waybar dan peralihannya, melalui telemetri widget argvus.
---

Widget telemetri adalah informasi sistem yang ditampilkan di taskbar (header, kunci, sistem, CPU
dan GPU, memori, penyimpanan, proses, jaringan, dan dasbor pengembang). Ini sedang dalam proses
paket `argvus`: tidak ada kolektor shell.

## Perintah

```text
argvus widget-telemetry block BLOCK ...   prints the text of one block (used by the bar)
argvus widget-telemetry toggle ...        switches the blocks on and off, and their order
```

`block` adalah jalur panas bilah: dipanggil pada setiap penyegaran. `toggle` mengubah blok mana
bilah menampilkan dan memindahkan jendela gulir.

Blok tersebut adalah `header`, `keys`, `system`, `cpu_gpu`, `memory`, `storage`, `processes`,
`network` dan `dev_dashboard`. Bilah tugas juga menggunakan probe `cpu-temp` dan `gpu`. Probe GPU menerima `--present` (GPU yang didukung terlihat di DRM/sysfs), `--available` (GPU dan perintah btop tersedia untuk panel), `--available-temp` (suhu yang dapat diatribusikan dapat dibaca), dan `--temp` (suhu tertinggi yang valid, atau `N/A`).

Suhu GPU disesuaikan dengan perangkat PCI GPU. Pembacaan kepemilikan NVIDIA menggunakan ID bus `nvidia-smi`/NVML; Pembacaan NVIDIA nouveau, AMD, dan Intel menggunakan hwmon hanya jika tautan perangkat sysfsnya secara unik cocok dengan GPU tersebut. Suhu taskbar adalah nilai valid maksimum di seluruh GPU yang terdeteksi. Pembacaan yang hilang, tidak valid, atau ambigu tetap tidak tersedia. Suhu milik NVIDIA memerlukan penyedia driver; Suhu AMD dan Intel bergantung pada kernel yang mengekspos sensor hwmon yang ditautkan. Suhu Intel tidak disuplai oleh panel GPU btop 1.4.7.

## Dimulai dengan sesi

Unit pengguna `argvus-widget-telemetry.service` memulai sesi hanya ketika widget aktif
diaktifkan. Sakelar utama adalah `effects.widget_telemetry_enabled` dalam konfigurasinya
diaktifkan secara default, sehingga akun baru memulai widget saat login.

Setelah sakelar menulis `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state`, file itu
memutuskan sebagai gantinya. Hanya baris pertama `enabled` yang memulai widget. Semakin tua
File `desktop-telemetry-state` dan `sysinfo-state` masih dibaca ketika file saat ini
hilang.

Label telemetri ringkas dalam sepuluh bahasa yang didukung. Pengidentifikasi teknis seperti CPU, GPU, IP, DE, WM dan Kernel tetap mempertahankan arti teknisnya. Uptime dibaca dari `/proc/uptime` dan ditampilkan dengan satuan yang disingkat (misalnya `1d 12h 49m`); itu tidak lagi diperluas menjadi kata-kata lengkap dari lokal sistem. Nama model perangkat keras dan nilai penyedia lainnya masih dapat memengaruhi lebar widget.

---
title: Sensor suhu
description: Ikhtisar suhu langsung didukung oleh lm_sensors JSON.
---

`argvus sensors` membuka panel Ratatui dalam proses yang digunakan oleh popup suhu CPU bilah tugas. Ini menjalankan `sensors -j`, mengelompokkan pembacaan berdasarkan nama chip dan label yang dilaporkan oleh perangkat keras, dan menyegarkan setiap dua detik. Pemantauan GPU ditangani oleh `argvus system-monitor --gpu`.

Pembacaannya menggunakan Celcius. Panel menampilkan `max`, `crit`, dan status alarm saat lm_sensors menyediakannya. Pengukur ditampilkan hanya ketika batas positif tersedia. `lm_sensors` yang hilang, kegagalan perintah, JSON tidak valid, dan pembacaan kosong tetap terlihat sebagai status yang dapat dipulihkan; percobaan ulang pemungutan suara pada siklus berikutnya.

Tekan `q` atau `Esc` untuk menutup. Bilah tugas membuka panel di Kitty melalui `argvus tui-terminal`; judul tab bawah yang dilokalkan menunjukkan petunjuk penutupan. Perintah ini memiliki rincian suhu CPU; ikon suhu GPU membuka monitor btop khusus GPU. Paket taskbar sudah bergantung pada `lm_sensors`, jadi tidak diperlukan paket atau ketergantungan terpisah.
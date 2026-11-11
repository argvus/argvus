---
title: Sesi dan systemd
description: Target sesi ARGVUS dan layanan penggunanya.
---

Definisi unit dikemas oleh `argvus-session` di bawah `/usr/share/argvus/session/config/systemd/user/`. Manajer sesi memiliki impor siklus hidup dan lingkungan. Repositori komponen memiliki perintah yang dijalankan oleh unitnya.

Gunakan `argvus sessionctl` alih-alih memulai komponen secara manual saat mendiagnosis sesi terintegrasi.
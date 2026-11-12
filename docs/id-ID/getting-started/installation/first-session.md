---
title: Sesi pertama
description: Mulai ARGVUS setelah instalasi.
---

Untuk login grafis, konfigurasikan paket integrasi penyambutan dengan `argvus greeter-setup --help` dan gunakan alur manajer tampilan yang dijelaskan dalam [Sesi](/docs/user-guide/sessions/).

Untuk sesi TTY, masuk ke terminal virtual dan jalankan:

```sh
argvus --start-desktop
```

Setelah login, gunakan `argvus sessionctl status` untuk memeriksa target pengguna ARGVUS. Jika kompositor atau shell tidak dapat dijalankan, ikuti [pemecahan masalah sesi](/docs/user-guide/troubleshooting/session-startup/).

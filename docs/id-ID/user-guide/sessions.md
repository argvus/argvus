---
title: Sesi
description: Memulai dan mengelola sesi grafis ARGVUS.
---

`argvus-session` memiliki titik masuk sesi dan siklus hidup `systemd --user`. Ini memulai Hyprland melalui `argvus start`, mengimpor lingkungan sesi dan mengelola target ARGVUS.

- [Sesi grafis](/docs/argvus-session/graphical-session/)
- [Sesi TTY](/docs/argvus-session/tty-session/)
- [Penyambut](/docs/components/argvus-greeter/)

Diagnostik yang berguna:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```

Lihat pengembang [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/) untuk kepemilikan layanan.

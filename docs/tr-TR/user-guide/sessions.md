---
title: Oturumları
description: Bir ARGVUS grafik oturumunu başlatın ve yönetin.
---

`argvus-session`, oturum giriş noktalarına ve `systemd --user` yaşam döngüsüne sahiptir. Hyprland'ı `argvus start` aracılığıyla başlatır, oturum ortamını içe aktarır ve ARGVUS hedefini yönetir.

- [Grafik oturumu](/docs/argvus-session/graphical-session/)
- [TTY oturumu](/docs/argvus-session/tty-session/)
- [Karşılayan](/docs/components/argvus-greeter/)

Faydalı teşhisler:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```

Hizmet sahipliği için geliştiriciye [çalışma zamanı yaşam döngüsü](/docs/developer-guide/architecture/runtime-lifecycle/) bakın.
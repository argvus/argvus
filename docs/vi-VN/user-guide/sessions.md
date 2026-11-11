---
title: Sessions
description: Bắt đầu và quản lý phiên đồ họa ARGVUS.
---

`argvus-session` sở hữu các điểm vào phiên và vòng đời `systemd --user`. Nó khởi động Hyprland thông qua `argvus start`, nhập môi trường phiên và quản lý mục tiêu ARGVUS.

- [Phiên đồ họa](/docs/argvus-session/graphical-session/)
- [Phiên TTY](/docs/argvus-session/tty-session/)
- [Chào](/docs/components/argvus-greeter/)

Chẩn đoán hữu ích:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```

Hãy liên hệ với nhà phát triển [vòng đời thời gian chạy](/docs/developer-guide/architecture/runtime-lifecycle/) để biết quyền sở hữu dịch vụ.
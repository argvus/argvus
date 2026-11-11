---
title: Phiên đầu tiên
description: Khởi động ARGVUS sau khi cài đặt.
---

Để đăng nhập bằng đồ họa, hãy định cấu hình tích hợp chào mừng đóng gói với `argvus greeter-setup --help` và sử dụng luồng trình quản lý hiển thị được mô tả trong [Phiên](/docs/user-guide/sessions/).

Đối với phiên TTY, đăng nhập vào thiết bị đầu cuối ảo và chạy:

```sh
argvus --start-desktop
```

Sau khi đăng nhập, sử dụng `argvus sessionctl status` để kiểm tra mục tiêu người dùng ARGVUS. Nếu bộ tổng hợp hoặc shell không khởi động, hãy làm theo [khắc phục sự cố phiên](/docs/user-guide/troubleshooting/session-startup/).
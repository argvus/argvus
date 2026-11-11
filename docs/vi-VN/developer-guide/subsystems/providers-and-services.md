---
title: Nhà cung cấp và dịch vụ
description: Provider packages and their system interfaces.
---

Các dự án của nhà cung cấp hiển thị các giao diện tập trung cho người tiêu dùng UI:

- `argvus network` và `argvus bluetooth` tích hợp Trình quản lý mạng và trình trợ giúp Bluetooth;
- `argvus-display` tích hợp trạng thái giám sát với Hyprland;
- `argvus-power` tích hợp các hoạt động không tải, khóa và cấp nguồn;
- `argvus-firewall` quản lý `argvus-firewall.service`;
- trình trợ giúp `argvus removable-devices` trong `argvus` tiêu thụ UDisks2;
- trình trợ giúp `argvus control-center` trong `argvus` sử dụng API của nhà cung cấp và giao diện D-Bus của hệ thống.

Trung tâm điều khiển là khách hàng của các nhà cung cấp này chứ không phải là khách hàng thay thế của họ.
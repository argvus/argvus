---
title: Bề mặt vỏ
description: Thanh tác vụ, bảng điều khiển và quyền sở hữu phép đo từ xa.
---

`argvus-waybar` cung cấp `/usr/bin/waybar` đã được vá. `argvus-taskbar` sở hữu các hành động và UX trên thanh tác vụ; thanh tác vụ JSONC và CSS mà nó gửi là các hình chiếu của `config.json` và `argvus-config` ghi lớp hoạt động trong `data/generated/waybar/` cùng với một bản sao được phản chiếu trong `data/waybar/`. `argvus-control-panel` sở hữu thẻ Quickshell. `argvus-widget-telemetry` sở hữu thanh đo từ xa tùy chọn và ngữ nghĩa khối của nó; JSONC và CSS của nó được ghi vào `data/waybar/` từ các giá trị mặc định được đóng gói khi thay đổi diện mạo, sau đó `argvus-config` chỉ viết lại các khối phông chữ và `ARGVUS_TELEMETRY_*` được phân tách. `argvus-session` sở hữu dịch vụ người dùng của họ.

Không có gói `argvus-shell` hoặc `argvus-shell.service` hiện tại trong mô hình thời gian chạy này.
---
title: Bề mặt vỏ
description: Thanh tác vụ, bảng điều khiển và quyền sở hữu phép đo từ xa.
---

`argvus-shell` cung cấp `Quickshell Taskbar` đã được vá. `argvus-shell` sở hữu các hành động và UX trên thanh tác vụ; thanh tác vụ JSONC và CSS mà nó gửi là các hình chiếu của `config.json` và `argvus-config` ghi lớp hoạt động trong `data/generated/taskbar/` cùng với một bản sao được phản chiếu trong `data/taskbar/`. `argvus-shell` sở hữu thẻ Quickshell. `argvus-shell` provides the optional Telemetry surface, while the `argvus widget-telemetry` command in `argvus` retains block-control semantics; JSONC và CSS của nó được ghi vào `data/taskbar/` từ các giá trị mặc định được đóng gói khi thay đổi diện mạo, sau đó `argvus-config` chỉ viết lại các khối phông chữ và `ARGVUS_TELEMETRY_*` được phân tách. `argvus-session` sở hữu dịch vụ người dùng của họ.

Không có gói `argvus-shell` hoặc `argvus-shell.service` hiện tại trong mô hình thời gian chạy này.
---
title: Vai trò kho lưu trữ
description: Trách nhiệm của kho nguồn ARGVUS sau khi hợp nhất không gian làm việc.
---

Các kho lưu trữ được nhóm theo trách nhiệm thay vì theo trang điều hướng:

- không gian làm việc: `argvus` (bộ điều phối, lệnh đang xử lý, trình trợ giúp GTK 4, thùng dùng chung, lời chào trong `crates/argvus-greeter` và dữ liệu của các lệnh);
- phiên và tích hợp: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash` và lời chào (`argvus-greeter`, được xây dựng từ `argvus/crates/argvus-greeter`);
- Giao diện người dùng bên ngoài không gian làm việc: Bảng điều khiển, thanh tác vụ (`argvus-taskbar`), trình khởi chạy và Waybar;
- nhà cung cấp: hiển thị, nguồn, tường lửa, thông báo;
- nội dung và bao bì: hình thức, hình nền, phông chữ, biểu tượng, hồ sơ.

## Hợp nhất vào không gian làm việc

Các kho lưu trữ này đã được sáp nhập vào `de/argvus`. Tài liệu của họ hiện nằm trong `docs/` của không gian làm việc và kho lưu trữ của họ chỉ được lưu giữ cho lịch sử:

| Kho lưu trữ cũ | Bây giờ | Tài liệu |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [dòng lệnh](/docs/reference/command-line/), [tệp cấu hình](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [Thành phần quốc tế hóa](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | dòng lệnh |
| `argvus-terminal` | `argvus terminal` | [Thành phần đầu cuối](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [Thành phần giám sát hệ thống](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [Thành phần mạng](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | dòng lệnh |
| `argvus-control-center` | `argvus control-center` (ứng dụng đầu cuối) | [Thành phần Trung tâm điều khiển](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (người trợ giúp GTK 4) | [Thành phần lịch trên thanh tác vụ](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (người trợ giúp GTK 4) | [Thành phần thiết bị có thể tháo rời](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (Người trợ giúp `splash`) | dòng lệnh |
| `argvus-tui-terminal` | `argvus tui-terminal` (thùng trong không gian làm việc) | dòng lệnh |
| `argvus-tui`, `argvus-theme` | Thùng chung `crates/argvus-tui` và `crates/argvus-theme` | ghi chú của nhà phát triển |
| `argvus-greeter` | Nguồn trong `crates/argvus-greeter`, được vận chuyển trong gói `argvus` | [thành phần lời chào](/docs/components/argvus-greeter/) |
| `argvus-themes` | Đã nghỉ hưu. Mỗi chủ đề chính thức là gói `argvus-theme-*` riêng. | [cài đặt chủ đề](/docs/user-guide/appearance/) |

Ghi chú dành cho nhà phát triển của các thành phần hợp nhất có trong `docs/internal/<repository>/`.

Ngôn ngữ không phải là ranh giới quyền sở hữu: mỗi thành phần giữ các trợ giúp thời gian chạy cùng với gói của nó. Một quy tắc cứng áp dụng cho các gói: chỉ `argvus-config` ghi bên dưới `data/generated/`. Một thành phần cần tiếp cận một ứng dụng không thể đọc `config.json` sẽ thêm một bộ điều hợp vào trình trợ giúp riêng của nó và cho phép người điều phối gọi nó sau cam kết chuẩn; nó không bao giờ ghi một tập tin được tạo ra. CLI công khai thuộc `/usr/bin`; các trình trợ giúp nội bộ thuộc về `/usr/lib/argvus/<component>` hoặc trong thư mục lệnh khi chúng cần thiết trong thời gian chạy; dữ liệu độc lập với kiến ​​trúc vẫn thuộc `/usr/share/argvus/<component>`.

Xem [ma trận gói](/docs/reference/package-matrix/) để biết ranh giới gói đã cài đặt.

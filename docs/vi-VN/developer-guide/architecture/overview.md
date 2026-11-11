---
title: Architecture overview
description: Cách ARGVUS hình thành môi trường máy tính để bàn tích hợp.
---

ARGVUS có năm lớp thời gian chạy:

1. điểm vào và điều phối trong `argvus`;
2. vòng đời phiên trong `argvus-session`;
3. bề mặt bộ tổng hợp và vỏ trong `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar` và `argvus-control-panel`;
4. các ứng dụng người dùng như Trung tâm điều khiển, lịch và màn hình hệ thống;
5. nhà cung cấp và tài sản để hiển thị, mạng, nguồn, giao diện, tài khoản và thông báo.

Trạng thái chia sẻ kết nối các lớp này. `argvus-config` giữ tài liệu chuẩn và là tác giả duy nhất của `data/generated/`, do đó, thay đổi chủ đề hoặc hiệu ứng sẽ tái tạo các tệp được nhiều gói sử dụng trong một giao dịch; dịch vụ phiên cung cấp quyền sở hữu vòng đời mà không có quyền sở hữu việc triển khai mọi thành phần.

##Ranh giới trách nhiệm

| Lớp | Trách nhiệm chính | Ví dụ |
| --- | --- | --- |
| Điểm vào và công văn | Khởi động ứng dụng máy tính để bàn và giải quyết cấu hình ARGVUS. | `argvus`, `argvus-app-profiles` |
| Vòng đời phiên | Chuẩn bị môi trường, khởi chạy Hyprland, đồng bộ hóa môi trường dịch vụ người dùng và tắt phiên riêng. | `argvus-session`, `argvus sessionctl` |
| Bộ tổng hợp và vỏ | Xác định hành vi của cửa sổ và bề mặt màn hình hiển thị. | `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar`, `argvus-control-panel` |
| Cấu hình | Sở hữu `config.json`, mô hình đường dẫn và mọi tệp người dùng được tạo. | `argvus-config` |
| Ứng dụng | Cung cấp các cài đặt và quy trình làm việc tập trung trên máy tính để bàn. | Trung tâm điều khiển, lịch, trình khởi chạy, màn hình hệ thống và các thiết bị di động |
| Nhà cung cấp và tài sản | Thực hiện các hoạt động miền và tài nguyên hình ảnh được chia sẻ. | hiển thị, mạng, nguồn, thông báo, giao diện, phông chữ và hình nền |

Gói `argvus` là mục tiêu cài đặt và điều phối viên, nhưng nó không phải là thời gian chạy nguyên khối. Các gói tập trung giữ quyền sở hữu các tệp nhị phân, cấu hình và dịch vụ của chúng. Trình quản lý phiên khởi động chúng tại thời điểm thích hợp trong phiên đồ họa mà không tiếp thu việc triển khai chúng.

Sự khác biệt này rất quan trọng khi gỡ lỗi: lỗi phiên trước tiên thuộc về `argvus-session`; lỗi thanh tác vụ thuộc về `argvus-taskbar`/`argvus-waybar`; và lỗi chủ đề hoặc hình nền được tạo thuộc về đường dẫn cấu hình phiên và giao diện.
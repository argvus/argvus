---
title: Đóng góp
description: Ranh giới đóng góp cho các dự án và tài liệu ARGVUS.
---

Theo dõi một tính năng thông qua tất cả các kho lưu trữ tiêu thụ trước khi thay đổi nó. Duy trì các hợp đồng nhà nước được chia sẻ, quyền sở hữu gói, quyền sở hữu dịch vụ và cả danh mục bản địa hóa.

`argvus-config` là tác giả duy nhất của `data/generated/`. Thêm hoặc thay đổi tệp người dùng ở đó, trong `crates/argvus-config/src/project.rs` và các mô-đun con của nó trong `project/` (tệp Hyprland trong `project/hyprland_projection.rs`), không phải trong trình trợ giúp shell. Nếu ứng dụng tiêu dùng không thể đọc `config.json`, hãy thêm bộ điều hợp trong gói sở hữu và gọi nó từ bộ điều phối sau cam kết chuẩn. Xóa `data/generated/`, chạy `argvus config apply` và xác nhận cây được xây dựng lại từng byte một từ `config.json`. Xác thực nguồn, tải trọng gói và hoạt động của phiên trực tiếp một cách riêng biệt; chỉ xây dựng nguồn không chứng minh được rằng phiên Wayland đã cài đặt sử dụng các tệp mới.

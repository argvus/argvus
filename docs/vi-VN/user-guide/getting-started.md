---
title: Cấu hình đầu tiên
description: Đường dẫn khởi động đầu tiên thực tế để định cấu hình ARGVUS.
---

Sau phiên đồ họa đầu tiên, bạn không cần định cấu hình mọi phần của ARGVUS. Bắt đầu với các cài đặt ảnh hưởng đến cảm giác trên màn hình, sau đó định cấu hình phần cứng và dịch vụ nếu cần.

## Một đơn hàng hữu ích

1. Đọc [tổng quan về máy tính để bàn](/docs/user-guide/desktop/) để xác định các cửa sổ, không gian làm việc, thanh tác vụ, Bảng điều khiển và trình khởi chạy.
2. Mở [Trung tâm điều khiển](/docs/components/argvus-control-center/) và chọn chủ đề, chế độ, điểm nhấn và hình nền trong **Giao diện**.
3. Điều chỉnh [cửa sổ và bố cục](/docs/argvus-hyprland/windows-and-layout/) nếu các khoảng trống, đường viền hoặc khoảng cách thanh tác vụ mặc định không phù hợp với màn hình của bạn.
4. Định cấu hình [thanh tác vụ](/docs/argvus-taskbar/taskbar/) và quyết định thẻ Bảng Điều khiển nào bạn muốn hiển thị.
5. Xem lại [phím tắt](/docs/argvus-hyprland/keyboard-shortcuts/) và chỉ thay đổi các liên kết mà bạn thực sự sử dụng.
6. Định cấu hình [chuột và bàn di chuột](/docs/argvus-hyprland/input/), màn hình, kết nối mạng và nguồn từ các trang cài đặt liên quan.

Bốn bước đầu tiên là cá nhân hóa tùy chọn. Các bước còn lại thường chỉ cần thiết khi phần cứng, ngôn ngữ hoặc quy trình làm việc của bạn yêu cầu chúng.

## Hai nơi để thay đổi cài đặt

Sử dụng **Trung tâm điều khiển** cho các cài đặt mô tả cách định cấu hình ARGVUS: giao diện, bố cục, phông chữ, phương thức nhập, phím tắt, ngôn ngữ, khu vực và các ứng dụng mặc định. Sử dụng **Bảng điều khiển** để biết trạng thái và các tác vụ thường xuyên trong phiên như âm lượng, độ sáng, trạng thái mạng, thông báo, điều khiển nguồn và phiên.

Xem [Trung tâm điều khiển và Bảng điều khiển](/docs/components/argvus-control-center/) để biết sự khác biệt đầy đủ và [Nơi định cấu hình mọi thứ](/docs/user-guide/where-to-configure/) để biết chỉ mục nhanh.

## Thay đổi, kiên trì và phục hồi

Hầu hết các thay đổi của Trung tâm điều khiển được áp dụng trong khi phiên đang chạy và được lưu dưới dạng trạng thái người dùng ARGVUS. Các thay đổi về giao diện được cam kết đối với cấu hình chuẩn và `argvus-config` chiếu các tệp được Hyprland sử dụng và shell từ nó. Chuyển đổi Bảng điều khiển có thể là một hành động trực tiếp hoặc thay đổi trạng thái phiên thay vì cấu hình cố định.

Khi một trang cung cấp **Đặt lại mặc định**, hãy sử dụng tác vụ trên trang đó thay vì xóa các tệp đã tạo. Giao diện có các hành động đặt lại riêng biệt cho các điểm nhấn, hình nền và giá trị bố cục; trang liên kết phím có hành động khôi phục riêng. Không có nút đặt lại chung nào cho mọi cài đặt trên màn hình.

## Các bước tiếp theo

- [Giao diện](/docs/user-guide/appearance/) — chủ đề, hình nền, hiệu ứng và bố cục.
- [Bố cục màn hình](/docs/argvus-hyprland/windows-and-layout/) — hiểu các khoảng trống, đường viền và không gian bảng điều khiển.
- [Thiết bị đầu vào](/docs/argvus-hyprland/input/) — điều khiển chuột và bàn di chuột.
- [Khắc phục sự cố chủ đề và hình nền](/docs/user-guide/troubleshooting/theme-and-wallpaper/) — khôi phục khi thay đổi hình ảnh không như bạn mong đợi.
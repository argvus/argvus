---
title: Tùy chỉnh màu đánh dấu
description: Thay đổi màu đánh dấu ARGVUS độc lập với chủ đề đã chọn và quay lại mặc định của chủ đề.
---

**Màu đánh dấu** là điểm nhấn được sử dụng cho đường viền tiêu điểm, vùng chọn và trạng thái hoạt động trên màn hình nền. Mỗi chủ đề đều có màu nổi bật mặc định nhưng bạn có thể thay thế bằng bất kỳ màu nào bạn muốn mà không cần thay đổi chủ đề.

## Thay đổi màu đánh dấu

1. Mở **Trung tâm điều khiển → Giao diện → Màu đánh dấu**.
2. Chọn **Chỉnh sửa màu đánh dấu**. Giá trị hiện tại được hiển thị bên cạnh tùy chọn.
3. Nhập màu RGB gồm sáu chữ số, có hoặc không có `#` ở đầu. Ví dụ: `3590BD` và `#3590BD` đều được chấp nhận.
4. Nhấn `Enter` để áp dụng.

Bạn cũng có thể dán màu vào trường này. ARGVUS chỉ chấp nhận văn bản được dán khi đó là giá trị RGB gồm sáu chữ số hợp lệ. Các dạng ngắn như `#123` và các giá trị có ký tự không phải thập lục phân bị từ chối kèm theo thông báo lỗi và màu không thay đổi.

Màu mới được áp dụng ngay cho màn hình nền và được lưu trong cấu hình ARGVUS của bạn.

## Trở về mặc định của chủ đề

Để quay lại màu được xác định theo chủ đề đã chọn, hãy mở **Trung tâm điều khiển → Giao diện → Màu đánh dấu** và chọn **Đặt lại về mặc định của chủ đề**.

## Màu nổi bật tương tác với chủ đề như thế nào

- Màu nổi bật bạn chọn là màu ghi đè độc lập. Nó vẫn tồn tại khi bạn chuyển sang chủ đề khác nên màu bạn chọn vẫn giữ nguyên.
- Chủ đề mà bạn áp dụng sau **Đặt lại về mặc định của chủ đề** sẽ lấy lại màu đánh dấu và sử dụng màu mặc định của chính nó.
- Xuất chủ đề đang hoạt động sẽ lưu trữ màu nổi bật đang hoạt động khi bạn xuất làm điểm nhấn cho bảng màu của chủ đề mới. Xem [Nhập và xuất chủ đề](/docs/components/argvus-themes/import-and-export/).

## Nơi lưu trữ giá trị

Màu nổi bật là một phần trong cấu hình ARGVUS của bạn, trong `$XDG_CONFIG_HOME/argvus/config.json` (thường là `~/.config/argvus/config.json`). Không chỉnh sửa các tệp được tạo trong `$XDG_CONFIG_HOME/argvus/data/generated`: ARGVUS ghi lại chúng từ cấu hình.
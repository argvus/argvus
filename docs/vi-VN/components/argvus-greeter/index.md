---
title: Chào hỏi
description: Lời chào đăng nhập bắt đầu, được vận chuyển trong gói argvus.
---

`argvus-greeter` là lời chào đăng nhập cho lời chào. Đây là thiết bị đầu cuối đầu tiên có bàn phím
giao diện người dùng phát hiện các phiên Wayland đã cài đặt, hiển thị các tài khoản với
hình đại diện và áp dụng chủ đề ARGVUS đang hoạt động cho tài khoản đã chọn.

Nó là một phần của gói `argvus`, gói duy nhất của máy tính để bàn. Nguồn của nó sống
trong `crates/argvus-greeter` của kho lưu trữ `argvus`, với tư cách là thành viên của Cargo gốc
không gian làm việc và nó được xây dựng cùng với phần còn lại của `argvus`.

Lời chào không tự xác thực người dùng. chào xử lý xác thực thông qua
PAM. Trình khởi chạy mở lời chào bằng `argvus tui-terminal`, vì vậy lời chào không
cần một chương trình đầu cuối riêng biệt.

Các tệp cấu hình lời chào được cài đặt trong `/etc/argvus`. sử dụng
`argvus greeter-setup --help` để xem các tùy chọn thiết lập mà gói cung cấp.
Để biết quy trình đăng nhập, hãy xem [Phiên](/docs/user-guide/sessions/).

Ổ cắm chuyển giao đăng nhập cấp phép cho mỗi khách hàng bằng thông tin xác thực ngang hàng của kernel. Chỉ root, chủ sở hữu trợ giúp và tài khoản được xác thực bằng lời chào mới có thể gửi lệnh điều khiển. Các tài khoản cục bộ khác bị từ chối mặc dù ổ cắm cho phép kết nối giữa những người dùng. Khung DRM đã chuẩn bị vẫn được gắn với bộ mô tả ban đầu của nó; người trợ giúp đợi bộ mô tả đó trước khi tiếp quản màn hình.

Khi trường xác thực được tập trung, các ký tự có thể in được (bao gồm `j` và `k`) sẽ nhập phản hồi. Tiêu điểm di chuyển Tab và Shift-Tab; Backspace xóa ký tự cuối cùng. Điều này áp dụng cho các lời nhắc bí mật và không bí mật.

Danh sách tài khoản cuộn để hiển thị tài khoản đã chọn khi bạn điều hướng hoặc thay đổi kích thước thiết bị đầu cuối.

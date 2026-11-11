---
title: ARGVUS Disk Usage Analyzer
description: Duyệt mức sử dụng đĩa tương tác với ncdu trong cửa sổ Kitty theo chủ đề.
---

Mở **Trình phân tích sử dụng đĩa ARGVUS** từ trình khởi chạy ứng dụng hoặc chạy:

```sh
argvus du
```

Máy phân tích sẽ mở một cửa sổ Kitty chuyên dụng, ở giữa và nhúng giao diện ncdu thực vào bên trong khung ARGVUS của nó. Nó bắt đầu trong `$HOME`. Chỉnh sửa đường dẫn thư mục hoặc chọn **Duyệt** để mở trình quản lý tệp đã định cấu hình. Với Superfile, điều hướng đến một thư mục và nhấn **Space** để chọn nó; `q` hoặc `Esc` hủy. Các trình quản lý tệp được định cấu hình khác mở bình thường nhưng có thể không trả về thư mục đã chọn, vì vậy hãy nhập hoặc dán đường dẫn vào trình phân tích.

Sử dụng Tab hoặc các phím mũi tên để di chuyển giữa thư mục, tùy chọn và hành động. Biểu đồ, tỷ lệ phần trăm, bảng màu tối và thứ tự thư mục đầu tiên được bật theo mặc định. Các tùy chọn quét và hiển thị còn lại có thể được chuyển đổi trước khi chọn **Phân tích**. Trong Kết quả, hãy sử dụng các điều khiển bàn phím của ncdu; `q` quay lại biểu mẫu và `Esc` dừng quá trình quét. Nhấn `d` để xóa tệp hoặc thư mục đã chọn. ncdu yêu cầu xác nhận; việc xóa là vĩnh viễn và sử dụng quyền của người dùng hiện tại. Không có đặc quyền nâng cao được yêu cầu.

Giao diện bên ngoài tuân theo chủ đề và ngôn ngữ ARGVUS đang hoạt động. Cần có Kitty và ncdu; gói `argvus-devtools` cung cấp ncdu. Nếu thiếu Kitty hoặc ncdu, ứng dụng sẽ báo cáo phần phụ thuộc bị thiếu.
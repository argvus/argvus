---
title: Projects
description: Bộ chọn dự án và cấu hình dự án thông qua các dự án argvus.
---

`argvus projects` liệt kê và mở các dự án của bạn. Đây là lệnh đang được xử lý của `argvus`
gói và `SUPER + O` sẽ mở bộ chọn.

## Lệnh

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position, full path or unique name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```

##Dự án là gì

Dự án là một thư mục được liệt kê trong cấu hình (`/projects/paths`) hoặc thư mục con của
một root được liệt kê trong `/projects/roots`. Có nhiều nhất chín dự án được liệt kê: số lượng dự án
Phím tắt `SUPER + CTRL + N`. Mở một dự án tập trung vào không gian làm việc Hyprland của nó. Lần đầu tiên một
dự án mở ra, ARGVUS khởi động một thiết bị đầu cuối và trình soạn thảo được đặt trong `/projects/editor` bên trong nó.

Cấu hình được đọc và ghi thông qua `argvus config`, do đó Trung tâm điều khiển và
shell xem danh sách tương tự.

## Không gian làm việc của dự án và thanh tác vụ

Các dự án sử dụng không gian làm việc Hyprland có tên thông thường, không phải không gian làm việc đặc biệt. `SUPER + O` hiển thị số dự án, thư mục và phím tắt `SUPER + CTRL + N` trực tiếp. Cả hai điểm vào đều tập trung vào cùng một không gian làm việc. Chỉ các không gian làm việc mở mới xuất hiện trên thanh tác vụ, có nhãn `Project: name`, trạng thái hoạt động và chú giải công cụ chứa đường dẫn và phím tắt đầy đủ. Nhấp vào nút không gian làm việc để quay lại nó. Thanh tác vụ đi theo màn hình của không gian làm việc.

Đường dẫn thư mục chuẩn xác định danh tính không gian làm việc ổn định; thay đổi thứ tự danh sách sẽ thay đổi phím tắt chứ không phải không gian làm việc. Các dự án có cùng tên thư mục vẫn tách biệt và các liên kết tượng trưng đến cùng một thư mục sẽ được loại bỏ trùng lặp. Việc khởi chạy thiết bị đầu cuối và trình soạn thảo nhắm mục tiêu rõ ràng đến không gian làm việc của dự án. Những người chỉnh sửa sử dụng lại quy trình/cửa sổ hiện có có thể giữ lại hành vi không gian làm việc của chính ứng dụng đó.

Không gian làm việc được mở trước khi cập nhật sẽ giữ nguyên các cửa sổ và tên gốc hiện có của chúng. Mở một dự án sau khi cập nhật sẽ sử dụng danh tính bắt nguồn từ đường dẫn mới của nó; đóng các cửa sổ dự án cũ khi thuận tiện. Không có liên kết hoặc di chuyển tự động nào được thực hiện vì tên cũ không xác định được thư mục một cách đáng tin cậy. Với tên thư mục trùng lặp, hãy sử dụng bộ chọn, một số hoặc đường dẫn chuẩn đầy đủ thay vì một tên mơ hồ.

Việc mở một dự án sẽ tái tạo nguyên tử `$XDG_RUNTIME_DIR/argvus/project-workspaces.json` từ danh sách dự án đã định cấu hình và các bản dịch. Đây là siêu dữ liệu phiên dành cho nhãn và chú giải công cụ của không gian làm việc, không phải cấu hình; không chỉnh sửa nó. Mở lại dự án sau khi thay đổi danh sách hoặc ngôn ngữ để làm mới nhãn của dự án. Cập nhật cả `argvus` và `argvus-waybar`, cùng với cấu hình `argvus-taskbar` cho sự tích hợp này.
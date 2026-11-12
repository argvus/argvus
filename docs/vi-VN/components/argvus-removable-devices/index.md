---
title: Thiết bị có thể tháo rời
description: Gắn kết và quản lý bộ lưu trữ di động từ ARGVUS.
---

`argvus-removable-devices` theo dõi UDisks2 và cung cấp mô-đun lưu trữ Thanh tác vụ ARGVUS cũng như menu thiết bị của nó. Menu này cũng có sẵn dưới dạng giao diện kiểu Rofi riêng biệt, do đó, các hành động tương tự của thiết bị có thể được sử dụng từ thanh tác vụ hoặc từ quy trình làm việc của trình khởi chạy.

Các hành động khả dụng là `open`, `mount`, `unmount`, `eject`, `poweroff`, `lock`, `unlock` và `copy`. Các lệnh `list`, `watch` và `menu` hiển thị trạng thái tương ứng, luồng thanh tác vụ và giao diện menu để tích hợp.

## Trình đơn thanh tác vụ

Với `"mode": "gui"` (mặc định của hệ thống), một cú nhấp chuột vào biểu tượng lưu trữ trên thanh tác vụ sẽ mở ra menu GTK được gắn vào Thanh tác vụ ARGVUS, giống như cửa sổ bật lên lịch: bên dưới thanh khi thanh tác vụ ở trên cùng của màn hình và bên trên nó khi thanh tác vụ ở dưới cùng. Mỗi hàng thiết bị có một menu con hành động chỉ có những hành động mà trạng thái của nó cho phép.
- **Con trỏ:** di chuột hoặc nhấp vào hàng thiết bị sẽ mở menu con của thiết bị đó. Menu con vẫn mở trong khi con trỏ đi qua các cạnh menu hoặc rời khỏi menu và nó chỉ đi theo một hàng khác sau khi con trỏ dừng ở đó một thời gian ngắn, do đó đường chéo đến menu con không đóng nó. Một cú nhấp chuột bên ngoài cả hai menu sẽ đóng cửa sổ bật lên.
- **Bàn phím:** `Up`/`Down` chọn một hàng, `Right`, `Enter` hoặc `Space` mở menu con (hoặc chạy **Làm mới**/**Quit**), `Enter` hoặc `Space` chạy hành động đã chọn, `Left` hoặc `Escape` quay lại danh sách thiết bị và `Escape` sẽ đóng cửa sổ bật lên.

**Sao chép đường dẫn** đặt điểm gắn kết vào bảng nhớ tạm thông qua `copy_command` (`wl-copy` mặc định, từ `wl-clipboard`), vì vậy đường dẫn vẫn còn đó sau khi đóng menu; một thông báo xác nhận điều đó khi `show_notification` được bật. **Khóa** được cung cấp cho mọi tập đĩa được mã hóa đã mở khóa và ngắt kết nối nó trước khi cần.

Hành động của trình quản lý tệp mặc định tuân theo trình quản lý tệp được chọn trong Trung tâm điều khiển ARGVUS. Điều này giữ cho các hoạt động của thiết bị di động nhất quán với các giá trị mặc định của ứng dụng của người dùng.

Cấu hình hệ thống là `/etc/argvus/removable-devices/config.json`. Phép chiếu cho mỗi người dùng do ARGVUS quản lý là `~/.config/argvus/data/removable-devices/config.json`, với `theme.css` phù hợp bên cạnh. Biểu định kiểu đó cũng được chiếu tới `~/.config/argvus/data/generated/removable-devices/theme.css` từ chủ đề, điểm nhấn và chế độ chuẩn, với dự phòng dòng/biến thể cho các chủ đề không gửi nội dung thiết bị di động chuyên dụng, do đó, việc thay đổi chủ đề sẽ giữ cho mô-đun nhất quán mà không cần bất kỳ bước thủ công nào. Đường dẫn thành phần kế thừa là đầu vào di chuyển; cấu hình người dùng sau này sẽ ghi đè mặc định của hệ thống.

Các cài đặt hữu ích bao gồm liệu mô-đun có bị ẩn khi không có thiết bị hay không (`hide_when_empty`), tên và dung lượng có được hiển thị hay không, thứ tự sắp xếp thiết bị, hành vi thông báo, phần phụ trợ menu và biểu tượng trạng thái. Biểu định kiểu của nó được hiển thị từ chủ đề ARGVUS đang hoạt động (`removable-devices/theme.css` của bản dựng chủ đề) và được sao chép vào `theme.css` của bảng điều khiển trên mọi thay đổi về chủ đề hoặc điểm nhấn.

Giao diện sử dụng ngôn ngữ người dùng từ `LANG` trong `$XDG_CONFIG_HOME/locale.conf`, với tiếng Anh làm mặc định cho người dùng chưa được định cấu hình. Các biến cục bộ được kế thừa không chọn ngôn ngữ giao diện.

## Danh mục ngôn ngữ được chia sẻ

Giao diện sử dụng danh mục `argvus-i18n` được chia sẻ cho tất cả mười ngôn ngữ. Văn bản được lưu trữ trong `resources/locales/<locale>/removable-devices.json` trong không gian làm việc chính và được cài đặt bởi `argvus`. Tiếng Anh là dự phòng; ngôn ngữ mới không còn yêu cầu từ điển song ngữ địa phương nữa.
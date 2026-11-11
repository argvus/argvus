---
title: Terminal
description: Mở thiết bị đầu cuối ARGVUS và hồ sơ ứng dụng thiết bị đầu cuối.
---

Mở thiết bị đầu cuối được cấu hình bằng:

```sh
argvus --terminal
```

`argvus terminal` (một tiểu ban của `argvus`) cung cấp khả năng tích hợp thiết bị đầu cuối và cấu hình Kitty. `argvus-app-profiles` cung cấp hồ sơ cho các ứng dụng đầu cuối như Superfile và Yazi; nó không cung cấp bộ điều phối `argvus`.

`argvus-config` chiếu chủ đề vào `data/generated/terminal/`, `argvus --yazi` và `argvus --spf` đọc cấu hình của chúng từ các cây `data/generated/yazi/` và `data/generated/superfile/` được chiếu thay vì từ các bản sao được hiện thực hóa vào thư mục chính của bạn. Ghi đè gốc hoàn chỉnh trong thư mục cấu hình riêng của ứng dụng vẫn thắng, vì vậy bạn có thể thay thế cây được chiếu bằng cấu hình của riêng mình.

Các thay đổi chủ đề sẽ tái tạo các tệp dẫn xuất của Kitty trong `$XDG_CACHE_HOME/argvus/argvus-terminal`. Trình tạo tuần tự hóa các bản cập nhật và thay thế từng tệp một cách nguyên tử. Nếu thiết bị đầu cuối vẫn không mở, hãy kiểm tra chủ đề đang hoạt động và tạo lại bộ đệm bằng:

```sh
argvus config get /appearance/theme --raw
argvus terminal --apply
argvus terminal --print-config
```

Thiết bị đầu cuối đọc chủ đề và giọng được quản lý từ
`$XDG_CONFIG_HOME/argvus/config.json` đầu tiên. `.active-theme` kế thừa và
Các tệp `.accent-color` vẫn là bản dự phòng cho các bản cài đặt cũ hơn.

## Minh bạch và mờ

Trong **Trung tâm điều khiển → Giao diện → Thiết bị đầu cuối**, định cấu hình **Độ trong suốt**
và **Blur** độc lập từ `0%` đến `100%`. Những thay đổi được cam kết với
Nút **Áp dụng** và tạo lại hồ sơ Kitty bên dưới
`$XDG_CACHE_HOME/argvus/argvus-terminal`; cửa sổ Kitty hiện tại nhận được
cấu hình được cập nhật thông qua tải lại.

Độ trong suốt của `0%` giữ cho nền mờ và `100%` làm cho nền mờ hoàn toàn
minh bạch. Blur sử dụng bán kính làm mờ nền được hỗ trợ của Kitty và chỉ có một
hiệu ứng có thể nhìn thấy khi cửa sổ trong suốt.

## Thông báo lệnh chạy dài

Các lệnh chạy trong 30 giây trở lên sẽ gửi thông báo khi chúng kết thúc,
miễn là cửa sổ terminal không tập trung. Thông báo sử dụng Kitty's
`notify_on_cmd_finish` với tính năng tích hợp shell đã được bật, vì vậy
`~/.zshrc` và `~/.bashrc` không cần thay đổi. Ngưỡng 30 giây được cố định
trong cấu hình Kitty.

## Chế độ ứng dụng TUI

Cửa sổ bật lên TUI trên thanh tác vụ và Trung tâm điều khiển ARGVUS sử dụng `argvus tui-terminal` với
hồ sơ ứng dụng Kitty chuyên dụng. Nó kế thừa chủ đề và phông chữ ARGVUS đang hoạt động,
gán một lớp cửa sổ ổn định và thoát ra cùng với lớp con của nó thay vì mở một
vỏ. Trung tâm điều khiển ẩn các tab và có Độ trong suốt và Độ mờ độc lập
giá trị; màn hình thanh tác vụ và cửa sổ bật lên nhiệt độ sử dụng thanh tab dưới cùng cho
tiêu đề `Press q to close` đã bản địa hóa. Trình khởi chạy chấp nhận `--title TITLE` để thiết lập
tiêu đề tab và cửa sổ Kitty cố định. Thiết bị đầu cuối bình thường giữ các tab của nó và
cài đặt.

Khi LANG không có trong môi trường trình khởi chạy, `argvus terminal` sẽ cung cấp ngôn ngữ cá nhân từ `$XDG_CONFIG_HOME/locale.conf` (tiếng Anh theo mặc định) trước khi Kitty khởi tạo bố cục bàn phím. Ghi đè môi trường miền địa phương hiện tại được giữ nguyên. Khởi động lại Kitty sau khi cập nhật: xuất LANG bên trong shell hiện có không khởi tạo lại bảng soạn thư của Kitty.

LC_ALL và LC_CTYPE trống được kế thừa từ quá trình khởi tạo phiên sẽ bị xóa khỏi môi trường con Kitty. Nếu không, Kitty coi ngôn ngữ trống là rõ ràng và không thể tải bảng soạn thảo của nó, ngay cả với LANG hợp lệ. Ghi đè không trống được giữ nguyên. Điều này cũng áp dụng cho các cấu hình mới và yêu cầu mở lại Kitty sau khi cập nhật trình khởi chạy.

Cửa sổ Trung tâm điều khiển chuyên dụng (`argvus-control-center`) nhận được ghi đè Kitty `confirm_os_window_close=0` mỗi lần khởi chạy, do đó, hành động đóng bộ tổng hợp không yêu cầu xác nhận. Việc ghi đè không được ghi vào hồ sơ thiết bị đầu cuối dùng chung; các TUI khác và thiết bị đầu cuối thông thường vẫn giữ chính sách hiện có của họ.
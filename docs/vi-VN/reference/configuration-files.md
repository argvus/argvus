---
title: Configuration files
description: Cấu hình người dùng, hệ thống, gói mặc định và ARGVUS được tạo.
---

| Lớp | Vị trí | Ý nghĩa |
| --- | --- | --- |
| Hệ thống | `/etc/argvus/` | Cấu hình quản trị viên cho các dịch vụ và nhà cung cấp. |
| Mặc định gói | `/usr/share/argvus/` | Giá trị mặc định của thành phần chỉ đọc và nội dung thời gian chạy. |
| Người dùng | `$XDG_CONFIG_HOME/argvus/config.json` | Tùy chọn di động Canonical. |
| Dữ liệu được quản lý | `$XDG_CONFIG_HOME/argvus/data/` | Dữ liệu thành phần, tệp tương thích và siêu dữ liệu nội bộ. |
| Đã tạo | `$XDG_CONFIG_HOME/argvus/data/generated/` | Các phép chiếu của `config.json`, chỉ được viết bởi `argvus-config`. Không bao giờ chỉnh sửa, không bao giờ sao chép ở nơi khác dưới dạng cấu hình và không bao giờ đọc lại dưới dạng nguồn. |
| Thời gian chạy | `$XDG_RUNTIME_DIR/` và `$XDG_CACHE_HOME/` | Ổ cắm, ổ khóa và trạng thái tạm thời. |

Trạng thái chủ đề, giọng nói, hình nền tùy chỉnh và hiệu ứng đang hoạt động là trạng thái hợp lý của người dùng. Mô hình logic được quản lý được ghi lại trong [Cấu hình Canonical](/docs/reference/argvus-config/). Các tệp Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, Yazi, Superfile, thiết bị đầu cuối, phông chữ và thiết bị di động đã tạo không được coi là nguồn thông tin xác thực độc lập.

`argvus-config` cũng phản chiếu lớp chủ đề vào `data/waybar/argvus-taskbar.{jsonc,css}` và `data/waybar/argvus-widget-telemetry.{jsonc,css}`. Các bản sao đó được thay thế từ các giá trị mặc định được đóng gói khi có sự thay đổi về hình thức, sau đó được viết lại các khối được quản lý được phân tách, vì vậy hãy coi chúng là các tệp có nguồn gốc thay vì các tệp được duy trì bằng tay. Cấu hình ứng dụng gốc trong `$XDG_CONFIG_HOME/<app>` thì khác: đó là mục tiêu bộ chuyển đổi được viết bởi người điều phối giao diện, không bao giờ được viết bởi máy chiếu.

## Ngoại lệ đối với quy tắc một người viết

`argvus config` là người viết duy nhất của `data/generated/`. Có một ngoại lệ đối với việc di chuyển:
khi một bản sao người dùng của một tập tin đóng gói được tạo ra, một tập tin mà phiên bản cũ hơn để lại bên dưới
`data/generated/` được chuyển đến vị trí người dùng của nó (`data/`), vì vậy người dùng sẽ giữ lại các chỉnh sửa của mình. các
việc di chuyển chỉ xảy ra đối với các tệp mà phiên bản hiện tại không tạo ra. Đừng dựa vào điều này
đường dẫn cho các tệp mới: thêm tệp được tạo trong hình chiếu, không phải ở đây.
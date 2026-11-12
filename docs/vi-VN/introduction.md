---
title: Giới thiệu
description: ARGVUS là gì, máy tính để bàn của nó hoạt động như thế nào và các thành phần của nó phù hợp ở đâu.
---

ARGVUS là môi trường máy tính để bàn Hyprland và Wayland mô-đun dành cho Arch Linux. Nó là một tập hợp các gói tích hợp chứ không phải là một ứng dụng nguyên khối: phiên, bộ tổng hợp, bề mặt shell, ứng dụng cài đặt, nhà cung cấp và nội dung trực quan hoạt động cùng nhau thông qua các hợp đồng trạng thái chung và thời gian chạy rõ ràng.

Trang này là bản đồ của tài liệu. Nó giải thích toàn bộ màn hình ở mức cao; các trang được liên kết chứa các chi tiết vận hành và triển khai.

## Cách thức hoạt động của ARGVUS

Phiên bình thường đi theo con đường này:

```text
graphical login or TTY
        ↓
argvus-session → argvus start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```

`argvus-session` sở hữu thiết lập môi trường và vòng đời phiên. Nó không triển khai mọi tính năng của máy tính để bàn: mỗi thành phần giữ quyền sở hữu các lệnh, cấu hình và tải trọng dịch vụ riêng.

Để cài đặt và khởi động, hãy xem [Cài đặt](/docs/getting-started/installation/) và [Phiên](/docs/user-guide/sessions/). Để biết quyền sở hữu dịch vụ, hãy liên hệ với nhà phát triển [vòng đời thời gian chạy](/docs/developer-guide/architecture/runtime-lifecycle/).

## Các lớp chính

### Điểm đầu vào và cơ sở hạ tầng

Gói `argvus` cung cấp bộ điều phối `/usr/bin/argvus` và điểm vào gói chính. `argvus-session` cung cấp các điểm vào đồ họa và TTY, mục tiêu người dùng và `argvus sessionctl`. Chức năng bản địa hóa và TUI được chia sẻ đến từ không gian làm việc `argvus` (các thùng `argvus-tui` và `argvus-i18n`) và lệnh `argvus i18n`.

### Bộ tổng hợp và vỏ máy tính để bàn

`argvus-hyprland` cung cấp cấu hình và tổ hợp phím Hyprland. `argvus-waybar` cung cấp tệp nhị phân Waybar đã vá, trong khi `argvus-taskbar` sở hữu cấu hình và hành động của thanh tác vụ. `argvus-control-panel` cung cấp bảng điều khiển Quickshell và `argvus widget-telemetry` cung cấp các tiện ích hệ thống tùy chọn, đang được xử lý trong `argvus`.

Xem [Desktop](/docs/user-guide/desktop/) để biết cách sử dụng và [bề mặt vỏ](/docs/developer-guide/subsystems/shell-surfaces/) để biết quyền sở hữu triển khai.

### Cài đặt và ứng dụng

`argvus control-center` là ứng dụng cài đặt bàn phím đầu tiên, được cung cấp dưới dạng trợ giúp trong `argvus`. Các công cụ lịch và thiết bị di động là công cụ trợ giúp GTK 4 trong `argvus`; màn hình hệ thống và thiết bị đầu cuối là các lệnh phụ của `argvus`; trình khởi chạy là `argvus-launcher`. Tất cả chúng đều được tích hợp với shell và bộ điều phối.

Xem [Ứng dụng](/docs/user-guide/applications/) và [tham chiếu lệnh](/docs/reference/command-line/).

### Nhà cung cấp và tích hợp

Màn hình, mạng, Bluetooth, nguồn, thông báo, tường lửa và bộ lưu trữ di động được cung cấp bởi các dự án tập trung. Trung tâm điều khiển và Bảng điều khiển sử dụng các nhà cung cấp này; họ không thay thế chúng.

Hình ảnh đăng nhập, khóa và khởi động cũng riêng biệt: `argvus-greeter` xử lý chào mừng, `argvus-lock` xử lý Hyprlock, trình trợ giúp chuyển giao được cung cấp trong `argvus` vẽ vòng quay đăng nhập, lớp phủ tải xử lý chuyển đổi chủ đề và `argvus-boot-splash` xử lý phần khởi động.

Xem [Phần cứng](/docs/user-guide/hardware/), [Quyền riêng tư và bảo mật](/docs/user-guide/privacy-and-security/) và nhà phát triển [tham chiếu thành phần](/docs/developer-guide/).

## Trạng thái hiển thị được chia sẻ

Các tùy chọn chủ đề, giọng, hình nền, phông chữ và hiệu ứng được lưu trữ dưới dạng trạng thái logic trong `$XDG_CONFIG_HOME/argvus` (thường là `~/.config/argvus`). `argvus-config` là thành phần duy nhất ghi `data/generated/`, nơi nó chiếu các tệp người tiêu dùng cho Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, thông báo, màn hình khóa, Yazi, Superfile và thiết bị đầu cuối. Rofi là ngoại lệ: `argvus launcher` đóng gói cấu hình và chủ đề của nó dưới dạng tệp đơn giản và các bản sao đã chỉnh sửa của người dùng nằm trong `data/rofi/`. Trình trợ giúp giao diện cam kết thay đổi trạng thái và sau đó điều chỉnh các ứng dụng bên ngoài; họ không bao giờ viết cây được tạo ra.Thư mục được tạo không phải là nguồn gốc của sự thật. Xem [Giao diện](/docs/user-guide/appearance/) và [cấu hình và trạng thái](/docs/developer-guide/architecture/configuration-and-state/).

Hệ thống thị giác cũng kiểm soát bố cục. Chế độ dính nhỏ gọn và vuông vức; Chế độ giãn cách sử dụng các khoảng trống cửa sổ lớn hơn, các góc tròn và lề rộng hơn xung quanh thanh tác vụ và vỏ. Thanh tác vụ mặc định ở cạnh trên nhưng hỗ trợ vị trí trên/dưới thông qua trạng thái bố cục được chia sẻ. Các hiệu ứng kiểm soát độ trong suốt, độ mờ, bóng và hình ảnh động trên bề mặt bộ tổng hợp và màn hình nền.

Xem [bảng chủ đề](/docs/user-guide/appearance/) để biết các dòng và màu hiện tại, [hiệu ứng](/docs/argvus-appearance/effects/) để biết hợp đồng hiệu ứng dùng chung và [thanh tác vụ](/docs/argvus-taskbar/taskbar/) để biết vị trí thanh và cửa sổ.

## ARGVUS là gì—và không phải là gì

ARGVUS là lớp môi trường máy tính để bàn trên Arch Linux. Arch Linux cung cấp hệ điều hành và các gói; Hyprland cung cấp thành phần Wayland; ARGVUS điều phối phiên đồ họa, bề mặt vỏ, cài đặt, giao diện, dịch vụ máy tính để bàn và các ứng dụng tích hợp.

Do đó, nó không chỉ là một cấu hình Hyprland hay một tập hợp các dotfile cá nhân. ARGVUS xác định điểm nhập phiên được hỗ trợ, sở hữu vòng đời của các dịch vụ máy tính để bàn, cung cấp mô hình giao diện và cài đặt chung, đồng thời kết nối các thành phần tập trung như mạng, màn hình, nguồn, thông báo, tài khoản và bộ lưu trữ di động.

ARGVUS không cần thay thế mọi ứng dụng. Nó tích hợp các ứng dụng thượng nguồn được chọn lọc trong đó đó là sự lựa chọn đáng tin cậy nhất, đồng thời duy trì cấu hình và hành vi trên toàn máy tính để bàn một cách mạch lạc.

Cái tên kết hợp hai ý tưởng. *ARGV*, từ *vector đối số*, là danh sách các đối số mà chương trình nhận được và phản ánh mục tiêu đầu tiên của bàn phím là điều khiển máy tính để bàn thông qua các lệnh và phím tắt. *US*, từ *Hệ thống người dùng*, phản ánh mục tiêu của máy tính để bàn tập trung vào người dùng. Để biết giải thích đầy đủ và âm thanh phát âm, hãy xem [Giới thiệu về ARGVUS](/about/).

Để biết ranh giới triển khai và quyền sở hữu gói, hãy xem nhà phát triển [tổng quan về kiến ​​trúc](/docs/developer-guide/architecture/overview/) và [vai trò kho lưu trữ](/docs/developer-guide/architecture/repository-roles/).

## Tiếp tục theo mục tiêu

- [Đọc Câu hỏi thường gặp](/docs/user-guide/faq/).
- [Cài đặt ARGVUS](/docs/getting-started/installation/).
- [Bắt đầu phiên đồ họa hoặc TTY](/docs/user-guide/sessions/).
- [Cấu hình chủ đề và hình nền](/docs/user-guide/appearance/).
- [Sử dụng thanh tác vụ và Bảng điều khiển](/docs/user-guide/desktop/).
- [Hiểu các lệnh, đường dẫn và dịch vụ](/docs/reference/).
- [Nghiên cứu kiến ​​trúc](/docs/developer-guide/architecture/overview/).

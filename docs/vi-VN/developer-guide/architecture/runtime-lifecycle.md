---
title: Vòng đời thời gian chạy
description: Khởi động phiên và quyền sở hữu dịch vụ người dùng.
---

`argvus-session` nhập môi trường, khởi động Hyprland thông qua `argvus start` và khởi động `argvus-session.target` sau khi bộ tổng hợp sẵn sàng. Mục tiêu kéo các dịch vụ shell, notification, hình nền, nhàn rỗi và clipboard. `argvus sessionctl` tọa độ bắt đầu, dừng, tải lại, khởi động lại, trạng thái và nhật ký.

Phiên dừng mục tiêu khi Hyprland thoát ra. Các dự án riêng lẻ vẫn chịu trách nhiệm về các tệp nhị phân và cấu hình của riêng mình.

## Khởi động

Điểm vào đồ họa là `/usr/share/wayland-sessions/argvus.desktop`, gọi `argvus session`. Phiên nhập môi trường cấu hình ARGVUS và XDG, đồng bộ hóa nó với kích hoạt `systemd --user` và D-Bus, đồng thời ủy quyền khởi động bộ tổng hợp cho `argvus start`.

`argvus start` xác thực cấu hình Hyprland, áp dụng dự phòng đóng gói khi tìm thấy tệp được tạo trong kho hoặc cấu hình người dùng không hợp lệ, phát hiện khả năng tương thích ảo hóa và khởi chạy Hyprland. Sau đó, cầu nối sẵn sàng của bộ tổng hợp gọi `argvus sessionctl ready`, bắt đầu mục tiêu ARGVUS với môi trường Wayland và Hyprland hiện tại.

Mục tiêu sử dụng bộ phận chuẩn bị một lần trước khi bắt đầu các thành phần chạy dài. Quá trình chuẩn bị sẽ áp dụng cấu hình, chủ đề và trạng thái nhấn đã tạo, trạng thái giám sát, tích hợp ứng dụng và môi trường phiên. Các dịch vụ liên tục thuộc sở hữu của `systemd --user`, không phải bởi các quy trình nền đặc biệt.

`argvus start` xếp hàng `argvus-config.service` (`--no-block`) ngay trước khi khởi chạy Hyprland: trình chiếu không cần bộ tổng hợp nên nó chồng lấp quá trình khởi động bộ tổng hợp. Sau đó, `argvus sessionctl ready` khởi động mục tiêu, mục tiêu này sẽ phát hiện đơn vị đó đang hoạt động hoặc chờ công việc của nó, do đó cấu hình chuẩn vẫn được chiếu trước khi chạy chuẩn bị. Khi mục tiêu đã bắt đầu, `sessionctl ready` đợi tối đa 1,5 giây để hình nền (bề mặt lớp nền) và thanh tác vụ được ánh xạ trước khi xóa lớp phủ tải, vì một dịch vụ đơn giản được tính là đã bắt đầu trước khi nó vẽ; một bề mặt có thiết bị không chạy sẽ không được chờ đợi và thời gian chờ được ghi lại là `session.desktop-surfaces`, với `session.wallpaper-surface` và `session.taskbar-surface` cho thời gian diễn ra của mỗi bề mặt. Một lỗi chiếu nghiêm trọng hoặc lỗi khởi động mục tiêu được ghi lại và dừng lớp phủ tải độc lập, ngăn chặn một vòng quay dường như vô tận. Người tiêu dùng trực quan tùy chọn phải giải quyết dự phòng hoặc báo cáo lỗi xuống cấp mà không thay thế mục tiêu phiên.

### Chi phí đăng nhập

Đăng nhập không có thay đổi cấu hình sẽ không lặp lại công việc tốn kém:

- Phép chiếu chỉ chạy công tắc chủ đề và bộ điều hợp hiệu ứng khi phần ngoại hình hoặc hiệu ứng thay đổi hoặc khi thiếu đầu ra thực của chúng (bản sao người dùng cụ thể hóa và `data/state/effects/<theme>.conf`).
- Quá trình chuẩn bị chỉ áp dụng chủ đề khởi động và dấu trọng âm ở trên chủ đề đó khi dấu trong `$XDG_CACHE_HOME/argvus/session/theme-startup.stamp` không còn khớp với các tệp chủ đề được đóng gói và dấu trọng âm. Nội dung tệp dấu vân tay được đóng dấu nên việc cài đặt lại các tệp giống hệt nhau sẽ không làm mất hiệu lực của tệp đó. Đầu vào chủ đề đã thay đổi hoặc giọng mới sẽ vô hiệu hóa nó một lần. Việc di chuyển tem dựa trên dấu thời gian cũ hơn cũng sẽ được xây dựng lại một lần. Khi chủ đề được áp dụng lại, quá trình chuẩn bị cũng sẽ tải lại Hyprland, bắt đầu trên các tệp trước đó.
- Cài đặt GTK (`gsettings`) chỉ được ghi khi chủ đề khởi động được áp dụng lại. Giữa các lần đăng nhập, dconf sẽ giữ chúng, vì vậy đăng nhập ổn định sẽ không chạy lại chúng.
- `argvus-dunst.service` là `Type=dbus`: nó được tính là đã bắt đầu khi Dunst sở hữu `org.freedesktop.Notifications`, do đó mục tiêu phiên không chờ trạng thái tạm dừng được khôi phục.
- Quá trình chuẩn bị chạy `spaces-switch.sh` và `borders-switch.sh` (`--apply-static`) cạnh nhau: chúng viết các tệp rời rạc (lề JSONC của các thanh và hình học hiệu quả cũng như chủ đề Rofi và CSS của các thanh).- Phép chiếu chỉ ghi lại một tệp được tạo khi nội dung của nó thay đổi. Các tệp không thay đổi sẽ được giữ nguyên, do đó, quá trình đăng nhập ổn định sẽ không đồng bộ hóa chúng lại.
- Bước hiển thị chuẩn bị (`argvus config display session-prepare`) chỉ áp dụng DPMS và các đầu ra bị vô hiệu hóa: nó không chiếu lại cũng như không tải lại bộ tổng hợp.
- `argvus-session-reload.service` chạy `argvus sessionctl reload-desktop --login`, chỉ tải lại Hyprland khi bảng kê khai chiếu liệt kê `hyprland` trong số những người tiêu dùng bị ảnh hưởng. Các thành phần máy tính để bàn không chờ đợi nó.

Công cụ quay vòng phiên (trình trợ giúp chuyển giao ở chế độ Wayland) được ánh xạ từ quá trình chuyển giao của bộ tổng hợp cho đến khi `argvus sessionctl ready` bắt đầu mục tiêu, do đó, một công cụ quay vòng duy nhất sẽ bao trùm toàn bộ quá trình khởi động. Nó không có `PartOf=`, vì việc tạo lại mục tiêu khi đăng nhập mới không được gỡ bỏ nó; `sessionctl` dừng nó một cách rõ ràng và `RuntimeMaxSec=30` kết thúc nó nếu không có điểm dừng nào xuất hiện.

Mỗi bước chuẩn bị, tải lại và bộ điều hợp trình chiếu sẽ thêm thời lượng của nó vào nhật ký phiên dưới dạng `<epoch-ms> timing <step> <N>ms`; xem [khắc phục sự cố khởi động phiên](/docs/user-guide/troubleshooting/session-startup/).

Giải pháp đường dẫn, chuẩn bị phiên và kế hoạch tải lại Hyprland đều thuộc sở hữu của Rust.
`argvus config session-prepare` và `argvus config session-reload-config` chạy
các bước khởi động và tải lại; tập lệnh `hypr-init.sh` cũ đã bị xóa.

## Dịch vụ và tắt máy

Mục tiêu thường sở hữu `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service` khi được bật, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, trình theo dõi clipboard, thông báo bố cục bàn phím và tích hợp PolicyKit. Lớp phủ tải phiên được bắt đầu bởi `argvus start`, không phải bởi mục tiêu. Lỗi trong dịch vụ tùy chọn không thay thế toàn bộ phiên.

Các dịch vụ cần kết nối máy khách Wayland, bao gồm Bảng điều khiển và trình quản lý không hoạt động, hãy đợi thời gian chạy Hyprland có thể sử dụng được trước khi khởi chạy ứng dụng khách của họ. Điều này tránh khởi động Quickshell hoặc hypridle dựa vào ổ cắm cũ trong quá trình chuyển giao bộ tổng hợp; nếu Hyprland được thay thế, systemd có thể khởi động lại thành phần sau khi có thời gian chạy mới.

Công tắc chủ đề tuần tự hóa các lần tải lại cam kết, xây dựng, trình chiếu và tiêu dùng của nó dưới dạng một hoạt động trong vòng đời dưới khóa chủ đề. Nó không tải lại `argvus-config.service` cũng như không khởi động lại dịch vụ: phép chiếu nó chạy chỉ ghi các tệp và để lại chủ đề cho công tắc trong khi khóa được giữ và các ứng dụng tiêu dùng đang chạy tải lại tại chỗ (xem [Giao diện và chủ đề](/docs/developer-guide/subsystems/appearance-and-themes/)).

Khi Hyprland thoát, `argvus start` sẽ dừng `argvus-session.target` và các dịch vụ liên kết với nó. Điều này ngăn chặn các quá trình thanh tác vụ, shell, thông báo, nhàn rỗi và clipboard thoát khỏi tình trạng đăng xuất.

## Đang tải lại

```sh
argvus sessionctl reload
```

Tải lại sẽ nhập lại môi trường đồ họa, sau đó **áp dụng** thế hệ đã cam kết khi lệnh miền trước đó chẳng hạn như `apply-theme`, `accent`, `set`/`patch` hoặc thay đổi khối đo từ xa tiện ích đã dự kiến ​​thay đổi. Trong trường hợp đó, nó khôi phục những người tiêu dùng được liệt kê trong gói kê khai và không gọi lại `argvus config project` vì việc định hướng lại sẽ xây dựng lại một cây giống hệt và loại bỏ kế hoạch. Phép chiếu, di chuyển và `ensure` chỉ được thực hiện khi không có gói đang chờ xử lý hợp lệ nào tồn tại - khi khởi động phiên hoặc sau khi `argvus config recover` phải khôi phục tài liệu trước đó. Chỉ cho phép lệnh cấm hoạt động hoàn toàn khi các phép chiếu và thời gian chạy được quản lý đều hiện hành. Nếu trạng thái chuẩn không thay đổi nhưng ổ cắm tổng hợp, dịch vụ hoặc ứng dụng tiêu dùng đã cũ, hãy tải lại để điều chỉnh thay vì quay lại sớm. Phân xuất thông thường bao gồm Hyprland, thanh tác vụ, đo từ xa tiện ích, Bảng điều khiển, thông báo, hình nền và chế độ chờ theo bảng kê khai chiếu, trong khi các tiêu dùng tùy chọn không thay đổi vẫn không bị ảnh hưởng. Mọi tệp trong `data/generated/` đều được viết bởi `argvus-config` tại thời điểm này, vì vậy phiên này không bao giờ hiển thị lại chính chủ đề đó. Sau đó, quá trình tải lại sẽ chạy `theme-adapters.sh`, điều này giúp điều hòa những người tiêu dùng không thể tự đọc tài liệu chuẩn hoặc phép chiếu đã xuất bản. Những bộ điều hợp đó chỉ đọc thế hệ đã cam kết: họ viết các tệp tiêu dùng và bộ nhớ đệm của riêng họ, đồng thời họ không bao giờ viết tài liệu chuẩn hoặc `data/generated/`, do đó, một hành động của người dùng vẫn phải trả một lần đột biến, một lần chiếu, một lần xuất bản và một lần tải lại. Chỉ sử dụng `argvus sessionctl restart <component>` khi một dịch vụ thuộc sở hữu duy nhất cần khởi động lại một cách rõ ràng. Kiểm tra [tham chiếu dịch vụ](/docs/reference/services/) để biết tên đơn vị chính xác.

Các hoạt động chiếu và tải lại được tuần tự hóa bằng khóa thời gian chạy của người dùng. Để theo dõi thời gian tạm thời, hãy đặt `ARGVUS_SESSION_PERF=1` trước khi gọi tải lại và kiểm tra `~/.local/state/argvus/session.log`; dấu vết ghi lại ranh giới giai đoạn mà không thay đổi hành vi tải lại thông thường.
---
title: File locations
description: Các tệp và thư mục ARGVUS đã cài đặt quan trọng.
---

- `/usr/bin/argvus*` — lệnh công khai.
- `/usr/share/argvus/hyprland/` — Cấu hình và trợ giúp Hyprland.
- `/usr/share/argvus/control-panel/` — QML và tập lệnh của Bảng điều khiển.
- `/usr/share/argvus/taskbar/` — cấu hình và tập lệnh trên thanh tác vụ.
- `/usr/share/argvus/lock/` — Cấu hình Hyprlock.
- `/usr/share/argvus/portal/` — tệp tích hợp cổng thông tin.
- `/usr/share/backgrounds/argvus/` — nội dung hình nền.
- `/etc/argvus/` — cấu hình nhà cung cấp hệ thống.
- `/etc/argvus/greeter.toml` — cấu hình lời chào.
- `/etc/argvus/removable-devices/config.json` — mặc định của hệ thống cho bộ lưu trữ di động.
- `$XDG_CONFIG_HOME/argvus/config.json` — tùy chọn di động chuẩn.
- `$XDG_CONFIG_HOME/argvus/data/` — quản lý dữ liệu thành phần, sao lưu, khóa và tạo đầu ra.
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — ghi đè thanh tác vụ Waybar gốc hoàn chỉnh tùy chọn.
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — bản sao thanh tác vụ ARGVUS được quản lý.
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — đã tạo các cấu hình Waybar, bao gồm `argvus-taskbar.css` và `argvus-widget-telemetry.{jsonc,css}`; những điều này được ưu tiên hơn các bản sao ở trên.
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — cũng được phản chiếu trong `data/waybar/`, được thay thế từ mặc định đóng gói khi thay đổi hình thức và sau đó có `ARGVUS_TELEMETRY_*` và các khối phông chữ được viết lại.
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — cây cấu hình Yazi dự kiến, với dự phòng nhận biết biến thể cho các chủ đề không mang hương vị Yazi.
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — bộ chủ đề Superfile dự kiến.
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — cấu hình thiết bị đầu cuối dự kiến.
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — bảng màu Qt6ct dự kiến.
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — tệp chủ đề GTK dự kiến.
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — bản sao của người dùng về cấu hình, chủ đề và chế độ Rofi. ARGVUS tạo từng bản sao khi nó thay đổi tệp lần đầu tiên; các tệp không thay đổi được đọc từ `/usr/share/argvus/launcher/config/`.
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — biểu định kiểu dự kiến ​​cho thiết bị di động.
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — Cài đặt tiêu điểm ARGVUS, lịch sử phiên và điểm kiểm tra phiên hiện tại.
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — trạng thái ARGVUS Focus tạm thời được thanh tác vụ sử dụng; bị xóa khi Focus thoát.
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — ARGVUS Cài đặt đồng hồ và đồng hồ thế giới, báo thức và điểm kiểm tra hẹn giờ/đồng hồ bấm giờ.
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — màu của ARGVUS Focus và ARGVUS Clock (chủ đề tiêu dùng `time`).
- `/usr/share/argvus/sounds/` — âm thanh của ARGVUS Focus và ARGVUS Clock (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`).
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — bản dựng chủ đề (`builds/<id>-<variant>-<hash>/`), được viết bởi `argvus theme render` và bằng mọi chuyển đổi chủ đề và thay đổi giọng, cũng như liên kết tượng trưng `current` tới bản dựng của chủ đề đang hoạt động.
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — người dùng Hyprland Lua ghi đè và các phép chiếu Hyprland gốc.
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — cấu hình Hyprland gốc hoàn chỉnh tùy chọn được chọn bởi `argvus start`; nó không phải là một lớp phủ gia tăng.
- `$XDG_STATE_HOME/argvus/config-projection.json` — bảng kê khai chiếu: tạo, hàm băm cấu hình hiệu quả, các phần đã thay đổi và công việc đang chờ xử lý. Được đọc bởi `argvus sessionctl reload`; không bao giờ là nguồn gốc của sự thật.
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — tùy chọn khối đo từ xa kế thừa. Chỉ đầu vào di chuyển; các giá trị chuẩn tồn tại trong `config.json` và tệp này không còn được ghi nữa.
- `$XDG_STATE_HOME/argvus/session.log` — nhật ký phiên đồ họa (thường là `~/.local/state/argvus/session.log`).
- `/usr/share/wayland-sessions/argvus.desktop` — mục nhập phiên đồ họa.
- `/usr/share/plymouth/themes/argvus/` — tài sản giật gân khởi động được đóng gói.

Tải trọng chính xác tùy theo từng gói; xem tài liệu tham khảo thành phần dành cho nhà phát triển và bản kê khai gói có liên quan.
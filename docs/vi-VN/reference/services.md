---
title: Dịch vụ
description: ARGVUS hệ thống và dịch vụ người dùng.
---

Mục tiêu người dùng chính là `argvus-session.target`. Các dịch vụ người dùng phổ biến bao gồm `argvus-taskbar.service`, `argvus-control-panel.service`, `argvus-widget-telemetry.service`, `argvus-hyprland.service`, `argvus-dunst.service`, `argvus-wallpaper.service`, `argvus-hypridle.service`, clipboard dịch vụ và `argvus-session-loading.service`.

Tường lửa là một dịch vụ hệ thống có tên `argvus-firewall.service`. Gói lịch cài đặt `argvus-taskbar-calendar.service` làm đơn vị người dùng và `argvus-clock.service` (bị tắt theo mặc định) khởi động Đồng hồ ARGVUS ẩn sau khi đăng nhập khi **Bắt đầu với phiên** được bật.

Kiểm tra các đơn vị đang hoạt động với:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```

## Đơn vị thuộc sở hữu của phiên

Gói phiên cài đặt các đơn vị người dùng này trong `/usr/lib/systemd/user` (tải trọng nguồn nằm trong cây cấu hình dùng chung của gói phiên):

| Đơn vị | Vai trò |
| --- | --- |
| `argvus-session.target` | Ranh giới trọn đời cho phiên đồ họa ARGVUS. |
| `argvus-session-prepare.service` | Chuẩn bị khởi động một lần và cấu hình được tạo. |
| `argvus-control-panel.service` | Bảng điều khiển Quickshell. |
| `argvus-taskbar.service` | Thanh tác vụ ARGVUS chính. |
| `argvus-widget-telemetry.service` | Bề mặt đo từ xa/sysinfo tùy chọn. |
| `argvus-hyprland.service` | Thiết bị một lần theo yêu cầu chạy `hyprctl reload`. |
| `argvus-config.service` | Chiếu cấu hình chuẩn (`argvus config project`); tải lại chạy `argvus sessionctl reload-config-transaction`. |
| `argvus-session-reload.service` | Tải lại đăng nhập (`argvus sessionctl reload-desktop --login`): chỉ tải lại Hyprland khi phép chiếu ảnh hưởng đến nó. |
| `argvus-ssh-agent.service` | Đại lý SSH cho phiên. |
| `argvus-secret-service.service` | Bắt đầu dịch vụ bí mật thông qua `argvus sessionctl start-secret-service`. |
| `argvus-wallpaper.service` | Chương trình phụ trợ hình nền đang hoạt động. |
| `argvus-dunst.service` | Daemon thông báo Dunst. |
| `argvus-hypridle.service` | Chính sách nhàn rỗi và khóa. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | Người theo dõi lịch sử clipboard. |
| `argvus-keyboard-layout.service` | Thông báo bố trí bàn phím. |
| `argvus-polkit.service` | Tích hợp đại lý PolicyKit. |
| `argvus-session-loading.service` | Lớp phủ tải phiên từ quá trình chuyển giao bộ tổng hợp cho đến khi các thành phần máy tính để bàn khởi động. |

`argvus-blueman-applet.service` cũng được vận chuyển theo gói phiên. Một số đơn vị là tùy chọn hoặc bắt đầu có điều kiện; không phải tất cả chúng đều hoạt động trong mỗi phiên.

Để kiểm tra định nghĩa hiệu quả của một đơn vị thay vì chỉ kiểm tra nguồn của nó:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```

`argvus-firewall.service` cấp hệ thống thuộc về `argvus-firewall`. Gói lịch vận chuyển riêng `argvus-taskbar-calendar.service` dưới dạng đơn vị người dùng. `argvus-clock.service` là đơn vị người dùng của gói `argvus`, `WantedBy=graphical-session.target`, được bật từ cài đặt Đồng hồ ARGVUS.

## Lệnh quản lý phiên

Lệnh `argvus sessionctl` quản lý vòng đời phiên và có thể khởi động lại các thành phần cụ thể mà không cần đăng xuất:

### Vòng đời của phiên

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```

### Khởi động lại theo thành phần cụ thể

Khởi động lại các dịch vụ riêng lẻ mà không ảnh hưởng đến toàn bộ phiên:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```

### Đồng bộ hóa môi trường

Nhập các biến môi trường vào phiên:

```sh
argvus sessionctl import-environment
```

Điều này đồng bộ hóa các biến Wayland, Hyprland, Qt, con trỏ và DBus giữa các dịch vụ người dùng shell và systemd.

## Tham chiếu đơn vị dịch vụ

Các đơn vị người dùng bổ sung có thể được quản lý trực tiếp bằng `systemctl --user`:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```

Nhật ký cho phiên khởi động đầu tiên được lưu trữ tại:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```

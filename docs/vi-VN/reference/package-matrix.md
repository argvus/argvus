---
title: Package matrix
description: ARGVUS package responsibilities.
---

| Trọn gói | Trách nhiệm |
| --- | --- |
| `argvus` | Gói chính: bộ điều phối, các lệnh trong quá trình (cấu hình, bản dịch, khóa, dự án, thiết bị đầu cuối, giám sát hệ thống, mạng, Bluetooth, đo từ xa, thiết bị đầu cuối TUI, thiết lập), dữ liệu được chia sẻ và danh mục i18n. Mọi gói thành phần đều phụ thuộc vào nó. |
| `argvus` (người trợ giúp) | Các trình trợ giúp trong cùng một gói: Trung tâm điều khiển và Trình phân tích mức sử dụng đĩa (ứng dụng đầu cuối), cùng với các ứng dụng GTK 4 cho lịch trên thanh tác vụ, thiết bị di động, lớp phủ tải, ARGVUS Focus và ARGVUS Clock. |
| `argvus` (người chào) | Lời chào đăng nhập cho lời chào, được vận chuyển trong cùng một gói và được xây dựng từ `crates/argvus-greeter`. |
| `argvus-desktop` | Cấu hình máy tính để bàn và ứng dụng hàng ngày được yêu cầu bởi gói chính `argvus`. Nó không phụ thuộc vào `argvus`, do đó sự phụ thuộc vẫn là một chiều. |
| `argvus-devtools`, `argvus-pentest` | Các siêu dữ liệu độc lập tùy chọn dành cho các công cụ phát triển và kiểm tra bảo mật được ủy quyền. Gói chính không cài đặt các cấu hình này. |
| `argvus-session` | Điểm vào phiên và vòng đời của người dùng. |
| `argvus-hyprland` | Cấu hình Hyprland. |
| `argvus-control-panel` | Bảng điều khiển Quickshell. |
| `argvus-taskbar`, `argvus-waybar` | Cấu hình Waybar và nhị phân được vá. |
| `argvus-branding` | Nội dung logo và wordmark. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Trạng thái ngoại hình và nội dung trực quan. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | Các nhà cung cấp máy tính để bàn. |
| `argvus-lock`, `argvus-boot-splash` | Màn hình khóa và khởi động giật gân. |
| `argvus-firewall` | Tường lửa, với các đặc quyền riêng. |
| `argvus-launcher` | Cấu hình trình khởi chạy và điểm vào. |
| `argvus-app-profiles` | Cấu hình chỉ dữ liệu cho các ứng dụng đầu cuối đi kèm. |
| `argvus-portal` | Cổng XDG và các giá trị mặc định của Wayland/DBus. |
| `argvus-theme-*` | Một gói tùy chọn cho mỗi nhóm chủ đề chính thức. |
| `argvus-game-snake`, `argvus-games` | Không bắt buộc. Snake là một dự án độc lập; `argvus-games` nhóm các trò chơi tùy chọn. |

## Gói thay thế

`argvus` tuyên bố `replaces`, `conflicts` và `provides` cho các gói được hấp thụ, do đó, bản nâng cấp sẽ loại bỏ chúng và các tệp của chúng không còn ở dạng mồ côi:

`argvus-config`, `argvus-i18n` (với `argvus-language-en-us` và `argvus-language-pt-br`), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` và `argvus-du`. `argvus` cũng thay thế `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices` và các gói chia nhỏ trước đây `argvus-gtk` và `argvus-greeter`, nó vẫn cung cấp để các gói tùy thuộc vào chúng tiếp tục được giải quyết. Trình quản lý tài khoản (`argvus accounts`) được xây dựng từ `crates/argvus-accounts` và được cài đặt dưới dạng `/usr/lib/argvus/commands/accounts`, với quy tắc polkit. Trình phân tích sử dụng đĩa được xây dựng từ `crates/argvus-du` và được cài đặt dưới dạng `/usr/lib/argvus/commands/argvus-du`.

`/usr/bin/argvus-terminal` vẫn được cài đặt làm trình khởi chạy tương thích cho `argvus terminal`. Thiết bị đầu cuối mặc định của Hyprland, danh mục Trung tâm điều khiển và các ứng dụng mặc định đã lưu vẫn đặt tên cho lệnh đó.

Các gói đã xóa `argvus-pwvucontrol` và `argvus-snappy-switcher` không được cung cấp bởi bất kỳ gói nào.

Không yêu cầu thứ tự cài đặt trong số các gói thành phần ARGVUS: `argvus` là gói chính của chúng và mỗi thành phần đều phụ thuộc vào gói đó. Cài đặt `argvus` cũng cài đặt cấu hình `argvus-desktop`; `argvus-devtools` và `argvus-pentest` vẫn được chọn tham gia. Sự hiện diện của một gói trong bảng này không có nghĩa là nó sở hữu toàn bộ tính năng: ví dụ: thanh tác vụ kết hợp `argvus-waybar`, `argvus-taskbar`, `argvus-session` và các nhà cung cấp tên miền.
---
title: Dòng lệnh
description: Lệnh argvus, các lệnh phụ của nó, các cờ được giữ cho các tổ hợp phím và nơi mỗi lệnh chạy.
---

`/usr/bin/argvus` là điểm vào công khai của ARGVUS. Đây là một chương trình Rust được xây dựng từ thùng `argvus-cli`. Mỗi lệnh phụ chạy bên trong chương trình đó, chạy một tệp thực thi trợ giúp từ thư mục lệnh hoặc chạy một tập lệnh đóng gói.

Chạy `argvus --help` để biết danh sách được cài đặt cùng với phiên bản của bạn. Các lệnh phụ được viết mà không có `--` ở đầu. Các biểu mẫu `--` chỉ được lưu giữ khi liên kết phím hoặc mục nhập trên màn hình sử dụng chúng.

## Nơi mỗi lệnh chạy

- **Đang xử lý:** lệnh là một thư viện của không gian làm việc và chạy bên trong `argvus`. Không có tệp thực thi riêng biệt nào tồn tại. Đó là `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`, `tui-terminal`, `sensors` và `setup`.
- **Trình trợ giúp:** một tệp thực thi riêng biệt trong `/usr/lib/argvus/commands/`, được cài đặt bởi gói `argvus`. Đó là `control-center` (ứng dụng đầu cuối), `argvus-du` (ra mắt cùng với `argvus du`) và các ứng dụng GTK 4 `taskbar-calendar`, `removable-devices`, `focus` và `clock`.
- **Tệp thực thi riêng tư:** tệp thực thi mà gói khác cài đặt trong `/usr/lib/argvus/commands/`. `argvus` chỉ khởi động nó. Chúng bao gồm `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` và Điểm vào `greeter-*`.
- **Tập lệnh:** tập lệnh shell được đóng gói chạy với `/bin/sh`. Đây là các lệnh tập lệnh Hyprland được liệt kê bên dưới.

`ARGVUS_COMMANDS_DIR` thay đổi thư mục lệnh để kiểm tra và kiểm tra `DESTDIR`. Một tệp thực thi riêng tư không được tìm thấy ở đó sẽ được tra cứu trên `PATH`.

## Lệnh phụ

| Lệnh | Chạy | Ghi chú |
| --- | --- | --- |
| `argvus config ...` | Đang xử lý | Mô hình cấu hình và công cụ. Cũng đạt được bởi `theme-switch` và `effects-toggle`. |
| `argvus theme ...` | Đang xử lý | `apply ID` là công tắc chủ đề (giống như `theme-switch`). Bản dựng chủ đề: `list` các chủ đề đã cài đặt, `check DIRECTORY` một chủ đề dựa trên các mẫu đã cài đặt, `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`. Xem [Giao diện và chủ đề](/docs/developer-guide/subsystems/appearance-and-themes/). |
| `argvus i18n ...` | Đang xử lý | Bản dịch: `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`. |
| `argvus keys ...` | Đang xử lý | Trình quản lý khóa SSH. `gpg` là bí danh cho các lệnh con GPG. |
| `argvus gpg ...` | Đang xử lý | Khóa GPG: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | Đang xử lý | Bộ chọn dự án. |
| `argvus terminal ...` | Đang xử lý | Cấu hình thiết bị đầu cuối ARGVUS. |
| `argvus system-monitor ...` | Đang xử lý | Giám sát hệ thống btop; `--gpu` chọn các hộp chỉ dành cho GPU và `--panel [--cpu|--mem|--gpu]` nhúng nó vào vỏ ARGVUS TUI. |
| `argvus du` | Người trợ giúp (ứng dụng đầu cuối) | Mở Trình phân tích sử dụng đĩa ARGVUS trong cửa sổ Kitty chuyên dụng và chạy ncdu. |
| `argvus network ...` | Đang xử lý | Trạng thái và điều khiển của Trình quản lý mạng (`networkctl` được giữ dưới dạng bí danh). |
| `argvus bluetooth ...` | Đang xử lý | Trạng thái và điều khiển Bluetooth (`bluetoothctl` được giữ làm bí danh). |
| `argvus widget-telemetry block ...` | Đang xử lý | Khối đo từ xa Waybar. Đây là con đường hot của quán bar. || `argvus widget-telemetry toggle ...` | Đang xử lý | Bật và tắt các khối đo từ xa cũng như thứ tự của chúng. |
| `argvus game-snake` | Thực thi riêng tư | Trò chơi Snake cuối cùng, từ dự án `argvus-game-snake` độc lập, một gói tùy chọn. Bên ngoài một thiết bị đầu cuối, nó sẽ mở một thiết bị đầu cuối. |
| `argvus control-center [page]` | Người trợ giúp (ứng dụng đầu cuối) | Mở Trung tâm điều khiển hoặc tập trung vào cửa sổ đã mở. |
| `argvus welcome [--if-needed]` | Người điều phối → Trung tâm điều khiển | Mở trang Chào mừng tích hợp trong Trung tâm điều khiển. `--if-needed` mở một lần trừ khi tiến trình của mỗi người dùng ghi lại rằng Chào mừng đã được hiển thị, hoàn thành hoặc loại bỏ. |
| `argvus taskbar-calendar [args]` | Ứng dụng GTK 4 | Mặc định là `toggle`. |
| `argvus removable-devices [args]` | Ứng dụng GTK 4 | Mặc định là `menu`. |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Ứng dụng GTK 4 | Cửa sổ bật lên tiêu điểm ARGVUS. Một phiên bản mỗi phiên. |
| `argvus focus status --waybar [--follow]` | Đang xử lý | Phát ra chỉ báo thanh tác vụ Focus dưới dạng Waybar JSON; Luồng `--follow` thay đổi mà không tải GTK. |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Ứng dụng GTK 4 | Cửa sổ bật lên Đồng hồ ARGVUS. Một phiên bản mỗi phiên; `--background` duy trì cảnh báo hoạt động mà không cần cửa sổ bật lên. |
| `argvus loading-theme ...` | `splash` được đóng gói tại `/usr/lib/argvus/loading-theme/splash` | Lớp phủ tải lại phiên trực tiếp; một chuyển đổi chủ đề không sử dụng nó. |
| `argvus session`, `start`, `tty`, `sessionctl` | Tệp thực thi riêng tư | Điểm vào phiên và bộ điều khiển phiên. |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | Tệp thực thi riêng tư | Điểm vào chào mừng. |
| `argvus accounts`, `firewall` | Tệp thực thi riêng tư | Tài khoản và công cụ tường lửa. |
| `argvus launcher`, `snippets`, `ssh`, `git` | Tệp thực thi riêng tư | Công cụ khởi chạy và chọn. |
| `argvus tui-terminal` | Đang xử lý | Trình bao bọc thiết bị đầu cuối cho các ứng dụng TUI. |
| `argvus sensors` | Đang xử lý | Tổng quan về nhiệt độ trực tiếp từ `sensors -j`. |
| `app-profiles-yazi-keymap` | Thực thi riêng tư | Người trợ giúp sơ đồ bàn phím. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | Tệp thực thi riêng tư | Thông báo, hiển thị và hành động trên thanh tác vụ. |
| `argvus setup [options]` | Đang xử lý | Sao chép các giá trị mặc định được đóng gói vào cấu hình người dùng. Xem bên dưới. |

Các đối số được chuyển tiếp không thay đổi, bao gồm khoảng trắng và giá trị trống. Đầu vào, đầu ra, lỗi, tín hiệu và trạng thái thoát tiêu chuẩn thuộc về lệnh chạy. Tên không xác định không thành công với gợi ý chạy `argvus --help`. Lệnh chạy trong quá trình sẽ từ chối một đối số không hợp lệ UTF-8 (trạng thái thoát 2) thay vì chuyển một giá trị đã thay đổi vào; các lệnh chạy một chương trình riêng biệt sẽ nhận được các byte không thay đổi.

## Cờ và bí danh

Những tên này tồn tại để các tổ hợp phím, mục trên màn hình và các tập lệnh cũ hơn tiếp tục hoạt động. Thích biểu mẫu lệnh phụ trong mã mới.

| Cờ hoặc bí danh | Tương đương |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (với các trang `about` và `apps`) |
| Trang `--displays`, `--monitors` | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | trình quản lý tệp (`spf`) |
| `--yazi`, `--yazy` | trình duyệt tệp (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | bắt đầu `tty` (không có đối số nào được chuyển tiếp) |
| `--version`, `-V`, `version` | in `ARGVUS <version>` |
| `help`, `--help`, `-h` | in cách sử dụng || `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

Trung tâm điều khiển chấp nhận cờ trang cũ làm đối số đầu tiên và ánh xạ chúng tới tên trang. Ví dụ: `--apps` trở thành `apps`, `--keyboard-layout` trở thành `locale` và `--hostname` trở thành `system`.

## Lệnh kịch bản Hyprland

Các tập lệnh này chạy các tập lệnh đóng gói và được cấu hình Hyprland sử dụng:

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`.

Một tập lệnh trong cấu hình của người dùng sẽ ghi đè tập lệnh được đóng gói. Thứ tự tra cứu giống như `_config_path` trong `hyprland.lua`: bản sao cấu hình của người dùng, sau đó là bản sao dữ liệu ARGVUS, sau đó là bản sao trạng thái, sau đó là bản sao được tạo và chỉ sau đó là tệp được đóng gói trong `/usr/share/argvus`.

## Ví dụ

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```

## Thiết lập

`argvus setup` sao chép các giá trị mặc định được đóng gói từ `/usr/share/argvus/<app>` sang `~/.config/argvus/data/<app>` để bạn có thể tùy chỉnh chúng. Nó không bao giờ chỉnh sửa các tập tin đóng gói.

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```

Một điểm đến đã tồn tại sẽ được báo cáo và để yên, trừ khi `--force` được cung cấp. Với `--force`, thư mục hiện có được đổi tên thành `<name>.bak-<timestamp>` trước khi sao chép.

Ứng dụng `argvus` rất đặc biệt: nó sao chép nội dung của thư mục được đóng gói vào thư mục gốc cấu hình. Vì gốc cấu hình đã tồn tại khi chạy thiết lập nên `--copy argvus` cần `--force` trong mã hiện tại.

Thiết lập xác nhận mọi tùy chọn trước khi nó thay đổi bất cứ điều gì. Từ chối tên ứng dụng không hợp lệ (trống, bắt đầu bằng `.` hoặc chứa `/`).

## Phiên và tải lại

`SUPER + SHIFT + R` chạy `argvus sessionctl reload`. Bộ điều khiển phiên áp dụng cấu hình đã tạo và chỉ khởi động lại các thành phần mà nó sở hữu. Nó không sao chép `/usr/share/argvus` vào thư mục chính và nó không thay thế tệp JSON không đúng định dạng.

## Ánh sáng ban đêm

Mở **Máy tính để bàn & Đầu vào > Hiển thị > Đèn ngủ** hoặc chạy `argvus control-center displays night-light`. Định cấu hình nhiệt độ, kích hoạt thủ công và lịch trình cục bộ cố định thông qua `argvus config display night-light`. Xem [Ánh sáng ban đêm](/docs/user-guide/night-light/) để biết các yêu cầu về lệnh, trạng thái và thời gian chạy.

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` chạy lệnh tương tác bên trong khung thiết bị đầu cuối ARGVUS được chia sẻ. Ctrl+E đóng khung; người gọi nó cung cấp phần chân trang được bản địa hóa. Sử dụng lệnh này bên trong thiết bị đầu cuối hoặc khi lệnh được truyền tới `argvus tui-terminal --term kitty -- ...`.

`argvus tui-terminal --term kitty --profile ai-agents` sử dụng cấu hình Kitty chuyên dụng với các tab ẩn và chỉ các phím tắt trong khay nhớ tạm. Người gọi cung cấp lệnh và tiêu đề; vị trí cửa sổ được xử lý theo quy tắc Hyprland cho lớp `argvus-ai`.
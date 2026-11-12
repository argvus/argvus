---
title: Cài đặt ARGVUS
description: Cài đặt, cập nhật, xóa và định cấu hình ARGVUS.
---

ARGVUS được phân phối dưới dạng gói Arch Linux thông qua kho lưu trữ gói ARGVUS chính thức.

## Trước khi cài đặt

Kích hoạt kho lưu trữ Arch Linux `multilib` chính thức trước khi cài đặt ARGVUS. Chỉnh sửa `/etc/pacman.conf` và bỏ ghi chú cả phần `[multilib]` và dòng `Include` của nó:

```ini
[multilib]
Include = /etc/pacman.d/mirrorlist
```

Sau đó đồng bộ hóa cơ sở dữ liệu gói và thực hiện nâng cấp toàn bộ hệ thống:

```sh
sudo pacman -Syu
```

Trình cài đặt nhanh định cấu hình kho lưu trữ ARGVUS nhưng không kích hoạt `multilib` cho bạn.

## Cài đặt nhanh

Khi cài đặt Arch Linux mới, bạn có thể cài đặt ARGVUS trực tiếp từ TTY mà không cần cài đặt môi trường máy tính để bàn khác trước.

Chạy:

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```

Trình cài đặt:

1. Nhập và ký cục bộ khóa ký kho lưu trữ ARGVUS.
2. Định cấu hình kho lưu trữ gói ARGVUS.
3. Cập nhật cơ sở dữ liệu gói.
4. Cài đặt gói `argvus` và các gói phụ thuộc của nó.

Sau khi cài đặt, hãy đăng xuất và chọn **ARGVUS** từ trình quản lý hiển thị của bạn.

> **Lưu ý:** Trình cài đặt nhanh yêu cầu `curl`, `sudo` và các công cụ quản lý gói Arch Linux tiêu chuẩn.

## Chỉ cấu hình kho lưu trữ

Để định cấu hình kho lưu trữ gói ARGVUS mà không cần cài đặt máy tính để bàn, hãy chạy:

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```

Cài đặt ARGVUS theo cách thủ công sau đó:

```sh
sudo pacman -Syu argvus
```

## Cài đặt thủ công

Nếu bạn muốn định cấu hình kho lưu trữ từng bước, hãy sử dụng các lệnh sau.

### 1. Tải xuống khóa ký kho lưu trữ

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```

### 2. Nhập khóa ký

```sh
sudo pacman-key --add /tmp/argvus.gpg
```

### 3. Ký khóa cục bộ

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```

### 4. Cấu hình kho ARGVUS

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```

Thêm cấu hình kho lưu trữ vào `/etc/pacman.conf`:

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```

Cấu hình kho lưu trữ kích hoạt các kho lưu trữ ARGVUS, bao gồm `argvus` và `argvus-extras`, với `SigLevel = Required TrustedOnly`. Pacman chỉ chấp nhận các gói khi chữ ký của chúng hợp lệ và được khóa cục bộ tin cậy.

### 5. Cài đặt ARGVUS

```sh
sudo pacman -Syu argvus
```

Sau khi cài đặt, hãy đăng xuất và chọn **ARGVUS** từ trình quản lý hiển thị của bạn.

## Cập nhật

Cập nhật ARGVUS cùng với phần còn lại của Arch Linux:

```sh
sudo pacman -Syu
```

Để chỉ cập nhật gói ARGVUS:

```sh
sudo pacman -S argvus
```

> **Khuyến nghị:** Arch Linux thường khuyến nghị nâng cấp toàn bộ hệ thống bằng `sudo pacman -Syu` thay vì cập nhật từng gói riêng lẻ.

## Xóa ARGVUS

Xóa ARGVUS trong khi vẫn giữ các phần phụ thuộc vẫn được yêu cầu bởi các gói đã cài đặt khác:

```sh
sudo pacman -R argvus
```

Xóa ARGVUS và các phần phụ thuộc không còn được yêu cầu bởi bất kỳ gói đã cài đặt nào:

```sh
sudo pacman -Rns argvus
```

### Xóa kho lưu trữ ARGVUS

Nếu bạn cũng muốn xóa kho lưu trữ gói, hãy xóa cấu hình của nó:

```sh
sudo rm /etc/pacman.d/argvus.conf
```

Sau đó xóa dòng tương ứng khỏi `/etc/pacman.conf`:

```text
Include = /etc/pacman.d/argvus.conf
```

Bạn có thể chỉnh sửa tập tin bằng:

```sh
sudo nano /etc/pacman.conf
```

### Xóa khóa ký

Để xóa khóa ký kho lưu trữ ARGVUS khỏi chuỗi khóa pacman, trước tiên hãy liệt kê các khóa:

```sh
sudo pacman-key --list
```

Sau đó xóa khóa ARGVUS bằng ID khóa của nó:

```sh
sudo pacman-key --delete KEY_ID
```

Thay thế `KEY_ID` bằng ID khóa ký ARGVUS.

> **Cảnh báo:** Chỉ xóa khóa này khi bạn cũng đang xóa kho lưu trữ ARGVUS. Không xóa các khóa vẫn được các kho lưu trữ khác yêu cầu.

## Cấu hình người dùng

Cài đặt gói không ghi cấu hình vào `$HOME`. ARGVUS đọc các giá trị mặc định được đóng gói từ `/usr/share/argvus` và hoạt động mà không cần sao chép chúng vào thư mục chính của bạn.

Cấu hình thời gian chạy tuân theo mức độ ưu tiên chung này:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```

`argvus-config` là thành phần duy nhất ghi `~/.config/argvus/data/generated/` và phép chiếu đó là lớp hoạt động: bản sao gốc hoặc được quản lý chỉ có hiệu lực trong khi không có tệp được tạo tương ứng.Cấu hình do ARGVUS quản lý có `~/.config/argvus/`. Ghi đè `$XDG_CONFIG_HOME/<app>` tùy chọn vẫn có sẵn cho các ứng dụng hỗ trợ cấu hình người dùng gốc.

Bộ điều phối `argvus --setup` là tùy chọn và dành cho hướng dẫn rõ ràng
ghi đè, không phục hồi. Nếu `~/.config/argvus` bị xóa, `SUPER + SHIFT + R`
tạo lại cấu hình chuẩn và trạng thái thời gian chạy được tạo thông qua thông thường
tải lại đường ống.

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```

`--copy-all` là danh sách cho phép rõ ràng của các tệp hồ sơ TUI được hỗ trợ; nó
không sao chép `/usr/share/argvus` và không sao chép tập lệnh, dịch vụ, nội dung,
các tệp được tạo hoặc `config.json`. Chạy `argvus --setup --help` để xem hiện tại
tùy chọn. Sau khi được sao chép, các tệp sẽ trở thành phần ghi đè và gói do người dùng sở hữu.
nâng cấp không thay thế chúng.

Để thay thế cấu hình đã sao chép hiện có, hãy sử dụng `--force`. ARGVUS tạo bản sao lưu có dấu thời gian trước khi thay thế nó:

```sh
argvus --setup --copy <app> --force
```

## Ghi đè người dùng Hyprland

Để biết hướng dẫn ghi đè Hyprland đầy đủ, bao gồm sự khác biệt giữa cấu hình gốc đầy đủ và các đoạn ARGVUS gia tăng, hãy xem [Ghi đè Hyprland](/docs/argvus-hyprland/hyprland-overrides/).

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```

### Dự phòng máy ảo

Một số máy ảo có thể gặp sự cố kết xuất với Hyprland, Kitty hoặc Quickshell. ARGVUS phát hiện các máy ảo sử dụng trình điều khiển đồ họa `vmwgfx` và cho phép kết xuất phần mềm khi cần thiết.

Nếu tính năng phát hiện không hoạt động trên máy ảo của bạn, hãy bật tính năng dự phòng theo cách thủ công trong `user.lua`:

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```

Sau khi tạo hoặc sửa đổi `user.lua`, hãy đăng xuất khỏi Hyprland và đăng nhập lại để biến được áp dụng cho các ứng dụng do phiên khởi chạy.

Xác minh nó với:

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```

Sản lượng dự kiến ​​là:

```text
1
```

> **Lưu ý:** `LIBGL_ALWAYS_SOFTWARE=1` buộc các ứng dụng Mesa/OpenGL sử dụng kết xuất phần mềm, thường thông qua `llvmpipe`. Nó có thể làm giảm hiệu suất đồ họa và nên được sử dụng làm phương án dự phòng cho các máy ảo bị ảnh hưởng.

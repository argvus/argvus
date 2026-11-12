---
title: Cài đặt chủ đề
description: Cài đặt gói chủ đề ARGVUS chính thức với pacman.
---

Các chủ đề chính thức được xuất bản dưới dạng gói Arch Linux. Mỗi chủ đề là gói riêng có tên `argvus-theme-<THEME_NAME>`.

## Nơi lấy chủ đề

Nguồn của mọi chủ đề chính thức đều có trong tổ chức dự án ARGVUS:

- Mỗi chủ đề là một kho lưu trữ có tên `argvus-theme-<THEME_NAME>` (ví dụ `argvus-theme-dracula`), được xây dựng dưới dạng gói có cùng tên.

Các gói chủ đề phụ thuộc vào `argvus-appearance`. Trước tiên hãy cài đặt ARGVUS, như được mô tả trong [Cài đặt](/docs/getting-started/installation/), để có thành phần ngoại hình.

## Cài đặt một chủ đề duy nhất

Để cài đặt mọi chủ đề chính thức cùng một lúc, hãy cài đặt siêu gói `argvus-themes`:

```bash
sudo pacman -S argvus-themes
```

Nó phụ thuộc vào tất cả các gói `argvus-theme-*` đã xuất bản. ARGVUS tối và ARGVUS
Đèn đã được `argvus-appearance` vận chuyển.

Cài đặt một chủ đề với tên gói của nó:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```

Ví dụ:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```

Gói cài đặt tệp kê khai chủ đề, `/usr/share/argvus/themes/<theme-id>/theme.toml` (tên, danh mục, hình nền và bảng màu, với `overrides/` tùy chọn) và hình nền của chủ đề trong `/usr/share/backgrounds/argvus/`. Các tệp của mọi thành phần được hiển thị từ bảng màu khi áp dụng chủ đề. Nó không ghi vào thư mục chính của bạn và nó không thay đổi chủ đề hiện tại của bạn. Sau đó, hãy chọn nó từ Trung tâm điều khiển, xem [Chủ đề chính thức](/docs/components/argvus-themes/official-themes/).

## Cài đặt một số chủ đề

Không có gói meta nào cài đặt mọi chủ đề. Cài đặt các chủ đề bạn muốn trong một lệnh:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## Các gói có sẵn

Các gói dưới đây là các gói chủ đề chính thức được xuất bản ngày hôm nay.

**Chủ đề tối**

| Trọn gói | Chủ đề |
| --- | --- |
| `argvus-theme-one-dark` | Một Bóng Tối |
| `argvus-theme-dracula` | Dracula |
| `argvus-theme-silver-dark` | Bạc tối |
| `argvus-theme-slate-dark` | Đá phiến tối |
| `argvus-theme-universe` | Vũ trụ |
| `argvus-theme-gruvbox-high-dark` | Gruvbox Cao Tối |
| `argvus-theme-gruvbox-dark` | Gruvbox tối |
| `argvus-theme-rose-pine` | Thông Rosé |
| `argvus-theme-tokyo-night` | Đêm Tokyo |
| `argvus-theme-solitude` | Cô đơn |
| `argvus-theme-sunset` | Hoàng hôn |
| `argvus-theme-matrix` | Ma trận |
| `argvus-theme-monokai-dark` | Monokai bóng tối |

**Chủ đề nhẹ nhàng**

| Trọn gói | Chủ đề |
| --- | --- |
| `argvus-theme-github-light` | Ánh sáng GitHub |
| `argvus-theme-solarized-light` | Ánh sáng mặt trời |
| `argvus-theme-nord-light` | Ánh sáng phương Bắc |
| `argvus-theme-one-light` | Một Ánh Sáng |
| `argvus-theme-everforest-light` | Ánh sáng Everforest |
| `argvus-theme-catppuccin-latte` | Latte Catppuccin |
| `argvus-theme-gruvbox-light` | Đèn Gruvbox |
| `argvus-theme-watercolor` | Màu nước |

Các chủ đề **ARGVUS Dark** và **ARGVUS Light** tích hợp sẵn không cần gói riêng; chúng đi kèm với `argvus-appearance`.

## Kiểm tra những gì đã được cài đặt

Liệt kê các gói chủ đề đã cài đặt với:

```bash
pacman -Qs argvus-theme
```

Mở Trung tâm điều khiển và đi tới **Giao diện → Chủ đề → Chủ đề chính thức** để xem các chủ đề hiện có sẵn trong bộ chọn. Một chủ đề đã được cài đặt nhưng bị thiếu trong danh sách đó là vấn đề về đóng gói; xem phần khắc phục sự cố của [Hướng dẫn chủ đề](/docs/components/argvus-themes/).

## Thêm chủ đề của riêng bạn

Chủ đề là một bản kê khai, vì vậy chủ đề của riêng bạn không cần gói. Tạo
`~/.local/share/argvus/themes/<id>/theme.toml`, trong đó tên thư mục là
ID chủ đề (chữ thường, chữ số và dấu gạch nối đơn):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```

Chỉ yêu cầu `bg`, `fg` và `accent`: mọi màu khác đều có màu mặc định
bắt nguồn từ chúng và bảng kê khai chỉ định nghĩa một cái để thay đổi nó. Kiểm tra nó,
sau đó áp dụng nó như bất kỳ chủ đề nào khác:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```

`argvus theme list` hiển thị chủ đề của bạn bên cạnh chủ đề chính thức (chính thức
chủ đề có cùng ID sẽ thắng). Chủ đề của bạn xuất hiện trong menu chủ đề
(`SUPER + Shift + T`); lời chào đăng nhập trước không thể đọc thư mục chính của bạn, vì vậy
nó giữ chủ đề chính thức cuối cùng mà bạn áp dụng trong khi một trong các chủ đề của bạn đang hoạt động.

Các bản sao theo chủ đề mà các phiên bản ARGVUS cũ hơn đã viết theo`~/.config/argvus/data/<app>/themes/` không còn được đọc. Chuyển đổi chủ đề đầu tiên
sau khi cập nhật sẽ chuyển chúng sang `~/.config/argvus/data/backups/theme-copies-<time>/`;
để giữ lại tệp bạn đã chỉnh sửa, hãy đặt tệp đó vào chủ đề của riêng bạn trong `overrides/` và
đặt tên nó trong bảng `[overrides]` của tệp kê khai.

## Xóa chủ đề

Xóa một gói chủ đề duy nhất với:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```

Việc xóa gói chủ đề sẽ không xóa các hồ sơ đã nhập của bạn. Nếu chủ đề đã xóa là chủ đề hiện đang được áp dụng, hãy chọn một chủ đề khác từ Trung tâm điều khiển trước khi xóa chủ đề đó.

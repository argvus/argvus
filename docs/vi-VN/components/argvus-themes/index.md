---
title: Chủ đề
description: Cài đặt, chọn, tùy chỉnh, nhập và xuất các chủ đề ARGVUS.
---

Chủ đề trong ARGVUS đặt màu sắc, bề mặt, mặc định hình nền và hình học bố cục được sử dụng trên màn hình: Hyprland, Waybar, Control Panel, thanh tác vụ, màn hình khóa, thông báo và ứng dụng ARGVUS. Hướng dẫn này bao gồm mọi thứ bạn cần để làm việc với chủ đề với tư cách là người dùng.

## Những gì có trong một chủ đề

Mỗi chủ đề chính thức là một bản sắc trực quan hoàn chỉnh với hai chế độ:

- **Dính** — bố cục nhỏ gọn với khoảng trống cửa sổ nhỏ, không có khoảng trống bên ngoài và thanh tác vụ nằm sát mép màn hình.
- **Giãn cách** — bố cục rộng rãi với các khoảng trống lớn hơn, các góc và lề được bo tròn xung quanh thanh tác vụ và bề mặt vỏ.

Việc chọn một chủ đề sẽ không làm thay đổi nhận dạng của nó: bảng màu, màu nổi bật mặc định và liên kết hình nền vẫn giữ nguyên giữa các chế độ. Chế độ này là một cài đặt riêng biệt, xem [Chủ đề chính thức](/docs/components/argvus-themes/official-themes/).

## Các trang trong hướng dẫn này

- [Cài đặt chủ đề](/docs/components/argvus-themes/installing-themes/) — nơi lấy các chủ đề chính thức và cách cài đặt một hoặc tất cả chúng với `pacman`.
- [Chủ đề chính thức](/docs/components/argvus-themes/official-themes/) — danh mục, màu nổi bật mặc định của từng chủ đề cũng như cách chọn chủ đề và chế độ của chủ đề đó.
- [Tùy chỉnh màu nổi bật](/docs/components/argvus-themes/highlight-color/) — thay đổi màu nhấn độc lập với chủ đề.
- [Nhập và xuất chủ đề](/docs/components/argvus-themes/import-and-export/) — xuất chủ đề đang hoạt động, với màu nổi bật và hình nền của bạn, làm chủ đề của riêng bạn và nhập chủ đề đó vào một máy khác.

## Tham khảo nhanh

| Nhiệm vụ | Ở đâu |
| --- | --- |
| Chọn một chủ đề | **Trung tâm điều khiển → Giao diện → Chủ đề → Chủ đề chính thức** |
| Chuyển dính và cách nhau | **Trung tâm điều khiển → Giao diện → Chế độ** |
| Thay đổi màu đánh dấu | **Trung tâm điều khiển → Giao diện → Màu nổi bật** |
| Xuất chủ đề đang hoạt động | **Trung tâm điều khiển → Giao diện → Chủ đề → Xuất chủ đề** |
| Nhập chủ đề | **Trung tâm điều khiển → Giao diện → Chủ đề → Nhập chủ đề** |
| Mở bộ chọn chủ đề từ bàn phím | `SUPER + SHIFT + T` |

## Chủ đề đến từ đâu

ARGVUS cung cấp hai chủ đề tích hợp sẵn, **ARGVUS Dark** và **ARGVUS Light**, với gói `argvus-appearance`. Mọi chủ đề chính thức khác là một gói `argvus-theme-<name>` riêng biệt. Việc cài đặt gói chủ đề sẽ thêm gói đó vào bộ chọn chủ đề; không cần khởi động lại phiên. Chủ đề của riêng bạn, được xuất hoặc nhập, được liệt kê riêng dưới dạng **Chủ đề tùy chỉnh**.

## Gợi ý bàn phím

Các trang chủ đề của Trung tâm điều khiển hiển thị các phím của chúng ở chân trang:

- `↑` / `↓` — điều hướng.
- `Enter` — áp dụng hoặc mở.
- `e` — xuất chủ đề đang hoạt động trên trang Chủ đề.
- `i` — nhập chủ đề trên trang Chủ đề.
- `d` — xóa chủ đề tùy chỉnh.
- `Esc` — quay lại.

## Khắc phục sự cố

- **Chủ đề tôi đã cài đặt không xuất hiện.** Xác nhận rằng gói đã được cài đặt với `pacman -Q argvus-theme-<name>`, sau đó mở lại Trung tâm điều khiển. Gói phải được cài đặt có `argvus-appearance`.
- **Màu đánh dấu đã thay đổi sau khi tôi chuyển đổi chủ đề.** Màu đánh dấu tùy chỉnh vẫn tồn tại khi chuyển đổi chủ đề. Sử dụng **Đặt lại về mặc định của chủ đề** để trả lại màu sắc riêng cho chủ đề.
- **Kho lưu trữ chủ đề không được liệt kê để nhập.** Nó phải nằm ngay trong thư mục chính của bạn và chứa `<id>/theme.toml`. Xem [Nhập và xuất chủ đề](/docs/components/argvus-themes/import-and-export/).

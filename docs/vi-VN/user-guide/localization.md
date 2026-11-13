---
title: Bản địa hóa
description: Sử dụng ARGVUS bằng tiếng Anh hoặc tiếng Bồ Đào Nha.
---

`argvus-i18n` cung cấp danh mục chia sẻ, `argvus i18n validate` lệnh và lệnh QML module `org.argvus.i18n`. ARGVUS Tàu 18 ngôn ngữ giao diện: Ả Rập, Bengali, Đức, Anh, Tây Ban Nha, Pháp, Ấn Độ, Indonesia, Ý, Nhật Bản, Hàn Quốc, Bồ Đào Nha Châu Âu, Nga, Thổ Nhĩ Kỳ, Urdu, Việt Nam và Simplified.

Sở hữu ARGVUS ứng dụng và trình bao tiêu thụ danh mục chung. Do đó, một chuỗi được dịch có thể được dùng bởi hơn một kho. A Rập và Urdu dùng bố trí phải- trái trong hỗ trợ GTK. Quickshell và giao diện thiết bị cuối.

Dùng ** Trung tâm điều khiển (Tuỳ thích)   Ngôn ngữ & Vùng để chọn ARGVUS Ngôn ngữ giao diện. Ghi `ARGVUS_LANGUAGE` phải `$XDG_CONFIG_HOME/locale.conf` (bình thường `~/.config/locale.conf`). Ví dụ, `ARGVUS_LANGUAGE=pt-PT` chọn người Bồ Đào Nha Châu Âu. Bồ Đào Nha Brazil (`pt-BR`) là một lựa chọn khác. Danh mục ngôn ngữ đã được cài đặt ARGVUSĐể nó không phụ thuộc vào `locale-gen`.

Bảng điều khiển tự động khởi chạy lại sau khi ngôn ngữ giao diện thay đổi. Khởi động một phiên chạy mới để áp dụng nó vào thành phần phiên chạy. Các thiết lập riêng biệt **Regional locale** stlomp `LANG` và điều khiển định dạng khu vực và ứng dụng bên thứ ba. Nó yêu cầu một hệ thống tạo ra địa phương; cho phép một trong `/etc/locale.gen` chạy `locale-gen` có quyền quản trị. Name

# Khi môi trường thay đổi #

ARGVUS ứng dụng đọc `ARGVUS_LANGUAGE` từ người dùng `locale.conf`Nếu vắng mặt, họ sẽ giải quyết khu vực `LANG` giá trị cho mục lục đã cài đặt. Biến số cục bộ không ghi đè lên một điều rõ ràng `ARGVUS_LANGUAGE` Thích hơn. Ứng dụng bên thứ ba tiếp tục sử dụng khu vực khu vực.

Xem xét các giá trị hiệu quả với:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

Kiểm tra xem `~/.config/locale.conf` chứa một hợp lệ `ARGVUS_LANGUAGE=<catalog>` giá trị. Để thay đổi định dạng vùng hoặc ngôn ngữ ứng dụng bên thứ ba, hãy dùng **Regional locale** và chọn một địa phương đã tạo ra. Bảng điều khiển được khởi chạy lại sau khi cả hai tùy thích thay đổi; đăng xuất và trong nếu các thành phần phiên chạy khác vẫn hiển thị ngôn ngữ giao diện cũ. Xem [các biến thay đổi]/docs/reference/environment-variablesCho các hợp đồng kỹ thuật và ưu tiên.

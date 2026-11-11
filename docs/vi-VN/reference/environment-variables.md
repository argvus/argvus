---
title: Biến môi trường
description: Hợp đồng môi trường được sử dụng bởi các thành phần phiên ARGVUS.
---

Phiên thiết lập tích hợp môi trường XDG và Wayland và có thể xuất các biến tương thích ảo hóa khi được phát hiện. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG` và `ARGVUS_BOOTSTRAP` được sử dụng bởi người trợ giúp phiên và người điều phối để giải quyết gốc cấu hình.

Coi các biến môi trường như hợp đồng thời gian chạy: xác minh nguồn hoặc `--help` của trình trợ giúp đã cài đặt trước khi thêm ghi đè liên tục.

Trạng thái dẫn xuất tồn tại dưới gốc trạng thái chứ không phải gốc cấu hình.
`XDG_STATE_HOME` (thường là `~/.local/state`) nhận `argvus/config-projection.json`,
bản kê khai phép chiếu `argvus-config` được xây dựng lại và `argvus sessionctl reload`
áp dụng, cộng với `argvus/session.log`. Cả hai đều không phải là nguồn gốc của sự thật.

## Môi trường ngôn ngữ

Tùy chọn ngôn ngữ chung duy nhất là `LANG` trong `$XDG_CONFIG_HOME/locale.conf` (`~/.config/locale.conf` mặc định). Nếu không có giá trị hợp lệ, ARGVUS chọn tiếng Anh (`en-US`), ngay cả khi `/etc/locale.conf`, PAM hoặc trình quản lý đăng nhập cung cấp tiếng Bồ Đào Nha. Các giá trị như `pt_BR.UTF-8` chuẩn hóa thành `pt-BR` cho danh mục.

Phiên áp dụng tùy chọn này trước khi khởi động các thành phần máy tính để bàn và nhập người dùng systemd và môi trường kích hoạt D-Bus. Nó thay thế `LANG`, `LC_MESSAGES`, `LANGUAGE` được kế thừa và xóa `LC_ALL`; cài đặt `LC_*` khu vực trong cùng một tệp được giữ nguyên. Tiếng Anh sử dụng `en_US.UTF-8` khi được tạo, nếu không thì `C.UTF-8`. Không có tập tin ưu tiên nào được tạo tự động. Đăng xuất và đăng nhập lại sau khi thay đổi ngôn ngữ.

`session.language`, `argvus/data/language` và `argvus/data/internal/language` bị bỏ qua dữ liệu cũ. Chúng không được nhập khẩu cũng như không được tái tạo; các tập tin hiện có được bảo tồn. `argvus config set` và `patch` từ chối ghi mới vào `session.language` và chuyển người gọi tới `locale.conf`.

Để chẩn đoán, hãy so sánh `argvus i18n locale`, `cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` và `systemctl --user show-environment`. `localectl status` mô tả hệ thống chứ không phải ngôn ngữ ARGVUS của người dùng. `ARGVUS_I18N_DIR` chọn thư mục danh mục để phát triển chứ không phải ngôn ngữ.
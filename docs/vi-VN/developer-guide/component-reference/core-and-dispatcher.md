---
title: Lõi và bộ điều phối
description: Không gian làm việc của argvus, bộ điều phối của nó và các thùng Rust được chia sẻ.
---

`argvus` là không gian làm việc Hàng hóa tại `de/argvus` và gói cung cấp `/usr/bin/argvus`. Thùng `argvus-cli` là bộ điều phối: nó đọc đối số đầu tiên và chạy lệnh thư viện trong quá trình xử lý, chạy trình trợ giúp GTK 4 hoặc tệp thực thi riêng tư hoặc chạy tập lệnh đóng gói. Danh sách lệnh con đầy đủ nằm trong [tham chiếu dòng lệnh](/docs/reference/command-line/).

Không gian làm việc cũng chứa các thùng mà các lệnh chia sẻ:

- `argvus-config`: kho cấu hình chuẩn và là tác giả duy nhất của cây được tạo ra (`data/generated/`). Nó cũng là một thư viện được sử dụng trong quá trình bởi `argvus config`, `theme-switch` và `effects-toggle`.
- `argvus-i18n` và `argvus-i18n-core`: các danh mục trong `resources/locales/`, xác thực của chúng và mô-đun QML `org.argvus.i18n`.
- `argvus-tui` và `argvus-theme`: tải giao diện và giao diện thiết bị đầu cuối dùng chung.
- `argvus-runtime`: các thông báo đã dịch và các trình trợ giúp quy trình được chia sẻ bởi các lệnh.

`argvus-session` vẫn là một kho lưu trữ riêng biệt. Nó cung cấp các điểm vào phiên (`session`, `start`, `tty`) và các đơn vị vòng đời, đồng thời `argvus sessionctl` tiếp cận bộ điều khiển của nó.
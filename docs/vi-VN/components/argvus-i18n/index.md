---
title: ARGVUS Quốc tế hóa
description: Danh mục ngôn ngữ, lựa chọn và hỗ trợ Unicode.
---

# Quốc tế hóa ARGVUS

ARGVUS giao hàng bằng tiếng Anh (`en-US`), tiếng Bồ Đào Nha Brazil (`pt-BR`), tiếng Tây Ban Nha (`es-ES`),
Tiếng Pháp (`fr-FR`), Tiếng Ý (`it-IT`), Tiếng Đức (`de-DE`), Tiếng Nga (`ru-RU`), Đơn giản hóa
Tiếng Trung (`zh-CN`), tiếng Nhật (`ja-JP`) và tiếng Hàn (`ko-KR`). Tiếng Anh là kinh điển
danh mục và dự phòng. Tám bản dịch bổ sung được thực hiện bằng máy
hỗ trợ dịch thuật và chỉnh sửa kỹ thuật; đánh giá của người bản xứ độc lập
chưa được chứng nhận.

## Chọn ngôn ngữ

Mở Trung tâm điều khiển > Tùy chọn > Ngôn ngữ & Khu vực > Ngôn ngữ hoặc Cài đặt >
Ngôn ngữ. Trang này khám phá các danh mục đã cài đặt và hiển thị tên gốc của chúng.
Nó lưu `LANG` trong `$XDG_CONFIG_HOME/locale.conf` (`~/.config/locale.conf` mặc định).
Trước tiên, hãy bật ngôn ngữ hệ thống UTF-8 tương ứng trong Ngôn ngữ hệ thống. Ví dụ
bao gồm `es_ES.UTF-8`, `zh_CN.UTF-8`, `ja_JP.UTF-8` và `ko_KR.UTF-8`.

Tùy chọn không sửa đổi `/etc/locale.conf`. Bình luận và thể loại khu vực
cài đặt được bảo tồn. Bảng điều khiển được khởi động lại sau khi lưu; đăng xuất và
quay lại để áp dụng ngôn ngữ trong suốt phiên. Người dùng chưa được định cấu hình sử dụng
Tiếng Anh. Các biến môi trường được kế thừa và các tệp ngôn ngữ ARGVUS kế thừa không
thay thế tùy chọn này. Lời chào đăng nhập chạy trước tùy chọn của người dùng
có sẵn và giữ lại ngôn ngữ tài khoản dịch vụ của riêng mình, tiếng Anh theo mặc định.

Các biến thể khu vực sử dụng danh mục đã cài đặt cho ngôn ngữ của chúng khi có thông tin chính xác
danh mục vắng mặt; ví dụ `es_MX.UTF-8` sử dụng `es-ES`. `zh-Hans` chọn
Tiếng Trung giản thể. Tiếng Trung phồn thể (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) là
không được ánh xạ âm thầm sang tiếng Trung giản thể và quay trở lại tiếng Anh.

## Danh mục và đóng góp

Nguồn là `resources/locales/<locale>/<domain>.json` trong kho `argvus`.
Danh mục đã cài đặt trực tiếp trong `/usr/share/argvus/i18n/`. Họ gửi hàng chính
Gói `argvus`. `locale.json` cung cấp mã định danh và tên gốc/tên hiển thị.
Không có danh mục PO riêng hoặc nền tảng dịch thuật trực tuyến bắt buộc.

Sử dụng các khóa ngữ nghĩa và giữ nguyên tên giữ chỗ, lệnh, tùy chọn, đường dẫn và
phím tắt. Dịch tin nhắn hoàn chỉnh thay vì ghép câu
từ các đoạn đã dịch. Khóa kinh điển mới cần dịch cả 10 khóa
ngôn ngữ. Giữ thuật ngữ nhất quán trong từng miền và xem xét bản dịch
trong ngữ cảnh, đặc biệt là xác thực, quyền và hành động phá hoại.

```sh
argvus i18n list
argvus i18n locale
argvus i18n get common welcome name=Alice
argvus i18n validate
```

`ARGVUS_I18N_DIR` chọn gốc danh mục thay thế để phát triển và dàn dựng
kiểm tra gói. Xác thực kiểm tra tên miền, khóa, giá trị chuỗi, phần giữ chỗ,
bản dịch trống và dạng số nhiều số nguyên dành riêng cho miền địa phương. Xác nhận chứng minh
tính toàn vẹn về cấu trúc chứ không phải độ chính xác về mặt ngôn ngữ.

## Unicode và phông chữ

`argvus-fonts` yêu cầu phông chữ Noto CJK ngoài tiếng Latin, biểu tượng và biểu tượng cảm xúc
các gia đình. Fontconfig cung cấp dự phòng cho tiếng Trung, tiếng Nhật và Hangul mà không cần
thay thế họ chính đã được người dùng chọn. Các ứng dụng và thiết bị đầu cuối vẫn
cần bật tính năng dự phòng phông chữ.

Trình trợ giúp văn bản Ratatui được chia sẻ đo các ô đầu cuối, bảo toàn biểu đồ mở rộng
cụm trong quá trình cắt bớt và bao bọc văn bản rộng. Trình soạn thảo văn bản di chuyển và xóa
toàn bộ đồ thị. Các khu vực cực kỳ hẹp không thể hiển thị glyph hai ô trong một
tế bào; trình trợ giúp bọc cứng sử dụng ký tự thay thế trong trường hợp đó. Biểu tượng cảm xúc và
các ký tự có độ rộng không rõ ràng vẫn có thể khác nhau giữa việc triển khai thiết bị đầu cuối/phông chữ.
Chính sách chiều rộng của thiết bị đầu cuối phải phù hợp với chiều rộng Unicode thông thường của Ratatui.

Lựa chọn ngôn ngữ không cài đặt phương thức nhập liệu hoặc thay đổi bố cục bàn phím.Sử dụng phương thức nhập tương thích với Wayland khi nhập tiếng Trung, tiếng Nhật hoặc tiếng Hàn.

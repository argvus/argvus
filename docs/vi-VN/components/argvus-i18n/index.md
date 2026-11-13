---
title: ARGVUS Quốc tế hóa
description: Danh mục ngôn ngữ, lựa chọn và hỗ trợ Unicode.
---

# Quốc tế hóa ARGVUS

ARGVUS tàu 18 giao diện ngôn ngữ: Ả Rập (`ar`), Ben-ga-li (`bn-BD`), Đức (`de-DE`)
Tiếng Anh`en-US`), Tây Ban Nha (`es-ES`), Pháp (`fr-FR`), Ấn Độ (`hi-IN`), Indonesia
(Tiếng cười)`id-ID`), Ý (Tiếng Ý)`it-IT`), tiếng Nhật (tiếng Nhật)`ja-JP`), Hàn Quốc (`ko-KR`), Bồ Đào Nha Brazil
(Tiếng cười)`pt-BR`), Bồ Đào Nha Châu Âu (`pt-PT`), Nga (`ru-RU`), Thổ Nhĩ Kỳ (`tr-TR`), Urdu
(Tiếng cười)`ur-PK`), Việt Nam (`vi-VN`) và Simplified Trung Hoa (`zh-CN`). Tiếng Anh là
danh mục chính thống và hậu phương. Các bản dịch được xuất bản với AI Trợ giúp và
Việc sửa chữa kỹ thuật; việc xem xét ngôn ngữ bản địa độc lập chưa được chứng nhận.

## Chọn ngôn ngữ

Trung tâm Điều khiển Mở > Tùy thích > Ngôn ngữ & Vùng > Ngôn ngữ Trang web phát hiện ra
đã cài đặt mục lục và cho thấy tên của họ. Nó tiết kiệm `ARGVUS_LANGUAGE` in
`$XDG_CONFIG_HOME/locale.conf` (mặc định `~/.config/locale.conf`), độc lập
`LANG`; tạo ra địa phương hệ thống không cần thiết. Bồ Đào Nha Brazil (`pt-BR`) và
Bồ Đào Nha Châu Âu (`pt-PT`) là những lựa chọn khác nhau.
Tùy thích không sửa đổi `/etc/locale.conf` hay vùng `LANG` cài đặt.
Ghi chú và thiết lập phân loại khu vực được bảo tồn. Name
sau khi lưu; bắt đầu một phiên chạy mới để áp dụng ngôn ngữ trong suốt các thành phần phiên chạy.
Người dùng không cấu hình sử dụng tiếng Anh. Name
`LANG`Định dạng khu vực và các ứng dụng bên thứ ba. Trình chào mừng đăng nhập chạy
Trước khi tùy thích của người dùng là có sẵn và giữ lại dịch vụ của riêng mình-ac Count
Ngôn ngữ, tiếng Anh theo mặc định.
Các biến thể vùng dùng danh mục đã cài đặt cho ngôn ngữ của họ khi chính xác
catalog vắng mặt; chẳng hạn `es_MX.UTF-8` sử dụng `es-ES`. `zh-Hans` lựa chọn
Truyền thống Trung Quốc. Truyền thống Trung Quốc (`zh-Hant`. `zh-TW`. `zh-HK`. `zh-MO`) là
không âm thầm lập bản đồ Trung Quốc và trở lại tiếng Anh.

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

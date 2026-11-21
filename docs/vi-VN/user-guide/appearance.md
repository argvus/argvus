---
title: Ngoại hình
description: Định cấu hình ngôn ngữ hình ảnh được chia sẻ bởi các thành phần ARGVUS.
---

Hệ thống giao diện áp dụng một trạng thái chủ đề cho các ứng dụng Hyprland, GTK, Qt, Waybar, Quickshell, Dunst, Hyprlock và ARGVUS.

## Nhóm chủ đề

Hai mươi ba họ bên dưới được xác định theo tải trọng chủ đề hiện tại. Cột màu là màu viền/dấu nhấn mặc định của dòng; người dùng có thể chọn một giọng RGB hợp lệ khác.

| Gia Đình | Đặt lại màu nổi bật | Nền tảng | Chế độ |
| --- | --- | --- | --- |
| Một Bóng Tối | `#61AFEF` | `#282C34` | Dính, cách nhau |
| Dracula | `#BD93F9` | `#282A36` | Dính, cách nhau |
| ARGVUS tối | `#3590bd` | `#111316` | Dính, cách nhau |
| Bạc đậm | `#595959` | `#111316` | Dính, cách nhau |
| Đá đen | `#7391a5` | `#2f3541` | Dính, cách nhau |
| Vũ trụ bóng tối | `#eeeeee` | `#000000` | Dính, cách nhau |
| ARGVUS Dark Gruvbox Cao | `#D79921` | `#282828` | Dính, cách nhau |
| ARGVUS Dark Gruvbox | `#D4BE98` | `#282828` | Dính, cách nhau |
| Ánh sáng ARGVUS | `#181818` | `#f7f7f7` | Dính, cách nhau |
| Ánh sáng GitHub | `#0969DA` | `#FFFFFF` | Dính, cách nhau |
| Latte Catppuccin | `#1E66F5` | `#EFF1F5` | Dính, cách nhau |
| ARGVUS Light Gruvbox | `#458588` | `#FBF1C7` | Dính, cách nhau |
| Thông Rosé | `#C4A7E7` | `#191724` | Dính, cách nhau |
| Tokyo-Đêm | `#7AA2F7` | `#1A1B26` | Dính, cách nhau |
| Cô đơn | `#798186` | `#101315` | Dính, cách nhau |
| Hoàng hôn | `#E2BE8A` | `#0F0F0F` | Dính, cách nhau |
| Ma trận | `#82FB9C` | `#0B0C16` | Dính, cách nhau |
| Monokai bóng tối | `#78DCE8` | `#2D2A2E` | Dính, cách nhau |
| Cô Sương | `#354B78` | `#131313` | Dính, cách nhau |
| Một Ánh Sáng | `#4078F2` | `#FAFAFA` | Dính, cách nhau |
| Ánh sáng Everforest | `#3A94C5` | `#FDF6E3` | Dính, cách nhau |
| Ánh sáng mặt trời | `#268BD2` | `#FDF6E3` | Dính, cách nhau |
| Ánh sáng phương Bắc | `#5E81AC` | `#ECEFF4` | Dính, cách nhau |
| Màu nước | `#1A639C` | `#E9E3D3` | Dính, cách nhau |

Màu hiển thị trong bảng là giá trị được ghi vào `config.json` khi họ đó được chọn hoặc khi sử dụng **Đặt lại về mặc định chủ đề**. Dấu RGB gồm sáu chữ số được chọn thủ công sẽ áp dụng cho đến khi chuyển đổi chủ đề tiếp theo, thay thế bằng mặc định của chủ đề đã chọn.

**Dính** là bố cục nhỏ gọn: các cửa sổ lát gạch sử dụng các khoảng trống nhỏ bên trong, các khoảng trống bên ngoài bằng 0 và thanh tác vụ nằm sát mép màn hình. **Spaced** thêm các khoảng trống lớn hơn, các góc tròn, bóng và lề xung quanh thanh tác vụ và bề mặt vỏ.

Các giá trị bố cục `float` hiện có và số nhận dạng chủ đề `-float` được nhận dạng là tên cũ cho Spaced. Các giá trị bố cục mới và số nhận dạng chủ đề đã giải quyết sử dụng `spaced` và `-spaced`.

Chế độ thay đổi hình học và xử lý bề mặt; nó không tạo ra họ màu thứ sáu. Xem [Chủ đề](/docs/user-guide/appearance/) để lựa chọn và [Màn hình nền](/docs/user-guide/desktop/) để biết vị trí của thanh tác vụ và cửa sổ.

- [Chủ đề](/docs/user-guide/appearance/)
- [Hình nền](/docs/argvus-appearance/wallpapers/)
- [Phông chữ và biểu tượng](/docs/argvus-appearance/fonts-and-icons/)
- [Hiệu ứng](/docs/argvus-appearance/effects/)

Trung tâm điều khiển hiển thị các cài đặt hướng tới người dùng. Việc triển khai được chia sẻ bởi `argvus-appearance`, `argvus-session`, các thành phần shell và gói tài sản. Xem [kiến trúc ngoại hình](/docs/developer-guide/subsystems/appearance-and-themes/).

## Cửa sổ chao đảo

Mở **Desktop & input → Hyprland → Animations → Windows** và sử dụng
Điều khiển **Lắc lư**. Tìm kiếm `wobble` hoặc mở trang trực tiếp:

```sh
argvus control-center hyprland animations windows
```

Wobble bị tắt theo mặc định. Giá trị mặc định ARGVUS của nó là độ cứng `100`, giảm chấn`10`, khối lượng `1`, cường độ `1` và độ phân giải lưới `8`. Độ cứng và khối lượng chấp nhận
`0.0001–1000`, giảm chấn `0–1000`, cường độ `0–2` và chia lưới một số nguyên `2–32`.
Độ phân giải lưới cao hơn làm tăng chi phí kết xuất. Sử dụng các phím mũi tên để điều chỉnh
giá trị hoặc Enter để nhập chúng. **Khôi phục mặc định Wobble** thay đổi bản nháp;
**Áp dụng** lưu tất cả các chỉnh sửa hoạt ảnh đang chờ xử lý và tải lại Hyprland một lần mà không cần
khởi động lại phiên. Việc quay lại trong Ảnh động sẽ giữ lại bản nháp được chia sẻ.

Công tắc Ảnh động toàn cầu sẽ ngăn chặn sự rung chuyển trong khi vẫn giữ nguyên trạng thái riêng lẻ của nó.
cài đặt. Các giá trị chuẩn nằm dưới `effects.wobble_*` ở trạng thái ARGVUS;
`hypr/state.lua` được tạo và không được chỉnh sửa thủ công.

Điều này yêu cầu Hyprland **0.57.0 hoặc mới hơn**, được thực thi bởi `argvus-hyprland`'s
phụ thuộc gói. Phiên bản 0.56 không được hỗ trợ cho cấu hình này. các
cường độ tối đa theo[phát hành mã nguồn 0.57.0](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp),
giới hạn ở `2`, mặc dù tài liệu mới hơn có thể liệt kê `3`.

## Bộ chọn chủ đề trực quan

Mở **ARGVUS Theme Selector** (`argvus theme-selector`), `argvus theme-switch` không có đối số hoặc `SUPER + Shift + T`. Lớp phủ GTK độc lập hiển thị trên màn hình đang hoạt động với bản xem trước ở giữa và các thẻ lân cận. Tìm kiếm hoặc lọc Tất cả/Tối/Sáng. Dùng phím mũi tên, con lăn hoặc thẻ bên; Enter áp dụng, Escape đóng. Duyệt không thay đổi màn hình nền. Chủ đề hệ thống và người dùng dùng chung danh mục hiện có. Ưu tiên ảnh `preview` hợp lệ; nếu không có, hình nền và bảng màu tạo bản minh họa, không tái hiện chính xác tùy chỉnh ứng dụng. Quy trình hiện có giữ Sticky/Spaced và cho phép thử lại khi lỗi. Ảnh thu nhỏ JXL cần `djxl` từ `libjxl`; bộ nhớ đệm XDG nằm tại `argvus/theme-selector/v2`.

Khi mở bộ chọn, ô tìm kiếm tự động nhận tiêu điểm để bạn có thể gõ ngay.

Ô tìm kiếm hỗ trợ các ký tự có dấu từ bố cục bàn phím đang dùng. Khi ô tìm kiếm có tiêu điểm, dùng phím mũi tên trái và phải để duyệt các kết quả đã lọc.

Khi mở bộ chọn, tiêu điểm ban đầu nằm ở phần điều hướng băng chuyền. Nhấn `/` để lấy hoặc bỏ tiêu điểm khỏi ô tìm kiếm; nhập liệu sẽ lọc giao diện khi ô tìm kiếm đang được chọn. **Tùy chỉnh** liệt kê giao diện do người dùng cài đặt; **Tối** và **Sáng** liệt kê giao diện hệ thống theo từng loại. Hàng tìm kiếm rộng bằng phần xem trước ở giữa. Sticky dùng góc vuông trên toàn giao diện; Spaced giữ góc bo tròn. Kiểu chữ đơn cách gọn gàng đồng bộ với các tiện ích ARGVUS.

Khi áp dụng, bản xem trước ở giữa tối đi và hiển thị biểu tượng tải. Tiêu điểm tìm kiếm dùng màu nhấn hiện tại; bộ lọc có một đường viền.

**ARGVUS Layout Mode** (`argvus layout-mode-menu`) (SUPER + Shift + M). Chọn chế độ bố cục, cách sắp xếp cửa sổ và vị trí thanh tác vụ. **Tab**: Sticky / Spaced ⇄ Tile / Floating · Top / Bottom. **↑ / ↓ / ← / →**: Chọn. **Enter**: Áp dụng. **Esc**: Đóng.

Bộ chọn giao diện và chế độ hiển thị tiến trình trong cửa sổ của mình, không có màn hình chờ phiên. Vị trí cuối của thanh tác vụ được chuẩn bị trước khi tải lại; các lần khởi động lại cần thiết được gộp trong một giao dịch systemd.

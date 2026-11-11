---
title: Configuration and state
description: Trạng thái chia sẻ, ghi đè và tạo các tệp thời gian chạy.
---

Root người dùng bình thường là `$XDG_CONFIG_HOME/argvus` (thường là `~/.config/argvus`). Tùy chọn logic được quản lý được lưu trữ trong `config.json`. Các tùy chọn về chủ đề, điểm nhấn, hình nền, khoảng cách và đường viền đều chuẩn ở đó; các tệp dấu chấm vẫn duy trì các dự đoán tương thích bắt nguồn cho Hyprland Lua và người dùng shell hiện tại trong khi những người tiêu dùng đó chuyển sang `data/`.

`argvus-config` là người viết duy nhất dưới `generated/`. Mọi thuộc sở hữu của ARGVUS
tệp người tiêu dùng là hình chiếu của `config.json`, bao gồm Hyprland, GTK, Qt6ct,
Waybar, Quickshell, Dunst, Hyprlock, thanh tác vụ Waybar và phép đo từ xa widget
profile, cây cấu hình Yazi và Superfile, thiết bị đầu cuối và hồ sơ Qt
cây, các khối được quản lý phông chữ và biểu định kiểu thiết bị di động. Linh kiện
giải quyết các ghi đè rõ ràng của người dùng, sau đó tạo đầu ra, sau đó mặc định gói. Rofi không phải là một phép chiếu: `argvus-launcher` sở hữu các tệp của nó và người dùng sao chép trực tiếp dưới `data/rofi/`.
Trình chiếu chủ đề/tải lại thông thường không bao giờ quảng bá các tệp được tạo hoặc đóng gói
thành phần ghi đè của người dùng. Tài liệu và công cụ phải xác định nguồn hợp lý
thay vì yêu cầu người dùng chỉnh sửa kết quả đã tạo.

Do đó, `argvus-appearance` sở hữu khả năng phối hợp theo chủ đề và giọng điệu, đồng thời
`argvus-config` sở hữu trạng thái chuẩn và phép chiếu có cấu trúc.
`argvus theme-switch` (giao dịch `argvus config theme-apply`) và `argvus config accent` xác thực, khóa và cam kết thông qua
`argvus-config` khi có sẵn, sau đó điều chỉnh những người tiêu dùng tương thích
không thể đọc trực tiếp `config.json`. Những bộ điều hợp đó là
tệp Qt6ct gốc, gợi ý GTK `settings.ini` và `gsettings`, thiết bị đầu cuối và
ứng dụng giám sát hệ thống, foot, cấu hình riêng của superfile
thư mục và lời chào xác thực trước. Người trợ giúp cần bộ chuyển đổi mới
phải thêm nó vào đó, không bao giờ được thêm vào `generated/`.

Quyền sở hữu thời gian chạy được phân chia có chủ ý theo trách nhiệm: Rust sở hữu
cửa hàng chuẩn, xác thực lược đồ, di chuyển, mô hình đường dẫn, khóa, nguyên tử
ghi, băm chiếu và trình kết xuất có cấu trúc. Shell chỉ còn lại như
các điểm vào hoặc bộ điều hợp tương thích nhỏ dành cho người tiêu dùng bên ngoài như
`hyprctl` và các tập lệnh chủ đề thuộc sở hữu nội dung. Python được dành riêng cho việc kiểm tra, kiểm tra
và những người trợ giúp phát triển; nó không phải là một phần của thời gian chạy máy tính để bàn quan trọng.

`argvus config apply` là ranh giới xây dựng lại (`project` vẫn là ranh giới
chính tả tương thích). Nó tái tạo các đầu ra bị thiếu ngay cả khi chuẩn
tài liệu không thay đổi, hiển thị máy chiếu ở chế độ dàn dựng và hoán đổi nguyên tử
Cây `data/` chỉ sau mỗi lần chiếu thành công. Vòng đời phiên vẫn thuộc quyền sở hữu của
`argvus sessionctl`, sử dụng bảng kê khai đó để lên kế hoạch tải lại mục tiêu.

Khi người gọi đã thực hiện một thế hệ — `apply-theme`, một dấu
thay đổi, `set`/`patch` hoặc thay đổi khối đo từ xa của tiện ích - `reload` **áp dụng**
thế hệ đó thay vì chiếu lại nó. Nó đọc kế hoạch từ
bảng kê khai, khôi phục người tiêu dùng được nhắm mục tiêu và chỉ các dự án khi bảng kê khai được
vắng mặt hoặc không có công việc chờ xử lý. Việc tái dự kiến vào thời điểm đó sẽ xây dựng lại một
cây giống hệt nhau và loại bỏ kế hoạch đột biến vừa tạo ra.

Bản kê khai cũng ghi lại quá trình tạo hoạt động, cấu hình hiệu quả
hàm băm, tên miền đã thay đổi và danh sách tệp được tạo. `data/internal/generation.json`
là điểm đánh dấu chiếu cục bộ dùng một lần và không bao giờ là tùy chọn của người dùng.
`argvus config status` so sánh các bản ghi này với cấu hình đã giải quyết
và báo cáo các tệp bị thiếu hoặc được tạo thêm.

Các đột biến tuân theo cùng một ranh giới động cơ: `UserConfig` được sao chép vào một
`NextUserConfig`, được xác thực, phân giải thành `EffectiveConfig`, được chiếu và
chỉ sau đó vẫn tồn tại. `.config-transaction.json` là trạng thái phục hồi hoạt động;`argvus config recover` xác định lại phiên bản chuẩn trước đó một cách dứt khoát
tài liệu sau sự cố giữa việc xuất bản `data/` và `config.json`. Tải lại là
chỉ được yêu cầu sau lần cam kết và lỗi tải lại không hoàn nguyên
cấu hình bền vững mạch lạc.

##Mô hình độ phân giải

```text
config.json logical model
    ↓
ARGVUS native compatibility state and per-app override
    ↓
ARGVUS generated runtime file
    ↓
/usr/share/argvus packaged default
    ↓
upstream default
```

Các ứng cử viên chính xác khác nhau tùy theo thành phần, nhưng nguyên tắc nhất quán: người dùng ghi đè sẽ thắng, đầu ra được tạo là trạng thái dẫn xuất và các tệp được đóng gói vẫn là đầu vào chỉ đọc. Phiên xuất `ARGVUS_CONFIG_HOME` và `ARGVUS_SYSTEM_CONFIG` để người trợ giúp và dịch vụ người dùng giải quyết cùng một gốc.

Tải lại phục hồi theo `recover → ensure → migrate → apply → runtime reload`,
và bước chiếu bị bỏ qua khi bảng kê khai đến đã mô tả một
thế hệ cam kết. `ensure`
hiện thực hóa `config.json` từ các giá trị mặc định của lược đồ chỉ khi cấu hình được
bị thiếu và điền vào các trường vắng mặt mà không thay thế `false`, `0` rõ ràng, hoặc
các giá trị tùy chỉnh. Một tài liệu hiện có không đúng định dạng vẫn là một lỗi thay vì
đang được thay thế một cách âm thầm.

Các đột biến liên tục so sánh các cấu hình cũ và mới đã được giải quyết thông qua
`ChangedDomains`. Sau khi cam kết, một yêu cầu tải lại được gửi qua
`argvus sessionctl`; lỗi tải lại được báo cáo và ghi lại mà không cần lăn
quay lại cấu hình đã cam kết. Khởi động phiên thực hiện khôi phục trước
hiện thực hóa và chiếu bootstrap, vì vậy người tiêu dùng không bắt đầu phản đối
giao dịch bị bỏ rơi

| Tiểu bang | Mục đích |
| --- | --- |
| `config.json` | Tùy chọn logic được quản lý theo tiêu chuẩn; nguồn duy nhất cho mọi thứ bên dưới `generated/`. |
| `.active-theme`, `.accent-color`, `.accent-custom` | Đầu vào di chuyển kế thừa; không phải nguồn thời gian chạy và không được tạo lại. |
| `.wallpaper-custom` | Đầu vào di chuyển kế thừa cho `appearance.wallpaper`; không phải là nguồn thời gian chạy. |
| `.spaces`, `.borders` | Đầu vào di chuyển bố cục cũ; các giá trị chuẩn được lưu trữ dưới `layout` trong `config.json`. |
| `fonts.conf` | Phép chiếu tương thích của các lựa chọn phông chữ chuẩn. |
| `generated/effects` | Khả năng tương thích của các hiệu ứng kinh điển; dùng một lần. |
| `$XDG_STATE_HOME/argvus/config-projection.json` | Bản kê khai chiếu: các phần đã thay đổi, người tiêu dùng bị ảnh hưởng và liệu có cần tải lại hay không. Được đọc bởi `reload`; không bao giờ là nguồn gốc của sự thật. |
| `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` | Tùy chọn khối từ xa kế thừa. Chỉ đọc dưới dạng dự phòng di chuyển; các giá trị chuẩn tồn tại ở `/control_panel/widget_telemetry_blocks` và tệp không còn được ghi nữa. |

Một thuộc tính là chuẩn khi nó được đọc từ `config.json` và được chiếu từ
ở đó. Một tài sản có được khi nó được dự kiến nhưng cũng có thể tiêu thụ được như một
đầu vào di chuyển. Bất cứ thứ gì một thành phần vẫn ghi trong thời gian chạy - bộ nhớ đệm, bản gốc
config trong `$XDG_CONFIG_HOME/<app>`, tệp trạng thái lời chào - là một bộ chuyển đổi
target và không được đọc lại dưới dạng cấu hình.

Projection bao gồm người tiêu dùng Hyprland, GTK/Qt, Waybar, Quickshell, thiết bị đầu cuối, thông báo, màn hình khóa và ứng dụng gốc. Hyprland đọc các màu đường viền hoạt động và không hoạt động từ chủ đề đã chọn; `appearance.accent_custom=true` rõ ràng là trường hợp duy nhất ghi đè đường viền hoạt động/nhóm bằng giọng người dùng. Chuyển đổi chủ đề sẽ áp dụng lại các giá trị này thông qua tải lại bộ tổng hợp để nó không thể giữ lại màu của chủ đề trước đó. Nếu cấu hình người dùng được tạo lại, các hiệu ứng mặc định sẽ được bật. Xóa `generated/` và chạy `argvus config apply` sẽ tạo lại từng byte một từ `config.json`; chỉnh sửa trực tiếp các tập tin được tạo ra không bền.
Chẩn đoán thời gian chạy phải xác định bộ thực thi hoàn chỉnh: đường dẫn nhị phân,
phiên bản/bản sửa đổi, gốc tài sản, gốc cấu hình, gốc dữ liệu, gốc được tạo, nội bộ
biểu hiện gốc và phép chiếu. `status --verbose` hiển thị bộ dữ liệu này và
so sánh thế hệ/băm đã xuất bản với cấu hình đã giải quyết hiện tại.
Kiểm tra gói phát triển và cài đặt là các chế độ riêng biệt; cũng không thể rơiâm thầm quay trở lại tệp thực thi không liên quan hoặc nhà của nhà phát triển. `apply` là
cũng là ranh giới điều chỉnh cho các mã định danh chủ đề kết hợp cũ: a
giá trị như `argvus-dark-spaced` trở thành `theme=argvus-dark` và
`variant=spaced` trước khi phân giải. Nhận dạng tài sản đóng gói không được ghi lại trong bảng kê khai trong bản phát hành này, do đó không có tệp chẩn đoán nào mô tả nó.
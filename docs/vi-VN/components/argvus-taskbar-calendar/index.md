---
title: Lịch
description: Sử dụng lịch trên thanh tác vụ ARGVUS.
---

`argvus-taskbar-calendar` là cửa sổ bật lên lịch GTK4 được mở từ mô-đun ngày Waybar hoặc bằng:

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```

Nó hỗ trợ các sự kiện địa phương, lời nhắc và nhập/xuất ICS. Dịch vụ người dùng của nó là `argvus-taskbar-calendar.service`. Sử dụng `argvus taskbar-calendar --help` để biết các tùy chọn được phiên bản đã cài đặt hỗ trợ.

Lịch tuân theo chủ đề ARGVUS đang hoạt động, bao gồm các biến thể Cố định và Khoảng cách. Thay đổi chủ đề được áp dụng thông qua bộ đệm người dùng đã tạo; sử dụng `argvus taskbar-calendar reload` nếu cửa sổ bật lên đang chạy cần được làm mới rõ ràng.

Khi được mở từ mô-đun ngày trên thanh tác vụ, cửa sổ bật lên được neo vào cạnh thanh: nó mở bên dưới thanh tác vụ trên cùng và phía trên thanh tác vụ dưới cùng, được căn chỉnh theo cạnh phải của mô-đun ngày. Khởi chạy trực tiếp mà không có hình học trên thanh tác vụ, hãy sử dụng con trỏ hoặc theo dõi dự phòng.

## Lệnh

```bash
argvus taskbar-calendar                 open the calendar
argvus taskbar-calendar toggle          show or hide it (the Waybar date module uses this)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          open the system configuration for editing (sudo)
argvus taskbar-calendar config-path     print the configuration path
argvus taskbar-calendar reload
argvus taskbar-calendar import file.ics
argvus taskbar-calendar export --output calendar.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```

## Sự kiện và lời nhắc

Sự kiện được tắt theo mặc định. Bật chúng trong cửa sổ bật lên trong **Cài đặt > Sự kiện**; cái
sự lựa chọn được lưu giữ trong thư mục trạng thái và cũng cho phép và bắt đầu
`argvus-taskbar-calendar.service`, hiển thị lời nhắc. Tắt chúng sẽ dừng
dịch vụ.
- **+ SỰ KIỆN** (hoặc `n`) mở trình chỉnh sửa cho ngày đã chọn. Sự kiện có thể được tạo ra và
  đã thay đổi kể từ hôm nay; một sự kiện của ngày hôm qua vẫn có thể được mở và xóa.
- Sự kiện mới bắt đầu lúc 09:00 và kéo dài theo thời lượng mặc định được đặt trong Cài đặt. Di chuyển
  thời gian bắt đầu giữ nguyên độ dài; thời gian kết thúc sớm hơn thời gian bắt đầu có nghĩa là sự kiện
  kết thúc vào ngày hôm sau.
- Mọi sự kiện đều hiển thị thông báo kèm theo âm thanh khi bắt đầu, bất kể sự kiện là gì
  Các hộp **Nhắc nhở** và **Lặp lại** cho biết. Một sự kiện được lưu sau khi nó bắt đầu (cả ngày
  sự kiện được tạo trong ngày chẳng hạn) sẽ thông báo trong vòng 30 giây.
- **Nhắc nhở** thêm thông báo trước đó, thời gian đã chọn trước khi bắt đầu (lúc
  bắt đầu khi thời gian đó đã trôi qua).
- Trong sự kiện kéo dài cả ngày, trường thời gian trở thành **THÔNG BÁO TẠI**: sự kiện bao gồm toàn bộ
  ngày và thông báo vào thời điểm đó (00:00 thông báo vào đầu ngày). thời gian là
  được giữ lại khi sự kiện được mở lại.
- Đối với các sự kiện diễn ra cả ngày, **Lặp lại** sẽ thông báo lại vào khoảng thời gian đã chọn sau thời gian đó.
- Chỉnh sửa một sự kiện mà không di chuyển phần bắt đầu của nó sẽ không lặp lại các lời nhắc đã được hiển thị.
- Thông báo tuân theo ngôn ngữ lịch (Cài đặt > Ngôn ngữ).
- Một ngày hiển thị một dấu chấm (`•`) khi có sự kiện diễn ra trên đó, bao gồm mọi ngày của một
  sự kiện qua đêm hoặc nhiều ngày. Ngày theo múi giờ địa phương.
- Các sự kiện cục bộ sẽ tự động bị xóa 10 phút sau khi chúng kết thúc. Nếu một sự kiện đã có
  kết thúc khi được lưu, nó vẫn khả dụng trong ít nhất 10 phút sau khi lưu.

Các tệp `.ics` đã nhập sẽ giữ nguyên ngày, múi giờ (`TZID`) và báo thức (`VALARM`).
Các sự kiện đã nhập cũng thông báo khi bắt đầu, bên cạnh các cảnh báo riêng của chúng. Sự kiện định kỳ (`RRULE`)
được lưu trữ và xuất, nhưng chỉ lần xuất hiện đầu tiên của chúng mới được hiển thị và nhắc nhở.
Sau khi cập nhật gói, dịch vụ nhắc nhở sẽ thoát và systemd khởi động phiên bản mới
trong vòng 30 giây. `sync` chỉ báo cáo các lịch từ xa đã được định cấu hình; Đồng bộ hóa CalDAV không
đã triển khai chưa.

## Đường dẫn

- Cấu hình: `/etc/argvus/taskbar/calendar/config.toml` (toàn hệ thống; mặc định đi kèm gói).
- Kiểu dáng: `/etc/argvus/taskbar/calendar/style.css` và `theme.css`; màu sắc của chủ đề ARGVUS đang hoạt động đến từ bản dựng chủ đề thông qua bộ đệm bên dưới.
- Dữ liệu: `$XDG_DATA_HOME/argvus-taskbar-calendar/` (hoặc `~/.local/share/argvus-taskbar-calendar/`).
- Trạng thái: `$XDG_STATE_HOME/argvus/taskbar/calendar/` (hoặc `~/.local/state/...`); sự kiện chuyển đổi được giữ ở đó.
- Bộ đệm: `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (hoặc `~/.cache/argvus-taskbar-calendar/`).

Màu đánh dấu được đọc từ `~/.config/argvus/data/.accent-color` và được áp dụng khi chạy
bên trên màu sắc của chủ đề.

Lựa chọn ngôn ngữ tự động theo người dùng `locale.conf`, với tiếng Anh làm mặc định. Tùy chọn ngôn ngữ lịch rõ ràng vẫn ghi đè lựa chọn tự động.Lịch trên thanh tác vụ, ARGVUS Focus, ARGVUS Clock và Control Panel loại trừ lẫn nhau: mở cái này sẽ đóng cái kia.

Lỗi đọc lịch được hiển thị riêng biệt với chương trình làm việc trống. Việc mở cơ sở dữ liệu, tải sự kiện hàng ngày và tải chỉ số hàng tháng có các thông điệp riêng biệt. Chọn lại một ngày để thử lại. Việc làm mới không thành công chỉ giữ lại dữ liệu đã tải trước đó trong cùng ngày hoặc tháng; kết quả hợp lệ từ truy vấn khác vẫn có sẵn.

## Danh mục ngôn ngữ được chia sẻ

Giao diện sử dụng danh mục `argvus-i18n` được chia sẻ cho tất cả mười ngôn ngữ. Văn bản được lưu trữ trong `resources/locales/<locale>/taskbar-calendar.json` trong không gian làm việc chính và được cài đặt bởi `argvus`. Tiếng Anh là dự phòng; ngôn ngữ mới không còn yêu cầu từ điển song ngữ địa phương nữa.

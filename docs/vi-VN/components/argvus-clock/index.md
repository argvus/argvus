---
title: ARGVUS Clock
description: Đồng hồ, đồng hồ thế giới, đồng hồ bấm giờ, đồng hồ đếm ngược và báo thức có âm thanh.
---

Đồng hồ ARGVUS là ứng dụng GTK 4 với các công cụ thời gian và thời gian hiện tại: đồng hồ thế giới, đồng hồ bấm giờ, đồng hồ đếm ngược và báo thức. Mở nó từ trình khởi chạy, bằng cách nhấp vào thời gian trên thanh tác vụ, với **SUPER + Shift + C** hoặc bằng:

```sh
argvus clock                     # open the popup near the pointer
argvus clock --toggle            # hide the popup if it is visible, show it otherwise
argvus clock --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus clock --page alarms       # open on a page: clock, world, stopwatch, timer, alarms
argvus clock --background        # run without a window, to keep alarms active
```

Đồng hồ ARGVUS mở ra dưới dạng cửa sổ bật lên được căn chỉnh với thanh tác vụ, với khoảng cách 4 pixel. Thanh tác vụ chuyển tọa độ nhấp chuột gốc để cửa sổ bật lên sử dụng màn hình được nhấp; khởi chạy nó mà không có tọa độ sử dụng vị trí con trỏ. Nếu con trỏ ở bên ngoài cửa sổ bật lên trong 5 giây, nó sẽ tự động đóng lại; di chuyển con trỏ vào cửa sổ bật lên sẽ hủy đếm ngược. **Esc** trên tab chính vẫn ẩn nó ngay lập tức và **Esc** trên toàn trang sẽ quay lại tab trước đó. `argvus --clock` là bí danh. Bấm vào ngày trên thanh tác vụ vẫn mở lịch; Đồng hồ ARGVUS không hiển thị sự kiện.

ARGVUS Focus, ARGVUS Clock và Control Panel có chung cạnh màn hình. Việc mở Đồng hồ sẽ đóng Tiêu điểm và Bảng điều khiển trước; mở một trong hai bề mặt đó sẽ đóng Đồng hồ.

## trang

- **Đồng hồ**: thời gian ở dạng chữ số lớn (24 giờ theo mặc định, 12 giờ trong cài đặt, giây tùy chọn), ngày được bản địa hóa, múi giờ địa phương và độ lệch UTC của nó cũng như báo thức tiếp theo.
- **Thế giới**: các thành phố được thêm bằng cách tìm kiếm thành phố hoặc múi giờ IANA (dấu và kiểu chữ bị bỏ qua). Mỗi cái đều hiển thị thời gian, sự khác biệt so với giờ địa phương và cho dù đó là ngày hôm qua, hôm nay hay ngày mai ở đó. Tiết kiệm ánh sáng ban ngày đến từ cơ sở dữ liệu múi giờ IANA; không có gì được tính toán với độ lệch cố định. Các thành phố có thể được sắp xếp lại và loại bỏ.
- **Đồng hồ bấm giờ**: bắt đầu, tạm dừng, tiếp tục, đặt lại và vòng (thời gian vòng và tổng số).
- **Hẹn giờ**: giờ, phút và giây hoặc cài đặt trước 5, 10, 15, 25, 30 và 60 phút. Tạm dừng, tiếp tục, khởi động lại và hủy. Khi nó kết thúc, nó reo lên như một hồi chuông cảnh báo. Bộ hẹn giờ này độc lập với các phiên Focus của ARGVUS.
- **Báo thức**: tạo, chỉnh sửa, bật, tắt và xóa báo thức. Mỗi cái có một thời gian, một tên tùy chọn, các ngày trong tuần nó lặp lại (không có nghĩa là nó đổ chuông một lần), âm thanh và thời lượng báo lại.

Bàn phím: **Ctrl+1** sang **Ctrl+5** chuyển trang, **Ctrl+N** tạo cảnh báo, **Ctrl+,** mở cài đặt, **Esc** quay lại, **Ctrl+W** đóng cửa sổ và **Ctrl+Q** thoát.

## Âm thanh báo động

Báo thức phát âm thanh thực cho đến khi bạn trả lời:

- Âm thanh: **Chime** (mặc định) và **Beacon**, được vận chuyển trong `/usr/share/argvus/sounds/`.
- **Âm lượng báo thức** (10 đến 100, mặc định 80) và **Nghe âm thanh báo thức** có trong cài đặt; trình chỉnh sửa có nút kiểm tra âm thanh đã chọn.
- Chuông báo thức hiển thị một thẻ ở đầu màn hình, phía trên các cửa sổ khác và một thông báo, cả hai đều có **Báo lại** và **Bỏ qua**. Khi thẻ được tập trung, **Enter** sẽ loại bỏ và **S** báo lại.
- Báo lại đổ chuông sau thời gian báo lại của báo thức (1 đến 60 phút, mặc định là 10).
- Báo thức không được trả lời sẽ dừng sau **Dừng đổ chuông sau** (5, 10, 15 hoặc 30 phút, mặc định là 10) và để lại thông báo.
- Hai báo thức đổ chuông cùng nhau đều được liệt kê; cái thứ hai không bao giờ im lặng cái đầu tiên.
- Nếu không phát được âm thanh thì card và cửa sổ báo lỗi và ghi lỗi. Đồng hồ ARGVUS không bao giờ báo cáo âm thanh không phát.

Tính năng phát lại sử dụng hỗ trợ đa phương tiện của GTK (GStreamer) nên hoạt động với PipeWire, PulseAudio hoặc ALSA. Gói này phụ thuộc vào `gst-plugins-base` (Ogg Vorbis) và `gst-plugins-good` (đầu ra âm thanh).

## Khi chuông báo thức đổ chuông

Báo thức chỉ đổ chuông khi Đồng hồ ARGVUS chạy trong phiên của bạn:

- Đóng cửa sổ giúp Đồng hồ ARGVUS chạy trong khi có báo thức, đồng hồ hẹn giờ đang chạy hoặc đồng hồ bấm giờ đang chạy (**Tiếp tục chạy khi cửa sổ đóng**, bật theo mặc định). Không có gì đang chờ xử lý, nó sẽ thoát.
- **Bắt đầu với phiên** kích hoạt đơn vị người dùng `argvus-clock.service`, chạy `argvus clock --background` sau khi đăng nhập. Nó được tắt theo mặc định.- Báo thức không thể đổ chuông khi bạn đăng xuất, máy tính tắt hoặc bị treo.
- Báo thức trễ đến 5 phút vẫn đổ chuông. Sau đó (đồng hồ không chạy hoặc hệ thống ở chế độ ngủ hoặc tắt) nó không đổ chuông: nó được báo cáo là **bị bỏ lỡ** trong một thông báo và trên trang Báo thức. Báo thức lặp lại tiếp tục vào ngày hôm sau; báo thức một lần sẽ tắt.
- Mỗi lần xuất hiện đều được lưu dưới dạng đã xử lý trước khi nó đổ chuông, do đó việc mở lại Đồng hồ ARGVUS không bao giờ đổ chuông cùng một báo thức hai lần và chỉ một phiên bản chạy mỗi phiên.
- Báo thức lặp lại được tính toán theo giờ địa phương vào các ngày trong tuần đã chọn, do đó, chúng giữ thời gian theo các thay đổi theo tiết kiệm ánh sáng ban ngày. Khoảng thời gian bị lò xo thay vòng ở phút đầu tiên sau khe hở; thời gian xảy ra hai lần trong lần đầu tiên đổ chuông.
- Đồng hồ bấm giờ tiếp tục đếm trong thời gian tạm dừng. Nếu nó kết thúc trong khi hệ thống đang ngủ, nó sẽ đổ chuông khi thức dậy và cho biết đã muộn như thế nào. Sau khi Đồng hồ ARGVUS đóng, đồng hồ hẹn giờ đang chạy hoặc đồng hồ bấm giờ sẽ tạm dừng hoạt động trở lại.

## Đường dẫn

| Đường dẫn | Nội dung |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/clock/settings.json` | Cài đặt và đồng hồ thế giới |
| `$XDG_DATA_HOME/argvus/clock/alarms.json` | Báo động |
| `$XDG_STATE_HOME/argvus/clock/timers.json` | Điểm kiểm tra đồng hồ bấm giờ và đồng hồ bấm giờ |
| `/usr/lib/systemd/user/argvus-clock.service` | Bắt đầu với phiên |
| `/usr/share/argvus/sounds/` | Âm thanh đóng gói |

Các tập tin được viết nguyên tử; một tệp không thể đọc được sẽ được đổi tên thành `<name>.corrupt-<time>` và các giá trị mặc định được sử dụng. Màu sắc tuân theo chủ đề ARGVUS đang hoạt động và màu nổi bật (người tiêu dùng chủ đề `time`). Các chuỗi đến từ miền dịch thuật `clock` và `time`.

Lịch trên thanh tác vụ, ARGVUS Focus, ARGVUS Clock và Control Panel loại trừ lẫn nhau: mở cái này sẽ đóng cái kia.
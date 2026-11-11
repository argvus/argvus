---
title: ARGVUS Focus
description: Pomodoro và các phiên tập trung với thời gian nghỉ, thông báo và lịch sử phiên cục bộ.
---

ARGVUS Focus là ứng dụng GTK 4 để làm việc trong các phiên tập trung được phân tách bằng thời gian nghỉ. Nó mở ra dưới dạng cửa sổ bật lên bên cạnh thanh tác vụ. Mở nó từ trình khởi chạy, bằng cách nhấp vào chỉ báo trên thanh tác vụ của nó, với **SUPER + Shift + F** hoặc bằng:

```sh
argvus focus              # open the popup near the pointer
argvus focus --toggle     # hide the popup if it is visible, show it otherwise
argvus focus --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus focus --start      # start or resume the current phase without opening the window
argvus focus --pause      # pause the running phase
argvus focus status --waybar         # print one Waybar JSON status and exit
argvus focus status --waybar --follow  # stream status changes for Waybar
```

Thanh tác vụ chuyển tọa độ nhấp chuột gốc để cửa sổ bật lên sử dụng màn hình được nhấp và căn chỉnh với thanh; phóng không có tọa độ sử dụng vị trí con trỏ. Nếu con trỏ ở bên ngoài cửa sổ bật lên trong 5 giây, nó sẽ tự động đóng lại; di chuyển con trỏ vào cửa sổ bật lên sẽ hủy việc đếm ngược đó. **Esc** trên trang chính vẫn ẩn ngay lập tức và **Esc** trên trang con sẽ quay lại trang trước. Chỉ báo trên thanh tác vụ chỉ hiển thị biểu tượng của nó khi không hoạt động và `MM:SS` còn lại trong một pha, với màu pha; giai đoạn tạm dừng sẽ giữ nguyên thời gian còn lại và thêm trạng thái tạm dừng. Lệnh `status` chạy bên trong `argvus` mà không khởi động GTK. `argvus --focus` là bí danh. Chỉ một phiên bản Focus chạy mỗi phiên: lần khởi chạy thứ hai sẽ chuyển các tùy chọn của nó cho phiên bản đang chạy.

ARGVUS Focus, ARGVUS Clock và Control Panel có chung cạnh màn hình. Việc mở Tiêu điểm sẽ đóng Đồng hồ và Bảng điều khiển trước; mở một trong hai bề mặt đó sẽ đóng Focus.

## Phiên

- Chế độ **Pomodoro** luân phiên lấy nét và nghỉ ngắn và kết thúc mỗi chu kỳ bằng thời gian nghỉ dài. Thời lượng mặc định là 25, 5 và 15 phút, có thời gian nghỉ dài sau 4 phiên tập trung.
- Chế độ **Phiên đơn** chạy một phiên tập trung có thời lượng riêng (45 phút theo mặc định) mà không có thời gian nghỉ.
- Cửa sổ hiển thị giai đoạn (tiêu điểm, nghỉ ngắn, nghỉ dài, mỗi giai đoạn có màu riêng), thời gian còn lại trong vòng tiến trình, vị trí trong chu kỳ và tổng thời gian của ngày hôm nay.
- **Bắt đầu tập trung / Bắt đầu ngắt**, **Tạm dừng**, **Tiếp tục**, **Hủy phiên** (bỏ qua giai đoạn và không ghi gì) và **Hoàn thành ngay** (kết thúc phiên tập trung sớm và ghi lại thời gian thực tế đã sử dụng). Trong thời gian nghỉ giải lao, **Bỏ qua giờ giải lao** quay lại tiêu điểm.
- Khi một giai đoạn kết thúc, giai đoạn tiếp theo sẽ chờ bạn, trừ khi **Bắt đầu tự động ngắt** hoặc **Bắt đầu lấy nét tự động sau khi nghỉ** được bật.

Bàn phím: **Space** bắt đầu hoặc tạm dừng, **Ctrl+,** mở cài đặt, **Ctrl+H** lịch sử, **Esc** quay lại, **Ctrl+W** ẩn cửa sổ và **Ctrl+Q** thoát.

## Cài đặt

Các cài đặt trước (Cổ điển 25/5/15, Mở rộng 50/10/30, Làm việc sâu 90/20/30) lấp đầy các độ dài và có thể chỉnh sửa được. Cài đặt cũng bao gồm các phiên trước thời gian nghỉ dài, tự động bắt đầu, thông báo khi kết thúc một giai đoạn và **âm thanh phiên**: một tiếng chuông ngắn có âm lượng riêng và nút **Kiểm tra âm thanh**. Những âm thanh này tách biệt với âm thanh báo thức của [Đồng hồ ARGVUS](/docs/components/argvus-clock/).

## Lịch sử

Mỗi phiên tập trung đã hoàn thành sẽ được ghi lại thông tin bắt đầu, kết thúc và thời gian sử dụng; nghỉ giải lao và các phiên bị hủy thì không. Trang lịch sử hiển thị các phiên và phút hôm nay, tổng số và các phiên gần đây. Các bản ghi cũ hơn 365 ngày sẽ bị loại bỏ. **Xóa lịch sử** yêu cầu xác nhận.

## Cửa sổ, nền và tạm dừng

- Đóng cửa sổ trong một chu kỳ chỉ ẩn nó: phiên tiếp tục và thông báo cho bạn khi giai đoạn kết thúc. Không có gì đang diễn ra, kết thúc.
- Nếu hệ thống bị treo trong khi một pha đang chạy, pha đó sẽ bị tạm dừng tại thời điểm tạm dừng và một thông báo cho biết hệ thống đã ngủ trong bao lâu. Ngủ không được tính là thời gian tập trung.
- Phiên được lưu sau mỗi thay đổi và cứ sau 30 giây. Nếu ARGVUS Focus bị đóng trong một giai đoạn (đăng xuất, gặp sự cố, **Ctrl+Q**), nó sẽ quay lại tạm dừng tại thời điểm đó vào lần mở tiếp theo; nó không bao giờ giả vờ rằng đồng hồ vẫn chạy trong khi nó không thể thông báo.
- Thời gian được đo trên đồng hồ thời gian khởi động của hệ thống nên việc thay đổi đồng hồ treo tường hoặc múi giờ không làm rút ngắn hoặc kéo dài một pha.## Đường dẫn

| Đường dẫn | Nội dung |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/focus/settings.json` | Cài đặt |
| `$XDG_DATA_HOME/argvus/focus/history.json` | Phiên đã hoàn thành |
| `$XDG_STATE_HOME/argvus/focus/session.json` | Điểm kiểm tra của phiên hiện tại |
| `/usr/share/argvus/sounds/argvus-bell.oga` | Chuông phiên |

Các tập tin được viết nguyên tử. Một tệp không thể đọc được được đổi tên thành `<name>.corrupt-<time>` và các giá trị mặc định được sử dụng; một tin nhắn cho bạn biết nó đã đi đâu.

Màu sắc tuân theo chủ đề ARGVUS đang hoạt động và màu nổi bật (chủ đề tiêu dùng `time`). Các chuỗi đến từ miền dịch thuật `focus` và `time`.

Lịch trên thanh tác vụ, ARGVUS Focus, ARGVUS Clock và Control Panel loại trừ lẫn nhau: mở cái này sẽ đóng cái kia.
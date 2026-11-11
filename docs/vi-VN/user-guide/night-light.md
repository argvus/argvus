---
title: Night Light
description: Điều chỉnh nhiệt độ màu và lịch trình cục bộ cố định trên Hyprland.
---

Mở **Trung tâm điều khiển > Màn hình nền & Đầu vào > Màn hình > Đèn ngủ**, tìm kiếm **Đèn ngủ** hoặc chạy `argvus control-center displays night-light`.

Night Light và lịch trình của nó ban đầu bị tắt. Chọn nhiệt độ từ **1000 đến 20000 Kelvin** (mặc định **4500 K**). Giá trị thấp hơn cho màu sắc ấm hơn. Trang hiển thị kích hoạt thủ công đã lưu, trạng thái hiệu lực và quá trình chuyển đổi tiếp theo. Nhập chỉnh sửa một giá trị; Esc hủy nó hoặc quay lại Hiển thị. Các giá trị không hợp lệ sẽ bị từ chối trước khi lưu. Lưu thay đổi và áp dụng ngay lập tức. Nếu việc áp dụng không thành công sau khi lưu, tùy chọn đã lưu vẫn hiển thị cùng với trạng thái hiệu quả thực tế và lỗi.

## Lên lịch và hành động nhanh chóng

Lịch trình cố định sử dụng múi giờ địa phương hiện tại. Sự bắt đầu được bao gồm và sự kết thúc được loại trừ; **20:00–07:00** đã qua nửa đêm. Bắt đầu và kết thúc phải khác nhau và sử dụng `HH:MM`. Không có vị trí hoặc tính toán năng lượng mặt trời được sử dụng.

Sử dụng thẻ **Hiển thị** của Bảng điều khiển hoặc **SUPER + Shift + B** để chuyển đổi nhanh. Bộ chọn độ sáng giữ lại tất cả các hành động về độ sáng và thêm Ánh sáng ban đêm khi có phần phụ trợ, kể cả trên màn hình không có phần phụ trợ độ sáng. Không có mô-đun thanh tác vụ cố định.

Khi tắt lịch trình, việc kích hoạt thủ công vẫn tiếp tục. Khi lập lịch bật, chuyển đổi nhanh sẽ tạo ghi đè thời gian chạy cho đến lần chuyển đổi được lên lịch tiếp theo. Việc thay đổi lịch trình hoặc nhiệt độ sẽ xóa nội dung ghi đè đó. Đăng xuất, phiên bản bộ tổng hợp mới hoặc chuyển đổi tiếp theo đã thay đổi (bao gồm thay đổi múi giờ) cũng làm mất hiệu lực của nó.

## Lệnh và trạng thái

Tất cả các giao diện đều sử dụng cùng một lệnh chung:

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```

`status` trả về lược đồ JSON phiên bản 1: `available`, `unavailable_reason`, `settings` (đã bật, nhiệt độ, lịch_enabled, bắt đầu, kết thúc), `active`, `effective_temperature`, `mode` (`manual` hoặc `scheduled`), `manual_override`, `next_transition` (ISO 8601 có offset), `minimum_temperature` và `maximum_temperature`. Các trường hiệu quả sẽ rỗng khi không có sẵn; nhiệt độ hiệu quả là vô giá trị khi nhận dạng được kích hoạt. Lỗi khả dụng được báo cáo trong trạng thái; lỗi hành động trả về giá trị khác 0 với chẩn đoán.

Tùy chọn chuẩn là `/displays/night_light`, được lưu trữ nguyên tử bởi `argvus config` trong `$XDG_CONFIG_HOME/argvus/config/displays.json`. Ghi đè dễ bay hơi và khóa hành động nằm trong `$XDG_RUNTIME_DIR/argvus/night-light/`. Các tập tin giám sát được tạo ra là độc lập. Không chỉnh sửa các tệp đã tạo để định cấu hình tính năng này.

## Thời gian chạy

Yêu cầu Hyprland và **hyprsunset >= 0.4.0**. Phiên bắt đầu `hyprsunset.service` ngược dòng một lần, áp dụng trạng thái đã lưu thông qua `argvus-night-light-init.service` và đánh giá lịch trình mỗi phút thông qua `argvus-night-light.timer` và `argvus-night-light.service`. Những thay đổi về đồng hồ và múi giờ cũng kích hoạt việc đánh giá. Khi tắt lịch trình, hoạt động định kỳ sẽ không còn trạng thái thủ công. Các ứng dụng lặp đi lặp lại truy vấn danh tính/nhiệt độ hiệu quả và bỏ qua việc ghi không thay đổi. Không cần tải lại bộ tổng hợp.

ARGVUS gửi một phần bổ sung systemd cho dịch vụ ngược dòng: nó bắt đầu bằng `--identity` và tách biệt việc khám phá cấu hình của nó với `/usr/share/argvus/session/config/night-light`. Điều này tránh các hồ sơ gốc cạnh tranh và giữ nguyên các tệp `hyprsunset.conf` cá nhân. Việc đăng nhập áp dụng cho dịch vụ ngược tuyến đó trong khi gói phiên ARGVUS được cài đặt, bao gồm cả các lần khởi động bên ngoài ARGVUS. Việc ghi đè systemd của người dùng hiện tại có thể được ưu tiên; kiểm tra `systemctl --user cat hyprsunset.service` khi chẩn đoán tình huống đó. Hyprsunset 0.4.0 từ chối đối số đường dẫn riêng của `--config`, do đó, việc khám phá cấu hình được tách biệt bằng các biến XDG cục bộ dịch vụ.Phần phụ trợ sử dụng Hyprsunset IPC chính thức (`temperature`, `identity`, `identity get`), với I/O giới hạn và đọc lại sau khi thay đổi. Không sử dụng trình đổ bóng, phần phụ trợ của bộ tổng hợp thay thế, ghi cấu hình hệ thống hoặc leo thang đặc quyền. Các điều khiển không khả dụng sẽ bị tắt trong Trung tâm điều khiển và ẩn trong thẻ Hiển thị.
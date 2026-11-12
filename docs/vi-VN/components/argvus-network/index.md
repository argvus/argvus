---
title: Mạng và Bluetooth
description: Quản lý NetworkManager và Bluetooth thông qua ARGVUS.
---

Lệnh phụ `argvus network` báo cáo và kiểm soát NetworkManager, đồng thời `argvus bluetooth` báo cáo và kiểm soát trạng thái cũng như tính khả dụng của bộ điều hợp Bluetooth. Cả hai đều chạy trong quá trình bên trong `argvus`; `networkctl` và `bluetoothctl` vẫn là bí danh. Bảng điều khiển và Waybar gọi các lệnh này.

Sử dụng `--help` trên một trong hai lệnh cho các lệnh phụ được cài đặt. Tích hợp giao diện người dùng mạng và Bluetooth tách biệt với các trình quản lý nâng cao tùy chọn như Blueman.

Trung tâm điều khiển tách trạng thái mạng khỏi các trang cấu hình: **Mạng → Trạng thái** tóm tắt trạng thái hiện tại, trong khi các giao diện, Wi-Fi, Ethernet, VPN, DNS, proxy và tường lửa là các tuyến riêng biệt khi nhà cung cấp của chúng có sẵn. **Bluetooth → Trạng thái**, **Thiết bị** và **Ghép nối** bao gồm trạng thái của bộ điều hợp và hoạt động của thiết bị. Các trang này thực hiện các hành động hệ thống do nhà cung cấp hỗ trợ; thanh tác vụ và Bảng điều khiển hiển thị bề mặt trạng thái/hành động nhanh.

Các thay đổi về mạng và Bluetooth có thể yêu cầu ủy quyền và có thể áp dụng ngay lập tức thông qua NetworkManager hoặc dịch vụ Bluetooth. Chúng không giống như việc thay đổi chỉ báo trên thanh tác vụ ARGVUS.

Trung tâm điều khiển gửi mật khẩu Wi-Fi tới nmcli thông qua đầu vào tiêu chuẩn với `--ask`; mật khẩu được loại trừ khỏi các đối số của quy trình và được xử lý lại khỏi các lỗi lệnh được trả về.

## Quản lý mạng bằng dòng lệnh

Sử dụng `argvus networkctl` để truy vấn và điều khiển mạng mà không cần giao diện đồ họa:

```sh
# Show network status
argvus networkctl status

# Toggle networking on/off
argvus networkctl enable
argvus networkctl disable

# Wi-Fi control
argvus networkctl wifi on
argvus networkctl wifi off

# Open the network manager
argvus networkctl manager
```

## Quản lý Bluetooth bằng dòng lệnh

Sử dụng `argvus bluetoothctl` cho bộ chuyển đổi Bluetooth:

```sh
# Show the adapter state (key=value lines)
argvus bluetoothctl status

# Exit 0 when an adapter is available, 1 otherwise
argvus bluetoothctl available

# Turn the adapter on or off, or toggle it
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Open the Bluetooth manager (Blueman)
argvus bluetoothctl manager
```

Khi kernel liệt kê không có bộ điều khiển nào trong `/sys/class/bluetooth`, `status` sẽ trả lời `status=no-controller` ngay lập tức mà không cần hỏi `bluetoothctl` (nếu không sẽ đợi BlueZ khi dịch vụ bị dừng). Mô-đun thanh tác vụ và thẻ Bảng điều khiển đọc cùng một trạng thái.

Sử dụng `--help` trên một trong hai lệnh để xem các tùy chọn và lệnh phụ bổ sung dành riêng cho phiên bản đã cài đặt của bạn.

Khi hộp thoại mật khẩu Wi-Fi mở, các bản cập nhật quét sẽ giữ nguyên SSID và loại bảo mật đã chọn. Nếu mạng đó biến mất hoặc tính bảo mật của mạng thay đổi, quá trình xác nhận sẽ dừng lại kèm theo lỗi và yêu cầu lựa chọn mới.

DNS thủ công gửi máy chủ IPv4 và IPv6 đến cài đặt Trình quản lý mạng tương ứng trước khi cập nhật tệp trình phân giải. Các liên kết tượng trưng của trình phân giải được quản lý dịch vụ được giữ nguyên bằng cách từ chối thay thế tệp. Việc quay lại thành công DNS tự động sẽ khôi phục và xóa bản sao lưu, do đó, chu trình thủ công sau đó sẽ lưu trạng thái trình phân giải hiện tại.

Việc đóng trình chỉnh sửa cấu hình mạng sẽ chờ trong khi công việc hiện tại của nó kết thúc, do đó việc lưu hoặc xóa cấu hình không thể bị gián đoạn bằng cách loại bỏ trình chỉnh sửa. Việc làm mới nền vẫn có thể bị hủy.

Các yêu cầu kết nối Wi-Fi chia sẻ bộ bảo vệ hành động mạng: một tác vụ đang chạy hoặc công việc chỉnh sửa hồ sơ hoàn thành trước khi một đột biến khác có thể bắt đầu. Thông báo tiến độ và hoàn thành tuân theo ngôn ngữ đã chọn.

Xác nhận xóa Bluetooth sẽ giữ nguyên địa chỉ thiết bị ban đầu ngay cả khi danh sách thiết bị được làm mới. Tỷ lệ phần trăm pin từ bluetoothctl được nhận dạng và các truy vấn chi tiết thiết bị chạy đồng thời có giới hạn và thời gian chờ riêng lẻ.

Mật khẩu Wi-Fi, DNS thủ công và các trường tìm kiếm mạng chấp nhận dán trong ngoặc. Các điều khiển đã dán và ngắt dòng được bỏ qua. Các phím tắt ký tự chỉ Ctrl hoặc chỉ Alt không được hỗ trợ sẽ không chèn chữ cái. Người chỉnh sửa chấp nhận các sự kiện AltGr có thể in được mã hóa dưới dạng Ctrl+Alt; Ctrl+S chỉ lưu mà không có Alt. Trình soạn thảo văn bản được chia sẻ giữ điều hướng dọc trong các cột trực quan và căn chỉnh cuộn ngang cho các ký tự rộng.Chỉ báo mạng trên thanh tác vụ theo dõi các sự kiện NetworkManager bằng `argvus networkctl waybar --follow`, với ảnh chụp nhanh ban đầu và kết nối lại màn hình tự động. Nó không còn phải đợi khoảng thời gian bỏ phiếu 10 giây nữa. Đã kết nối, truy cập Internet hạn chế và bị ngắt kết nối vẫn là các trạng thái riêng biệt. Trạng thái Wi-Fi đọc các điểm truy cập được lưu trong bộ nhớ đệm mà không yêu cầu quét. `argvus networkctl waybar` phát ra một ảnh chụp nhanh JSON của Waybar.

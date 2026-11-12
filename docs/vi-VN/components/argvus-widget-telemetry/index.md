---
title: Widget đo từ xa
description: Khối đo từ xa Waybar và chuyển đổi chúng, thông qua phép đo từ xa tiện ích argvus.
---

Tiện ích đo từ xa là thông tin hệ thống được hiển thị trên thanh tác vụ (tiêu đề, phím, hệ thống, CPU
và GPU, bộ nhớ, bộ lưu trữ, quy trình, mạng và bảng điều khiển dành cho nhà phát triển). Nó đang trong quá trình
gói `argvus`: không có bộ sưu tập vỏ.

## Lệnh

```text
argvus widget-telemetry block BLOCK ...   prints the text of one block (used by the bar)
argvus widget-telemetry toggle ...        switches the blocks on and off, and their order
```

`block` là đường dẫn nóng của thanh: nó được gọi mỗi lần làm mới. `toggle` thay đổi khối nào
thanh hiển thị và di chuyển cửa sổ cuộn.

Các khối là `header`, `keys`, `system`, `cpu_gpu`, `memory`, `storage`, `processes`,
`network` và `dev_dashboard`. Thanh tác vụ cũng sử dụng đầu dò `cpu-temp` và `gpu`. Đầu dò GPU chấp nhận `--present` (GPU được hỗ trợ hiển thị trong DRM/sysfs), `--available` (GPU và lệnh btop có sẵn cho bảng điều khiển), `--available-temp` (có thể đọc nhiệt độ có thể quy định) và `--temp` (nhiệt độ hợp lệ cao nhất hoặc `N/A`).

Nhiệt độ GPU được khớp với thiết bị PCI của GPU. Các bài đọc độc quyền của NVIDIA sử dụng ID bus `nvidia-smi`/NVML; Các bài đọc của NVIDIA nouveau, AMD và Intel chỉ sử dụng hwmon khi liên kết thiết bị sysfs của nó phù hợp duy nhất với GPU đó. Nhiệt độ thanh tác vụ là giá trị hợp lệ tối đa trên các GPU được phát hiện. Các bài đọc bị thiếu, không hợp lệ hoặc mơ hồ vẫn không có sẵn. Nhiệt độ độc quyền của NVIDIA yêu cầu nhà cung cấp trình điều khiển; Nhiệt độ của AMD và Intel phụ thuộc vào hạt nhân hiển thị cảm biến hwmon được liên kết. Nhiệt độ Intel không được cung cấp bởi bảng GPU của btop 1.4.7.

## Bắt đầu từ phiên

Đơn vị người dùng `argvus-widget-telemetry.service` chỉ bắt đầu với phiên khi tiện ích được
đã bật. Công tắc chính có cấu hình là `effects.widget_telemetry_enabled`.
được bật theo mặc định, vì vậy tài khoản mới sẽ khởi động tiện ích khi đăng nhập.

Khi nút chuyển đổi đã ghi `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state`, tệp đó
quyết định thay thế. Chỉ một dòng đầu tiên của `enabled` khởi động tiện ích. Càng lớn tuổi
Các tệp `desktop-telemetry-state` và `sysinfo-state` vẫn được đọc khi tệp hiện tại được
thiếu.

Nhãn từ xa được thu gọn bằng tất cả mười ngôn ngữ được hỗ trợ. Các mã định danh kỹ thuật như CPU, GPU, IP, DE, WM và Kernel vẫn giữ nguyên ý nghĩa kỹ thuật của chúng. Thời gian hoạt động được đọc từ `/proc/uptime` và hiển thị bằng đơn vị viết tắt (ví dụ `1d 12h 49m`); nó không còn mở rộng thành các từ đầy đủ từ ngôn ngữ hệ thống. Tên mẫu phần cứng và các giá trị khác của nhà cung cấp vẫn có thể ảnh hưởng đến độ rộng của tiện ích.

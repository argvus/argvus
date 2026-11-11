---
title: Màn hình hệ thống
description: Mở màn hình hệ thống ARGVUS.
---

Mở màn hình được cấu hình bằng:

```sh
argvus --system-monitor
```

Lệnh con `argvus system-monitor` cung cấp điểm vào gốc và tích hợp btop. Đo từ xa hiển thị trên thanh tác vụ là một tính năng riêng biệt được cung cấp bởi `argvus widget-telemetry`. Nhấp vào các mô-đun CPU, Bộ nhớ và GPU sẽ mở btop bên trong bảng ARGVUS TUI với tiêu đề có thương hiệu và chân trang `Esc Close`. Chế độ GPU hiển thị tối đa sáu hộp GPU và không có hộp CPU, bộ nhớ, mạng hoặc quy trình. Nhấn `q` hoặc `Esc` để đóng bảng điều khiển. Tab Kitty phía dưới hiển thị gợi ý `Press q to close` đã bản địa hóa. Các lệnh gọi `argvus system-monitor --cpu`, `--mem` và `--gpu` độc lập khởi chạy btop trực tiếp với các hộp đã chọn.

Bảng điều khiển GPU phụ thuộc vào khả năng của bản dựng btop đã cài đặt và nhà cung cấp thiết bị của hệ thống. Giám sát NVIDIA cần hỗ trợ GPU trong btop, Thư viện quản lý NVIDIA từ trình điều khiển NVIDIA đã cài đặt và quyền truy cập vào thiết bị. Giám sát AMD sử dụng ROCm SMI, đây là tùy chọn và có thể không được cài đặt. Dữ liệu GPU Intel được đọc từ sysfs và phụ thuộc vào quyền truy cập của người dùng vào các giao diện đó; btop 1.4.7 báo cáo mức sử dụng, nguồn điện và xung nhịp của Intel nhưng không cung cấp nhiệt độ Intel. NVIDIA nouveau và các trình điều khiển khác có thể hiển thị nhiệt độ GPU trên thanh tác vụ thông qua hwmon được liên kết với thiết bị, trong khi sự hỗ trợ của nhà cung cấp btop phụ thuộc vào trình điều khiển và thư viện có sẵn. ARGVUS không thay đổi quyền của thiết bị hoặc chạy btop với các đặc quyền nâng cao. Nếu không có GPU nào hiển thị hoặc btop không hiển thị hộp GPU trong vòng ba giây do không có hỗ trợ hoặc nhà cung cấp thì bảng điều khiển sẽ hiển thị thông báo không có sẵn cục bộ.

Màn hình sẽ đọc chủ đề và giọng đã chọn từ
`$XDG_CONFIG_HOME/argvus/config.json` đầu tiên. Các tập tin btop được tạo ra của nó vẫn còn
trạng thái bộ đệm có nguồn gốc; `.active-theme` và `.accent-color` tương thích
dự phòng. Vì btop giữ tập tin chủ đề riêng nên màn hình được điều chỉnh bằng
giao dịch chủ đề (`argvus theme-switch`) dưới dạng bộ điều hợp bên ngoài sau khi thay đổi chuẩn
cam kết; nó không phải là một phần của cây được tạo ra.

## Hợp đồng thời gian chạy

- Chủ đề và điểm nhấn đến từ cấu hình chuẩn (`argvus config get
  /xuất hiện/theme` and `/xuất hiện/accent`). The files `~/.config/argvus/data/.active-theme`
  và `.accent-color` là dự phòng.
- Cấu hình và chủ đề btop được tạo được ghi vào
  `$XDG_CACHE_HOME/argvus/argvus-system-monitor`. Chúng là trạng thái dẫn xuất; không chỉnh sửa chúng.
- Các giá trị mặc định được cài đặt trong `/usr/share/argvus-system-monitor/btop`.
- Các phím tắt mô-đun (`--mem`, `--cpu`, `--net`, `--proc`) chỉ được đọc làm đối số đầu tiên.
  Truyền `-c`/`--config` hoặc `--themes-dir` của btop chạy btop với các đường dẫn đó và ghi không
  cấu hình.
- `--panel [--cpu|--mem|--gpu]` nhúng btop vào vỏ ARGVUS TUI. Chế độ này được sử dụng bởi thanh tác vụ;
  không có `--panel`, màn hình tiếp tục tự thay thế bằng btop.
---
title: Máy ảo
description: Chạy ARGVUS với phần cứng đồ họa ảo.
---

`argvus start` phát hiện ảo hóa và xuất cài đặt tương thích cho các phiên bị ảnh hưởng, bao gồm cả dự phòng kết xuất phần mềm và con trỏ/công cụ sửa đổi khi môi trường được phát hiện yêu cầu.

Nếu máy ảo vẫn không hiển thị, hãy kiểm tra môi trường phiên và nhật ký trước khi thêm ghi đè. Xem [khắc phục sự cố đồ họa](/docs/user-guide/troubleshooting/graphics-and-virtual-machines/).Khi phát hiện ảo hóa, `argvus start` xuất các biến tương thích cho phiên làm việc, bao gồm fallback phần mềm-rendering, phần mềm-con trỏ và bộ sửa đổi. Điều này làm cho đường dẫn đăng nhập bình thường là điều đầu tiên cần thử; không thêm ghi đè vĩnh viễn chỉ vì máy tính để bàn đang chạy trong máy ảo.

Nếu việc phát hiện không đủ, hãy sử dụng ghi đè Hyprland của người dùng như một fallback chẩn đoán:

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```

Chỉ thêm điều này khi ngăn xếp đồ họa VM yêu cầu:

```lua
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```

Đăng xuất và đăng nhập lại sau khi thay đổi môi trường phiên. Xác nhận giá trị hoạt động với `echo $LIBGL_ALWAYS_SOFTWARE`; kết xuất phần mềm có thể làm cho máy tính để bàn có thể sử dụng được nhưng có thể giảm hiệu suất.

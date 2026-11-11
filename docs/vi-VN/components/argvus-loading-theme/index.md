---
title: Đang tải lớp phủ
description: Lớp phủ được hiển thị trong khi phiên tải lại trực tiếp.
---

Lớp phủ tải là bề mặt vỏ lớp GTK 4 được hiển thị trong khi ARGVUS tải lại phiên trực tiếp
(`argvus sessionctl reload`). Công tắc chủ đề không sử dụng nó: màn hình thay đổi tại chỗ. Nó là một trợ giúp được vận chuyển trong gói `argvus` tại
`/usr/lib/argvus/loading-theme/splash` và nó không được đặt trong `/usr/bin`.

Thông tin đăng nhập không sử dụng nó: phần chào mừng và vòng quay bắt đầu phiên là chế độ Wayland của
trình trợ giúp chuyển giao, vẽ cùng một vòng quay không có GTK và hiển thị khung hình đầu tiên của nó trong
mili giây (xem [Chào, khóa và giật gân](/docs/developer-guide/subsystems/greeter-lock-and-splash/)).
Với `--spinner-only`, lớp phủ chỉ hiển thị vòng quay đó và thoát trên `SIGTERM`. Với `--detached`, nó không thoát khi quá trình bắt đầu nó thoát; tải lại phiên sử dụng cả hai cờ và dừng nó thông qua tệp pid của nó.
Nó đọc chủ đề đã chọn từ thư mục dữ liệu ARGVUS và quay lại ARGVUS được đóng gói
Bảng màu tối. Nó chỉ báo cáo tình trạng sẵn sàng sau khung hình đầu tiên của mọi bề mặt mà nó ánh xạ chứ không phải khi
quá trình bắt đầu.
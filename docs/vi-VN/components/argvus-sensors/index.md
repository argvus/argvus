---
title: Cảm biến nhiệt độ
description: Tổng quan về nhiệt độ trực tiếp được hỗ trợ bởi lm_sensors JSON.
---

`argvus sensors` mở bảng Ratatui đang trong quá trình được sử dụng bởi cửa sổ bật lên nhiệt độ CPU trên thanh tác vụ. Nó chạy `sensors -j`, nhóm các bài đọc theo tên chip và nhãn do phần cứng báo cáo và làm mới hai giây một lần. Giám sát GPU được xử lý bởi `argvus system-monitor --gpu`.

Bài đọc sử dụng độ C. Bảng này hiển thị `max`, `crit` và trạng thái cảnh báo khi lm_sensors cung cấp chúng. Một thước đo chỉ được hiển thị khi có sẵn giới hạn dương. Thiếu `lm_sensors`, lỗi lệnh, JSON không hợp lệ và số đọc trống vẫn hiển thị dưới dạng trạng thái có thể phục hồi; thử lại việc bỏ phiếu trong chu kỳ tiếp theo.

Nhấn `q` hoặc `Esc` để đóng. Thanh tác vụ mở bảng trong Kitty thông qua `argvus tui-terminal`; tiêu đề tab dưới cùng được bản địa hóa của nó hiển thị gợi ý đóng. Lệnh này sở hữu chi tiết nhiệt độ CPU; biểu tượng nhiệt độ GPU sẽ mở màn hình btop chỉ dành cho GPU. Gói thanh tác vụ đã phụ thuộc vào `lm_sensors`, do đó không cần gói hoặc phần phụ thuộc riêng biệt.

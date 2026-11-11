---
title: Phiên và hệ thống
description: Mục tiêu phiên ARGVUS và các dịch vụ người dùng của nó.
---

Các định nghĩa đơn vị được đóng gói bởi `argvus-session` trong `/usr/share/argvus/session/config/systemd/user/`. Trình quản lý phiên sở hữu quá trình nhập vòng đời và môi trường. Kho thành phần sở hữu các lệnh được thực thi bởi đơn vị của chúng.

Sử dụng `argvus sessionctl` thay vì khởi động một thành phần theo cách thủ công khi chẩn đoán phiên tích hợp.
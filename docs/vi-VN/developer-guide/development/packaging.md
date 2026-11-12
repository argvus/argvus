---
title: Bao bì
description: Xây dựng và kiểm tra các gói ARGVUS Arch.
---

Mỗi thành phần sở hữu gói Arch trong kho lưu trữ của nó. Kiểm tra `packaging/archlinux/`, các chức năng của gói và danh sách tệp đã cài đặt trước khi ghi lại tải trọng. Các giá trị mặc định của gói trong `/usr/share/argvus` và cấu hình quản trị viên trong `/etc/argvus` không thể thay thế cho nhau.

Để xác minh thời gian chạy, hãy so sánh nội dung gói với `pacman -Ql <package>` và kiểm tra riêng các phần ghi đè của người dùng đang hoạt động.

Các bản dựng cục bộ giữ các đối tượng Hàng hóa trong `build/cargo-target`, bên ngoài cây nguồn gói, do đó, `make build` lặp lại chỉ biên dịch lại những gì đã thay đổi. `make clean` xóa thư mục đó và bản dựng tiếp theo sẽ bắt đầu nguội.

Không suy ra tải trọng thời gian chạy chỉ từ tên kho lưu trữ. Một gói có thể sở hữu các tệp nhị phân, dịch vụ người dùng, mặc định `/usr/share/argvus`, cấu hình hoặc nội dung quản trị viên `/etc/argvus`. Xác nhận tải trọng thực tế bằng `PKGBUILD` và `pacman -Ql` của kho lưu trữ sau khi cài đặt.
